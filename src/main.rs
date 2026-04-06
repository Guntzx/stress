#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::NativeOptions;

mod cli;
mod config;
mod gui;
mod load_test;
mod models;
mod monitor;
mod report_generator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(subcommand) = args.get(1) {
        match subcommand.as_str() {
            "update" => {
                cli::update().await?;
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
                eprintln!("[ERROR] Subcomando desconocido: '{}'", other);
                eprintln!("        Ejecuta 'stress help' para ver los comandos disponibles.");
                std::process::exit(1);
            }
        }
    }

    tracing_subscriber::fmt::init();

    let options = NativeOptions::default();
    eframe::run_native(
        "Test Stress - Pruebas de Carga",
        options,
        Box::new(|cc| Ok(Box::new(gui::TestStressApp::new(cc)))),
    )?;

    Ok(())
}
