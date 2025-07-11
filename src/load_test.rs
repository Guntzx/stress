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

    pub async fn run_single_test(
        &self,
        request: &TestRequest,
        base_url: &str,
        iterations: u32,
        concurrent_requests: u32,
        wait_time: u64,
        output_dir: &str,
    ) -> Result<TestSummary, Box<dyn std::error::Error>> {
        info!("Iniciando prueba individual: {}", request.description);
        
        let progress_bar = self.create_progress_bar(iterations, &request.description);
        let mut results = Vec::new();

        // Crear directorio de salida
        fs::create_dir_all(output_dir)?;

        for batch_start in (0..iterations).step_by(concurrent_requests as usize) {
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
                progress_bar.inc(1);
            }

            if batch_end < iterations {
                sleep(Duration::from_secs(wait_time)).await;
            }
        }

        let message = format!("Prueba {} completada", request.description);
        let static_message: &'static str = Box::leak(message.into_boxed_str());
        progress_bar.finish_with_message(static_message);
        self.save_test_results(&results, output_dir, &request.description)?;

        Ok(self.calculate_summary(&results, TestType::Single, &request.description))
    }

    pub async fn run_single_test_with_cancel(
        &self,
        request: &TestRequest,
        base_url: &str,
        iterations: u32,
        concurrent_requests: u32,
        wait_time: u64,
        output_dir: &str,
        cancel_flag: std::sync::Arc<std::sync::Mutex<bool>>,
        cancelled: &mut bool,
    ) -> Result<TestSummary, Box<dyn std::error::Error>> {
        info!("Iniciando prueba individual: {}", request.description);
        let progress_bar = self.create_progress_bar(iterations, &request.description);
        let mut results = Vec::new();
        std::fs::create_dir_all(output_dir)?;
        for batch_start in (0..iterations).step_by(concurrent_requests as usize) {
            let batch_end = (batch_start + concurrent_requests).min(iterations);
            let batch_futures: Vec<_> = (batch_start..batch_end)
                .map(|i| self.execute_request(i + 1, request, base_url))
                .collect();
            let batch_results = futures::stream::iter(batch_futures)
                .buffer_unordered(concurrent_requests as usize)
                .collect::<Vec<_>>()
                .await;
            for result in batch_results {
                if let Ok(result) = result {
                    results.push(result);
                }
                progress_bar.inc(1);
            }
            // Verificar cancelación
            if *cancel_flag.lock().unwrap() {
                *cancelled = true;
                break;
            }
            if batch_end < iterations {
                tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
            }
        }
        let message = if *cancelled {
            format!("Prueba {} cancelada", request.description)
        } else {
            format!("Prueba {} completada", request.description)
        };
        let static_message: &'static str = Box::leak(message.into_boxed_str());
        progress_bar.finish_with_message(static_message);
        self.save_test_results(&results, output_dir, &request.description)?;
        Ok(self.calculate_summary(&results, TestType::Single, &request.description))
    }

    pub async fn run_single_test_with_progress(
        &self,
        request: &TestRequest,
        base_url: &str,
        iterations: u32,
        concurrent_requests: u32,
        wait_time: u64,
        output_dir: &str,
        progress_sender: mpsc::Sender<f32>,
    ) -> Result<TestSummary, Box<dyn std::error::Error>> {
        info!("Iniciando prueba individual: {}", request.description);
        
        let progress_bar = self.create_progress_bar(iterations, &request.description);
        let mut results = Vec::new();
        let mut completed = 0;

        // Crear directorio de salida
        fs::create_dir_all(output_dir)?;

        for batch_start in (0..iterations).step_by(concurrent_requests as usize) {
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

            if batch_end < iterations {
                sleep(Duration::from_secs(wait_time)).await;
            }
        }

        let message = format!("Prueba {} completada", request.description);
        let static_message: &'static str = Box::leak(message.into_boxed_str());
        progress_bar.finish_with_message(static_message);
        self.save_test_results(&results, output_dir, &request.description)?;

        Ok(self.calculate_summary(&results, TestType::Single, &request.description))
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

            if batch_end < iterations {
                sleep(Duration::from_secs(wait_time)).await;
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

    pub async fn run_suite_test(
        &self,
        suite: &TestSuite,
    ) -> Result<Vec<TestSummary>, Box<dyn std::error::Error>> {
        info!("Iniciando suite de pruebas: {}", suite.name);
        
        let mut all_summaries = Vec::new();

        // Crear directorio de salida
        fs::create_dir_all(&suite.output_dir)?;

        for (index, request) in suite.requests.iter().enumerate() {
            info!("Ejecutando petición {}/{}: {}", index + 1, suite.requests.len(), request.description);
            
            let summary = self
                .run_single_test(
                    request,
                    &suite.base_url,
                    suite.iterations,
                    suite.concurrent_requests,
                    suite.wait_time,
                    &suite.output_dir,
                )
                .await?;
            
            all_summaries.push(summary);
        }

        Ok(all_summaries)
    }

    pub async fn run_suite_test_with_cancel(
        &self,
        suite: &TestSuite,
        cancel_flag: std::sync::Arc<std::sync::Mutex<bool>>,
        cancelled: &mut bool,
    ) -> Result<Vec<TestSummary>, Box<dyn std::error::Error>> {
        info!("Iniciando suite de pruebas: {}", suite.name);
        let mut all_summaries = Vec::new();
        std::fs::create_dir_all(&suite.output_dir)?;
        for (index, request) in suite.requests.iter().enumerate() {
            if *cancel_flag.lock().unwrap() {
                *cancelled = true;
                break;
            }
            info!("Ejecutando petición {}/{}: {}", index + 1, suite.requests.len(), request.description);
            let summary = self
                .run_single_test_with_cancel(
                    request,
                    &suite.base_url,
                    suite.iterations,
                    suite.concurrent_requests,
                    suite.wait_time,
                    &suite.output_dir,
                    cancel_flag.clone(),
                    cancelled,
                )
                .await?;
            all_summaries.push(summary);
            if *cancelled {
                break;
            }
        }
        Ok(all_summaries)
    }

    pub async fn run_suite_test_with_progress(
        &self,
        suite: &TestSuite,
        progress_sender: mpsc::Sender<f32>,
    ) -> Result<Vec<TestSummary>, Box<dyn std::error::Error>> {
        info!("Iniciando suite de pruebas: {}", suite.name);
        
        let mut all_summaries = Vec::new();
        let total_requests = suite.requests.len();
        let mut completed_requests = 0;

        // Crear directorio de salida
        fs::create_dir_all(&suite.output_dir)?;

        for (index, request) in suite.requests.iter().enumerate() {
            info!("Ejecutando petición {}/{}: {}", index + 1, suite.requests.len(), request.description);
            
            let summary = self
                .run_single_test(
                    request,
                    &suite.base_url,
                    suite.iterations,
                    suite.concurrent_requests,
                    suite.wait_time,
                    &suite.output_dir,
                )
                .await?;
            
            all_summaries.push(summary);
            completed_requests += 1;
            
            // Enviar progreso actualizado
            let progress = completed_requests as f32 / total_requests as f32;
            let _ = progress_sender.send(progress);
        }

        Ok(all_summaries)
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
            
            let summary = self
                .run_single_test(
                    request,
                    &suite.base_url,
                    suite.iterations,
                    suite.concurrent_requests,
                    suite.wait_time,
                    &suite.output_dir,
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

        // Construir URL completa
        let mut url = format!("{}{}", base_url, request.endpoint);
        
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

        // Agregar headers
        for header in &request.headers {
            req_builder = req_builder.header(&header.name, &header.value);
        }

        // Agregar body si existe y el método lo soporta
        if let Some(body) = &request.body {
            match request.method {
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH => {
                    req_builder = req_builder.json(&serde_json::from_str::<serde_json::Value>(body)?);
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
        let filename = format!("{}/{}_{}.csv", output_dir, test_name.replace(" ", "_"), timestamp);
        
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