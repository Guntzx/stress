// postman.rs — Importa colecciones Postman (v2.x) y sus archivos de entorno.
// Aplana carpetas, resuelve {{variables}} (el entorno pisa a la colección),
// traduce auth bearer/basic/apikey a headers y devuelve peticiones listas para la suite.

use crate::curl_parser::{base64_encode, split_url};
use crate::models::{HttpHeader, HttpMethod, TestRequest};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct ImportedCollection {
    pub name: String,
    pub base_url: String,
    pub requests: Vec<TestRequest>,
    /// Variables {{x}} sin valor: hay que completarlas a mano antes de correr.
    pub sin_valor: Vec<String>,
    /// Cosas que no se pudieron traducir (body form-data, URL sin `raw`, …).
    pub avisos: Vec<String>,
    /// Cuántas variables quedaron resueltas (colección + entorno).
    pub vars_usadas: usize,
}

/// Importa uno o varios archivos: la colección y, opcionalmente, entornos.
/// El orden no importa, se reconocen por su forma (`item` vs `values`).
pub fn import_files(paths: &[PathBuf]) -> Result<ImportedCollection, String> {
    let mut coleccion: Option<Value> = None;
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut vars_env: HashMap<String, String> = HashMap::new();

    for p in paths {
        let txt = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", nombre_archivo(p)))?;
        let json: Value = serde_json::from_str(&txt)
            .map_err(|e| format!("{}: JSON inválido ({e})", nombre_archivo(p)))?;

        if json.get("item").is_some() {
            leer_vars(json.get("variable"), &mut vars);
            coleccion = Some(json);
        } else if json.get("values").is_some() {
            leer_vars(json.get("values"), &mut vars_env);
        } else {
            return Err(format!("{}: no es colección ni entorno Postman", nombre_archivo(p)));
        }
    }

    let coleccion = coleccion
        .ok_or("Falta la colección: elegí también el archivo con las peticiones")?;
    vars.extend(vars_env); // el entorno pisa a la colección

    let mut ctx = Ctx { vars, sin_valor: BTreeSet::new(), avisos: Vec::new() };
    let mut brutas: Vec<(String, TestRequest)> = Vec::new();
    recolectar(&coleccion, coleccion.get("auth"), &mut brutas, &mut ctx);

    if brutas.is_empty() {
        return Err("La colección no trae peticiones utilizables".into());
    }

    // Una sola base_url para toda la suite: gana la más repetida. Las peticiones
    // de otro host conservan la URL completa en el endpoint (load_test la respeta).
    let base_url = base_mayoritaria(&brutas);
    let requests = brutas
        .into_iter()
        .map(|(url, mut req)| {
            let (base, endpoint) = split_url(&url);
            req.endpoint = if base == base_url { endpoint } else { url };
            req
        })
        .collect();

    Ok(ImportedCollection {
        name: coleccion["info"]["name"].as_str().unwrap_or("Colección Postman").to_string(),
        base_url,
        requests,
        sin_valor: ctx.sin_valor.into_iter().collect(),
        avisos: ctx.avisos,
        vars_usadas: ctx.vars.len(),
    })
}

// ── Recorrido del árbol ──────────────────────────────────────────────────────

/// Las carpetas anidan `item` dentro de `item`; la auth se hereda hacia abajo.
fn recolectar<'a>(
    nodo: &'a Value,
    auth_heredada: Option<&'a Value>,
    out: &mut Vec<(String, TestRequest)>,
    ctx: &mut Ctx,
) {
    let auth = nodo.get("auth").or(auth_heredada);

    if let Some(hijos) = nodo.get("item").and_then(|i| i.as_array()) {
        for hijo in hijos {
            recolectar(hijo, auth, out, ctx);
        }
        return;
    }

    let Some(req) = nodo.get("request") else { return };
    let nombre = nodo.get("name").and_then(|v| v.as_str()).unwrap_or("sin nombre").to_string();

    let Some(url_bruta) = url_de(req.get("url")) else {
        ctx.avisos.push(format!("{nombre}: sin URL legible"));
        return;
    };
    let url = ctx.resolver(&url_bruta);

    let method = req
        .get("method")
        .and_then(|v| v.as_str())
        .and_then(|m| HttpMethod::from_str(m).ok())
        .unwrap_or(HttpMethod::GET);

    let mut headers: Vec<HttpHeader> = Vec::new();
    if let Some(hs) = req.get("header").and_then(|v| v.as_array()) {
        for h in hs {
            if desactivado(h) {
                continue;
            }
            let (Some(k), Some(v)) = (texto(h, "key"), texto(h, "value")) else { continue };
            if k.trim().is_empty() {
                continue;
            }
            let (k, v) = (ctx.resolver(&k), ctx.resolver(&v));
            headers.push(HttpHeader { name: k, value: v });
        }
    }

    if let Some(a) = req.get("auth").or(auth) {
        if let Some(h) = header_de_auth(a, ctx) {
            headers.push(h);
        }
    }

    let (body, ct) = cuerpo(req.get("body"), ctx, &nombre);
    if let (Some(ct), false) = (ct, tiene_header(&headers, "content-type")) {
        headers.push(HttpHeader { name: "Content-Type".into(), value: ct.into() });
    }

    out.push((
        url,
        TestRequest {
            method,
            endpoint: String::new(), // se completa al elegir la base_url
            headers,
            query_params: Vec::new(),
            body,
            description: nombre,
        },
    ));
}

