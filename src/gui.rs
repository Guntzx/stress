// gui.rs — Capa de interfaz gráfica basada en Slint
// Mantiene toda la lógica de negocio intacta; sólo reemplaza el backend de UI.

use crate::config::{
    get_output_directory, save_config, load_config,
    list_saved_configs, delete_config, list_configs_with_info,
};
use crate::load_test::LoadTester;
use crate::models::{HttpHeader, HttpMethod, SavedConfig, TestRequest, TestSuite, TestSummary};
use crate::report_generator::generate_excel_report_from_files;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use slint::{ModelRc, SharedString, VecModel};

// Genera los tipos Rust desde los archivos .slint compilados por build.rs
slint::include_modules!();

// ── Estado interno de la aplicación (accedido sólo desde el hilo principal) ──
struct AppState {
    // Resultados y logs compartidos con hilos de ejecución
    results:         Arc<Mutex<Vec<TestSummary>>>,
    logs:            Arc<Mutex<Vec<String>>>,

    // Control de ejecución en curso
    is_running:       bool,
    cancel_flag:      Option<Arc<Mutex<bool>>>,
    completion_rx:    Option<mpsc::Receiver<()>>,
    progress_rx:      Option<mpsc::Receiver<f32>>,

    // Lista local de peticiones de la suite (sólo hilo UI)
    suite_requests:   Vec<TestRequest>,

    // Caché para detectar cuando hay nuevos resultados
    last_result_count: usize,
}

impl AppState {
    fn new() -> Self {
        Self {
            results:           Arc::new(Mutex::new(Vec::new())),
            logs:              Arc::new(Mutex::new(Vec::new())),
            is_running:        false,
            cancel_flag:       None,
            completion_rx:     None,
            progress_rx:       None,
            suite_requests:    Vec::new(),
            last_result_count: 0,
        }
    }
}

