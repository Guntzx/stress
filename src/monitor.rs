use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;
use sysinfo::System;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Local>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub active_connections: u32,
    pub load_average: f64,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub monitoring_type: MonitoringType,
    pub interval_ms: u64,
    pub max_history: usize,
    pub monitor_cpu: bool,
    pub monitor_memory: bool,
    pub monitor_disk: bool,
    pub monitor_network: bool,
    pub ssh_config: SSHConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MonitoringType {
    Local,
    SSH,
}

impl Default for MonitoringType {
    fn default() -> Self {
        MonitoringType::Local
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            monitoring_type: MonitoringType::Local,
            interval_ms: 1000, // 1 segundo
            max_history: 300,  // 5 minutos de historia
            monitor_cpu: true,
            monitor_memory: true,
            monitor_disk: true,
            monitor_network: true,
            ssh_config: SSHConfig::default(),
        }
    }
}

pub struct SystemMonitor {
    config: MonitoringConfig,
    metrics_history: Arc<Mutex<VecDeque<SystemMetrics>>>,
    is_monitoring: Arc<Mutex<bool>>,
}

impl SystemMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics_history: Arc::new(Mutex::new(VecDeque::new())),
            is_monitoring: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn start_monitoring(&mut self) {
        let mut is_monitoring = self.is_monitoring.lock().unwrap();
        *is_monitoring = true;
        drop(is_monitoring);
        
        let metrics_history = Arc::clone(&self.metrics_history);
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let config = self.config.clone();
        
        std::thread::spawn(move || {
            let mut system = System::new_all();

            // Conexión SSH si es necesario
            let mut ssh_session: Option<Session> = None;
            if config.monitoring_type == MonitoringType::SSH {
                if let Ok(session) = Self::create_ssh_session(&config.ssh_config) {
                    ssh_session = Some(session);
                }
            }

            while *is_monitoring.lock().unwrap() {
                let timestamp = Local::now();

                let metrics = if config.monitoring_type == MonitoringType::Local {
                    Self::get_local_metrics(&mut system, &config, timestamp)
                } else {
                    // Monitoreo SSH
                    if let Some(ref session) = ssh_session {
                        Self::get_ssh_metrics(session, &config, timestamp)
                    } else {
                        // Fallback a métricas locales si SSH falla
                        Self::get_local_metrics(&mut system, &config, timestamp)
                    }
                };
                
                // Actualizar historial
                if let Ok(mut history) = metrics_history.lock() {
                    history.push_back(metrics);
                    if history.len() > config.max_history {
                        history.pop_front();
                    }
                }
                
                std::thread::sleep(Duration::from_millis(config.interval_ms));
            }
        });
    }
    
    fn create_ssh_session(ssh_config: &SSHConfig) -> Result<Session, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(&ssh_config.username, &ssh_config.password)?;
        Ok(sess)
    }
    
    fn get_local_metrics(
        system: &mut System,
        config: &MonitoringConfig,
        timestamp: DateTime<Local>
    ) -> SystemMetrics {
        system.refresh_all();

        let cpu_usage = if config.monitor_cpu {
            system.global_cpu_info().cpu_usage()
        } else {
            0.0
        };

        let (memory_usage, memory_used, memory_total) = if config.monitor_memory {
            let total = system.total_memory();
            let used = system.used_memory();
            let usage = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };
            (usage, used, total)
        } else {
            (0.0, 0, 0)
        };

        SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            memory_used,
            memory_total,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            active_connections: 0,
            load_average: 0.0,
            temperature: None,
        }
    }
    
    fn get_ssh_metrics(session: &Session, config: &MonitoringConfig, timestamp: DateTime<Local>) -> SystemMetrics {
        let mut cpu_usage = 0.0;
        let mut memory_usage = 0.0;
        let mut memory_used = 0u64;
        let mut memory_total = 0u64;
        let mut load_average = 0.0;
        
        // Obtener CPU usage
        if config.monitor_cpu {
            if let Ok(output) = Self::execute_ssh_command(session, "top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1") {
                if let Ok(usage) = output.trim().parse::<f32>() {
                    cpu_usage = usage;
                }
            }
        }
        
        // Obtener memoria
        if config.monitor_memory {
            if let Ok(output) = Self::execute_ssh_command(session, "free | grep Mem") {
                let parts: Vec<&str> = output.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                        memory_total = total * 1024; // Convertir KB a bytes
                        memory_used = used * 1024;
                        memory_usage = if memory_total > 0 { (memory_used as f32 / memory_total as f32) * 100.0 } else { 0.0 };
                    }
                }
            }
        }
        
        // Obtener load average
        if let Ok(output) = Self::execute_ssh_command(session, "cat /proc/loadavg | awk '{print $1}'") {
            if let Ok(load) = output.trim().parse::<f64>() {
                load_average = load;
            }
        }
        
        SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            memory_used,
            memory_total,
            disk_read_bytes: 0, // TODO: Implementar métricas de disco SSH
            disk_write_bytes: 0,
            network_rx_bytes: 0, // TODO: Implementar métricas de red SSH
            network_tx_bytes: 0,
            active_connections: 0,
            load_average,
            temperature: None,
        }
    }
    
    fn execute_ssh_command(session: &Session, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut channel = session.channel_session()?;
        channel.exec(command)?;
        
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        
        Ok(output)
    }
    
    pub fn stop_monitoring(&mut self) {
        let mut is_monitoring = self.is_monitoring.lock().unwrap();
        *is_monitoring = false;
    }
    
    pub fn get_current_metrics(&mut self) -> Option<SystemMetrics> {
        if let Ok(history) = self.metrics_history.lock() {
            history.back().cloned()
        } else {
            None
        }
    }
    
    pub fn update_config(&mut self, config: MonitoringConfig) {
        self.config = config;
    }
}

