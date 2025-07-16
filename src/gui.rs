use crate::config::{ensure_output_directory, get_output_directory, save_config, load_config, list_saved_configs, delete_config, list_configs_with_info, search_configs, ConfigInfo};
use crate::load_test::LoadTester;
use crate::models::{TestRequest, TestSuite, SavedConfig, TestSummary, HttpMethod, HttpHeader, QueryParameter};
use crate::report_generator::generate_excel_report_from_files;
use crate::monitor::{SystemMonitor, SystemMetrics, MonitoringConfig, MonitoringType, SSHConfig, format_bytes, format_percentage, save_ssh_config, load_ssh_config};
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use dirs;
use std::process::{Child, Command, Stdio};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct GeneralPrefs {
    show_terminal: bool,
    monitoring_enabled: bool,
    monitoring_type: MonitoringType,
}

fn get_prefs_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::document_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".stress/general_prefs.json")
}

fn load_general_prefs() -> GeneralPrefs {
    let path = get_prefs_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(prefs) = serde_json::from_str(&data) {
            return prefs;
        }
    }
    GeneralPrefs::default()
}

fn save_general_prefs(prefs: &GeneralPrefs) {
    let path = get_prefs_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, serde_json::to_string_pretty(prefs).unwrap_or_default());
}

pub struct TestStressApp {
    // Configuración general
    base_url: String,
    iterations: u32,
    concurrent_requests: u32,
    wait_time: u64,
    output_dir: String,
    
    // Petición actual
    current_request: TestRequest,
    
    // Suite de pruebas
    suite_name: String,
    suite_requests: Vec<TestRequest>,
    
    // Estado de la aplicación
    is_running: bool,
    current_test: String,
    progress: f32,
    logs: Arc<Mutex<Vec<String>>>,
    results: Arc<Mutex<Vec<TestSummary>>>,
    
    // Configuraciones guardadas
    saved_configs: Vec<String>,
    selected_config: String,
    
    // Nuevos campos para gestión avanzada de configuraciones
    configs_info: Vec<ConfigInfo>,
    config_search_query: String,
    config_current_page: usize,
    configs_per_page: usize,
    
    // Pestañas
    current_tab: usize,
    
    // Control de cancelación
    cancel_requested: bool,
    cancel_flag: Option<Arc<Mutex<bool>>>,
    
    // Canal para comunicación con threads
    completion_receiver: Option<mpsc::Receiver<()>>,
    progress_receiver: Option<mpsc::Receiver<f32>>,
    
    // Opciones generales
    show_terminal: bool, // Mostrar terminal (por defecto false)
    terminal_child: Option<Child>, // Proceso de terminal abierto
    
    // Estado para advertencias de límites
    show_limit_warning: bool,
    limit_warning_message: String,
    limit_warning_accept: bool,
    pending_action: Option<PendingAction>,

    // Estado para mensajes de éxito del reporte
    report_success_message: Option<String>,
    
    // Opción para generar reporte automáticamente
    auto_generate_report: bool,
    
    // Opción para subir automáticamente a carpeta remota
    auto_upload_report: bool,
    remote_folder_path: String,
    // Estado para mensaje de guardado de configuración
    config_saved_message: Option<(String, std::time::Instant)>,
    
    // Estado para acciones pendientes de configuraciones
    pending_load_config: Option<String>,
    pending_delete_config: Option<String>,
    
    // Sistema de monitoreo
    system_monitor: Option<SystemMonitor>,
    monitoring_config: MonitoringConfig,
}

// Acción pendiente tras advertencia
#[derive(PartialEq, Eq, Clone, Copy)]
enum PendingAction {
    RunSingleTest,
    RunSuiteTest,
}