// ── Punto de entrada público ──────────────────────────────────────────────────
pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    let window = AppWindow::new()?;
    let state  = Rc::new(RefCell::new(AppState::new()));

    // ── Carga inicial de datos en la UI ───────────────────────────────────────
    populate_initial_data(&window);

    // ─────────────────────────────────────────────────────────────────────────
    // CALLBACKS — Tab 0: Prueba Individual
    // ─────────────────────────────────────────────────────────────────────────

    // Ejecutar prueba individual
    window.on_ejecutar_individual({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st  = state.borrow_mut();
            if st.is_running { return; }

            let request  = build_request_from_ui(&w);
            let base_url = w.get_url_base().to_string();
            let iters    = parse_u32(&w.get_iteraciones(), 10);
            let conc     = parse_u32(&w.get_peticiones_simultaneas(), 1);
            let wait     = 1u64;
            let out_dir  = w.get_dir_salida().to_string();
            let excel    = w.get_auto_excel();
            let upload   = w.get_auto_subir();
            let remote   = w.get_carpeta_remota().to_string();

            let logs    = st.logs.clone();
            let results = st.results.clone();

            let (done_tx, done_rx)   = mpsc::channel();
            let (prog_tx, prog_rx)   = mpsc::channel();
            let cancel               = Arc::new(Mutex::new(false));
            let cancel_clone         = cancel.clone();

            st.is_running     = true;
            st.cancel_flag    = Some(cancel);
            st.completion_rx  = Some(done_rx);
            st.progress_rx    = Some(prog_rx);
            drop(st);

            w.set_ejecutando(true);
            w.set_estado("RUNNING".into());
            w.set_barra_progreso("░░░░░░░░░░ 0%".into());

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Runtime Tokio");
                rt.block_on(async move {
                    if let Err(e) = execute_single_test(
                        &request, &base_url, iters, conc, wait,
                        &out_dir, logs, results, prog_tx,
                        cancel_clone, excel, upload, remote,
                    ).await {
                        eprintln!("[error] prueba individual: {e}");
                    }
                    let _ = done_tx.send(());
                });
            });
        }
    });

    // Cancelar ejecución en curso
    window.on_cancelar({
        let state = state.clone();
        move || {
            let mut st = state.borrow_mut();
            if let Some(ref flag) = st.cancel_flag {
                if let Ok(mut f) = flag.lock() {
                    *f = true;
                }
            }
            st.is_running = false;
        }
    });

    // Seleccionar directorio de salida (diálogo nativo)
    window.on_seleccionar_dir_salida({
        let window_weak = window.as_weak();
        move || {
            let window_weak = window_weak.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.to_string_lossy().to_string();
                    slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_dir_salida(path_str.into());
                        }
                    }).ok();
                }
            });
        }
    });

    // Seleccionar carpeta remota (diálogo nativo)
    window.on_seleccionar_dir_remota({
        let window_weak = window.as_weak();
        move || {
            let window_weak = window_weak.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.to_string_lossy().to_string();
                    slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_carpeta_remota(path_str.into());
                        }
                    }).ok();
                }
            });
        }
    });

    // Cargar configuración guardada en la UI
    window.on_cargar_config({
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let configs = list_saved_configs().unwrap_or_default();
            let Some(name) = configs.get(idx as usize) else { return };
            if let Ok(cfg) = load_config(name) {
                apply_config_to_ui(&w, &cfg);
            }
        }
    });

    // Guardar configuración actual
    window.on_guardar_config({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let st = state.borrow();
            let tab = w.get_tab_activo();

            let config = if tab == 0 {
                // Petición individual
                let req = build_request_from_ui(&w);
                let desc = req.description.clone();
                SavedConfig {
                    name: format!("{} - {}", desc, chrono::Local::now().format("%Y%m%d_%H%M%S")),
                    base_url: w.get_url_base().to_string(),
                    requests: vec![req],
                    iterations: parse_u32(&w.get_iteraciones(), 10),
                    concurrent_requests: parse_u32(&w.get_peticiones_simultaneas(), 1),
                    wait_time: 1,
                    output_dir: w.get_dir_salida().to_string(),
                    auto_generate_report: w.get_auto_excel(),
                    auto_upload_report: w.get_auto_subir(),
                    remote_folder_path: w.get_carpeta_remota().to_string(),
                    created_at: chrono::Local::now(),
                    description: Some(format!("Petición: {}", desc)),
                }
            } else {
                // Suite de pruebas
                let nombre = w.get_suite_nombre().to_string();
                let n = st.suite_requests.len();
                SavedConfig {
                    name: nombre.clone(),
                    base_url: w.get_url_base().to_string(),
                    requests: st.suite_requests.clone(),
                    iterations: parse_u32(&w.get_iteraciones(), 10),
                    concurrent_requests: parse_u32(&w.get_peticiones_simultaneas(), 1),
                    wait_time: 1,
                    output_dir: w.get_dir_salida().to_string(),
                    auto_generate_report: w.get_auto_excel(),
                    auto_upload_report: w.get_auto_subir(),
                    remote_folder_path: w.get_carpeta_remota().to_string(),
                    created_at: chrono::Local::now(),
                    description: Some(format!("Suite con {n} peticiones")),
                }
            };

            drop(st);
            if let Err(e) = save_config(&config) {
                eprintln!("[error] guardar config: {e}");
            } else {
                refresh_configs_in_ui(&w);
            }
        }
    });

    // ─────────────────────────────────────────────────────────────────────────
    // CALLBACKS — Tab 1: Suite de Pruebas
    // ─────────────────────────────────────────────────────────────────────────

    // Agregar nueva petición vacía a la suite
    window.on_suite_agregar({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let req = TestRequest::default();
            st.suite_requests.push(req);
            sync_suite_to_ui(&w, &st.suite_requests);
        }
    });

    // Eliminar petición seleccionada de la suite
    window.on_suite_eliminar({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let i = idx as usize;
            if i < st.suite_requests.len() {
                st.suite_requests.remove(i);
                let new_sel = if st.suite_requests.is_empty() { -1 }
                    else { (i as i32 - 1).max(0) };
                drop(st);
                w.set_suite_seleccionado(new_sel);
                let st2 = state.borrow();
                sync_suite_to_ui(&w, &st2.suite_requests);
            }
        }
    });

    // Mover petición hacia arriba en la suite
    window.on_suite_mover_arriba({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let i = idx as usize;
            if i > 0 && i < st.suite_requests.len() {
                st.suite_requests.swap(i - 1, i);
                drop(st);
                w.set_suite_seleccionado(idx - 1);
                let st2 = state.borrow();
                sync_suite_to_ui(&w, &st2.suite_requests);
            }
        }
    });

    // Mover petición hacia abajo en la suite
    window.on_suite_mover_abajo({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let i = idx as usize;
            if i + 1 < st.suite_requests.len() {
                st.suite_requests.swap(i, i + 1);
                drop(st);
                w.set_suite_seleccionado(idx + 1);
                let st2 = state.borrow();
                sync_suite_to_ui(&w, &st2.suite_requests);
            }
        }
    });

    // Ejecutar suite de pruebas
    window.on_ejecutar_suite({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            if st.is_running { return; }
            if st.suite_requests.is_empty() {
                eprintln!("[warn] suite vacía");
                return;
            }

            let suite = TestSuite {
                name:                w.get_suite_nombre().to_string(),
                base_url:            w.get_url_base().to_string(),
                requests:            st.suite_requests.clone(),
                iterations:          parse_u32(&w.get_iteraciones(), 10),
                concurrent_requests: parse_u32(&w.get_peticiones_simultaneas(), 1),
                wait_time:           1,
                output_dir:          w.get_dir_salida().to_string(),
            };
            let excel  = w.get_auto_excel();
            let upload = w.get_auto_subir();
            let remote = w.get_carpeta_remota().to_string();

            let logs    = st.logs.clone();
            let results = st.results.clone();

            let (done_tx, done_rx) = mpsc::channel();
            let (prog_tx, prog_rx) = mpsc::channel();
            let cancel             = Arc::new(Mutex::new(false));
            let cancel_clone       = cancel.clone();

            st.is_running    = true;
            st.cancel_flag   = Some(cancel);
            st.completion_rx = Some(done_rx);
            st.progress_rx   = Some(prog_rx);
            drop(st);

            w.set_ejecutando(true);
            w.set_estado("RUNNING".into());
            w.set_barra_progreso("░░░░░░░░░░ 0%".into());

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Runtime Tokio");
                rt.block_on(async move {
                    if let Err(e) = execute_suite_test(
                        &suite, logs, results, prog_tx,
                        cancel_clone, excel, upload, remote,
                    ).await {
                        eprintln!("[error] suite: {e}");
                    }
                    let _ = done_tx.send(());
                });
            });
        }
    });

    // ─────────────────────────────────────────────────────────────────────────
    // CALLBACKS — Tab 2: Configuraciones
    // ─────────────────────────────────────────────────────────────────────────

    // Cargar config seleccionada en la UI (desde tab Configs)
    window.on_config_cargar({
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let configs = list_configs_with_info().unwrap_or_default();
            let Some(info) = configs.get(idx as usize) else { return };
            if let Ok(cfg) = load_config(&info.name) {
                apply_config_to_ui(&w, &cfg);
                w.set_tab_activo(0);
            }
        }
    });

    // Guardar una nueva config con los datos del formulario
    window.on_config_guardar({
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let nombre = w.get_form_nombre().to_string();
            if nombre.trim().is_empty() {
                eprintln!("[warn] nombre de config vacío");
                return;
            }
            let config = SavedConfig {
                name: nombre.clone(),
                base_url: w.get_form_url().to_string(),
                requests: vec![TestRequest::default()],
                iterations: parse_u32(&w.get_form_iter(), 10),
                concurrent_requests: parse_u32(&w.get_form_concurrentes(), 1),
                wait_time: 1,
                output_dir: "./results".to_string(),
                auto_generate_report: true,
                auto_upload_report: false,
                remote_folder_path: String::new(),
                created_at: chrono::Local::now(),
                description: Some(w.get_form_desc().to_string()),
            };
            if let Err(e) = save_config(&config) {
                eprintln!("[error] guardar config: {e}");
            } else {
                refresh_configs_in_ui(&w);
            }
        }
    });

    // Duplicar config seleccionada
    window.on_config_duplicar({
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let configs = list_configs_with_info().unwrap_or_default();
            let Some(info) = configs.get(idx as usize) else { return };
            if let Ok(mut cfg) = load_config(&info.name) {
                cfg.name = format!("{}_copia_{}", cfg.name, chrono::Local::now().format("%H%M%S"));
                cfg.created_at = chrono::Local::now();
                if let Err(e) = save_config(&cfg) {
                    eprintln!("[error] duplicar config: {e}");
                } else {
                    refresh_configs_in_ui(&w);
                }
            }
        }
    });

    // Eliminar config seleccionada
    window.on_config_eliminar({
        let window_weak = window.as_weak();
        move |idx| {
            let Some(w) = window_weak.upgrade() else { return };
            let configs = list_configs_with_info().unwrap_or_default();
            let Some(info) = configs.get(idx as usize) else { return };
            if let Err(e) = delete_config(&info.name) {
                eprintln!("[error] eliminar config: {e}");
            } else {
                w.set_config_info_idx(-1);
                refresh_configs_in_ui(&w);
            }
        }
    });

    // ─────────────────────────────────────────────────────────────────────────
    // CALLBACKS — Tab 3: Resultados
    // ─────────────────────────────────────────────────────────────────────────

    // Limpiar resultados de la tabla
    window.on_limpiar_resultados({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let st = state.borrow();
            if let Ok(mut r) = st.results.lock() { r.clear(); }
            drop(st);
            state.borrow_mut().last_result_count = 0;
            w.set_resultados(ModelRc::new(VecModel::from(vec![])));
            reset_metrics(&w);
        }
    });

    // Exportar resultados a Excel
    window.on_exportar_excel({
        let state       = state.clone();
        let window_weak = window.as_weak();
        move || {
            let Some(w) = window_weak.upgrade() else { return };
            let st = state.borrow();
            let out_dir = w.get_dir_salida().to_string();
            let results = st.results.clone();
            drop(st);

            std::thread::spawn(move || {
                let results_lock = results.lock().unwrap();
                if results_lock.is_empty() { return; }
                drop(results_lock);

                // Buscar CSVs en el directorio de salida
                let csv_files = find_csv_files_in_dir(&out_dir);
                if csv_files.is_empty() { return; }

                let reports_dir = format!("{}/reports", out_dir);
                let _ = fs::create_dir_all(&reports_dir);
                let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let excel_path = format!("{}/report_{}.xlsx", reports_dir, ts);
                if let Err(e) = generate_excel_report_from_files(&csv_files, &excel_path) {
                    eprintln!("[error] exportar excel: {e}");
                } else {
                    let _ = open::that(reports_dir);
                }
            });
        }
    });

    // ─────────────────────────────────────────────────────────────────────────
    // CALLBACKS — Tab 4: Opciones Generales
    // ─────────────────────────────────────────────────────────────────────────

    window.on_guardar_general({
        let window_weak = window.as_weak();
        move || {
            // Persistir preferencias generales (monitoreo, etc.)
            let Some(w) = window_weak.upgrade() else { return };
            let _monitoreo = w.get_monitoreo();
            // Aquí se pueden guardar preferencias en JSON de ser necesario.
        }
    });

    window.on_seleccionar_dir_reportes({
        let window_weak = window.as_weak();
        move || {
            let window_weak = window_weak.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.to_string_lossy().to_string();
                    slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_dir_reportes(path_str.into());
                        }
                    }).ok();
                }
            });
        }
    });

    // Editar petición de la suite (por ahora sólo selecciona; edición inline pendiente)
    window.on_suite_editar({
        move |_idx| {
            // TODO: abrir diálogo modal para editar la petición seleccionada
        }
    });

    // ─────────────────────────────────────────────────────────────────────────
    // TIMER — Polling de progreso y actualización de UI
    // ─────────────────────────────────────────────────────────────────────────
    let _progress_timer = slint::Timer::default();
    _progress_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(120),
        {
            let state       = state.clone();
            let window_weak = window.as_weak();
            move || {
                let Some(w) = window_weak.upgrade() else { return };
                let mut st = state.borrow_mut();

                if !st.is_running { return; }

                // Actualizar barra de progreso
                let mut last_progress = 0.0f32;
                if let Some(ref rx) = st.progress_rx {
                    while let Ok(p) = rx.try_recv() {
                        last_progress = p;
                    }
                    if last_progress > 0.0 {
                        let filled = (last_progress * 10.0).round() as usize;
                        let empty  = 10usize.saturating_sub(filled);
                        let bar = format!("{}{} {:.0}%",
                            "█".repeat(filled),
                            "░".repeat(empty),
                            last_progress * 100.0,
                        );
                        w.set_barra_progreso(bar.into());
                    }
                }

                // Detectar completación
                let completed = if let Some(ref rx) = st.completion_rx {
                    rx.try_recv().is_ok()
                } else {
                    false
                };

                if completed {
                    st.is_running    = false;
                    st.completion_rx = None;
                    st.progress_rx   = None;
                    let results_arc  = st.results.clone();
                    drop(st);

                    w.set_ejecutando(false);
                    w.set_estado("DONE".into());
                    w.set_barra_progreso("██████████ 100%".into());

                    // Actualizar tabla y métricas en la UI
                    refresh_results_in_ui(&w, &results_arc);
                    // Ir al tab de resultados automáticamente
                    w.set_tab_activo(3);
                }
            }
        },
    );

    window.run()?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS — Conversión de datos UI ↔ modelos Rust
