use crate::config::{ensure_output_directory, get_output_directory, save_config, load_config, list_saved_configs, delete_config};
use crate::load_test::LoadTester;
use crate::models::{TestRequest, TestSuite, SavedConfig, TestSummary, HttpMethod, HttpHeader, QueryParameter};
use crate::report_generator::generate_excel_report;
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
        };
        // Solo abrir terminal si la preferencia está activa y no hay terminal abierta
        if app.show_terminal {
            app.open_terminal();
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
                if let Err(e) = Self::execute_suite_test_with_progress_and_cancel(&suite, logs, results, progress_sender, cancel_flag_clone).await {
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
        
        Ok(())
    }
    
    async fn execute_suite_test_with_progress_and_cancel(
        suite: &TestSuite,
        logs: Arc<Mutex<Vec<String>>>,
        results: Arc<Mutex<Vec<TestSummary>>>,
        progress_sender: mpsc::Sender<f32>,
        cancel_flag: Arc<Mutex<bool>>,
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
        corrected_suite.output_dir = final_output_dir;
        
        // Crear tester y ejecutar suite con cancelación
        let tester = LoadTester::new();
        let summaries = tester.run_suite_test_with_progress_and_cancel(&corrected_suite, progress_sender, cancel_flag).await?;
        
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
            }
        } else {
            // Guardar suite de pruebas
            SavedConfig {
                name: self.suite_name.clone(),
                base_url: self.base_url.clone(),
                requests: self.suite_requests.clone(),
            }
        };
        save_config(&config)?;
        println!("Configuración guardada exitosamente: {}", config.name);
        self.refresh_configs();
        Ok(())
    }
    
    fn load_config(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = load_config(name)?;
        
        // Cargar la configuración en la pestaña apropiada
        self.base_url = config.base_url;
        
        if config.requests.len() == 1 {
            // Es una petición individual, cargar en pestaña 0
            self.current_tab = 0;
            self.current_request = config.requests[0].clone();
        } else {
            // Es una suite, cargar en pestaña 1
            self.current_tab = 1;
            self.suite_name = config.name.clone();
            self.suite_requests = config.requests.clone();
        }
        
        Ok(())
    }
    
    fn refresh_configs(&mut self) {
        self.saved_configs = list_saved_configs().unwrap_or_default();
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
    }
}

impl TestStressApp {
    fn render_single_test_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Prueba Individual");
        