// ── Piezas sueltas ───────────────────────────────────────────────────────────

/// Postman 2.x guarda la URL como string o como objeto con `raw`.
fn url_de(v: Option<&Value>) -> Option<String> {
    let v = v?;
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    v.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string())
}

/// Devuelve (body, content-type sugerido).
fn cuerpo(b: Option<&Value>, ctx: &mut Ctx, nombre: &str) -> (Option<String>, Option<&'static str>) {
    let Some(b) = b else { return (None, None) };
    match b.get("mode").and_then(|m| m.as_str()).unwrap_or("raw") {
        "raw" => {
            let raw = b.get("raw").and_then(|r| r.as_str()).unwrap_or("");
            let raw = ctx.resolver(raw);
            if raw.trim().is_empty() { (None, None) } else { (Some(raw), None) }
        }
        "urlencoded" => {
            let pares: Vec<String> = b
                .get("urlencoded")
                .and_then(|u| u.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|e| !desactivado(e))
                        .filter_map(|e| {
                            let k = texto(e, "key")?;
                            Some(format!("{}={}", k, texto(e, "value").unwrap_or_default()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            if pares.is_empty() {
                (None, None)
            } else {
                (Some(ctx.resolver(&pares.join("&"))), Some("application/x-www-form-urlencoded"))
            }
        }
        "graphql" => {
            let Some(g) = b.get("graphql") else { return (None, None) };
            let query = g.get("query").and_then(|q| q.as_str()).unwrap_or("");
            // `variables` viene como string con JSON adentro.
            let variables = g
                .get("variables")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .unwrap_or(Value::Null);
            let cuerpo = serde_json::json!({ "query": query, "variables": variables }).to_string();
            (Some(ctx.resolver(&cuerpo)), Some("application/json"))
        }
        otro => {
            ctx.avisos.push(format!("{nombre}: body \"{otro}\" no soportado"));
            (None, None)
        }
    }
}

/// bearer / basic / apikey-en-header → Authorization. El resto se ignora.
fn header_de_auth(a: &Value, ctx: &mut Ctx) -> Option<HttpHeader> {
    let tipo = a.get("type").and_then(|t| t.as_str())?;
    let nodo = a.get(tipo)?;
    // v2.1 usa [{key, value}]; v2.0 usa {key: value}.
    let campo = |k: &str| -> Option<String> {
        match nodo.as_array() {
            Some(arr) => arr
                .iter()
                .find(|e| e.get("key").and_then(|x| x.as_str()) == Some(k))
                .and_then(|e| texto(e, "value")),
            None => nodo.get(k).and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    };

    match tipo {
        "bearer" => {
            let token = ctx.resolver(&campo("token")?);
            Some(HttpHeader { name: "Authorization".into(), value: format!("Bearer {token}") })
        }
        "basic" => {
            let usuario = ctx.resolver(&campo("username").unwrap_or_default());
            let clave = ctx.resolver(&campo("password").unwrap_or_default());
            let par = format!("{usuario}:{clave}");
            Some(HttpHeader {
                name: "Authorization".into(),
                value: format!("Basic {}", base64_encode(par.as_bytes())),
            })
        }
        // `in: query` no se puede representar sin tocar la URL: se ignora.
        "apikey" if campo("in").as_deref() != Some("query") => {
            let nombre = ctx.resolver(&campo("key")?);
            let valor = ctx.resolver(&campo("value").unwrap_or_default());
            Some(HttpHeader { name: nombre, value: valor })
        }
        _ => None,
    }
}

/// La base_url de la suite: el host más repetido de la colección.
fn base_mayoritaria(brutas: &[(String, TestRequest)]) -> String {
    let mut cuenta: HashMap<String, usize> = HashMap::new();
    for (url, _) in brutas {
        *cuenta.entry(split_url(url).0).or_default() += 1;
    }
    cuenta
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))) // empate: alfabético estable
        .map(|(base, _)| base)
        .unwrap_or_default()
}

fn leer_vars(v: Option<&Value>, out: &mut HashMap<String, String>) {
    let Some(arr) = v.and_then(|x| x.as_array()) else { return };
    for e in arr {
        let Some(k) = e.get("key").and_then(|k| k.as_str()) else { continue };
        if desactivado(e) || e.get("enabled").and_then(|x| x.as_bool()) == Some(false) {
            continue;
        }
        let valor = match e.get("value") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Null) | None => String::new(),
            Some(otro) => otro.to_string(),
        };
        // Un secreto no exportado llega vacío: mejor que figure como pendiente.
        if valor.is_empty() {
            continue;
        }
        out.insert(k.to_string(), valor);
    }
}

