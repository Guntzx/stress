#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod cli;
mod config;
mod gui;
mod load_test;
mod models;
mod report_generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(subcommand) = args.get(1) {
        // Crear runtime Tokio para los subcomandos CLI asíncronos
        let rt = tokio::runtime::Runtime::new()?;
        match subcommand.as_str() {
            "update" => {
                rt.block_on(cli::update())?;
                return Ok(());
            }
            "uninstall" => {
                cli::uninstall()?;
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                cli::print_help();
                return Ok(());
            }
            other => {
                eprintln!("[ERROR] Subcomando desconocido: '{other}'");
                eprintln!("        Ejecuta 'stress help' para ver los comandos disponibles.");
                std::process::exit(1);
            }
        }
    }

    // Inicializar logging
    tracing_subscriber::fmt::init();

    // Lanzar la interfaz gráfica Slint
    gui::run_app()
}