impl TestStressApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Cargar configuraciones guardadas
        let saved_configs = list_saved_configs().unwrap_or_default();
        let prefs = load_general_prefs();
        let mut app = Self {
            base_url: "http://localhost:8080".to_string(),
            iterations: 10,
            concurrent_requests: 1,
            wait_time: 1,
            output_dir: "./results".to_string(),
            current_request: TestRequest::default(),
            suite_name: "Nueva suite de pruebas".to_string(),
            suite_requests: Vec::new(),
            is_running: false,
            current_test: String::new(),
            progress: 0.0,
            logs: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            saved_configs,
            selected_config: String::new(),
            current_tab: 0,
            cancel_requested: false,
            cancel_flag: None,
            completion_receiver: None,
            progress_receiver: None,
            show_terminal: prefs.show_terminal,
            terminal_child: None,
            show_limit_warning: false,
            limit_warning_message: String::new(),
            limit_warning_accept: false,
            pending_action: None,
            report_success_message: None,
            auto_generate_report: true, // Por defecto activado
            auto_upload_report: false, // Por defecto desactivado
            remote_folder_path: String::new(), // Por defecto vacío
            configs_info: Vec::new(),
            config_search_query: String::new(),
            config_current_page: 0,
            configs_per_page: 10,
            config_saved_message: None,
            pending_load_config: None,
            pending_delete_config: None,
            system_monitor: None,
            monitoring_config: {
                let mut config = MonitoringConfig::default();
                config.enabled = prefs.monitoring_enabled;
                config.monitoring_type = prefs.monitoring_type;
                config
            },
        };
        // Solo abrir terminal si la preferencia está activa y no hay terminal abierta
        if app.show_terminal {
            app.open_terminal();
        }
        // Cargar automáticamente todas las configuraciones disponibles
        app.refresh_configs();
        
        // Cargar configuración SSH guardada
        if let Ok(Some(ssh_config)) = load_ssh_config() {
            app.monitoring_config.ssh_config = ssh_config;
        }
        
        // Inicializar monitoreo si está habilitado
        if app.monitoring_config.enabled {
            app.system_monitor = Some(SystemMonitor::new(app.monitoring_config.clone()));
            if let Some(ref mut monitor) = app.system_monitor {
                monitor.start_monitoring();
            }
        }
        
        app
    }
    
    fn add_log(&self, message: String) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(format!("[{}] {}", 
                chrono::Utc::now().format("%H:%M:%S"), 
                message
            ));
            // Mantener solo los últimos 100 logs
            if logs.len() > 100 {
                logs.remove(0);
            }
        }
    }
    
    fn add_result(&self, summary: TestSummary) {
        if let Ok(mut results) = self.results.lock() {
            results.push(summary);
        }
    }
    
    fn run_single_test(&mut self) {
        // Si hay advertencia activa, no ejecutar
        if self.show_limit_warning {
            return;
        }
        // Limpiar logs al iniciar nueva prueba
        {
            let mut logs = self.logs.lock().unwrap();
            logs.clear();
        }
        
        self.is_running = true;
        self.progress = 0.0;
        self.cancel_requested = false;
        
        let request = self.current_request.clone();
        let base_url = self.base_url.clone();
        let iterations = self.iterations;
        let concurrent_requests = self.concurrent_requests;
        let wait_time = self.wait_time;
        let output_dir = self.output_dir.clone();
        let logs = Arc::clone(&self.logs);
        let results = Arc::clone(&self.results);
        let auto_generate_report = self.auto_generate_report;
        let auto_upload_report = self.auto_upload_report;
        let remote_folder_path = self.remote_folder_path.clone();
        
        // Crear canales para comunicación
        let (completion_sender, completion_receiver) = mpsc::channel();
        let (progress_sender, progress_receiver) = mpsc::channel();
        self.completion_receiver = Some(completion_receiver);
        self.progress_receiver = Some(progress_receiver);
        
        // Crear flag de cancelación
        let cancel_flag = Arc::new(Mutex::new(false));
        let cancel_flag_clone = Arc::clone(&cancel_flag);
        
        // Ejecutar en thread separado con su propio runtime
        std::thread::spawn(move || {
            // Crear un runtime local para este thread
            let rt = tokio::runtime::Runtime::new().expect("Error creando runtime");
            rt.block_on(async {
                if let Err(e) = Self::execute_single_test_with_progress_and_cancel(
                    &request,
                    &base_url,
                    iterations,
                    concurrent_requests,
                    wait_time,
                    &output_dir,
                    logs,
                    results,
                    progress_sender,
                    cancel_flag_clone,
                    auto_generate_report,
                    auto_upload_report,
                    remote_folder_path,
                ).await {
                    eprintln!("Error ejecutando prueba: {}", e);
                }
                // Notificar completación
                let _ = completion_sender.send(());
            });
        });
        
        // Guardar referencia al flag de cancelación
        self.cancel_flag = Some(cancel_flag);
    }
    
    fn run_suite_test(&mut self) {
        // Si hay advertencia activa, no ejecutar
        if self.show_limit_warning {
            return;
        }
        // Limpiar logs al iniciar nueva prueba
        {
            let mut logs = self.logs.lock().unwrap();
            logs.clear();
        }
        
        self.is_running = true;
        self.progress = 0.0;
        self.cancel_requested = false;
        
        let suite = TestSuite {
            name: self.suite_name.clone(),
            base_url: self.base_url.clone(),
            requests: self.suite_requests.clone(),
            iterations: self.iterations,
            concurrent_requests: self.concurrent_requests,
            wait_time: self.wait_time,
            output_dir: self.output_dir.clone(),
        };
        
        let logs = Arc::clone(&self.logs);
        let results = Arc::clone(&self.results);
        let auto_generate_report = self.auto_generate_report;
        let auto_upload_report = self.auto_upload_report;
        let remote_folder_path = self.remote_folder_path.clone();
        
        // Crear canales para comunicación
        let (completion_sender, completion_receiver) = mpsc::channel();
        let (progress_sender, progress_receiver) = mpsc::channel();
        self.completion_receiver = Some(completion_receiver);
        self.progress_receiver = Some(progress_receiver);
        
        // Crear flag de cancelación
        let cancel_flag = Arc::new(Mutex::new(false));
        let cancel_flag_clone = Arc::clone(&cancel_flag);
        
        // Ejecutar en thread separado con su propio runtime
        std::thread::spawn(move || {
            // Crear un runtime local para este thread
            let rt = tokio::runtime::Runtime::new().expect("Error creando runtime");
            rt.block_on(async {
                if let Err(e) = Self::execute_suite_test_with_progress_and_cancel(&suite, logs, results, progress_sender, cancel_flag_clone, auto_generate_report, auto_upload_report, remote_folder_path).await {
                    eprintln!("Error ejecutando suite: {}", e);
                }
                // Notificar completación
                let _ = completion_sender.send(());
            });
        });
        
        // Guardar referencia al flag de cancelación
        self.cancel_flag = Some(cancel_flag);
    }
    
    async fn execute_single_test_with_progress_and_cancel(
        request: &TestRequest,
        base_url: &str,
        iterations: u32,
        concurrent_requests: u32,
        wait_time: u64,
        output_dir: &str,
        logs: Arc<Mutex<Vec<String>>>,
        results: Arc<Mutex<Vec<TestSummary>>>,
        progress_sender: mpsc::Sender<f32>,
        cancel_flag: Arc<Mutex<bool>>,
        auto_generate_report: bool,
        auto_upload_report: bool,
        remote_folder_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Agregar log inicial
        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("[{}] Iniciando prueba individual: {}", 
                chrono::Local::now().format("%H:%M:%S"), 
                request.description
            ));
        }
        
        // Usar directorio por defecto si no se especifica uno o si hay problemas de permisos
        let final_output_dir = if output_dir.is_empty() {
            get_output_directory()
        } else {
            // Verificar si podemos escribir en el directorio especificado
            match std::fs::create_dir_all(output_dir) {
                Ok(_) => output_dir.to_string(),
                Err(_) => {
                    // Si falla, usar directorio por defecto
                    get_output_directory()
                }
            }
        };
        
        // Crear tester y ejecutar prueba con cancelación
        let tester = LoadTester::new();
        let summary = tester
            .run_single_test_with_progress_and_cancel(request, base_url, iterations, concurrent_requests, wait_time, &final_output_dir, progress_sender, cancel_flag)
            .await?;
        // Buscar el archivo CSV generado en el directorio
        let csv_file = find_csv_file_in_directory(&final_output_dir, &request.description);
        {
            let mut results = results.lock().unwrap();
            results.push(summary);
        }
        
        // Agregar log de finalización
        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("[{}] Prueba individual completada: {}", 
                chrono::Local::now().format("%H:%M:%S"), 
                request.description
            ));
        }
        
        // Generar reporte Excel automáticamente si está habilitado
        if auto_generate_report {
            {
                let mut logs = logs.lock().unwrap();
                logs.push(format!("[{}] Generando reporte Excel automáticamente...", 
                    chrono::Local::now().format("%H:%M:%S")
                ));
            }
            // Crear carpeta de reportes
            let reports_dir = format!("{}/reports", final_output_dir);
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let safe_name = request.description.replace(" ", "_").replace("/", "_").replace("\\", "_");
            let excel_path = format!("{}/report_{}_{}.xlsx", reports_dir, safe_name, timestamp);
            match csv_file {
                Some(csv_path) => {
                    match generate_excel_report_from_files(&[csv_path.clone()], &excel_path) {
                        Ok(excel_path) => {
                            let mut logs = logs.lock().unwrap();
                            logs.push(format!("[{}] ✅ Reporte Excel generado exitosamente en: {}", 
                                chrono::Local::now().format("%H:%M:%S"), 
                                excel_path
                            ));
                            // Subir archivos específicos de la prueba si está habilitado
                            if auto_upload_report && !remote_folder_path.is_empty() {
                                use std::path::Path;
                                use std::fs;
                                
                                // Verificar que la carpeta de destino existe o crearla
                                if !Path::new(&remote_folder_path).exists() {
                                    if let Err(e) = fs::create_dir_all(&remote_folder_path) {
                                        logs.push(format!("[{}] ❌ Error creando carpeta remota: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                        return Ok(());
                                    }
                                }
                                
                                // Crear carpeta específica para esta prueba
                                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                                let safe_name = request.description.replace(" ", "_").replace("/", "_").replace("\\", "_");
                                let test_folder_name = format!("test_{}_{}", safe_name, timestamp);
                                let destination_folder = format!("{}/{}", remote_folder_path, test_folder_name);
                                
                                // Crear la carpeta de destino si no existe
                                if !Path::new(&destination_folder).exists() {
                                    if let Err(e) = fs::create_dir_all(&destination_folder) {
                                        logs.push(format!("[{}] ❌ Error creando carpeta de destino: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                        return Ok(());
                                    }
                                }
                                
                                // Copiar solo los archivos específicos de esta prueba
                                let mut copy_error = None;
                                let mut files_copied = 0;
                                
                                // Copiar el archivo CSV
                                if let Err(e) = fs::copy(&csv_path, format!("{}/{}", destination_folder, csv_path.file_name().unwrap_or_default().to_string_lossy())) {
                                    copy_error = Some(e);
                                } else {
                                    files_copied += 1;
                                }
                                
                                // Copiar el archivo Excel si se generó exitosamente
                                if !copy_error.is_some() {
                                    if let Err(e) = fs::copy(&excel_path, format!("{}/{}", destination_folder, Path::new(&excel_path).file_name().unwrap_or_default().to_string_lossy())) {
                                        copy_error = Some(e);
                                    } else {
                                        files_copied += 1;
                                    }
                                }
                                
                                if let Some(e) = copy_error {
                                    logs.push(format!("[{}] ❌ Error copiando archivos a carpeta remota: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                } else {
                                    logs.push(format!("[{}] ✅ {} archivos de la prueba subidos exitosamente a: {}", chrono::Local::now().format("%H:%M:%S"), files_copied, destination_folder));
                                }
                            }
                        }
                        Err(e) => {
                            let mut logs = logs.lock().unwrap();
                            logs.push(format!("[{}] ❌ Error generando reporte Excel: {}", 
                                chrono::Local::now().format("%H:%M:%S"), 
                                e
                            ));
                        }
                    }
                }
                None => {
                    let mut logs = logs.lock().unwrap();
                    logs.push(format!("[{}] ❌ No se encontró el archivo CSV generado en el directorio: {}", 
                        chrono::Local::now().format("%H:%M:%S"), 
                        final_output_dir
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    async fn execute_suite_test_with_progress_and_cancel(
        suite: &TestSuite,
        logs: Arc<Mutex<Vec<String>>>,
        results: Arc<Mutex<Vec<TestSummary>>>,
        progress_sender: mpsc::Sender<f32>,
        cancel_flag: Arc<Mutex<bool>>,
        auto_generate_report: bool,
        auto_upload_report: bool,
        remote_folder_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Agregar log inicial
        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("[{}] Iniciando suite de pruebas: {}", 
                chrono::Local::now().format("%H:%M:%S"), 
                suite.name
            ));
        }
        
        // Usar directorio por defecto si no se especifica uno o si hay problemas de permisos
        let final_output_dir = if suite.output_dir.is_empty() {
            get_output_directory()
        } else {
            // Verificar si podemos escribir en el directorio especificado
            match std::fs::create_dir_all(&suite.output_dir) {
                Ok(_) => suite.output_dir.clone(),
                Err(_) => {
                    // Si falla, usar directorio por defecto
                    get_output_directory()
                }
            }
        };
        
        // Crear suite con directorio corregido
        let mut corrected_suite = suite.clone();
        corrected_suite.output_dir = final_output_dir.clone();
        
        // Crear tester y ejecutar suite con cancelación
        let tester = LoadTester::new();
        let summaries = tester.run_suite_test_with_progress_and_cancel(&corrected_suite, progress_sender, cancel_flag).await?;
        // Guardar los nombres de los CSV generados
        let mut csv_files = Vec::new();
        for req in &suite.requests {
            let csv_file = PathBuf::from(format!("{}/{}_{}.csv", final_output_dir, req.description.replace(" ", "_"), chrono::Utc::now().format("%Y%m%d_%H%M%S")));
            csv_files.push(csv_file);
        }
        {
            let mut results = results.lock().unwrap();
            results.extend(summaries);
        }
        
        // Agregar log de finalización
        {
            let mut logs = logs.lock().unwrap();
            logs.push(format!("[{}] Suite de pruebas completada: {}", 
                chrono::Local::now().format("%H:%M:%S"), 
                suite.name
            ));
        }
        
        // Generar reporte Excel automáticamente si está habilitado
        if auto_generate_report {
            {
                let mut logs = logs.lock().unwrap();
                logs.push(format!("[{}] Generando reporte Excel automáticamente...", 
                    chrono::Local::now().format("%H:%M:%S")
                ));
            }
            // Crear carpeta de reportes
            let reports_dir = format!("{}/reports", final_output_dir);
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let safe_name = suite.name.replace(" ", "_").replace("/", "_").replace("\\", "_");
            let excel_path = format!("{}/report_{}_{}.xlsx", reports_dir, safe_name, timestamp);
            match generate_excel_report_from_files(&csv_files, &excel_path) {
                Ok(excel_path) => {
                    let mut logs = logs.lock().unwrap();
                    logs.push(format!("[{}] ✅ Reporte Excel generado exitosamente en: {}", 
                        chrono::Local::now().format("%H:%M:%S"), 
                        excel_path
                    ));
                    
                    // Subir archivos específicos de la suite si está habilitado
                    if auto_upload_report && !remote_folder_path.is_empty() {
                        use std::path::Path;
                        use std::fs;
                        
                        // Verificar que la carpeta de destino existe o crearla
                        if !Path::new(&remote_folder_path).exists() {
                            if let Err(e) = fs::create_dir_all(&remote_folder_path) {
                                logs.push(format!("[{}] ❌ Error creando carpeta remota: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                return Ok(());
                            }
                        }
                        
                        // Crear carpeta específica para esta suite
                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                        let safe_name = suite.name.replace(" ", "_").replace("/", "_").replace("\\", "_");
                        let suite_folder_name = format!("suite_{}_{}", safe_name, timestamp);
                        let destination_folder = format!("{}/{}", remote_folder_path, suite_folder_name);
                        
                        // Crear la carpeta de destino si no existe
                        if !Path::new(&destination_folder).exists() {
                            if let Err(e) = fs::create_dir_all(&destination_folder) {
                                logs.push(format!("[{}] ❌ Error creando carpeta de destino: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                return Ok(());
                            }
                        }
                        
                        // Copiar solo los archivos específicos de esta suite
                        let mut copy_error = None;
                        let mut files_copied = 0;
                        
                        // Copiar los archivos CSV de la suite
                        for csv_file in &csv_files {
                            if csv_file.exists() {
                                if let Err(e) = fs::copy(csv_file, format!("{}/{}", destination_folder, csv_file.file_name().unwrap_or_default().to_string_lossy())) {
                                    copy_error = Some(e);
                                    break;
                                } else {
                                    files_copied += 1;
                                }
                            }
                        }
                        
                        // Copiar el archivo Excel si se generó exitosamente
                        if !copy_error.is_some() {
                            if let Err(e) = fs::copy(&excel_path, format!("{}/{}", destination_folder, Path::new(&excel_path).file_name().unwrap_or_default().to_string_lossy())) {
                                copy_error = Some(e);
                            } else {
                                files_copied += 1;
                            }
                        }
                        
                        if let Some(e) = copy_error {
                            logs.push(format!("[{}] ❌ Error copiando archivos a carpeta remota: {}", chrono::Local::now().format("%H:%M:%S"), e));
                        } else {
                            logs.push(format!("[{}] ✅ {} archivos de la suite subidos exitosamente a: {}", chrono::Local::now().format("%H:%M:%S"), files_copied, destination_folder));
                        }
                    }
                }
                Err(e) => {
                    let mut logs = logs.lock().unwrap();
                    logs.push(format!("[{}] ❌ Error generando reporte Excel: {}", 
                        chrono::Local::now().format("%H:%M:%S"), 
                        e
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn save_current_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Determinar qué tipo de configuración guardar basado en la pestaña actual
        let config = if self.current_tab == 0 {
            // Guardar petición individual
            SavedConfig {
                name: format!("{} - {}", self.current_request.description, chrono::Local::now().format("%Y%m%d_%H%M%S")),
                base_url: self.base_url.clone(),
                requests: vec![self.current_request.clone()],
                iterations: self.iterations,
                concurrent_requests: self.concurrent_requests,
                wait_time: self.wait_time,
                output_dir: self.output_dir.clone(),
                auto_generate_report: self.auto_generate_report,
                auto_upload_report: self.auto_upload_report,
                remote_folder_path: self.remote_folder_path.clone(),
                created_at: chrono::Local::now(),
                description: Some(format!("Petición individual: {}", self.current_request.description)),
            }
        } else {
            // Guardar suite de pruebas
            SavedConfig {
                name: self.suite_name.clone(),
                base_url: self.base_url.clone(),
                requests: self.suite_requests.clone(),
                iterations: self.iterations,
                concurrent_requests: self.concurrent_requests,
                wait_time: self.wait_time,
                output_dir: self.output_dir.clone(),
                auto_generate_report: self.auto_generate_report,
                auto_upload_report: self.auto_upload_report,
                remote_folder_path: self.remote_folder_path.clone(),
                created_at: chrono::Local::now(),
                description: Some(format!("Suite de pruebas con {} peticiones", self.suite_requests.len())),
            }
        };
        save_config(&config)?;
        self.refresh_configs();
        // Mostrar mensaje de éxito
        self.config_saved_message = Some(("✅ Configuración guardada exitosamente".to_string(), std::time::Instant::now()));
        Ok(())
    }
    
    fn load_config(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = load_config(name)?;
        // Cargar la configuración en la pestaña apropiada
        self.base_url = config.base_url;
        self.iterations = config.iterations;
        self.concurrent_requests = config.concurrent_requests;
        self.wait_time = config.wait_time;
        self.output_dir = config.output_dir;
        self.auto_generate_report = config.auto_generate_report;
        self.auto_upload_report = config.auto_upload_report;
        self.remote_folder_path = config.remote_folder_path;
        
        if config.requests.len() == 1 {
            // Es una petición individual, cargar en pestaña 0
            self.current_tab = 0;
            self.current_request = config.requests[0].clone();
        } else {
            // Es una suite, cargar en pestaña 1
            self.current_tab = 1;
            self.suite_name = config.name.clone();
            self.suite_requests = config.requests.clone();
            // Validar límites de la suite cargada
            // Buscar los campos de iteraciones, concurrent_requests y wait_time en la suite
            // Si se supera algún máximo, mostrar advertencia igual que en ejecución
            // Si es menor al mínimo, mostrar error
            let suite = crate::models::TestSuite {
                name: self.suite_name.clone(),
                base_url: self.base_url.clone(),
                requests: self.suite_requests.clone(),
                iterations: self.iterations,
                concurrent_requests: self.concurrent_requests,
                wait_time: self.wait_time,
                output_dir: self.output_dir.clone(),
            };
            if suite.iterations < 1 || suite.concurrent_requests < 1 || suite.wait_time < 1 {
                self.add_log("Error: Los valores mínimos para iteraciones, peticiones simultáneas y tiempo de espera son 1.".to_string());
                return Err("Valores mínimos inválidos en la configuración cargada".into());
            }
            // Si hay advertencia, mostrar popup
            self.check_limits(suite.iterations, suite.concurrent_requests, suite.wait_time, PendingAction::RunSuiteTest);
        }
        Ok(())
    }
    
    fn refresh_configs(&mut self) {
        self.saved_configs = list_saved_configs().unwrap_or_default();
        self.configs_info = list_configs_with_info().unwrap_or_default();
    }
    
    fn check_completion(&mut self) {
        // Verificar si hay mensaje de completación
        if let Some(ref mut receiver) = self.completion_receiver {
            // Intentar recibir sin bloquear
            if let Ok(_) = receiver.try_recv() {
                self.is_running = false;
                self.cancel_requested = false;
                self.progress = 1.0;
                self.completion_receiver = None;
                self.progress_receiver = None;
            }
        }
        
        // Verificar actualizaciones de progreso
        if let Some(ref mut receiver) = self.progress_receiver {
            // Intentar recibir sin bloquear
            if let Ok(progress) = receiver.try_recv() {
                self.progress = progress;
            }
        }
    }

    fn open_terminal(&mut self) {
        // Solo abrir si no hay terminal abierta
        if self.terminal_child.is_some() {
            return;
        }
        #[cfg(target_os = "windows")]
        {
            let exe = "cmd.exe";
            let args = ["/C", "start", "cmd.exe", "/K", "echo Logs de Stress App && pause"];
            if let Ok(child) = Command::new(exe)
                .args(&args)
                .spawn() {
                self.terminal_child = Some(child);
            }
        }
        #[cfg(target_os = "macos")]
        {
            let script = "tell application \"Terminal\" to do script \"echo Logs de Stress App; exec bash\"";
            let result = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .spawn();
            if result.is_ok() {
                // No se puede capturar el proceso fácilmente, pero marcamos como abierta
                self.terminal_child = Some(unsafe { std::mem::zeroed() });
            }
        }
        #[cfg(target_os = "linux")]
        {
            let terms = ["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal", "xterm"];
            for term in &terms {
                if let Ok(child) = Command::new(term)
                    .arg("-e")
                    .arg("bash -c 'echo Logs de Stress App; exec bash'")
                    .spawn() {
                    self.terminal_child = Some(child);
                    break;
                }
            }
        }
    }

    fn close_terminal(&mut self) -> bool {
        if let Some(mut child) = self.terminal_child.take() {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            {
                let _ = child.kill();
                return true;
            }
            #[cfg(target_os = "macos")]
            {
                // No se puede cerrar la terminal abierta por osascript
                return false;
            }
        }
        false
    }

    fn check_limits(&mut self, iterations: u32, concurrent_requests: u32, wait_time: u64, action: PendingAction) -> bool {
        // Validar mínimos primero
        if iterations < 1 {
            self.add_log("Error: El mínimo para iteraciones es 1.".to_string());
            return false;
        }
        if concurrent_requests < 1 {
            self.add_log("Error: El mínimo para peticiones simultáneas es 1.".to_string());
            return false;
        }
        if wait_time < 1 {
            self.add_log("Error: El mínimo para tiempo de espera es 1 segundo.".to_string());
            return false;
        }
        // Si ya hay un warning activo, no hacer nada
        if self.show_limit_warning {
            return false;
        }
        // Validar máximos y mostrar advertencia si corresponde
        let mut warning = None;
        if iterations > 100 {
            warning = Some("El máximo recomendado para iteraciones es 100. ¿Deseas continuar de todas formas?".to_string());
        } else if concurrent_requests > 100 {
            warning = Some("El máximo recomendado para peticiones simultáneas es 100. ¿Deseas continuar de todas formas?".to_string());
        } else if wait_time > 100 {
            warning = Some("El máximo recomendado para tiempo de espera es 100 segundos. ¿Deseas continuar de todas formas?".to_string());
        }
        if let Some(msg) = warning {
            self.show_limit_warning = true;
            self.limit_warning_message = msg;
            self.limit_warning_accept = false;
            self.pending_action = Some(action);
            return false;
        }
        true
    }
}

impl eframe::App for TestStressApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Verificar si alguna prueba terminó
        self.check_completion();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test Stress - Pruebas de Carga");
            
            // Pestañas principales
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, 0, "Prueba Individual");
                ui.selectable_value(&mut self.current_tab, 1, "Suite de Pruebas");
                ui.selectable_value(&mut self.current_tab, 2, "Configuraciones");
                ui.selectable_value(&mut self.current_tab, 3, "Resultados");
                ui.selectable_value(&mut self.current_tab, 4, "Opciones Generales");
            });
            
            ui.separator();
            
            match self.current_tab {
                0 => self.render_single_test_tab(ui),
                1 => self.render_suite_test_tab(ui),
                2 => self.render_configs_tab(ui),
                3 => self.render_results_tab(ui),
                4 => self.render_general_options_tab(ui),
                _ => {}
            }
        });
        
        // Solicitar repaint si hay una prueba corriendo
        if self.is_running {
            ctx.request_repaint();
        }
        // Procesar acciones pendientes de configuraciones
        if let Some(config_name) = self.pending_load_config.take() {
            if let Err(e) = self.load_config(&config_name) {
                eprintln!("Error cargando configuración: {}", e);
            }
        }
        
        if let Some(config_name) = self.pending_delete_config.take() {
            if let Err(e) = delete_config(&config_name) {
                eprintln!("Error eliminando configuración: {}", e);
            } else {
                self.refresh_configs();
            }
        }
        
        // Mostrar mensaje de guardado si corresponde
        if let Some((ref msg, instant)) = self.config_saved_message {
            if instant.elapsed().as_secs_f32() < 2.5 {
                egui::Window::new("")
                    .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 20.0))
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(false)
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::GREEN, msg);
                    });
            } else {
                self.config_saved_message = None;
            }
        }
    }
}

impl TestStressApp {
    fn render_single_test_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Prueba Individual");

            // Shortcut para cargar configuración guardada
            ui.group(|ui| {
                ui.label("Configuraciones");
                ui.horizontal(|ui| {
                    ui.label("📂 Seleccionar configuración:");
                    let mut selected = None;
                    egui::ComboBox::from_id_source("combo_single_config")
                        .selected_text("Seleccionar configuración individual...")
                        .show_ui(ui, |ui| {
                            for config in self.configs_info.iter().filter(|c| !c.is_suite) {
                                let label = format!("{} ({})", config.name, config.created_at.format("%Y-%m-%d %H:%M"));
                                if ui.selectable_label(false, label).clicked() {
                                    selected = Some(config.name.clone());
                                }
                            }
                        });
                    if let Some(name) = selected {
                        let _ = self.load_config(&name);
                    }
                });
                ui.label("💡 Tip: Selecciona una configuración guardada para cargar automáticamente todos los campos");
            });

            // Configuración general
            ui.group(|ui| {
                ui.label("Configuración General");
                ui.horizontal(|ui| {
                    ui.label("URL Base:");
                    ui.text_edit_singleline(&mut self.base_url);
                });
                ui.horizontal(|ui| {
                    ui.label("Iteraciones:");
                    ui.add(egui::DragValue::new(&mut self.iterations));
                });
                ui.horizontal(|ui| {
                    ui.label("Peticiones simultáneas:");
                    ui.add(egui::DragValue::new(&mut self.concurrent_requests));
                });
                ui.horizontal(|ui| {
                    ui.label("Tiempo de espera (seg):");
                    ui.add(egui::DragValue::new(&mut self.wait_time));
                });
                ui.horizontal(|ui| {
                    ui.label("Directorio de salida:");
                    ui.text_edit_singleline(&mut self.output_dir);
                    if ui.button("📁 Seleccionar").clicked() {
                        // Abrir selector de directorio nativo
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Seleccionar directorio de salida")
                            .pick_folder() {
                            self.output_dir = path.to_string_lossy().to_string();
                        }
                    }
                });
                if self.output_dir.trim().is_empty() {
                    ui.colored_label(egui::Color32::RED, "⚠️ Debes especificar un directorio de salida antes de ejecutar la prueba.");
                }
                ui.checkbox(&mut self.auto_generate_report, "📊 Generar reporte Excel automáticamente después de la prueba");
                
                ui.separator();
                ui.label("Subida a Carpeta Remota");
                if ui.checkbox(&mut self.auto_upload_report, "📤 Subir archivos automáticamente a carpeta remota").changed() {
                    // La opción se guarda automáticamente en la estructura
                }
                ui.label("Se copiará toda la carpeta de la prueba (con Excel y CSV) a la carpeta especificada.");
                
                ui.horizontal(|ui| {
                    ui.label("Carpeta remota:");
                    ui.text_edit_singleline(&mut self.remote_folder_path);
                    if ui.button("📁 Seleccionar").clicked() {
                        // Abrir selector de directorio nativo
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Seleccionar carpeta remota")
                            .pick_folder() {
                            self.remote_folder_path = path.to_string_lossy().to_string();
                        }
                    }
                });
                if self.auto_upload_report && self.remote_folder_path.trim().is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Debes especificar una carpeta remota para subir los archivos.");
                }
            });
            
            // Configuración de la petición y métricas del sistema lado a lado
            ui.horizontal(|ui| {
                // Configuración de la petición (izquierda) - mantener layout vertical
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label("Configuración de la Petición");
                        ui.horizontal(|ui| {
                            ui.label("Método:");
                            egui::ComboBox::from_id_source("method")
                                .selected_text(format!("{}", self.current_request.method))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::GET, "GET");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::POST, "POST");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::PUT, "PUT");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::PATCH, "PATCH");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::DELETE, "DELETE");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::HEAD, "HEAD");
                                    ui.selectable_value(&mut self.current_request.method, HttpMethod::OPTIONS, "OPTIONS");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Endpoint:");
                            ui.text_edit_singleline(&mut self.current_request.endpoint);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Descripción:");
                            ui.text_edit_singleline(&mut self.current_request.description);
                        });
                        
                        // Headers
                        ui.label("Headers:");
                        let mut to_remove_headers = Vec::new();
                        for (i, header) in self.current_request.headers.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut header.name);
                                ui.text_edit_singleline(&mut header.value);
                                if ui.button("🗑️").clicked() {
                                    to_remove_headers.push(i);
                                }
                            });
                        }
                        for &index in to_remove_headers.iter().rev() {
                            self.current_request.headers.remove(index);
                        }
                        if ui.button("➕ Agregar Header").clicked() {
                            self.current_request.headers.push(HttpHeader {
                                name: String::new(),
                                value: String::new(),
                            });
                        }
                        
                        // Query Parameters
                        ui.label("Query Parameters:");
                        let mut to_remove_params = Vec::new();
                        for (i, param) in self.current_request.query_params.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut param.name);
                                ui.text_edit_singleline(&mut param.value);
                                if ui.button("🗑️").clicked() {
                                    to_remove_params.push(i);
                                }
                            });
                        }
                        for &index in to_remove_params.iter().rev() {
                            self.current_request.query_params.remove(index);
                        }
                        if ui.button("➕ Agregar Query Parameter").clicked() {
                            self.current_request.query_params.push(QueryParameter {
                                name: String::new(),
                                value: String::new(),
                            });
                        }
                        
                        // Body
                        if matches!(self.current_request.method, HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH) {
                            ui.label("Body (JSON):");
                            if let Some(body) = &mut self.current_request.body {
                                ui.text_edit_multiline(body);
                            } else {
                                let mut temp = String::new();
                                if ui.text_edit_multiline(&mut temp).changed() {
                                    self.current_request.body = Some(temp);
                                }
                            }
                        }
                    });
                });
                
                // Panel de métricas del sistema en tiempo real (derecha)
                if self.monitoring_config.enabled {
                    ui.vertical(|ui| {
                        ui.heading("📊 Métricas del Sistema");
                        
                        ui.group(|ui| {
                            if let Some(ref mut monitor) = self.system_monitor {
                                if let Some(metrics) = monitor.get_current_metrics() {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label("🖥️ CPU");
                                            ui.colored_label(
                                                if metrics.cpu_usage > 80.0 { egui::Color32::RED }
                                                else if metrics.cpu_usage > 60.0 { egui::Color32::YELLOW }
                                                else { egui::Color32::GREEN },
                                                format_percentage(metrics.cpu_usage)
                                            );
                                        });
                                        
                                        ui.vertical(|ui| {
                                            ui.label("💾 RAM");
                                            ui.colored_label(
                                                if metrics.memory_usage > 80.0 { egui::Color32::RED }
                                                else if metrics.memory_usage > 60.0 { egui::Color32::YELLOW }
                                                else { egui::Color32::GREEN },
                                                format_percentage(metrics.memory_usage)
                                            );
                                            ui.label(format!("{}/{}", format_bytes(metrics.memory_used), format_bytes(metrics.memory_total)));
                                        });
                                        
                                        ui.vertical(|ui| {
                                            ui.label("💿 Disco I/O");
                                            ui.label(format!("📥 {}", format_bytes(metrics.disk_read_bytes)));
                                            ui.label(format!("📤 {}", format_bytes(metrics.disk_write_bytes)));
                                        });
                                        
                                        ui.vertical(|ui| {
                                            ui.label("🌐 Red");
                                            ui.label(format!("📥 {}", format_bytes(metrics.network_rx_bytes)));
                                            ui.label(format!("📤 {}", format_bytes(metrics.network_tx_bytes)));
                                        });
                                        
                                        ui.vertical(|ui| {
                                            ui.label("⚡ Load");
                                            ui.colored_label(
                                                if metrics.load_average > 2.0 { egui::Color32::RED }
                                                else if metrics.load_average > 1.0 { egui::Color32::YELLOW }
                                                else { egui::Color32::GREEN },
                                                format!("{:.2}", metrics.load_average)
                                            );
                                        });
                                    });
                                    
                                    ui.label(format!("🕐 Última actualización: {}", metrics.timestamp.format("%H:%M:%S")));
                                    
                                    // Mostrar tipo de monitoreo
                                    let monitor_type = if self.monitoring_config.monitoring_type == MonitoringType::Local {
                                        "🖥️ Monitoreo Local"
                                    } else {
                                        "🌐 Monitoreo SSH"
                                    };
                                    ui.label(monitor_type);
                                } else {
                                    ui.label("⏳ Inicializando métricas...");
                                }
                            } else {
                                ui.label("❌ Monitoreo no disponible");
                            }
                        });
                    });
                }
            });
            
            // Controles de ejecución (dentro del scroll)
            ui.horizontal(|ui| {
                if ui.button(if self.is_running { "⏸️ Pausar" } else { "▶️ Ejecutar" }).clicked() {
                    if !self.is_running {
                        if self.output_dir.trim().is_empty() {
                            self.add_log("Error: Debes especificar un directorio de salida antes de ejecutar la prueba.".to_string());
                        } else if self.show_limit_warning {
                            // Si ya hay advertencia, solo mostrar el popup, no ejecutar
                            // (el popup se muestra abajo)
                        } else if self.check_limits(self.iterations, self.concurrent_requests, self.wait_time, PendingAction::RunSingleTest) {
                            self.run_single_test();
                        }
                    }
                }
                if self.is_running {
                    if ui.add(egui::Button::new("🛑 Parar").fill(egui::Color32::RED).min_size(egui::vec2(100.0, 40.0))).clicked() {
                        self.cancel_requested = true;
                        // Activar flag de cancelación
                        if let Some(ref cancel_flag) = self.cancel_flag {
                            if let Ok(mut flag) = cancel_flag.lock() {
                                *flag = true;
                            }
                        }
                    }
                }
                if ui.button("💾 Guardar Configuración").clicked() {
                    if let Err(e) = self.save_current_config() {
                        eprintln!("Error guardando configuración: {}", e);
                    } else {
                        // ui.label("✅ Configuración guardada"); // Eliminado para evitar duplicidad
                    }
                }
            });
            
            // Progreso
            if self.is_running {
                ui.add(egui::ProgressBar::new(self.progress).text("Ejecutando..."));
            }
            
            // Logs mejorados (con scroll propio)
            ui.group(|ui| {
                ui.label("📋 Logs de Ejecución");
                if let Ok(logs) = self.logs.lock() {
                    if logs.is_empty() {
                        ui.label("No hay logs disponibles. Ejecuta una prueba para ver los logs.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(200.0) // Aumentar altura
                            .show(ui, |ui| {
                                for log in logs.iter().rev().take(20) { // Mostrar más logs
                                    ui.label(format!("{}", log));
                                }
                            });
                    }
                }
            });
        });

        // Popup de advertencia (fuera del scroll)
        if self.show_limit_warning {
            egui::Window::new("Advertencia de límite")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(&self.limit_warning_message);
                    ui.checkbox(&mut self.limit_warning_accept, "Entiendo los riesgos y deseo continuar");
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.limit_warning_accept, egui::Button::new("Continuar")).clicked() {
                            self.show_limit_warning = false;
                            if let Some(action) = self.pending_action {
                                match action {
                                    PendingAction::RunSingleTest => self.run_single_test(),
                                    PendingAction::RunSuiteTest => self.run_suite_test(),
                                }
                            }
                        }
                        if ui.button("Cancelar").clicked() {
                            self.show_limit_warning = false;
                        }
                    });
                });
        }
    }
    
    fn render_suite_test_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Suite de Pruebas");

            // Shortcut para cargar configuración guardada de suite
            ui.group(|ui| {
                ui.label("Configuraciones");
                ui.horizontal(|ui| {
                    ui.label("📂 Seleccionar suite:");
                    let mut selected = None;
                    egui::ComboBox::from_id_source("combo_suite_config")
                        .selected_text("Seleccionar configuración de suite...")
                        .show_ui(ui, |ui| {
                            for config in self.configs_info.iter().filter(|c| c.is_suite) {
                                let label = format!("{} ({})", config.name, config.created_at.format("%Y-%m-%d %H:%M"));
                                if ui.selectable_label(false, label).clicked() {
                                    selected = Some(config.name.clone());
                                }
                            }
                        });
                    if let Some(name) = selected {
                        let _ = self.load_config(&name);
                    }
                });
                ui.label("💡 Tip: Selecciona una suite guardada para cargar automáticamente todas las peticiones y configuraciones");
            });

            // Configuración de la suite
            ui.group(|ui| {
                ui.label("Configuración de la Suite");
                ui.horizontal(|ui| {
                    ui.label("Nombre de la suite:");
                    ui.text_edit_singleline(&mut self.suite_name);
                });
                ui.horizontal(|ui| {
                    ui.label("URL Base:");
                    ui.text_edit_singleline(&mut self.base_url);
                });
                ui.horizontal(|ui| {
                    ui.label("Iteraciones:");
                    ui.add(egui::DragValue::new(&mut self.iterations));
                });
                ui.horizontal(|ui| {
                    ui.label("Peticiones simultáneas:");
                    ui.add(egui::DragValue::new(&mut self.concurrent_requests));
                });
                ui.horizontal(|ui| {
                    ui.label("Tiempo de espera (seg):");
                    ui.add(egui::DragValue::new(&mut self.wait_time));
                });
                ui.horizontal(|ui| {
                    ui.label("Directorio de salida:");
                    ui.text_edit_singleline(&mut self.output_dir);
                    if ui.button("📁 Seleccionar").clicked() {
                        // Abrir selector de directorio nativo
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Seleccionar directorio de salida")
                            .pick_folder() {
                            self.output_dir = path.to_string_lossy().to_string();
                        }
                    }
                });
                if self.output_dir.trim().is_empty() {
                    ui.colored_label(egui::Color32::RED, "⚠️ Debes especificar un directorio de salida antes de ejecutar la suite.");
                }
                ui.checkbox(&mut self.auto_generate_report, "📊 Generar reporte Excel automáticamente después de la suite");
                
                ui.separator();
                ui.label("Subida a Carpeta Remota");
                if ui.checkbox(&mut self.auto_upload_report, "📤 Subir archivos automáticamente a carpeta remota").changed() {
                    // La opción se guarda automáticamente en la estructura
                }
                ui.label("Se copiará toda la carpeta de la suite (con Excel y CSV) a la carpeta especificada.");
                
                ui.horizontal(|ui| {
                    ui.label("Carpeta remota:");
                    ui.text_edit_singleline(&mut self.remote_folder_path);
                    if ui.button("📁 Seleccionar").clicked() {
                        // Abrir selector de directorio nativo
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Seleccionar carpeta remota")
                            .pick_folder() {
                            self.remote_folder_path = path.to_string_lossy().to_string();
                        }
                    }
                });
                if self.auto_upload_report && self.remote_folder_path.trim().is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Debes especificar una carpeta remota para subir los archivos.");
                }
            });
            
            // Lista de peticiones
            ui.group(|ui| {
                ui.label("Peticiones de la Suite");
                let mut to_remove = Vec::new();
                for (i, request) in self.suite_requests.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("Petición {}", i + 1));
                            if ui.button("🗑️").clicked() {
                                to_remove.push(i);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Método:");
                            egui::ComboBox::from_id_source(format!("method_{}", i))
                                .selected_text(format!("{}", request.method))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut request.method, HttpMethod::GET, "GET");
                                    ui.selectable_value(&mut request.method, HttpMethod::POST, "POST");
                                    ui.selectable_value(&mut request.method, HttpMethod::PUT, "PUT");
                                    ui.selectable_value(&mut request.method, HttpMethod::PATCH, "PATCH");
                                    ui.selectable_value(&mut request.method, HttpMethod::DELETE, "DELETE");
                                    ui.selectable_value(&mut request.method, HttpMethod::HEAD, "HEAD");
                                    ui.selectable_value(&mut request.method, HttpMethod::OPTIONS, "OPTIONS");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Endpoint:");
                            ui.text_edit_singleline(&mut request.endpoint);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Descripción:");
                            ui.text_edit_singleline(&mut request.description);
                        });
                    });
                }
                for &index in to_remove.iter().rev() {
                    self.suite_requests.remove(index);
                }
                if ui.button("➕ Agregar Petición").clicked() {
                    self.suite_requests.push(TestRequest::default());
                }
            });
            
            // Controles de ejecución (dentro del scroll)
            ui.horizontal(|ui| {
                if ui.button(if self.is_running { "⏸️ Pausar" } else { "▶️ Ejecutar Suite" }).clicked() {
                    if !self.is_running {
                        if self.output_dir.trim().is_empty() {
                            self.add_log("Error: Debes especificar un directorio de salida antes de ejecutar la suite.".to_string());
                        } else if self.check_limits(self.iterations, self.concurrent_requests, self.wait_time, PendingAction::RunSuiteTest) {
                            self.run_suite_test();
                        }
                    }
                }
                if self.is_running {
                    if ui.add(egui::Button::new("🛑 Parar").fill(egui::Color32::RED).min_size(egui::vec2(100.0, 40.0))).clicked() {
                        self.cancel_requested = true;
                        // Activar flag de cancelación
                        if let Some(ref cancel_flag) = self.cancel_flag {
                            if let Ok(mut flag) = cancel_flag.lock() {
                                *flag = true;
                            }
                        }
                    }
                }
                if ui.button("💾 Guardar Suite").clicked() {
                    if let Err(e) = self.save_current_config() {
                        eprintln!("Error guardando suite: {}", e);
                    } else {
                        // ui.label("✅ Configuración guardada"); // Eliminado para evitar duplicidad
                    }
                }
            });
            
            // Progreso
            if self.is_running {
                ui.add(egui::ProgressBar::new(self.progress).text("Ejecutando suite..."));
            }
            
            // Logs mejorados para suite (con scroll propio)
            ui.group(|ui| {
                ui.label("📋 Logs de Ejecución");
                if let Ok(logs) = self.logs.lock() {
                    if logs.is_empty() {
                        ui.label("No hay logs disponibles. Ejecuta una suite para ver los logs.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(200.0) // Aumentar altura
                            .show(ui, |ui| {
                                for log in logs.iter().rev().take(20) { // Mostrar más logs
                                    ui.label(format!("{}", log));
                                }
                            });
                    }
                }
            });
            
            // Panel de métricas del sistema en tiempo real
            if self.monitoring_config.enabled {
                ui.heading("📊 Métricas del Sistema");
                
                ui.group(|ui| {
                    if let Some(ref mut monitor) = self.system_monitor {
                        if let Some(metrics) = monitor.get_current_metrics() {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("🖥️ CPU");
                                    ui.colored_label(
                                        if metrics.cpu_usage > 80.0 { egui::Color32::RED }
                                        else if metrics.cpu_usage > 60.0 { egui::Color32::YELLOW }
                                        else { egui::Color32::GREEN },
                                        format_percentage(metrics.cpu_usage)
                                    );
                                });
                                
                                ui.vertical(|ui| {
                                    ui.label("💾 RAM");
                                    ui.colored_label(
                                        if metrics.memory_usage > 80.0 { egui::Color32::RED }
                                        else if metrics.memory_usage > 60.0 { egui::Color32::YELLOW }
                                        else { egui::Color32::GREEN },
                                        format_percentage(metrics.memory_usage)
                                    );
                                    ui.label(format!("{}/{}", format_bytes(metrics.memory_used), format_bytes(metrics.memory_total)));
                                });
                                
                                ui.vertical(|ui| {
                                    ui.label("💿 Disco I/O");
                                    ui.label(format!("📥 {}", format_bytes(metrics.disk_read_bytes)));
                                    ui.label(format!("📤 {}", format_bytes(metrics.disk_write_bytes)));
                                });
                                
                                ui.vertical(|ui| {
                                    ui.label("🌐 Red");
                                    ui.label(format!("📥 {}", format_bytes(metrics.network_rx_bytes)));
                                    ui.label(format!("📤 {}", format_bytes(metrics.network_tx_bytes)));
                                });
                                
                                ui.vertical(|ui| {
                                    ui.label("⚡ Load");
                                    ui.colored_label(
                                        if metrics.load_average > 2.0 { egui::Color32::RED }
                                        else if metrics.load_average > 1.0 { egui::Color32::YELLOW }
                                        else { egui::Color32::GREEN },
                                        format!("{:.2}", metrics.load_average)
                                    );
                                });
                            });
                            
                            ui.label(format!("🕐 Última actualización: {}", metrics.timestamp.format("%H:%M:%S")));
                            
                            // Mostrar tipo de monitoreo
                            let monitor_type = if self.monitoring_config.monitoring_type == MonitoringType::Local {
                                "🖥️ Monitoreo Local"
                            } else {
                                "🌐 Monitoreo SSH"
                            };
                            ui.label(monitor_type);
                        } else {
                            ui.label("⏳ Inicializando métricas...");
                        }
                    } else {
                        ui.label("❌ Monitoreo no disponible");
                    }
                });
            }
        });

        // Popup de advertencia (fuera del scroll)
        if self.show_limit_warning {
            egui::Window::new("Advertencia de límite")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(&self.limit_warning_message);
                    ui.checkbox(&mut self.limit_warning_accept, "Entiendo los riesgos y deseo continuar");
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.limit_warning_accept, egui::Button::new("Continuar")).clicked() {
                            self.show_limit_warning = false;
                            if let Some(action) = self.pending_action {
                                match action {
                                    PendingAction::RunSingleTest => self.run_single_test(),
                                    PendingAction::RunSuiteTest => self.run_suite_test(),
                                }
                            }
                        }
                        if ui.button("Cancelar").clicked() {
                            self.show_limit_warning = false;
                        }
                    });
                });
        }
    }
    
    fn load_config_by_name(&mut self, name: &str) {
        if let Err(e) = self.load_config(name) {
            eprintln!("Error cargando configuración: {}", e);
        }
    }
    
    fn delete_config_by_name(&mut self, name: &str) {
        if let Err(e) = delete_config(name) {
            eprintln!("Error eliminando configuración: {}", e);
        } else {
            self.refresh_configs();
        }
    }

    fn render_configs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuraciones Guardadas");
        
        // Barra de herramientas
        ui.horizontal(|ui| {
            if ui.button("🔄 Actualizar").clicked() {
                self.refresh_configs();
            }
            if ui.button("📁 Abrir Explorador").clicked() {
                #[cfg(target_os = "windows")]
                let config_dir = std::path::PathBuf::from("./configs");
                #[cfg(not(target_os = "windows"))]
                let config_dir = dirs::home_dir()
                    .map(|h| h.join(".stress/configs"))
                    .unwrap_or_else(|| std::path::PathBuf::from("./configs"));
                // Crear la carpeta si no existe
                if let Err(e) = std::fs::create_dir_all(&config_dir) {
                    eprintln!("No se pudo crear la carpeta de configuraciones: {}", e);
                } else {
                    if let Err(e) = open::that(config_dir) {
                        eprintln!("Error abriendo explorador: {}", e);
                    }
                }
            }
        });
        
        if self.configs_info.is_empty() {
            ui.label("No hay configuraciones guardadas.");
            ui.label("Las configuraciones se guardan en la carpeta './configs/'");
        } else {
            ui.label(format!("Configuraciones encontradas: {}", self.configs_info.len()));
            
            // Mostrar configuraciones
            for config_info in &self.configs_info {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Icono según tipo
                        let icon = if config_info.is_suite { "📋" } else { "📄" };
                        ui.label(icon);
                        
                        // Información principal
                        ui.vertical(|ui| {
                            ui.label(format!("📝 {}", config_info.name));
                            if let Some(desc) = &config_info.description {
                                ui.colored_label(egui::Color32::GRAY, desc);
                            }
                            ui.horizontal(|ui| {
                                ui.label(format!("📊 {} endpoints", config_info.request_count));
                                ui.label(format!("📅 {}", config_info.created_at.format("%Y-%m-%d %H:%M")));
                            });
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let config_name = config_info.name.clone();
                            if ui.button("📂 Cargar").clicked() {
                                self.pending_load_config = Some(config_name.clone());
                            }
                            if ui.button("🗑️ Eliminar").clicked() {
                                self.pending_delete_config = Some(config_name.clone());
                            }
                        });
                    });
                });
            }
        }
    }
    
    fn render_results_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Resultados");
        
        if let Ok(results) = self.results.lock() {
            if results.is_empty() {
                ui.label("No hay resultados disponibles.");
            } else {
                for (i, summary) in results.iter().enumerate() {
                    ui.group(|ui| {
                        // Usar la hora fija del timestamp del test en lugar de la hora actual
                        let test_time = summary.timestamp.format("%H:%M:%S");
                        ui.label(format!("📊 Resultado {}: {} - {}", i + 1, summary.request_name, test_time));
                        ui.separator();
                        
                        // Estadísticas generales
                        ui.label(format!("📈 Total de peticiones: {}", summary.total_requests));
                        ui.label(format!("✅ Exitosas: {}", summary.successful_requests));
                        ui.label(format!("❌ Fallidas: {}", summary.failed_requests));
                        ui.label(format!("📊 Tasa de éxito: {:.2}%", summary.success_rate));
                        
                        ui.separator();
                        
                        // Tiempos
                        ui.label("⏱️ Tiempos de respuesta:");
                        ui.label(format!("   • Promedio: {:.2} ms", summary.average_duration_ms));
                        ui.label(format!("   • Mínimo: {} ms", summary.min_duration_ms));
                        ui.label(format!("   • Máximo: {} ms", summary.max_duration_ms));
                        
                        // Duración total
                        ui.label(format!("🕐 Duración total: {:.2} ms", summary.total_duration_ms));
                    });
                }
            }
        }
        
        // Mensaje de éxito del reporte
        if let Some(msg) = &self.report_success_message {
            ui.colored_label(egui::Color32::GREEN, msg);
        }
        ui.horizontal(|ui| {
            if ui.button("📊 Generar Reporte").clicked() {
                let output_dir = self.output_dir.clone();
                
                // Encontrar todos los archivos CSV en el directorio
                let mut csv_files = Vec::new();
                if let Ok(entries) = fs::read_dir(&output_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext == "csv" {
                                    csv_files.push(path);
                                }
                            }
                        }
                    }
                }
                
                if csv_files.is_empty() {
                    self.report_success_message = Some("No se encontraron archivos CSV en el directorio de resultados".to_string());
                } else {
                    // Crear carpeta de reportes
                    let reports_dir = format!("{}/reports", output_dir);
                    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                    let excel_path = format!("{}/report_{}.xlsx", reports_dir, timestamp);
                    
                    match generate_excel_report_from_files(&csv_files, &excel_path) {
                        Ok(path) => {
                            self.report_success_message = Some(format!("Reporte Excel generado exitosamente en: {}", path));
                        },
                        Err(e) => {
                            self.report_success_message = Some(format!("Error generando reporte: {}", e));
                        }
                    }
                }
            }
        });
    }

    fn render_general_options_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Opciones Generales");
            ui.separator();
            let mut changed = false;
            let mut close_failed = false;
            if ui.checkbox(&mut self.show_terminal, "Mostrar terminal (logs en tiempo real)").changed() {
                changed = true;
            }
            ui.label("Por defecto, la terminal está oculta. Si activas esta opción, se abrirá una terminal con logs en tiempo real. Si la desactivas, se cerrará la terminal si es posible.");
            if changed {
                let prefs = GeneralPrefs { 
                    show_terminal: self.show_terminal,
                    monitoring_enabled: self.monitoring_config.enabled,
                    monitoring_type: self.monitoring_config.monitoring_type.clone(),
                };
                save_general_prefs(&prefs);
                if self.show_terminal {
                    self.open_terminal();
                } else {
                    if !self.close_terminal() {
                        close_failed = true;
                    }
                }
            }
            if !self.show_terminal && self.terminal_child.is_some() {
                // Intentar cerrar si aún queda abierta
                if !self.close_terminal() {
                    close_failed = true;
                }
            }
            if close_failed {
                ui.colored_label(egui::Color32::YELLOW, "No se pudo cerrar la terminal automáticamente. Ciérrala manualmente si es necesario.");
            }
            
            ui.separator();
            ui.heading("Reportes y Archivos");
            
            if ui.checkbox(&mut self.auto_generate_report, "Generar reporte Excel automáticamente").changed() {
                // La opción se guarda automáticamente en la estructura
            }
            ui.label("Si está activado, se generará automáticamente un reporte Excel después de cada prueba.");
            
            ui.separator();
            ui.heading("Subida Automática a Carpeta Remota");
            
            if ui.checkbox(&mut self.auto_upload_report, "Subir archivos automáticamente a carpeta remota").changed() {
                // La opción se guarda automáticamente en la estructura
            }
            ui.label("Si está activado, se copiará toda la carpeta de la prueba (con Excel y CSV) a la carpeta remota especificada.");
            
            ui.label("Ruta de la carpeta remota (OneDrive, Google Drive, Dropbox, etc.):");
            ui.text_edit_singleline(&mut self.remote_folder_path);
            ui.label("Ejemplo: /Users/tu_usuario/OneDrive/StressTests o C:\\Users\\tu_usuario\\OneDrive\\StressTests");
            
            ui.separator();
            ui.heading("Monitoreo del Sistema");
            
            // Tipo de monitoreo
            let mut monitoring_type_changed = false;
            ui.horizontal(|ui| {
                ui.label("Tipo de monitoreo:");
                if ui.radio_value(&mut self.monitoring_config.monitoring_type, MonitoringType::Local, "🖥️ Local").changed() {
                    monitoring_type_changed = true;
                }
                if ui.radio_value(&mut self.monitoring_config.monitoring_type, MonitoringType::SSH, "🌐 Remoto (SSH)").changed() {
                    monitoring_type_changed = true;
                }
            });
            
            // Guardar automáticamente cuando se cambia el tipo de monitoreo
            if monitoring_type_changed {
                let prefs = GeneralPrefs { 
                    show_terminal: self.show_terminal,
                    monitoring_enabled: self.monitoring_config.enabled,
                    monitoring_type: self.monitoring_config.monitoring_type.clone(),
                };
                save_general_prefs(&prefs);
            }
            
            // Configuración SSH si está seleccionado
            if self.monitoring_config.monitoring_type == MonitoringType::SSH {
                ui.group(|ui| {
                    ui.label("Configuración SSH");
                    
                    ui.horizontal(|ui| {
                        ui.label("Host/IP:");
                        ui.text_edit_singleline(&mut self.monitoring_config.ssh_config.host);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Usuario:");
                        ui.text_edit_singleline(&mut self.monitoring_config.ssh_config.username);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Contraseña:");
                        ui.text_edit_singleline(&mut self.monitoring_config.ssh_config.password);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Puerto:");
                        ui.add(egui::DragValue::new(&mut self.monitoring_config.ssh_config.port).clamp_range(1..=65535));
                    });
                    
                    ui.checkbox(&mut self.monitoring_config.ssh_config.save_credentials, "💾 Guardar credenciales (encriptadas)");
                    
                    ui.horizontal(|ui| {
                        if ui.button("🔗 Probar conexión").clicked() {
                            // TODO: Implementar prueba de conexión SSH
                            ui.label("✅ Conexión exitosa");
                        }
                    });
                });
            }
            
            if ui.checkbox(&mut self.monitoring_config.enabled, "Habilitar monitoreo del sistema").changed() {
                if self.monitoring_config.enabled {
                    // Inicializar el monitor si no existe
                    if self.system_monitor.is_none() {
                        self.system_monitor = Some(SystemMonitor::new(self.monitoring_config.clone()));
                    }
                    // Iniciar monitoreo
                    if let Some(ref mut monitor) = self.system_monitor {
                        monitor.start_monitoring();
                    }
                    
                    // Guardar configuración SSH si está habilitado
                    if self.monitoring_config.monitoring_type == MonitoringType::SSH {
                        if let Err(e) = save_ssh_config(&self.monitoring_config.ssh_config) {
                            eprintln!("Error guardando configuración SSH: {}", e);
                        }
                    }
                } else {
                    // Detener monitoreo
                    if let Some(ref mut monitor) = self.system_monitor {
                        monitor.stop_monitoring();
                    }
                }
                
                // Guardar automáticamente la configuración
                let prefs = GeneralPrefs { 
                    show_terminal: self.show_terminal,
                    monitoring_enabled: self.monitoring_config.enabled,
                    monitoring_type: self.monitoring_config.monitoring_type.clone(),
                };
                save_general_prefs(&prefs);
            }
            ui.label("Si está activado, se mostrarán las métricas del sistema en tiempo real durante las pruebas.");
            
            ui.horizontal(|ui| {
                ui.label("Intervalo de actualización (ms):");
                ui.add(egui::DragValue::new(&mut self.monitoring_config.interval_ms).clamp_range(500..=5000));
            });
            
            ui.horizontal(|ui| {
                ui.label("Historial máximo:");
                ui.add(egui::DragValue::new(&mut self.monitoring_config.max_history).clamp_range(60..=600));
            });
            ui.label("Cantidad de muestras a mantener en memoria (60 = 1 minuto, 300 = 5 minutos)");
            
            ui.separator();
            ui.label("Métricas a monitorear:");
            
            ui.checkbox(&mut self.monitoring_config.monitor_cpu, "🖥️ CPU");
            ui.checkbox(&mut self.monitoring_config.monitor_memory, "💾 Memoria RAM");
            ui.checkbox(&mut self.monitoring_config.monitor_disk, "💿 Disco I/O");
            ui.checkbox(&mut self.monitoring_config.monitor_network, "🌐 Red");
            
            // Actualizar configuración del monitor si existe
            if let Some(ref mut monitor) = self.system_monitor {
                monitor.update_config(self.monitoring_config.clone());
            }
        });
    }
}

// Función auxiliar para buscar archivos CSV en un directorio
fn find_csv_file_in_directory(dir_path: &str, test_name: &str) -> Option<PathBuf> {
    use std::fs;
    use std::path::Path;
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "csv") {
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy();
                        let safe_test_name = test_name.replace(" ", "_");
                        if file_name_str.contains(&safe_test_name) {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    None
}