// Funciones de utilidad para formatear métricas
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;
    
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", value as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", value, UNITS[unit_index])
    }
}

pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value)
}

// Estructuras para monitoreo SSH remoto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub save_credentials: bool,
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            username: String::new(),
            password: String::new(),
            port: 22,
            save_credentials: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSSHConfig {
    pub host: String,
    pub username: String,
    pub encrypted_password: String,
    pub port: u16,
}

// Funciones para encriptación de credenciales
pub fn encrypt_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};
    use base64::{Engine as _, engine::general_purpose};
    
    // Clave fija para simplificar (en producción debería ser más segura)
    let key = Key::<Aes256Gcm>::from_slice(b"my-32-byte-secret-key-for-aes-256-gcm");
    let cipher = Aes256Gcm::new(key);
    
    // Nonce aleatorio
    let nonce = Nonce::from_slice(b"unique-nonce-12");
    
    // Encriptar
    let ciphertext = cipher.encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("Error encriptando: {}", e))?;
    
    // Codificar en base64
    Ok(general_purpose::STANDARD.encode(ciphertext))
}

pub fn decrypt_password(encrypted_password: &str) -> Result<String, Box<dyn std::error::Error>> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};
    use base64::{Engine as _, engine::general_purpose};
    
    // Clave fija para simplificar
    let key = Key::<Aes256Gcm>::from_slice(b"my-32-byte-secret-key-for-aes-256-gcm");
    let cipher = Aes256Gcm::new(key);
    
    // Nonce
    let nonce = Nonce::from_slice(b"unique-nonce-12");
    
    // Decodificar de base64
    let ciphertext = general_purpose::STANDARD.decode(encrypted_password)?;
    
    // Desencriptar
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Error desencriptando: {}", e))?;
    
    Ok(String::from_utf8(plaintext)?)
}

// Funciones para guardar/cargar configuración SSH
pub fn save_ssh_config(config: &SSHConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    if !config.save_credentials {
        return Ok(());
    }
    
    let encrypted_password = encrypt_password(&config.password)?;
    let encrypted_config = EncryptedSSHConfig {
        host: config.host.clone(),
        username: config.username.clone(),
        encrypted_password,
        port: config.port,
    };
    
    // Guardar en el directorio de la aplicación
    let config_dir = get_ssh_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    
    let config_file = config_dir.join("ssh_config.json");
    let json = serde_json::to_string_pretty(&encrypted_config)?;
    fs::write(config_file, json)?;
    
    Ok(())
}

pub fn load_ssh_config() -> Result<Option<SSHConfig>, Box<dyn std::error::Error>> {
    use std::fs;

    let config_file = get_ssh_config_dir()?.join("ssh_config.json");
    
    if !config_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(config_file)?;
    let encrypted_config: EncryptedSSHConfig = serde_json::from_str(&content)?;
    
    let password = decrypt_password(&encrypted_config.encrypted_password)?;
    
    Ok(Some(SSHConfig {
        host: encrypted_config.host,
        username: encrypted_config.username,
        password,
        port: encrypted_config.port,
        save_credentials: true,
    }))
}

fn get_ssh_config_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use std::env;
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(documents) = env::var("USERPROFILE") {
            Ok(PathBuf::from(format!("{}\\Documents\\Stress", documents)))
        } else {
            Ok(PathBuf::from("."))
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(format!("{}/.stress", home)))
        } else {
            Ok(PathBuf::from("."))
        }
    }
} 