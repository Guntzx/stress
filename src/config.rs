use std::fs;
use std::path::PathBuf;
use crate::models::SavedConfig;
use std::env;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub description: Option<String>,
    pub request_count: usize,
    pub is_suite: bool,
}

#[cfg(target_os = "windows")]
use std::process::Command;

fn get_user_data_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        // En Windows, usar el directorio de documentos del usuario
        if let Ok(documents) = std::env::var("USERPROFILE") {
            format!("{}\\Documents\\Stress", documents)
        } else {
            // Fallback al directorio actual
            ".".to_string()
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // En Unix/Linux/macOS, usar HOME
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

#[cfg(target_os = "windows")]
fn is_admin() -> bool {
    use std::process::Command;
    let output = Command::new("net")
        .args(&["session"])
        .output();
    output.is_ok() && output.unwrap().status.success()
}

#[cfg(target_os = "windows")]
fn request_admin_privileges() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // Obtener la ruta del ejecutable actual
    let exe_path = std::env::current_exe()?;
    
    // Crear comando para ejecutar como administrador
    let mut cmd = Command::new("powershell");
    cmd.args(&[
        "Start-Process",
        &format!("'{}'", exe_path.to_string_lossy()),
        "-Verb", "RunAs",
        "-ArgumentList", "--gui"
    ]);
    
    // Ejecutar el comando
    let status = cmd.status()?;
    
    if status.success() {
        // Salir del proceso actual ya que se inició uno nuevo con privilegios
        std::process::exit(0);
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn request_admin_privileges_with_args() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    let exe_path = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();
    let args_str = args.join(" ");
    let mut cmd = Command::new("powershell");
    cmd.args(&[
        "Start-Process",
        &format!("'{}'", exe_path.to_string_lossy()),
        "-Verb", "RunAs",
        "-ArgumentList", &format!("'{}'", args_str),
    ]);
    let status = cmd.status()?;
    if status.success() {
        std::process::exit(0);
    }
    Err("No se pudo obtener privilegios de administrador (UAC) o el usuario lo rechazó".into())
}

#[cfg(not(target_os = "windows"))]
fn request_sudo_privileges_with_args() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    let exe_path = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cmd = Command::new("sudo");
    cmd.arg(exe_path);
    for arg in args {
        cmd.arg(arg);
    }
    let status = cmd.status()?;
    if status.success() {
        std::process::exit(0);
    }
    Err("No se pudo obtener privilegios de administrador (sudo) o el usuario lo rechazó".into())
}

fn ensure_directory_with_fallback(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    match fs::create_dir_all(path) {
        Ok(_) => Ok(path.to_string()),
        Err(e) => {
            #[cfg(target_os = "windows")]
            {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!("Se requieren permisos de administrador para acceder a: {}", path);
                    eprintln!("Intentando relanzar con privilegios elevados...");
                    if let Err(err) = request_admin_privileges_with_args() {
                        eprintln!("Error: {}", err);
                        return Err(Box::new(e));
                    } else {
                        Ok(path.to_string())
                    }
                } else {
                    Err(Box::new(e))
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!("Se requieren permisos de administrador para acceder a: {}", path);
                    eprintln!("Intentando relanzar con sudo...");
                    if let Err(err) = request_sudo_privileges_with_args() {
                        eprintln!("Error: {}", err);
                        return Err(Box::new(e));
                    } else {
                        Ok(path.to_string())
                    }
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }
}

pub fn get_output_directory() -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let path = format!("{}\\results\\{}", user_dir, timestamp);
        match ensure_directory_with_fallback(&path) {
            Ok(dir) => dir,
            Err(_) => {
                // Fallback final: directorio temporal
                let temp_dir = std::env::temp_dir();
                format!("{}\\Stress\\results\\{}", temp_dir.to_string_lossy(), timestamp)
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        format!("{}/.stress/results/{}", user_dir, timestamp)
    }
}

pub fn ensure_output_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    ensure_directory_with_fallback(path)?;
    Ok(())
}

pub fn save_config(config: &SavedConfig) -> Result<(), Box<dyn std::error::Error>> {
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let config_dir = format!("{}\\configs", user_dir);
        let final_config_dir = ensure_directory_with_fallback(&config_dir)?;
        
        let config_file = format!("{}\\{}.json", final_config_dir, config.name);
        let json = serde_json::to_string_pretty(config)?;
        fs::write(config_file, json)?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let config_dir = format!("{}/.stress/configs", user_dir);
        fs::create_dir_all(&config_dir)?;
        
        let config_file = format!("{}/{}.json", config_dir, config.name);
        let json = serde_json::to_string_pretty(config)?;
        fs::write(config_file, json)?;
    }
    
    Ok(())
}