fn desactivado(v: &Value) -> bool {
    v.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false)
}

fn texto(v: &Value, campo: &str) -> Option<String> {
    match v.get(campo)? {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        otro => Some(otro.to_string()),
    }
}

fn tiene_header(headers: &[HttpHeader], nombre: &str) -> bool {
    headers.iter().any(|h| h.name.eq_ignore_ascii_case(nombre))
}

fn nombre_archivo(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string())
}

// ── Sustitución de {{variables}} ─────────────────────────────────────────────

struct Ctx {
    vars: HashMap<String, String>,
    sin_valor: BTreeSet<String>,
    avisos: Vec<String>,
}

impl Ctx {
    /// Reemplaza {{clave}} por su valor. Repite hasta 3 vueltas porque una
    /// variable puede contener otra ({{base}} = "{{proto}}://{{host}}").
    fn resolver(&mut self, texto: &str) -> String {
        let mut actual = texto.to_string();
        for _ in 0..3 {
            if !actual.contains("{{") {
                break;
            }
            let mut nuevo = String::with_capacity(actual.len());
            let mut resto = actual.as_str();
            while let Some(ini) = resto.find("{{") {
                let Some(largo) = resto[ini..].find("}}") else { break };
                let fin = ini + largo;
                nuevo.push_str(&resto[..ini]);
                let clave = resto[ini + 2..fin].trim().to_string();
                match self.vars.get(&clave) {
                    Some(v) => nuevo.push_str(v),
                    None => {
                        self.sin_valor.insert(clave);
                        nuevo.push_str(&resto[ini..fin + 2]);
                    }
                }
                resto = &resto[fin + 2..];
            }
            nuevo.push_str(resto);
            if nuevo == actual {
                break;
            }
            actual = nuevo;
        }
        actual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escribir(nombre: &str, contenido: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("stress_{}_{}", std::process::id(), nombre));
        std::fs::write(&p, contenido).unwrap();
        p
    }

