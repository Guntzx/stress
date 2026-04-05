use std::path::PathBuf;
use umya_spreadsheet::*;
use crate::models::TestResult;
use tracing::info;

pub fn generate_excel_report_from_files(csv_files: &[PathBuf], excel_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("Generando reporte Excel desde archivos: {:?}", csv_files);
    let mut book = umya_spreadsheet::new_file();
    
    // Leer todos los resultados
    let mut all_results: Vec<TestResult> = Vec::new();
    for csv_path in csv_files {
        let mut rdr = csv::Reader::from_path(csv_path)?;
        for result in rdr.deserialize() {
            let r: TestResult = result?;
            all_results.push(r);
        }
    }
    
    // Crear hoja de resumen y escribir datos
    let summary_sheet = book.new_sheet("Resumen").unwrap();
    summary_sheet.get_cell_mut((1, 1)).set_value("Total requests");
    summary_sheet.get_cell_mut((2, 1)).set_value(all_results.len().to_string());
    
    let success_count = all_results.iter().filter(|r| r.success).count() as u32;
    let fail_count = all_results.len() as u32 - success_count;
    
    summary_sheet.get_cell_mut((1, 2)).set_value("Successful requests");
    summary_sheet.get_cell_mut((2, 2)).set_value(success_count.to_string());
    summary_sheet.get_cell_mut((1, 3)).set_value("Failed requests");
    summary_sheet.get_cell_mut((2, 3)).set_value(fail_count.to_string());
    
    let total_duration: u64 = all_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = if all_results.len() > 0 {
        total_duration as f64 / all_results.len() as f64
    } else { 0.0 };
    let min_duration = all_results.iter().map(|r| r.duration_ms).min().unwrap_or(0);
    let max_duration = all_results.iter().map(|r| r.duration_ms).max().unwrap_or(0);
    
    summary_sheet.get_cell_mut((1, 4)).set_value("Total duration (ms)");
    summary_sheet.get_cell_mut((2, 4)).set_value((total_duration as i64).to_string());
    summary_sheet.get_cell_mut((1, 5)).set_value("Average duration (ms)");
    summary_sheet.get_cell_mut((2, 5)).set_value(format!("{:.2}", avg_duration));
    summary_sheet.get_cell_mut((1, 6)).set_value("Min duration (ms)");
    summary_sheet.get_cell_mut((2, 6)).set_value((min_duration as i64).to_string());
    summary_sheet.get_cell_mut((1, 7)).set_value("Max duration (ms)");
    summary_sheet.get_cell_mut((2, 7)).set_value((max_duration as i64).to_string());
    
    let success_rate = if all_results.len() > 0 {
        (success_count as f64 / all_results.len() as f64) * 100.0
    } else { 0.0 };
    summary_sheet.get_cell_mut((1, 8)).set_value("Success rate (%)");
    summary_sheet.get_cell_mut((2, 8)).set_value(format!("{:.2}", success_rate));
    
    // Una hoja por cada archivo CSV
    for csv_path in csv_files {
        let file_name = csv_path.file_stem().unwrap().to_string_lossy();
        let sheet = book.new_sheet(file_name.to_string()).unwrap();
        // Escribir encabezados
        let headers = [
            "Test Type", "Request Name", "Iteration", "Start Time", "End Time", 
            "Duration (ms)", "Success", "Status Code", "Error"
        ];
        for (i, h) in headers.iter().enumerate() {
            sheet.get_cell_mut(((i+1) as u32, 1)).set_value(*h);
        }
        // Escribir filas
        let mut rdr = csv::Reader::from_path(csv_path)?;
        for (row, result) in rdr.deserialize().enumerate() {
            let r: TestResult = result?;
            sheet.get_cell_mut((1, (row+2) as u32)).set_value(format!("{}", r.test_type));
            sheet.get_cell_mut((2, (row+2) as u32)).set_value(&r.request_name);
            sheet.get_cell_mut((3, (row+2) as u32)).set_value(r.iteration.to_string());
            sheet.get_cell_mut((4, (row+2) as u32)).set_value(r.start_time.to_rfc3339());
            sheet.get_cell_mut((5, (row+2) as u32)).set_value(r.end_time.to_rfc3339());
            sheet.get_cell_mut((6, (row+2) as u32)).set_value(r.duration_ms.to_string());
            sheet.get_cell_mut((7, (row+2) as u32)).set_value(if r.success {"Yes"} else {"No"});
            sheet.get_cell_mut((8, (row+2) as u32)).set_value(r.status_code.map(|c| c.to_string()).unwrap_or_default());
            sheet.get_cell_mut((9, (row+2) as u32)).set_value(r.error.unwrap_or_default());
        }
    }
    
    // Guardar el archivo Excel
    if let Some(parent) = std::path::Path::new(excel_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    writer::xlsx::write(&book, excel_path)?;
    println!("Reporte Excel generado exitosamente en: {}", excel_path);
    Ok(excel_path.to_string())
} 