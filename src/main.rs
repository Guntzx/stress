#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::NativeOptions;

mod config;
mod gui;
mod load_test;
mod models;
mod monitor;
mod report_generator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let options = NativeOptions::default();
    eframe::run_native(
        "Test Stress - Pruebas de Carga",
        options,
        Box::new(|cc| Box::new(gui::TestStressApp::new(cc))),
    )?;

    Ok(())
}
