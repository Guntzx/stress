// curl_parser.rs — Convierte un comando `curl` en base_url + TestRequest.
// Soporta: -X/--request, -H/--header, -d/--data(-raw/-binary/-ascii/-urlencode), --json,
// -b/--cookie, -u/--user, --oauth2-bearer, -A, -e, -G/--get, -I/--head, --url y la URL suelta.
// Ignora banderas de transporte (-s, -k, -L, -i, --compressed…) y consume el valor de las que
// llevan uno (-o, -m, -x, …) para que no termine tomándose como URL.

use crate::models::{HttpHeader, HttpMethod, TestRequest};
use std::str::FromStr;

pub struct ParsedCurl {
    pub base_url: String,
    pub request: TestRequest,
}

/// Banderas con valor que no nos interesan: hay que consumir el valor igual o
/// terminaría siendo la URL (`curl -o out.json https://x` → "out.json").
const IGNORED_WITH_VALUE: &[&str] = &[
    "-o", "--output", "-w", "--write-out", "-m", "--max-time", "--connect-timeout",
    "-x", "--proxy", "-c", "--cookie-jar", "-E", "--cert", "--key", "--cacert",
    "--capath", "-K", "--config", "-D", "--dump-header", "--limit-rate", "--retry",
    "--retry-delay", "--retry-max-time", "--resolve", "--interface", "--max-redirs",
    "--max-filesize", "--proto", "-C", "--continue-at", "-r", "--range", "-z",
    "--time-cond", "--local-port", "--ciphers", "-Y", "--speed-limit", "-y", "--speed-time",
];

/// Banderas que este modelo no puede representar: mejor avisar que importar a medias.
const UNSUPPORTED: &[&str] = &["-F", "--form", "--form-string", "-T", "--upload-file"];