// ─────────────────────────────────────────────────────────────────────────────

/// Construye un TestRequest a partir de los valores actuales de la ventana Slint.
fn build_request_from_ui(w: &AppWindow) -> TestRequest {
    let method_idx = w.get_metodo_idx();
    let method = match method_idx {
        0 => HttpMethod::GET,
        1 => HttpMethod::POST,
        2 => HttpMethod::PUT,
        3 => HttpMethod::DELETE,
        _ => HttpMethod::PATCH,
    };
    let body = if method_idx == 0 {
        None
    } else {
        let s = w.get_body_json().to_string();
        if s.trim().is_empty() || s == "{}" { None } else { Some(s) }
    };
    TestRequest {
        method,
        endpoint:     w.get_endpoint().to_string(),
        headers:      parse_headers_json(&w.get_headers_json()),
        query_params: Vec::new(),
        body,
        description:  w.get_descripcion().to_string(),
    }
}

/// Aplica una SavedConfig a todos los campos de la ventana Slint.
fn apply_config_to_ui(w: &AppWindow, cfg: &SavedConfig) {
    w.set_url_base(cfg.base_url.clone().into());
    w.set_iteraciones(cfg.iterations.to_string().into());
    w.set_peticiones_simultaneas(cfg.concurrent_requests.to_string().into());
    w.set_timeout_seg("30".into());
    w.set_dir_salida(cfg.output_dir.clone().into());
    w.set_auto_excel(cfg.auto_generate_report);
    w.set_auto_subir(cfg.auto_upload_report);
    w.set_carpeta_remota(cfg.remote_folder_path.clone().into());

    if let Some(req) = cfg.requests.first() {
        let method_idx: i32 = match req.method {
            HttpMethod::GET     => 0,
            HttpMethod::POST    => 1,
            HttpMethod::PUT     => 2,
            HttpMethod::DELETE  => 3,
            HttpMethod::PATCH   => 4,
            _                   => 0,
        };
        w.set_metodo_idx(method_idx);
        w.set_endpoint(req.endpoint.clone().into());
        w.set_descripcion(req.description.clone().into());

        let headers_json = serde_json::to_string_pretty(
            &req.headers.iter()
                .map(|h| serde_json::json!({h.name.clone(): h.value.clone()}))
                .collect::<Vec<_>>()
        ).unwrap_or_else(|_| "{}".to_string());
        w.set_headers_json(headers_json.into());

        let body = req.body.clone().unwrap_or_default();
        w.set_body_json(body.into());
    }
}

