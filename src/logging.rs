use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use chrono::Local;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppLogger {
    pub run_id: String,
    pub log_path: PathBuf,
    file: Arc<Mutex<File>>,
    emit_stdout: bool,
}

impl AppLogger {
    pub fn new() -> Result<Self> {
        Self::new_with_base_dir(std::env::current_dir()?.as_path(), true)
    }

    pub fn new_for_tests() -> Self {
        let base = std::env::temp_dir().join("notebooklm_runner_tests");
        Self::new_with_base_dir(&base, false).expect("test logger creation")
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    fn new_with_base_dir(base_dir: &Path, emit_stdout: bool) -> Result<Self> {
        let logs_dir = base_dir.join("logs");
        fs::create_dir_all(&logs_dir)
            .with_context(|| format!("failed to create logs dir: {}", logs_dir.display()))?;

        let date = Local::now().format("%Y%m%d").to_string();
        let log_path = logs_dir.join(format!("notebooklm-{date}.log"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("failed to open log file: {}", log_path.display()))?;

        let run_id = Uuid::new_v4().to_string();
        Ok(Self {
            run_id,
            log_path,
            file: Arc::new(Mutex::new(file)),
            emit_stdout,
        })
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{timestamp}] [{}] [{level}] {message}", self.run_id);
        if self.emit_stdout {
            println!("{line}");
        }
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file, "{line}");
        }
    }
}