pub fn load_config(name: &str) -> Result<SavedConfig, Box<dyn std::error::Error>> {
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let config_file = format!("{}\\configs\\{}.json", user_dir, name);
        let content = fs::read_to_string(config_file)?;
        let config: SavedConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let config_file = format!("{}/.stress/configs/{}.json", user_dir, name);
        let content = fs::read_to_string(config_file)?;
        let config: SavedConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

pub fn list_saved_configs() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let config_dir = format!("{}\\configs", user_dir);
        if !PathBuf::from(&config_dir).exists() {
            return Ok(Vec::new());
        }
        
        let mut configs = Vec::new();
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Some(name) = entry.path().file_stem() {
                        configs.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(configs)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let config_dir = format!("{}/.stress/configs", user_dir);
        if !PathBuf::from(&config_dir).exists() {
            return Ok(Vec::new());
        }
        
        let mut configs = Vec::new();
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Some(name) = entry.path().file_stem() {
                        configs.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(configs)
    }
}

pub fn delete_config(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let config_file = format!("{}\\configs\\{}.json", user_dir, name);
        fs::remove_file(config_file)?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let config_file = format!("{}/.stress/configs/{}.json", user_dir, name);
        fs::remove_file(config_file)?;
    }
    
    Ok(())
}

pub fn get_config_info(name: &str) -> Result<ConfigInfo, Box<dyn std::error::Error>> {
    let config = load_config(name)?;
    Ok(ConfigInfo {
        name: config.name.clone(),
        created_at: config.created_at,
        description: config.description.clone(),
        request_count: config.requests.len(),
        is_suite: config.requests.len() > 1,
    })
}

pub fn list_configs_with_info() -> Result<Vec<ConfigInfo>, Box<dyn std::error::Error>> {
    let user_dir = get_user_data_dir();
    
    #[cfg(target_os = "windows")]
    {
        let config_dir = format!("{}\\configs", user_dir);
        if !PathBuf::from(&config_dir).exists() {
            return Ok(Vec::new());
        }
        
        let mut configs = Vec::new();
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Some(name) = entry.path().file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        if let Ok(info) = get_config_info(&name_str) {
                            configs.push(info);
                        }
                    }
                }
            }
        }
        // Ordenar por fecha de creación (más reciente primero)
        configs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(configs)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let config_dir = format!("{}/.stress/configs", user_dir);
        if !PathBuf::from(&config_dir).exists() {
            return Ok(Vec::new());
        }
        
        let mut configs = Vec::new();
        for entry in fs::read_dir(config_dir)? {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Some(name) = entry.path().file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        if let Ok(info) = get_config_info(&name_str) {
                            configs.push(info);
                        }
                    }
                }
            }
        }
        // Ordenar por fecha de creación (más reciente primero)
        configs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(configs)
    }
}

pub fn search_configs(query: &str) -> Result<Vec<ConfigInfo>, Box<dyn std::error::Error>> {
    let all_configs = list_configs_with_info()?;
    let query_lower = query.to_lowercase();
    
    let filtered = all_configs.into_iter()
        .filter(|config| {
            config.name.to_lowercase().contains(&query_lower) ||
            config.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
        })
        .collect();
    
    Ok(filtered)
} 