/// Recarga las listas de configuraciones guardadas en la UI.
fn refresh_configs_in_ui(w: &AppWindow) {
    let names = list_saved_configs().unwrap_or_default();
    let slint_names: Vec<SharedString> = names.iter().map(|s| s.as_str().into()).collect();
    w.set_configs_lista(ModelRc::new(VecModel::from(slint_names)));

    let infos = list_configs_with_info().unwrap_or_default();
    let slint_infos: Vec<ConfigItemData> = infos.iter().map(|ci| ConfigItemData {
        name:          ci.name.clone().into(),
        created_at:    ci.created_at.format("%Y-%m-%d %H:%M").to_string().into(),
        request_count: ci.request_count as i32,
        is_suite:      ci.is_suite,
    }).collect();
    w.set_configs_info(ModelRc::new(VecModel::from(slint_infos)));
}

/// Sincroniza la lista de peticiones de la suite al modelo Slint.
fn sync_suite_to_ui(w: &AppWindow, requests: &[TestRequest]) {
    let slint_reqs: Vec<SuiteRequestData> = requests.iter().map(|r| SuiteRequestData {
        method:      r.method.to_string().into(),
        endpoint:    r.endpoint.clone().into(),
        description: r.description.clone().into(),
    }).collect();
    w.set_suite_requests(ModelRc::new(VecModel::from(slint_reqs)));
}

