use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::System;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

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
    pub interval_ms: u64,
    pub max_history: usize,
    pub monitor_cpu: bool,
    pub monitor_memory: bool,
    pub monitor_disk: bool,
    pub monitor_network: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_ms: 1000,
            max_history: 300,
            monitor_cpu: true,
            monitor_memory: true,
            monitor_disk: true,
            monitor_network: true,
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

            while *is_monitoring.lock().unwrap() {
                let timestamp = Local::now();
                let metrics = Self::get_local_metrics(&mut system, &config, timestamp);

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

    fn get_local_metrics(
        system: &mut System,
        config: &MonitoringConfig,
        timestamp: DateTime<Local>,
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
