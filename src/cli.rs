use crate::config::{ensure_output_directory, get_output_directory, save_config, load_config, list_saved_configs, delete_config};
use crate::load_test::LoadTester;
use crate::models::{TestRequest, TestSuite, SavedConfig, HttpMethod, HttpHeader, QueryParameter};
use crate::report_generator::generate_excel_report;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use std::io::{self, Write};

fn check_limits_cli(iterations: u32, concurrent: u32, wait_time: u64) {
    if iterations < 1 {
        eprintln!("Error: El mínimo permitido para iteraciones es 1.");
        std::process::exit(1);
    }
    if concurrent < 1 {
        eprintln!("Error: El mínimo permitido para peticiones simultáneas es 1.");
        std::process::exit(1);
    }
    if wait_time < 1 {
        eprintln!("Error: El mínimo permitido para tiempo de espera es 1 segundo.");
        std::process::exit(1);
    }
    let mut warning = None;
    if iterations > 100 {
        warning = Some("El máximo recomendado para iteraciones es 100. ¿Deseas continuar de todos modos? (escribe: deseo continuar de todos modos)");
    } else if concurrent > 100 {
        warning = Some("El máximo recomendado para peticiones simultáneas es 100. ¿Deseas continuar de todos modos? (escribe: deseo continuar de todos modos)");
    } else if wait_time > 100 {
        warning = Some("El máximo recomendado para tiempo de espera es 100 segundos. ¿Deseas continuar de todos modos? (escribe: deseo continuar de todos modos)");
    }
    if let Some(msg) = warning {
        println!("ADVERTENCIA: {}", msg);
        print!("Confirma: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error leyendo la confirmación");
            std::process::exit(1);
        }
        if input.trim() != "deseo continuar de todos modos" {
            println!("Operación cancelada por el usuario.");
            std::process::exit(1);
        }
    }
}

pub async fn run_single_test(
    request: &TestRequest,
    base_url: &str,
    iterations: u32,
    concurrent: u32,
    wait_time: u64,
    output_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Ejecutando prueba individual: {}", request.description);
    
    // Validar límites
    check_limits_cli(iterations, concurrent, wait_time);
    
    // Validar que el directorio de salida sea obligatorio
    let output_path = match output_dir {
        Some(ref path) if !path.to_string_lossy().trim().is_empty() => path.to_string_lossy().to_string(),
        _ => {
            eprintln!("Error: Debes especificar el directorio de salida con --output-dir");
            std::process::exit(1);
        }
    };
    ensure_output_directory(&output_path)?;
    
    // Crear tester y ejecutar prueba
    let tester = LoadTester::new();
    let summary = tester
        .run_single_test(request, base_url, iterations, concurrent, wait_time, &output_path)
        .await?;
    
    // Mostrar resumen
    print_summary(&summary);
    
    Ok(())
}

pub async fn run_suite_test(suite: &TestSuite) -> Result<(), Box<dyn std::error::Error>> {
    info!("Ejecutando suite de pruebas: {}", suite.name);
    
    // Validar límites
    check_limits_cli(suite.iterations, suite.concurrent_requests, suite.wait_time);
    
    // Crear tester y ejecutar suite
    let tester = LoadTester::new();
    let summaries = tester.run_suite_test(suite).await?;
    
    // Mostrar resumen de cada petición
    for summary in &summaries {
        print_summary(summary);
        println!();
    }
    
    Ok(())
}

pub async fn save_test_config(config: &SavedConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Guardando configuración: {}", config.name);
    save_config(config)?;
    println!("Configuración guardada exitosamente: {}", config.name);
    Ok(())
}

pub async fn load_test_config(name: &str) -> Result<SavedConfig, Box<dyn std::error::Error>> {
    info!("Cargando configuración: {}", name);
    let config = load_config(name)?;
    // Validar límites si la config tiene requests tipo suite
    if config.requests.len() > 1 {
        // Intentar deserializar como suite
        if let Ok(suite) = serde_json::from_str::<crate::models::TestSuite>(&serde_json::to_string(&config).unwrap()) {
            check_limits_cli(suite.iterations, suite.concurrent_requests, suite.wait_time);
        }
    }
    println!("Configuración cargada exitosamente: {}", name);
    Ok(config)
}

pub async fn list_configs() -> Result<(), Box<dyn std::error::Error>> {
    let configs = list_saved_configs()?;
    if configs.is_empty() {
        println!("No hay configuraciones guardadas.");
    } else {
        println!("Configuraciones disponibles:");
        for config in configs {
            println!("  - {}", config);
        }
    }
    Ok(())
}

pub async fn delete_test_config(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Eliminando configuración: {}", name);
    delete_config(name)?;
    println!("Configuración eliminada exitosamente: {}", name);
    Ok(())
}

pub async fn generate_report(results_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generando reporte Excel");
    
    let results_path = results_dir.to_string_lossy();
    
    if !results_dir.exists() {
        return Err(format!("El directorio {} no existe", results_path).into());
    }
    
    generate_excel_report(&results_path).await?;
    
    println!("Reporte Excel generado exitosamente en: {}/excels", results_path);
    
    Ok(())
}

fn print_summary(summary: &crate::models::TestSummary) {
    println!("\n=== Resumen de {} ===", summary.request_name);
    println!("Total de peticiones: {}", summary.total_requests);
    println!("Peticiones exitosas: {}", summary.successful_requests);
    println!("Peticiones fallidas: {}", summary.failed_requests);
    println!("Tasa de éxito: {:.2}%", summary.success_rate);
    println!("Tiempo total: {} ms", summary.total_duration_ms);
    println!("Tiempo promedio: {:.2} ms", summary.average_duration_ms);
    println!("Tiempo mínimo: {} ms", summary.min_duration_ms);
    println!("Tiempo máximo: {} ms", summary.max_duration_ms);
} 