/// Actualiza la tabla de resultados y las métricas resumen.
fn refresh_results_in_ui(w: &AppWindow, results_arc: &Arc<Mutex<Vec<TestSummary>>>) {
    let Ok(results) = results_arc.lock() else { return };
    if results.is_empty() {
        reset_metrics(w);
        return;
    }

    let slint_results: Vec<ResultItemData> = results.iter().map(|r| ResultItemData {
        request_name:  r.request_name.clone().into(),
        method:        "HTTP".into(),
        total_requests: r.total_requests as i32,
        successful:    r.successful_requests as i32,
        failed:        r.failed_requests as i32,
        avg_ms:        r.average_duration_ms as f32,
        max_ms:        r.max_duration_ms as i32,
        success_rate:  r.success_rate as f32,
    }).collect();
    w.set_resultados(ModelRc::new(VecModel::from(slint_results)));

    // Métricas resumen agregadas
    let total: u32   = results.iter().map(|r| r.total_requests).sum();
    let ok: u32      = results.iter().map(|r| r.successful_requests).sum();
    let tasa = if total > 0 { ok as f64 / total as f64 * 100.0 } else { 0.0 };
    let avg: f64 = results.iter().map(|r| r.average_duration_ms).sum::<f64>() / results.len() as f64;
    let max: u64 = results.iter().map(|r| r.max_duration_ms).max().unwrap_or(0);
    let min: u64 = results.iter().map(|r| r.min_duration_ms).min().unwrap_or(0);

    w.set_res_total(total.to_string().into());
    w.set_res_tasa(format!("{tasa:.1}%").into());
    w.set_res_avg(format!("{avg:.0}ms").into());
    w.set_res_max(format!("{max}ms").into());
    w.set_res_min(format!("{min}ms").into());
    w.set_res_p95("N/A".into());
    w.set_res_p99("N/A".into());

    // Gráfico ASCII de distribución de tiempos
    w.set_ascii_chart(generate_ascii_chart(&results).into());
}