        // Configuración general
        ui.group(|ui| {
            ui.label("Configuración General");
            ui.horizontal(|ui| {
                ui.label("URL Base:");
                ui.text_edit_singleline(&mut self.base_url);
            });
            ui.horizontal(|ui| {
                ui.label("Iteraciones:");
                ui.add(egui::DragValue::new(&mut self.iterations).clamp_range(1..=10000));
            });
            ui.horizontal(|ui| {
                ui.label("Peticiones simultáneas:");
                ui.add(egui::DragValue::new(&mut self.concurrent_requests).clamp_range(1..=100));
            });
            ui.horizontal(|ui| {
                ui.label("Tiempo de espera (seg):");
                ui.add(egui::DragValue::new(&mut self.wait_time).clamp_range(0..=60));
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
        });
        
        // Configuración de la petición
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
                let mut remove = false;
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut header.name);
                    ui.label(":");
                    ui.text_edit_singleline(&mut header.value);
                    if ui.button("❌").clicked() {
                        remove = true;
                    }
                });
                if remove {
                    to_remove_headers.push(i);
                }
            }
            for &i in to_remove_headers.iter().rev() {
                self.current_request.headers.remove(i);
            }
            if ui.button("+ Agregar Header").clicked() {
                self.current_request.headers.push(HttpHeader {
                    name: String::new(),
                    value: String::new(),
                });
            }
            
            // Query parameters
            ui.label("Query Parameters:");
            let mut to_remove_query = Vec::new();
            for (i, param) in self.current_request.query_params.iter_mut().enumerate() {
                let mut remove = false;
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut param.name);
                    ui.label("=");
                    ui.text_edit_singleline(&mut param.value);
                    if ui.button("❌").clicked() {
                        remove = true;
                    }
                });
                if remove {
                    to_remove_query.push(i);
                }
            }
            for &i in to_remove_query.iter().rev() {
                self.current_request.query_params.remove(i);
            }
            if ui.button("+ Agregar Query Param").clicked() {
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
        
        // Controles
        ui.horizontal(|ui| {
            if ui.button(if self.is_running { "⏸️ Pausar" } else { "▶️ Ejecutar" }).clicked() {
                if !self.is_running {
                    if self.output_dir.trim().is_empty() {
                        self.add_log("Error: Debes especificar un directorio de salida antes de ejecutar la prueba.".to_string());
                    } else {
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
                    ui.label("✅ Configuración guardada");
                }
            }
        });
        
        // Progreso
        if self.is_running {
            ui.add(egui::ProgressBar::new(self.progress).text("Ejecutando..."));
        }
        
        // Logs mejorados
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
    }
    
    fn render_suite_test_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Suite de Pruebas");
        
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
                ui.add(egui::DragValue::new(&mut self.iterations).clamp_range(1..=10000));
            });
            ui.horizontal(|ui| {
                ui.label("Peticiones simultáneas:");
                ui.add(egui::DragValue::new(&mut self.concurrent_requests).clamp_range(1..=100));
            });
            ui.horizontal(|ui| {
                ui.label("Tiempo de espera (seg):");
                ui.add(egui::DragValue::new(&mut self.wait_time).clamp_range(0..=60));
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
        });
        
        // Lista de peticiones
        ui.group(|ui| {
            ui.label("Peticiones en la Suite");
            let mut to_remove = Vec::new();
            for (i, request) in self.suite_requests.iter_mut().enumerate() {
                let mut remove = false;
                ui.horizontal(|ui| {
                    ui.label(format!("{}. {} {} - {}", i + 1, request.method, request.endpoint, request.description));
                    if ui.button("❌").clicked() {
                        remove = true;
                    }
                });
                if remove {
                    to_remove.push(i);
                }
            }
            // Eliminar fuera del bucle
            for &i in to_remove.iter().rev() {
                self.suite_requests.remove(i);
            }
            if ui.button("+ Agregar Petición").clicked() {
                self.suite_requests.push(TestRequest::default());
            }
        });
        
        // Controles
        ui.horizontal(|ui| {
            if ui.button(if self.is_running { "⏸️ Pausar" } else { "▶️ Ejecutar Suite" }).clicked() {
                if !self.is_running {
                    if self.output_dir.trim().is_empty() {
                        self.add_log("Error: Debes especificar un directorio de salida antes de ejecutar la suite.".to_string());
                    } else {
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
                }
            }
        });
        
        // Progreso
        if self.is_running {
            ui.add(egui::ProgressBar::new(self.progress).text("Ejecutando suite..."));
        }
        
        // Logs mejorados para suite
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
    }
    
    fn render_configs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuraciones Guardadas");
        
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
        
        if self.saved_configs.is_empty() {
            ui.label("No hay configuraciones guardadas.");
            ui.label("Las configuraciones se guardan en la carpeta './configs/'");
        } else {
            ui.label(format!("Configuraciones encontradas: {}", self.saved_configs.len()));
            for config_name in self.saved_configs.clone() {
                ui.horizontal(|ui| {
                    ui.label(&config_name);
                    if ui.button("📂 Cargar").clicked() {
                        if let Err(e) = self.load_config(&config_name) {
                            eprintln!("Error cargando configuración: {}", e);
                        } else {
                            ui.label("✅ Cargada");
                        }
                    }
                    if ui.button("🗑️ Eliminar").clicked() {
                        if let Err(e) = delete_config(&config_name) {
                            eprintln!("Error eliminando configuración: {}", e);
                        } else {
                            self.refresh_configs();
                        }
                    }
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
        
        ui.horizontal(|ui| {
            if ui.button("📊 Generar Reporte").clicked() {
                // TODO: Implementar generación de reporte
            }
        });
    }

    fn render_general_options_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Opciones Generales");
        ui.separator();
        let mut changed = false;
        let mut close_failed = false;
        if ui.checkbox(&mut self.show_terminal, "Mostrar terminal (logs en tiempo real)").changed() {
            changed = true;
        }
        ui.label("Por defecto, la terminal está oculta. Si activas esta opción, se abrirá una terminal con logs en tiempo real. Si la desactivas, se cerrará la terminal si es posible.");
        if changed {
            let prefs = GeneralPrefs { show_terminal: self.show_terminal };
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
    }
} 