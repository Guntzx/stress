#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use clap::{Parser, Subcommand};
use eframe::NativeOptions;
use std::path::PathBuf;

mod cli;
mod config;
mod gui;
mod load_test;
mod models;
mod report_generator;

#[derive(Parser)]
#[command(name = "stress")]
#[command(about = "Aplicación de pruebas de carga con interfaz gráfica y línea de comandos")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Ejecutar en modo GUI
    #[arg(long, short)]
    gui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Ejecutar prueba individual
    Single {
        /// URL base del servidor
        #[arg(short, long)]
        base_url: String,
        
        /// Método HTTP (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
        #[arg(short, long, default_value = "GET")]
        method: String,
        
        /// Endpoint a probar
        #[arg(short, long)]
        endpoint: String,
        
        /// Descripción de la prueba
        #[arg(short, long, default_value = "Prueba individual")]
        description: String,
        
        /// Número de iteraciones
        #[arg(short, long, default_value = "10")]
        iterations: u32,
        
        /// Peticiones simultáneas
        #[arg(short, long, default_value = "1")]
        concurrent: u32,
        
        /// Tiempo de espera entre lotes (segundos)
        #[arg(short, long, default_value = "1")]
        wait_time: u64,
        
        /// Directorio de salida
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        
        /// Headers HTTP (formato: "nombre:valor")
        #[arg(long)]
        headers: Vec<String>,
        
        /// Parámetros de query (formato: "nombre=valor")
        #[arg(long)]
        query_params: Vec<String>,
        
        /// Body JSON
        #[arg(long)]
        body: Option<String>,
    },
    
    /// Ejecutar suite de pruebas
    Suite {
        /// Archivo JSON con la configuración de la suite
        #[arg(short, long)]
        config_file: PathBuf,
    },
    
    /// Guardar configuración
    Save {
        /// Nombre de la configuración
        #[arg(short, long)]
        name: String,
        
        /// URL base del servidor
        #[arg(short, long)]
        base_url: String,
        
        /// Archivo JSON con las peticiones
        #[arg(short, long)]
        requests_file: PathBuf,
    },
    
    /// Cargar configuración
    Load {
        /// Nombre de la configuración
        #[arg(short, long)]
        name: String,
    },
    
    /// Listar configuraciones guardadas
    List,
    
    /// Eliminar configuración
    Delete {
        /// Nombre de la configuración
        #[arg(short, long)]
        name: String,
    },
    
    /// Generar reporte Excel
    Report {
        /// Directorio con resultados
        #[arg(short, long)]
        results_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Single { 
            base_url, 
            method, 
            endpoint, 
            description, 
            iterations, 
            concurrent, 
            wait_time, 
            output_dir, 
            headers, 
            query_params, 
            body 
        }) => {
            // Parsear método HTTP
            let http_method: models::HttpMethod = method.parse()?;
            
            // Parsear headers
            let http_headers: Vec<models::HttpHeader> = headers
                .iter()
                .filter_map(|h| {
                    let parts: Vec<&str> = h.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some(models::HttpHeader {
                            name: parts[0].trim().to_string(),
                            value: parts[1].trim().to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            // Parsear query parameters
            let query_params: Vec<models::QueryParameter> = query_params
                .iter()
                .filter_map(|q| {
                    let parts: Vec<&str> = q.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some(models::QueryParameter {
                            name: parts[0].trim().to_string(),
                            value: parts[1].trim().to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            let request = models::TestRequest {
                method: http_method,
                endpoint,
                headers: http_headers,
                query_params,
                body,
                description,
            };
            
            cli::run_single_test(&request, &base_url, iterations, concurrent, wait_time, output_dir).await?;
        }
        Some(Commands::Suite { config_file }) => {
            let content = std::fs::read_to_string(config_file)?;
            let suite: models::TestSuite = serde_json::from_str(&content)?;
            cli::run_suite_test(&suite).await?;
        }
        Some(Commands::Save { name, base_url, requests_file }) => {
            let content = std::fs::read_to_string(requests_file)?;
            let requests: Vec<models::TestRequest> = serde_json::from_str(&content)?;
            let now = chrono::Local::now();
            let config = models::SavedConfig {
                name,
                base_url,
                requests: requests.clone(),
                iterations: 10,
                concurrent_requests: 1,
                wait_time: 1,
                output_dir: "./results".to_string(),
                auto_generate_report: true,
                auto_upload_report: false,
                remote_folder_path: String::new(),
                created_at: now,
                description: Some(format!("Guardado por CLI con {} peticiones", requests.len())),
            };
            cli::save_test_config(&config).await?;
        }
        Some(Commands::Load { name }) => {
            let _config = cli::load_test_config(&name).await?;
        }
        Some(Commands::List) => {
            cli::list_configs().await?;
        }
        Some(Commands::Delete { name }) => {
            cli::delete_test_config(&name).await?;
        }
        Some(Commands::Report { results_dir }) => {
            cli::generate_report(&results_dir)?;
        }
        None => {
            // Por defecto, abrir interfaz gráfica si no se especifica comando
            if cli.gui || std::env::args().len() == 1 {
                // Ejecutar interfaz gráfica
                let options = NativeOptions::default();
                
                eframe::run_native(
                    "Test Stress - Pruebas de Carga",
                    options,
                    Box::new(|cc| Box::new(gui::TestStressApp::new(cc))),
                )?;
            } else {
                // Mostrar ayuda solo si se especifica explícitamente
                println!("Test Stress - Aplicación de Pruebas de Carga");
                println!();
                println!("Uso:");
                    println!("  stress                               # Ejecutar interfaz gráfica (por defecto)");
    println!("  stress --gui                         # Ejecutar interfaz gráfica");
    println!("  stress single --help                 # Ver ayuda para prueba individual");
    println!("  stress suite --help                  # Ver ayuda para suite de pruebas");
    println!("  stress save --help                   # Ver ayuda para guardar configuración");
    println!("  stress load --help                   # Ver ayuda para cargar configuración");
    println!("  stress list                          # Listar configuraciones");
    println!("  stress delete --help                 # Ver ayuda para eliminar configuración");
    println!("  stress report --help                 # Ver ayuda para generar reportes");
            }
        }
    }
    
    Ok(())
} 