/// Resetea todos los campos de métricas a cero.
fn reset_metrics(w: &AppWindow) {
    w.set_res_total("0".into());
    w.set_res_tasa("0%".into());
    w.set_res_avg("0ms".into());
    w.set_res_max("0ms".into());
    w.set_res_min("0ms".into());
    w.set_res_p95("N/A".into());
    w.set_res_p99("N/A".into());
    w.set_ascii_chart("// sin datos de resultados".into());
}

/// Genera un gráfico de barras ASCII con la distribución de tiempos de respuesta.
fn generate_ascii_chart(results: &[TestSummary]) -> String {
    if results.is_empty() {
        return "// sin datos".to_string();
    }

    // Usar el tiempo promedio de cada petición para clasificar
    let buckets: [(&str, f64, f64); 5] = [
        ("  <10ms", 0.0,    10.0),
        ("10-50ms", 10.0,   50.0),
        ("50-100 ", 50.0,  100.0),
        (" 100-1s", 100.0, 1000.0),
        ("  >1000", 1000.0, f64::MAX),
    ];

    let total = results.len() as f64;
    let mut lines = Vec::new();
    lines.push("// distribución de tiempos promedio".to_string());
    lines.push(String::new());

    for (label, lo, hi) in &buckets {
        let count = results.iter()
            .filter(|r| r.average_duration_ms >= *lo && r.average_duration_ms < *hi)
            .count() as f64;
        let pct = if total > 0.0 { count / total } else { 0.0 };
        let filled = (pct * 20.0).round() as usize;
        let bar = "█".repeat(filled);
        lines.push(format!("{} │{:<20}│ {:5.1}%  ({})", label, bar, pct * 100.0, count as usize));
    }

    lines.join("\n")
}