/// Parsea un comando curl completo. Devuelve error si no encuentra URL.
pub fn parse_curl(input: &str) -> Result<ParsedCurl, String> {
    let tokens = tokenize(&normalize(input));
    if tokens.is_empty() {
        return Err("Comando vacío".into());
    }

    let mut method: Option<HttpMethod> = None;
    let mut headers: Vec<HttpHeader> = Vec::new();
    let mut body: Option<String> = None;
    let mut url: Option<String> = None;
    let mut bare: Vec<String> = Vec::new();
    let mut as_query = false; // -G / --get

    let mut i = 0;
    // Saltar el primer token si es "curl"
    if tokens[0].eq_ignore_ascii_case("curl") {
        i = 1;
    }

    while i < tokens.len() {
        let tok = tokens[i].clone();
        // Permite forma --flag=value
        let (flag, inline_val) = match tok.split_once('=') {
            Some((f, v)) if f.starts_with('-') => (f.to_string(), Some(v.to_string())),
            _ => (tok.clone(), None),
        };

        let mut next_val = || -> Option<String> {
            if let Some(v) = &inline_val {
                Some(v.clone())
            } else {
                i += 1;
                tokens.get(i).cloned()
            }
        };

        match flag.as_str() {
            "-X" | "--request" => {
                if let Some(v) = next_val() {
                    method = Some(HttpMethod::from_str(&v)?);
                }
            }
            "-H" | "--header" => {
                if let Some(v) = next_val() {
                    if let Some((name, value)) = v.split_once(':') {
                        let value = value.trim();
                        // `-H 'Accept;'` en curl borra el header; los vacíos no aportan.
                        if !value.is_empty() {
                            push_header(&mut headers, name.trim(), value);
                        }
                    }
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii"
            | "--data-urlencode" => {
                if let Some(v) = next_val() {
                    body = Some(match &body {
                        Some(prev) => format!("{prev}&{v}"),
                        None => v,
                    });
                }
            }
            // curl >= 7.82: body JSON + Content-Type/Accept automáticos
            "--json" => {
                if let Some(v) = next_val() {
                    body = Some(v);
                    push_header(&mut headers, "Content-Type", "application/json");
                    push_header(&mut headers, "Accept", "application/json");
                }
            }
            "-G" | "--get" => as_query = true,
            "-I" | "--head" => method = Some(HttpMethod::HEAD),
            "-b" | "--cookie" => {
                if let Some(v) = next_val() {
                    push_header(&mut headers, "Cookie", &v);
                }
            }
            "-A" | "--user-agent" => {
                if let Some(v) = next_val() {
                    push_header(&mut headers, "User-Agent", &v);
                }
            }
            "-e" | "--referer" => {
                if let Some(v) = next_val() {
                    push_header(&mut headers, "Referer", &v);
                }
            }
            "-u" | "--user" => {
                if let Some(v) = next_val() {
                    let auth = format!("Basic {}", base64_encode(v.as_bytes()));
                    push_header(&mut headers, "Authorization", &auth);
                }
            }
            "--oauth2-bearer" => {
                if let Some(v) = next_val() {
                    push_header(&mut headers, "Authorization", &format!("Bearer {v}"));
                }
            }
            "--url" => url = next_val(),
            f if UNSUPPORTED.contains(&f) => {
                return Err(format!("{f} (multipart / upload) no se puede importar"));
            }
            f if IGNORED_WITH_VALUE.contains(&f) => {
                next_val();
            }
            // ponytail: el resto de banderas sin valor (-s, -k, -L, -i, --compressed…) se ignoran.
            f if f.starts_with('-') && f != "-" => {}
            _ => bare.push(tok),
        }
        i += 1;
    }

    // La URL es el primer token suelto con esquema; si ninguno lo trae, el primero a secas.
    let url = url
        .or_else(|| bare.iter().find(|t| t.contains("://")).cloned())
        .or_else(|| bare.first().cloned())
        .ok_or("No se encontró URL en el comando curl")?;
    let (base_url, mut endpoint) = split_url(&url);

    // Si hay body y no se especificó método, curl usa POST.
    let mut method = method.unwrap_or(if body.is_some() {
        HttpMethod::POST
    } else {
        HttpMethod::GET
    });

    // -G: los datos viajan en la query, no en el body.
    if as_query {
        if let Some(data) = body.take() {
            let sep = if endpoint.contains('?') { '&' } else { '?' };
            endpoint = format!("{endpoint}{sep}{data}");
        }
        method = HttpMethod::GET;
    }

    let description = format!("{} {}", method, label_for(&endpoint, &base_url));

    Ok(ParsedCurl {
        base_url,
        request: TestRequest {
            method,
            endpoint,
            headers,
            query_params: Vec::new(),
            body,
            description,
        },
    })
}

/// Agrega un header pisando el anterior si se repite (la UI los guarda como objeto JSON).
fn push_header(headers: &mut Vec<HttpHeader>, name: &str, value: &str) {
    if let Some(h) = headers.iter_mut().find(|h| h.name.eq_ignore_ascii_case(name)) {
        h.value = value.to_string();
        return;
    }
    headers.push(HttpHeader { name: name.to_string(), value: value.to_string() });
}

/// Nombre corto para la descripción: último segmento del path (o el host).
/// Va al nombre del CSV, así que conviene que sea corto y sin `/` ni `?`.
fn label_for(endpoint: &str, base_url: &str) -> String {
    let path = endpoint.split('?').next().unwrap_or("/");
    match path.rsplit('/').find(|s| !s.is_empty()) {
        Some(seg) => seg.to_string(),
        None => base_url.rsplit("://").next().unwrap_or(base_url).to_string(),
    }
}

/// Divide una URL en (scheme://host[:port], /path?query#frag).
/// Sin esquema se asume https:// — reqwest lo necesita para poder pedir.
pub fn split_url(url: &str) -> (String, String) {
    let url = url.trim();
    let owned;
    let url = match url.contains("://") {
        true => url,
        false => {
            owned = format!("https://{url}");
            owned.as_str()
        }
    };
    let after_scheme = url.find("://").map(|p| p + 3).unwrap_or(0);
    match url[after_scheme..].find(['/', '?']) {
        Some(rel) => {
            let split_at = after_scheme + rel;
            let rest = &url[split_at..];
            let endpoint = if rest.starts_with('?') { format!("/{rest}") } else { rest.to_string() };
            (url[..split_at].to_string(), endpoint)
        }
        None => (url.to_string(), "/".to_string()),
    }
}

/// Comillas tipográficas y BOM: llegan al pegar desde Slack, Word o un chat.
fn normalize(input: &str) -> String {
    input
        .trim_start_matches('\u{feff}')
        .replace(['\u{2018}', '\u{2019}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"")
}

/// Tokeniza respetando comillas simples/dobles, `$'...'` (ANSI-C, lo que emite
/// "Copy as cURL" cuando el body trae escapes) y continuaciones de línea (`\` + newline).
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut has_token = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '$' if chars.peek() == Some(&'\'') => {
                chars.next();
                has_token = true;
                while let Some(ch) = chars.next() {
                    if ch == '\'' {
                        break;
                    }
                    if ch == '\\' {
                        match chars.next() {
                            Some('n') => cur.push('\n'),
                            Some('t') => cur.push('\t'),
                            Some('r') => cur.push('\r'),
                            Some(other) => cur.push(other),
                            None => {}
                        }
                        continue;
                    }
                    cur.push(ch);
                }
            }
            '\'' => {
                has_token = true;
                for ch in chars.by_ref() {
                    if ch == '\'' {
                        break;
                    }
                    cur.push(ch);
                }
            }
            '"' => {
                has_token = true;
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    if ch == '\\' {
                        if let Some(&n) = chars.peek() {
                            if n == '"' || n == '\\' || n == '$' || n == '`' {
                                cur.push(n);
                                chars.next();
                                continue;
                            }
                        }
                    }
                    cur.push(ch);
                }
            }
            '\\' => {
                // Continuación de línea o escape simple
                match chars.peek() {
                    Some('\n') | Some('\r') => {
                        chars.next();
                    }
                    Some(&n) => {
                        has_token = true;
                        cur.push(n);
                        chars.next();
                    }
                    None => {}
                }
            }
            c if c.is_whitespace() => {
                if has_token {
                    tokens.push(std::mem::take(&mut cur));
                    has_token = false;
                }
            }
            c => {
                has_token = true;
                cur.push(c);
            }
        }
    }
    if has_token {
        tokens.push(cur);
    }
    tokens
}

