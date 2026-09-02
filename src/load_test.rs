use crate::models::*;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs;
use std::sync::mpsc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::info;
use std::sync::Arc;
use std::sync::Mutex;

pub struct LoadTester {
    client: Client,
}

impl LoadTester {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Error creando cliente HTTP");

        Self { client }
    }

    pub async fn run_single_test_with_progress_and_cancel(
        &self,
        request: &TestRequest,
        base_url: &str,
        iterations: u32,
        concurrent_requests: u32,
        wait_time: u64,
        output_dir: &str,
        progress_sender: mpsc::Sender<f32>,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> Result<TestSummary, Box<dyn std::error::Error>> {
        info!("Iniciando prueba individual: {}", request.description);
        
        let progress_bar = self.create_progress_bar(iterations, &request.description);
        let mut results = Vec::new();
        let mut completed = 0;
        let mut cancelled = false;

        // Crear directorio de salida
        fs::create_dir_all(output_dir)?;

        for batch_start in (0..iterations).step_by(concurrent_requests as usize) {
            // Verificar cancelación antes de cada lote
            if *cancel_flag.lock().unwrap() {
                cancelled = true;
                break;
            }
            
            let batch_end = (batch_start + concurrent_requests).min(iterations);

            let batch_futures: Vec<_> = (batch_start..batch_end)
                .map(|i| self.execute_request(i + 1, request, base_url))
                .collect();

            let batch_results = stream::iter(batch_futures)
                .buffer_unordered(concurrent_requests as usize)
                .collect::<Vec<_>>()
                .await;

            for result in batch_results {
                if let Ok(result) = result {
                    results.push(result);
                }
                completed += 1;
                progress_bar.inc(1);
                
                // Enviar progreso actualizado
                let progress = completed as f32 / iterations as f32;
                let _ = progress_sender.send(progress);
            }

            if batch_end < iterations && esperar_o_cancelar(wait_time, &cancel_flag).await {
                cancelled = true;
                break;
            }
        }

        let message = if cancelled {
            format!("Prueba {} cancelada", request.description)
        } else {
            format!("Prueba {} completada", request.description)
        };
        let static_message: &'static str = Box::leak(message.into_boxed_str());
        progress_bar.finish_with_message(static_message);
        self.save_test_results(&results, output_dir, &request.description)?;

        Ok(self.calculate_summary(&results, TestType::Single, &request.description))
    }

    pub async fn run_suite_test_with_progress_and_cancel(
        &self,
        suite: &TestSuite,
        progress_sender: mpsc::Sender<f32>,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> Result<Vec<TestSummary>, Box<dyn std::error::Error>> {
        info!("Iniciando suite de pruebas: {}", suite.name);
        
        let mut all_summaries = Vec::new();
        let total_requests = suite.requests.len();
        let mut completed_requests = 0;
        let mut cancelled = false;

        // Crear directorio de salida
        fs::create_dir_all(&suite.output_dir)?;

        for (index, request) in suite.requests.iter().enumerate() {
            // Verificar cancelación antes de cada petición
            if *cancel_flag.lock().unwrap() {
                cancelled = true;
                break;
            }
            
            info!("Ejecutando petición {}/{}: {}", index + 1, suite.requests.len(), request.description);
            
            // Variante cancelable: la otra ignora el flag y había que esperar a que
            // terminara la petición completa antes de que el botón hiciera efecto.
            let (sin_uso, _) = mpsc::channel();
            let summary = self
                .run_single_test_with_progress_and_cancel(
                    request,
                    &suite.base_url,
                    suite.iterations,
                    suite.concurrent_requests,
                    suite.wait_time,
                    &suite.output_dir,
                    sin_uso,
                    cancel_flag.clone(),
                )
                .await?;
            
            all_summaries.push(summary);
            completed_requests += 1;
            
            // Enviar progreso actualizado
            let progress = completed_requests as f32 / total_requests as f32;
            let _ = progress_sender.send(progress);
        }

        if cancelled {
            info!("Suite de pruebas cancelada: {}", suite.name);
        }

        Ok(all_summaries)
    }

    async fn execute_request(
        &self,
        iteration: u32,
        request: &TestRequest,
        base_url: &str,
    ) -> Result<TestResult, Box<dyn std::error::Error>> {
        let start_time = Utc::now();
        let start_instant = Instant::now();

        // Construir URL completa. Un endpoint absoluto (colección Postman con
        // varios hosts) se usa tal cual y se ignora la base_url de la suite.
        let mut url = if request.endpoint.starts_with("http://") || request.endpoint.starts_with("https://") {
            request.endpoint.clone()
        } else {
            format!("{}{}", base_url, request.endpoint)
        };
        
        // Agregar query parameters si existen
        if !request.query_params.is_empty() {
            let query_string: Vec<String> = request.query_params
                .iter()
                .map(|param| format!("{}={}", param.name, param.value))
                .collect();
            url.push_str(&format!("?{}", query_string.join("&")));
        }

        // Construir request
        let mut req_builder = match request.method {
            HttpMethod::GET => self.client.get(&url),
            HttpMethod::POST => self.client.post(&url),
            HttpMethod::PUT => self.client.put(&url),
            HttpMethod::PATCH => self.client.patch(&url),
            HttpMethod::DELETE => self.client.delete(&url),
            HttpMethod::HEAD => self.client.head(&url),
            HttpMethod::OPTIONS => self.client.request(reqwest::Method::OPTIONS, &url),
        };

        // Agregar headers (solo los válidos)
        for header in &request.headers {
            let name = header.name.trim();
            let value = header.value.trim();
            if !name.is_empty() && !value.is_empty() {
                req_builder = req_builder.header(name, value);
            }
        }

        // Agregar body si existe y el método lo soporta
        if let Some(body) = &request.body {
            match request.method {
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH | HttpMethod::DELETE => {
                    // El body va crudo: antes se parseaba como JSON y un body
                    // form-urlencoded (`user=a&pass=b`) cortaba la petición con error,
                    // y esa iteración se descartaba sin aparecer en el CSV.
                    let tiene_ct = request.headers.iter()
                        .any(|h| h.name.eq_ignore_ascii_case("content-type"));
                    if !tiene_ct && serde_json::from_str::<serde_json::Value>(body).is_ok() {
                        req_builder = req_builder.header("Content-Type", "application/json");
                    }
                    req_builder = req_builder.body(body.clone());
                }
                _ => {}
            }
        }

        let response = req_builder.send().await;
        let end_time = Utc::now();
        let duration_ms = start_instant.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                
                let success = status.is_success();
                
                Ok(TestResult {
                    test_type: TestType::Single,
                    request_name: request.description.clone(),
                    iteration,
                    start_time,
                    end_time,
                    duration_ms,
                    success,
                    status_code: Some(status.as_u16()),
                    response: if success { Some(body.clone()) } else { None },
                    error: if !success { Some(format!("HTTP {}: {}", status, body)) } else { None },
                })
            }
            Err(e) => {
                Ok(TestResult {
                    test_type: TestType::Single,
                    request_name: request.description.clone(),
                    iteration,
                    start_time,
                    end_time,
                    duration_ms,
                    success: false,
                    status_code: None,
                    response: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    fn create_progress_bar(&self, total: u32, label: &str) -> ProgressBar {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(label.to_string());
        pb
    }

    fn save_test_results(
        &self,
        results: &[TestResult],
        output_dir: &str,
        test_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}/{}_{}.csv", output_dir, sanitize_name(test_name), timestamp);
        
        let mut wtr = csv::Writer::from_path(&filename)?;
        
        for result in results {
            wtr.serialize(result)?;
        }
        
        wtr.flush()?;
        info!("Resultados guardados en: {}", filename);
        
        Ok(())
    }

    fn calculate_summary(&self, results: &[TestResult], test_type: TestType, request_name: &str) -> TestSummary {
        let total_requests = results.len() as u32;
        let successful_requests = results.iter().filter(|r| r.success).count() as u32;
        let failed_requests = total_requests - successful_requests;
        
        let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();
        let average_duration = if total_requests > 0 {
            total_duration as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let min_duration = results.iter().map(|r| r.duration_ms).min().unwrap_or(0);
        let max_duration = results.iter().map(|r| r.duration_ms).max().unwrap_or(0);
        
        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        // Usar la hora del primer resultado como timestamp fijo del test
        let timestamp = results.first().map(|r| r.start_time).unwrap_or_else(chrono::Utc::now);

        TestSummary {
            test_type,
            request_name: request_name.to_string(),
            total_requests,
            successful_requests,
            failed_requests,
            total_duration_ms: total_duration,
            average_duration_ms: average_duration,
            min_duration_ms: min_duration,
            max_duration_ms: max_duration,
            success_rate,
            timestamp,
        }
    }
}

/// Espera entre lotes, pero troceada: corta apenas se pide cancelar en vez de
/// dormir el intervalo completo. Devuelve true si hay que cancelar.
async fn esperar_o_cancelar(segundos: u64, cancel_flag: &Arc<Mutex<bool>>) -> bool {
    let paso = Duration::from_millis(100);
    let mut restante = Duration::from_secs(segundos);
    while !restante.is_zero() {
        if *cancel_flag.lock().unwrap() {
            return true;
        }
        let dormir = paso.min(restante);
        sleep(dormir).await;
        restante -= dormir;
    }
    *cancel_flag.lock().unwrap()
}

/// Nombre de archivo seguro a partir de la descripción de la prueba.
/// Una descripción tipo `POST /v1/users?active=true` armaba una ruta inválida
/// y el CSV fallaba al escribirse recién al terminar toda la prueba.
pub fn sanitize_name(name: &str) -> String {
    let limpio: String = name
        .trim()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect();
    let limpio = limpio.trim_matches('_').to_string();
    if limpio.is_empty() { "prueba".to_string() } else { limpio }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::channel;

    /// Servidor mínimo: responde 200 a todo y publica lo que recibió.
    /// Contesta `Connection: close` para no tener que manejar keep-alive.
    fn echo_server() -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            for sock in listener.incoming() {
                let Ok(mut sock) = sock else { return };
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut raw = Vec::new();
                    let mut buf = [0u8; 1024];
                    loop {
                        let Ok(n) = sock.read(&mut buf) else { return };
                        raw.extend_from_slice(&buf[..n]);
                        let texto = String::from_utf8_lossy(&raw).to_string();
                        let Some(fin_headers) = texto.find("\r\n\r\n") else { continue };
                        let largo: usize = texto
                            .lines()
                            .find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse().unwrap_or(0)))
                            .unwrap_or(0);
                        if texto.len() >= fin_headers + 4 + largo || n == 0 {
                            let _ = sock.write_all(
                                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                            );
                            let _ = tx.send(texto);
                            return;
                        }
                    }
                });
            }
        });

        (base, rx)
    }

    fn peticion_simple() -> TestRequest {
        TestRequest {
            method: HttpMethod::GET,
            endpoint: "/ping".to_string(),
            headers: Vec::new(),
            query_params: Vec::new(),
            body: None,
            description: "ping".to_string(),
        }
    }

    #[tokio::test]
    async fn body_form_viaja_crudo_y_el_csv_se_escribe() {
        let (base, rx) = echo_server();
        let out = std::env::temp_dir().join(format!("stress_test_{}", std::process::id()));
        let out = out.to_string_lossy().to_string();

        let request = TestRequest {
            method: HttpMethod::POST,
            endpoint: "/login".to_string(),
            headers: vec![HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/x-www-form-urlencoded".to_string(),
            }],
            query_params: Vec::new(),
            body: Some("user=a&pass=b".to_string()),
            // Con `/` y `?` — antes rompía la ruta del CSV al terminar la prueba.
            description: "POST /login?x=1".to_string(),
        };

        let (prog, _prog_rx) = mpsc::channel();
        let resumen = LoadTester::new()
            .run_single_test_with_progress_and_cancel(
                &request, &base, 1, 1, 0, &out, prog, Arc::new(Mutex::new(false)),
            )
            .await
            .unwrap();

        let recibido = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(recibido.ends_with("user=a&pass=b"), "body crudo, sin JSON: {recibido}");
        assert!(recibido.to_lowercase().contains("content-type: application/x-www-form-urlencoded"));
        assert_eq!(resumen.successful_requests, 1, "la iteración no se descartó");

        let csv = fs::read_dir(&out).unwrap().filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("POST__login_x_1_"));
        assert!(csv, "el CSV se guardó con nombre saneado");
        fs::remove_dir_all(&out).ok();
    }

    /// Cadena completa: colección Postman → suite → peticiones reales.
    /// Necesita la API de juguete levantada:
    ///     python3 demo/api_demo.py
    ///     cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn suite_demo_contra_api_local() {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let coleccion = raiz.join("demo/demo.postman_collection.json");
        if !coleccion.exists() {
            eprintln!("sin demo/ local, test omitido");
            return;
        }
        let col = crate::postman::import_files(&[
            coleccion,
            raiz.join("demo/demo.postman_environment.json"),
        ])
        .unwrap();

        let out = std::env::temp_dir().join(format!("stress_demo_{}", std::process::id()));
        let suite = TestSuite {
            name: col.name,
            base_url: col.base_url,
            requests: col.requests,
            iterations: 3,
            concurrent_requests: 3,
            wait_time: 0,
            output_dir: out.to_string_lossy().to_string(),
        };

        let (tx, _rx) = mpsc::channel();
        let resumenes = LoadTester::new()
            .run_suite_test_with_progress_and_cancel(&suite, tx, Arc::new(Mutex::new(false)))
            .await
            .unwrap();

        assert_eq!(resumenes.len(), 8, "una fila por petición");
        for r in &resumenes {
            println!("{:<20} {:>3}/{:<3} ok   prom {:>7.1} ms   máx {:>5} ms",
                r.request_name, r.successful_requests, r.total_requests,
                r.average_duration_ms, r.max_duration_ms);
            // /flaky falla a propósito 1 de cada 5; el resto debe ir 100%.
            if r.request_name != "Endpoint inestable" {
                assert_eq!(r.failed_requests, 0, "{} falló: revisá la API demo", r.request_name);
            }
        }
        fs::remove_dir_all(&out).ok();
    }

    /// Cancelar tiene que cortar durante la espera entre lotes, no al final.
    /// Sin el troceado, 50 iteraciones con 1s de espera tardaban ~49s en parar.
    #[tokio::test]
    async fn cancelar_corta_la_espera_entre_lotes() {
        let (base, _rx) = echo_server();
        let out = std::env::temp_dir().join(format!("stress_cancel_{}", std::process::id()));
        let out = out.to_string_lossy().to_string();

        let flag = Arc::new(Mutex::new(false));
        let flag_ui = flag.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            *flag_ui.lock().unwrap() = true;
        });

        let (prog, _prog_rx) = mpsc::channel();
        let inicio = Instant::now();
        let resumen = LoadTester::new()
            .run_single_test_with_progress_and_cancel(
                &peticion_simple(), &base, 50, 1, 1, &out, prog, flag,
            )
            .await
            .unwrap();

        assert!(inicio.elapsed() < Duration::from_secs(5), "tardó {:?}", inicio.elapsed());
        assert!(resumen.total_requests < 50, "no cortó: {} de 50", resumen.total_requests);
        assert!(resumen.total_requests > 0, "cortó antes de empezar");
        fs::remove_dir_all(&out).ok();
    }

    #[test]
    fn nombres_de_archivo_seguros() {
        assert_eq!(sanitize_name("POST /v1/users?active=true"), "POST__v1_users_active_true");
        assert_eq!(sanitize_name("  "), "prueba");
    }
}