/// Carga los datos iniciales de la aplicación en la ventana Slint.
fn populate_initial_data(w: &AppWindow) {
    refresh_configs_in_ui(w);
    w.set_suite_requests(ModelRc::new(VecModel::from(vec![])));
    w.set_resultados(ModelRc::new(VecModel::from(vec![])));
    reset_metrics(w);
    w.set_app_version(env!("CARGO_PKG_VERSION").into());
}

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS — Parseo de datos
// ─────────────────────────────────────────────────────────────────────────────

fn parse_u32(s: &SharedString, default: u32) -> u32 {
    s.to_string().trim().parse::<u32>().unwrap_or(default)
}

/// Parsea un JSON de headers al formato Vec<HttpHeader>.
fn parse_headers_json(json_str: &SharedString) -> Vec<HttpHeader> {
    let s = json_str.to_string();
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) else { return Vec::new() };
    let Some(obj) = val.as_object() else { return Vec::new() };
    obj.iter().map(|(k, v)| HttpHeader {
        name:  k.clone(),
        value: v.as_str().unwrap_or("").to_string(),
    }).collect()
}

/// Busca archivos CSV en un directorio de salida.
fn find_csv_files_in_dir(dir_path: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir_path) else { return Vec::new() };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "csv"))
        .collect()
}

/// Busca un archivo CSV específico por nombre de prueba.
fn find_csv_file_in_directory(dir_path: &str, test_name: &str) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(dir_path) else { return None };
    let safe_name = test_name.replace(' ', "_");
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "csv") {
            let name = path.file_name()?.to_string_lossy().to_string();
            if name.contains(&safe_name) { return Some(path); }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// LÓGICA DE EJECUCIÓN — funciones asíncronas (sin modificar la lógica original)
// ─────────────────────────────────────────────────────────────────────────────

/// Ejecuta una prueba HTTP individual con soporte de cancelación y progreso.
async fn execute_single_test(
    request:            &TestRequest,
    base_url:           &str,
    iterations:         u32,
    concurrent:         u32,
    wait_time:          u64,
    output_dir:         &str,
    logs:               Arc<Mutex<Vec<String>>>,
    results:            Arc<Mutex<Vec<TestSummary>>>,
    progress_tx:        mpsc::Sender<f32>,
    cancel_flag:        Arc<Mutex<bool>>,
    auto_excel:         bool,
    auto_upload:        bool,
    remote_folder_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut l = logs.lock().unwrap();
        l.push(format!("[{}] Iniciando prueba: {}", chrono::Local::now().format("%H:%M:%S"), request.description));
    }

    let final_dir = resolve_output_dir(output_dir);

    let tester  = LoadTester::new();
    let summary = tester.run_single_test_with_progress_and_cancel(
        request, base_url, iterations, concurrent, wait_time,
        &final_dir, progress_tx, cancel_flag,
    ).await?;

    let csv_file = find_csv_file_in_directory(&final_dir, &request.description);

    {
        let mut r = results.lock().unwrap();
        r.push(summary);
    }
    {
        let mut l = logs.lock().unwrap();
        l.push(format!("[{}] Prueba completada: {}", chrono::Local::now().format("%H:%M:%S"), request.description));
    }

    if auto_excel {
        let reports_dir = format!("{}/reports", final_dir);
        let ts          = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let safe        = request.description.replace([' ', '/','\\'], "_");
        let excel_path  = format!("{}/report_{}_{}.xlsx", reports_dir, safe, ts);

        if let Some(csv) = csv_file {
            match generate_excel_report_from_files(&[csv.clone()], &excel_path) {
                Ok(out) => {
                    let mut l = logs.lock().unwrap();
                    l.push(format!("[{}] ✅ Excel: {}", chrono::Local::now().format("%H:%M:%S"), out));
                    upload_files_if_needed(
                        &[csv, PathBuf::from(&excel_path)],
                        &remote_folder_path,
                        &safe, auto_upload, &mut l,
                    );
                }
                Err(e) => {
                    let mut l = logs.lock().unwrap();
                    l.push(format!("[{}] ❌ Error Excel: {e}", chrono::Local::now().format("%H:%M:%S")));
                }
            }
        }
    }

    Ok(())
}