/// base64 estándar sin dependencias (para -u user:pass → Authorization: Basic).
pub fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_curl() {
        let cmd = "curl -X POST 'https://api.example.com:8443/v1/users?active=true' \
                   -H 'Content-Type: application/json' \
                   -H \"Authorization: Bearer abc123\" \
                   --data-raw '{\"name\":\"ada\"}'";
        let p = parse_curl(cmd).unwrap();
        assert_eq!(p.base_url, "https://api.example.com:8443");
        assert_eq!(p.request.endpoint, "/v1/users?active=true");
        assert_eq!(p.request.method, HttpMethod::POST);
        assert_eq!(p.request.headers.len(), 2);
        assert_eq!(p.request.headers[0].name, "Content-Type");
        assert_eq!(p.request.body.as_deref(), Some("{\"name\":\"ada\"}"));
        assert_eq!(p.request.description, "POST users");
    }

    #[test]
    fn data_implies_post() {
        let p = parse_curl("curl http://localhost:8080/login -d 'user=a&pass=b'").unwrap();
        assert_eq!(p.request.method, HttpMethod::POST);
        assert_eq!(p.base_url, "http://localhost:8080");
        assert_eq!(p.request.endpoint, "/login");
        assert_eq!(p.request.body.as_deref(), Some("user=a&pass=b"));
    }

    #[test]
    fn bare_get_no_path() {
        let p = parse_curl("curl https://example.com").unwrap();
        assert_eq!(p.base_url, "https://example.com");
        assert_eq!(p.request.endpoint, "/");
        assert_eq!(p.request.method, HttpMethod::GET);
        assert_eq!(p.request.description, "GET example.com");
    }

    #[test]
    fn flag_with_equals_and_basic_auth() {
        let p = parse_curl("curl --url=https://x.io/a --user=aladdin:opensesame").unwrap();
        assert_eq!(p.base_url, "https://x.io");
        assert_eq!(p.request.endpoint, "/a");
        assert_eq!(p.request.headers[0].name, "Authorization");
        assert_eq!(p.request.headers[0].value, "Basic YWxhZGRpbjpvcGVuc2VzYW1l");
    }

    #[test]
    fn no_url_errors() {
        assert!(parse_curl("curl -X GET").is_err());
        assert!(parse_curl("curl -F 'file=@a.png' https://x.io/up").is_err());
        assert!(parse_curl("curl -X BREW https://x.io").is_err());
    }

    #[test]
    fn value_flags_dont_steal_the_url() {
        let p = parse_curl("curl -s -o /tmp/out.json -m 5 https://api.x.io/v1/ping").unwrap();
        assert_eq!(p.base_url, "https://api.x.io");
        assert_eq!(p.request.endpoint, "/v1/ping");
    }

    // Lo que emite Chrome/Firefox "Copy as cURL" cuando el body lleva escapes.
    #[test]
    fn ansi_c_quoting_and_compressed() {
        let cmd = "curl 'https://api.x.io/v1/login' \\\n  -H $'Cookie: a=1; b=2' \\\n  \
                   --data-raw $'{\\n  \"pass\": \"a\\'b\"\\n}' --compressed";
        let p = parse_curl(cmd).unwrap();
        assert_eq!(p.request.method, HttpMethod::POST);
        assert_eq!(p.request.headers[0].value, "a=1; b=2");
        assert_eq!(p.request.body.as_deref(), Some("{\n  \"pass\": \"a'b\"\n}"));
    }

    #[test]
    fn get_flag_moves_data_to_query() {
        let p = parse_curl("curl -G https://x.io/search?lang=es -d 'q=bus' -d 'page=2'").unwrap();
        assert_eq!(p.request.method, HttpMethod::GET);
        assert_eq!(p.request.endpoint, "/search?lang=es&q=bus&page=2");
        assert!(p.request.body.is_none());
    }

    #[test]
    fn scheme_less_url_and_query_only() {
        let p = parse_curl("curl api.x.io/v1/a").unwrap();
        assert_eq!(p.base_url, "https://api.x.io");
        assert_eq!(p.request.endpoint, "/v1/a");

        let p = parse_curl("curl 'https://x.io?a=1'").unwrap();
        assert_eq!(p.base_url, "https://x.io");
        assert_eq!(p.request.endpoint, "/?a=1");
    }

    #[test]
    fn json_flag_sets_headers() {
        let p = parse_curl("curl --json '{\"a\":1}' https://x.io/v1/j").unwrap();
        assert_eq!(p.request.method, HttpMethod::POST);
        assert_eq!(p.request.headers.len(), 2);
        assert_eq!(p.request.body.as_deref(), Some("{\"a\":1}"));
    }

    #[test]
    fn smart_quotes_from_a_chat_paste() {
        let p = parse_curl("curl \u{2018}https://x.io/v1/a\u{2019} -H \u{201c}Accept: */*\u{201d}").unwrap();
        assert_eq!(p.request.endpoint, "/v1/a");
        assert_eq!(p.request.headers[0].value, "*/*");
    }
}
