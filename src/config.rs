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


fn get_configs_dir() -> PathBuf {
    let user_dir = get_user_data_dir();
    #[cfg(target_os = "windows")]
    { PathBuf::from(format!("{}\\configs", user_dir)) }
    #[cfg(not(target_os = "windows"))]
    { PathBuf::from(format!("{}/.stress/configs", user_dir)) }
}

pub fn save_config(config: &SavedConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_configs_dir();
    ensure_directory_with_fallback(&config_dir.to_string_lossy())?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_dir.join(format!("{}.json", config.name)), json)?;
    Ok(())
}

pub fn load_config(name: &str) -> Result<SavedConfig, Box<dyn std::error::Error>> {
    let config_file = get_configs_dir().join(format!("{}.json", name));
    let content = fs::read_to_string(config_file)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn list_saved_configs() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let config_dir = get_configs_dir();
    if !config_dir.exists() {
        return Ok(Vec::new());
    }
    let mut configs = Vec::new();
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            if let Some(name) = path.file_stem() {
                configs.push(name.to_string_lossy().to_string());
            }
        }
    }
    Ok(configs)
}

pub fn delete_config(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(get_configs_dir().join(format!("{}.json", name)))?;
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
    let config_dir = get_configs_dir();
    if !config_dir.exists() {
        return Ok(Vec::new());
    }
    let mut configs = Vec::new();
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            if let Some(name) = path.file_stem() {
                let name_str = name.to_string_lossy().to_string();
                if let Ok(info) = get_config_info(&name_str) {
                    configs.push(info);
                }
            }
        }
    }
    configs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(configs)
}

 