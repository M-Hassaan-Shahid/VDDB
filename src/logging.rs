use log::{Level, LevelFilter, Metadata, Record};
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

pub struct Logger {
    file: Mutex<File>,
    level: LevelFilter,
}

impl Logger {
    pub fn new(log_path: &Path, level: LevelFilter) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(Logger {
            file: Mutex::new(file),
            level,
        })
    }

    pub fn init(log_path: &Path, level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
        let logger = Logger::new(log_path, level)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(level);
        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
            
            let log_entry = format!(
                "[{}] {} [{}:{}] {} - {}\n",
                timestamp,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.target(),
                record.args()
            );

            if let Ok(mut file) = self.file.lock() {
                let _ = file.write_all(log_entry.as_bytes());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

pub fn setup_logging(log_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(log_dir)?;
    
    let log_file = log_dir.join("vddb.log");
    Logger::init(&log_file, LevelFilter::Info)?;
    
    info!("Logging initialized at {}", log_file.display());
    Ok(())
}

pub fn log_error(error: &dyn std::error::Error) {
    error!("Error occurred: {}", error);
    if let Some(source) = error.source() {
        error!("Caused by: {}", source);
    }
}

pub fn log_operation(operation: &str, details: &str) {
    info!("Operation: {} - {}", operation, details);
}

pub fn log_query(query: &str, params: Option<&str>) {
    if let Some(params) = params {
        info!("Query: {} with params: {}", query, params);
    } else {
        info!("Query: {}", query);
    }
}

pub fn log_transaction(tx_id: &str, action: &str) {
    info!("Transaction {}: {}", tx_id, action);
}

pub fn log_metrics(metric_name: &str, value: f64) {
    info!("Metric {}: {}", metric_name, value);
}

pub fn log_security(event: &str, details: &str) {
    warn!("Security Event: {} - {}", event, details);
}

pub fn log_audit(action: &str, user: &str, resource: &str) {
    info!("Audit: User {} performed {} on {}", user, action, resource);
}

pub fn log_performance(operation: &str, duration_ms: u64) {
    info!("Performance: {} took {}ms", operation, duration_ms);
}

pub fn log_system(event: &str, details: &str) {
    info!("System: {} - {}", event, details);
}

pub fn log_backup(backup_id: &str, status: &str) {
    info!("Backup {}: {}", backup_id, status);
}

pub fn log_recovery(recovery_id: &str, status: &str) {
    info!("Recovery {}: {}", recovery_id, status);
}

pub fn log_replication(replication_id: &str, status: &str) {
    info!("Replication {}: {}", replication_id, status);
}

pub fn log_maintenance(task: &str, status: &str) {
    info!("Maintenance {}: {}", task, status);
}

pub fn log_monitoring(metric: &str, value: &str) {
    info!("Monitoring {}: {}", metric, value);
}

pub fn log_alert(alert_id: &str, severity: &str, message: &str) {
    warn!("Alert {} [{}]: {}", alert_id, severity, message);
}

pub fn log_compliance(requirement: &str, status: &str) {
    info!("Compliance {}: {}", requirement, status);
}

pub fn log_data_quality(metric: &str, value: &str) {
    info!("Data Quality {}: {}", metric, value);
}

pub fn log_data_lineage(source: &str, target: &str) {
    info!("Data Lineage: {} -> {}", source, target);
}

pub fn log_data_governance(policy: &str, status: &str) {
    info!("Data Governance {}: {}", policy, status);
}

pub fn log_data_privacy(action: &str, details: &str) {
    info!("Data Privacy {}: {}", action, details);
}

pub fn log_data_security(event: &str, details: &str) {
    warn!("Data Security {}: {}", event, details);
}

pub fn log_data_compliance(requirement: &str, status: &str) {
    info!("Data Compliance {}: {}", requirement, status);
}

pub fn log_data_audit(action: &str, user: &str, resource: &str) {
    info!("Data Audit: User {} performed {} on {}", user, action, resource);
}

pub fn log_data_monitoring(metric: &str, value: &str) {
    info!("Data Monitoring {}: {}", metric, value);
}

pub fn log_data_alert(alert_id: &str, severity: &str, message: &str) {
    warn!("Data Alert {} [{}]: {}", alert_id, severity, message);
}

pub fn log_data_metrics(metric: &str, value: &str) {
    info!("Data Metrics {}: {}", metric, value);
}

pub fn log_data_performance(operation: &str, duration_ms: u64) {
    info!("Data Performance: {} took {}ms", operation, duration_ms);
}

pub fn log_data_capacity(metric: &str, value: &str) {
    info!("Data Capacity {}: {}", metric, value);
}

pub fn log_data_availability(status: &str) {
    info!("Data Availability: {}", status);
}

pub fn log_data_durability(status: &str) {
    info!("Data Durability: {}", status);
}

pub fn log_data_integrity(check: &str, status: &str) {
    info!("Data Integrity {}: {}", check, status);
}

pub fn log_data_confidentiality(action: &str, details: &str) {
    info!("Data Confidentiality {}: {}", action, details);
} 