    const COLECCION: &str = r#"{
      "info": { "name": "API Kupos" },
      "variable": [ { "key": "base", "value": "https://api.kupos.cl" } ],
      "auth": { "type": "bearer", "bearer": [ { "key": "token", "value": "{{token}}" } ] },
      "item": [
        {
          "name": "Auth",
          "item": [
            {
              "name": "Login",
              "request": {
                "method": "POST",
                "header": [
                  { "key": "Content-Type", "value": "application/json" },
                  { "key": "X-Viejo", "value": "no", "disabled": true }
                ],
                "body": { "mode": "raw", "raw": "{\"user\":\"{{usuario}}\"}" },
                "url": { "raw": "{{base}}/v1/login" }
              }
            }
          ]
        },
        {
          "name": "Buscar servicios",
          "request": {
            "method": "GET",
            "url": { "raw": "{{base}}/v1/services?origen=1" }
          }
        },
        {
          "name": "Form legacy",
          "request": {
            "method": "POST",
            "auth": { "type": "basic", "basic": [
              { "key": "username", "value": "aladdin" }, { "key": "password", "value": "opensesame" } ] },
            "body": { "mode": "urlencoded", "urlencoded": [
              { "key": "a", "value": "1" }, { "key": "b", "value": "2", "disabled": true } ] },
            "url": { "raw": "https://legacy.otro.cl/form" }
          }
        }
      ]
    }"#;

    const ENTORNO: &str = r#"{
      "name": "QA",
      "values": [
        { "key": "token", "value": "abc123", "enabled": true },
        { "key": "usuario", "value": "ada", "enabled": true },
        { "key": "secreto", "value": "", "enabled": true }
      ]
    }"#;

    #[test]
    fn importa_coleccion_con_entorno() {
        let col = escribir("col.json", COLECCION);
        let env = escribir("env.json", ENTORNO);
        // El orden de los archivos no importa.
        let c = import_files(&[env.clone(), col.clone()]).unwrap();

        assert_eq!(c.name, "API Kupos");
        assert_eq!(c.base_url, "https://api.kupos.cl", "gana el host mayoritario");
        assert_eq!(c.requests.len(), 3, "carpetas aplanadas");

        let login = &c.requests[0];
        assert_eq!(login.description, "Login");
        assert_eq!(login.method, HttpMethod::POST);
        assert_eq!(login.endpoint, "/v1/login");
        assert_eq!(login.body.as_deref(), Some(r#"{"user":"ada"}"#), "variable del entorno resuelta");
        assert!(login.headers.iter().any(|h| h.name == "Authorization" && h.value == "Bearer abc123"),
                "auth de la colección heredada");
        assert!(!login.headers.iter().any(|h| h.name == "X-Viejo"), "header disabled fuera");

        assert_eq!(c.requests[1].endpoint, "/v1/services?origen=1");

        // Otro host: se guarda la URL completa, la base de la suite no sirve.
        let form = &c.requests[2];
        assert_eq!(form.endpoint, "https://legacy.otro.cl/form");
        assert_eq!(form.body.as_deref(), Some("a=1"), "el par disabled no viaja");
        assert!(form.headers.iter().any(|h| h.value == "Basic YWxhZGRpbjpvcGVuc2VzYW1l"),
                "la auth propia pisa la heredada");
        assert!(form.headers.iter().any(|h| h.name == "Content-Type"
                && h.value == "application/x-www-form-urlencoded"));

        std::fs::remove_file(col).ok();
        std::fs::remove_file(env).ok();
    }

    #[test]
    fn sin_entorno_avisa_que_falta_la_variable() {
        let col = escribir("col2.json", COLECCION);
        let c = import_files(&[col.clone()]).unwrap();
        assert_eq!(c.sin_valor, vec!["token".to_string(), "usuario".to_string()]);
        assert_eq!(c.requests[0].body.as_deref(), Some(r#"{"user":"{{usuario}}"}"#));
        std::fs::remove_file(col).ok();
    }

    #[test]
    fn archivo_que_no_es_postman() {
        let malo = escribir("malo.json", r#"{"hola": 1}"#);
        assert!(import_files(&[malo.clone()]).is_err());
        assert!(import_files(&[]).is_err(), "sin colección no hay nada que importar");
        std::fs::remove_file(malo).ok();
    }

    /// La demo que se distribuye en demo/ tiene que importar sin pendientes.
    #[test]
    fn la_demo_del_repo_importa_completa() {
        let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
        let coleccion = raiz.join("demo/demo.postman_collection.json");
        // demo/ está en .gitignore: en un clon limpio no existe.
        if !coleccion.exists() {
            eprintln!("sin demo/ local, test omitido");
            return;
        }
        let c = import_files(&[coleccion, raiz.join("demo/demo.postman_environment.json")]).unwrap();

        assert_eq!(c.base_url, "http://127.0.0.1:8099");
        assert_eq!(c.requests.len(), 8);
        assert!(c.sin_valor.is_empty(), "el entorno cubre todo: {:?}", c.sin_valor);
        assert!(c.avisos.is_empty(), "{:?}", c.avisos);

        let buscar = |n: &str| c.requests.iter().find(|r| r.description == n).unwrap();

        // `noauth` en la petición pisa el bearer de la colección
        assert!(!buscar("Health").headers.iter().any(|h| h.name == "Authorization"));

        let servicios = buscar("Buscar servicios");
        assert_eq!(servicios.endpoint, "/services?origen=1&destino=2");
        assert!(servicios.headers.iter().any(|h| h.value == "Bearer demo-token-123"));

        assert_eq!(buscar("Form legacy").body.as_deref(), Some("rut=11111111-1"));
        assert_eq!(buscar("Borrar reserva").method, HttpMethod::DELETE);
        assert!(buscar("Login").body.as_deref().unwrap().contains("\"ada\""));
    }

    #[test]
    fn variables_anidadas() {
        let mut ctx = Ctx {
            vars: HashMap::from([
                ("base".into(), "{{proto}}://{{host}}".into()),
                ("proto".into(), "https".into()),
                ("host".into(), "api.x.io".into()),
            ]),
            sin_valor: BTreeSet::new(),
            avisos: Vec::new(),
        };
        assert_eq!(ctx.resolver("{{base}}/v1"), "https://api.x.io/v1");
        assert_eq!(ctx.resolver("{{ nada }}"), "{{ nada }}");
        assert!(ctx.sin_valor.contains("nada"));
    }
}