/// Ejecuta una suite completa de pruebas HTTP con soporte de cancelación y progreso.
async fn execute_suite_test(
    suite:              &TestSuite,
    logs:               Arc<Mutex<Vec<String>>>,
    results:            Arc<Mutex<Vec<TestSummary>>>,
    progress_tx:        mpsc::Sender<f32>,
    cancel_flag:        Arc<Mutex<bool>>,
    auto_excel:         bool,
    auto_upload:        bool,
    remote_folder_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut l = logs.lock().unwrap();
        l.push(format!("[{}] Iniciando suite: {}", chrono::Local::now().format("%H:%M:%S"), suite.name));
    }

    let final_dir = resolve_output_dir(&suite.output_dir);
    let mut corrected = suite.clone();
    corrected.output_dir = final_dir.clone();

    let tester    = LoadTester::new();
    let summaries = tester.run_suite_test_with_progress_and_cancel(&corrected, progress_tx, cancel_flag).await?;

    {
        let mut r = results.lock().unwrap();
        r.extend(summaries);
    }
    {
        let mut l = logs.lock().unwrap();
        l.push(format!("[{}] Suite completada: {}", chrono::Local::now().format("%H:%M:%S"), suite.name));
    }

    if auto_excel {
        let csv_files   = find_csv_files_in_dir(&final_dir);
        let reports_dir = format!("{}/reports", final_dir);
        let ts          = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let safe        = suite.name.replace([' ', '/','\\'], "_");
        let excel_path  = format!("{}/report_{}_{}.xlsx", reports_dir, safe, ts);

        match generate_excel_report_from_files(&csv_files, &excel_path) {
            Ok(out) => {
                let mut l = logs.lock().unwrap();
                l.push(format!("[{}] ✅ Excel: {}", chrono::Local::now().format("%H:%M:%S"), out));
                let mut files: Vec<PathBuf> = csv_files;
                files.push(PathBuf::from(&excel_path));
                upload_files_if_needed(&files, &remote_folder_path, &safe, auto_upload, &mut l);
            }
            Err(e) => {
                let mut l = logs.lock().unwrap();
                l.push(format!("[{}] ❌ Error Excel: {e}", chrono::Local::now().format("%H:%M:%S")));
            }
        }
    }

    Ok(())
}

/// Sube archivos a una carpeta remota (copia local) si el flag está habilitado.
fn upload_files_if_needed(
    files:         &[PathBuf],
    remote_folder: &str,
    label:         &str,
    do_upload:     bool,
    logs:          &mut Vec<String>,
) {
    if !do_upload || remote_folder.is_empty() { return; }

    let ts   = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let dest = format!("{}/test_{}_{}", remote_folder, label, ts);

    if let Err(e) = fs::create_dir_all(&dest) {
        logs.push(format!("[{}] ❌ Crear carpeta remota: {e}", chrono::Local::now().format("%H:%M:%S")));
        return;
    }

    let mut count = 0usize;
    for file in files {
        if !file.exists() { continue; }
        let file_name = file.file_name().unwrap_or_default().to_string_lossy();
        let target = format!("{}/{}", dest, file_name);
        if fs::copy(file, &target).is_ok() { count += 1; }
    }
    logs.push(format!("[{}] ✅ {count} archivos subidos a: {dest}", chrono::Local::now().format("%H:%M:%S")));
}

/// Resuelve el directorio de salida real, creándolo si es necesario.
fn resolve_output_dir(requested: &str) -> String {
    if requested.is_empty() {
        return get_output_directory();
    }
    match fs::create_dir_all(requested) {
        Ok(_)  => requested.to_string(),
        Err(_) => get_output_directory(),
    }
}
