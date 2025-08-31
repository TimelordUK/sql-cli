use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::logging::{LogEntry, LogRingBuffer};

/// Global dual logger instance
static DUAL_LOGGER: OnceLock<DualLogger> = OnceLock::new();

/// Cross-platform log directory
fn get_log_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // Windows: Use %TEMP% or %LOCALAPPDATA%
        std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("TEMP"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\temp"))
            .join("sql-cli")
    } else {
        // Unix-like: Use /tmp or $HOME/.local/share
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("sql-cli")
                .join("logs")
        } else {
            PathBuf::from("/tmp").join("sql-cli")
        }
    }
}

/// Dual logger that writes to both ring buffer and file
pub struct DualLogger {
    ring_buffer: LogRingBuffer,
    log_file: Arc<Mutex<Option<File>>>,
    log_path: PathBuf,
}

impl DualLogger {
    pub fn new() -> Self {
        let log_dir = get_log_dir();

        // Create log directory if it doesn't exist
        let _ = std::fs::create_dir_all(&log_dir);

        // Create timestamped log file
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let log_filename = format!("sql-cli_{}.log", timestamp);
        let log_path = log_dir.join(&log_filename);

        // Create a "latest.log" pointer - different approach for different OS
        let latest_path = log_dir.join("latest.log");

        #[cfg(unix)]
        {
            // On Unix, use symlink (doesn't require elevated privileges)
            let _ = std::fs::remove_file(&latest_path); // Remove old symlink
            let _ = std::os::unix::fs::symlink(&log_path, &latest_path);
        }

        #[cfg(windows)]
        {
            // On Windows, write a text file with the path to the actual log
            // This avoids needing admin rights for symlinks
            let pointer_content = format!("Current log file: {}\n", log_path.display());
            let _ = std::fs::write(&latest_path, pointer_content);

            // Also create a batch file for easy tailing
            let tail_script = log_dir.join("tail-latest.bat");
            let script_content = format!(
                "@echo off\necho Tailing: {}\ntype \"{}\" && timeout /t 2 >nul && goto :loop\n:loop\ntype \"{}\" 2>nul\ntimeout /t 1 >nul\ngoto :loop",
                log_path.display(),
                log_path.display(),
                log_path.display()
            );
            let _ = std::fs::write(&tail_script, script_content);
        }

        // Open log file
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        // Don't print here - let main.rs handle the announcement

        Self {
            ring_buffer: LogRingBuffer::new(),
            log_file: Arc::new(Mutex::new(log_file)),
            log_path,
        }
    }

    /// Log a message to both ring buffer and file
    pub fn log(&self, level: &str, target: &str, message: &str) {
        let entry = LogEntry::new(
            match level {
                "ERROR" => tracing::Level::ERROR,
                "WARN" => tracing::Level::WARN,
                "INFO" => tracing::Level::INFO,
                "DEBUG" => tracing::Level::DEBUG,
                _ => tracing::Level::TRACE,
            },
            target,
            message.to_string(),
        );

        // To ring buffer (for F5 display)
        self.ring_buffer.push(entry.clone());

        // To file (for persistent history)
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                let log_line = format!(
                    "[{}] {} [{}] {}\n",
                    entry.timestamp, entry.level, entry.target, entry.message
                );
                let _ = file.write_all(log_line.as_bytes());
                let _ = file.flush(); // Important for crash debugging!
            }
        }

        // Also to stderr if DEBUG env var set
        if std::env::var("SQL_CLI_DEBUG").is_ok() {
            eprintln!("{}", entry.format_for_display());
        }
    }

    /// Get the ring buffer for F5 display
    pub fn ring_buffer(&self) -> &LogRingBuffer {
        &self.ring_buffer
    }

    /// Get the log file path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Force flush the log file
    pub fn flush(&self) {
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize the global dual logger
pub fn init_dual_logger() -> &'static DualLogger {
    DUAL_LOGGER.get_or_init(|| DualLogger::new())
}

/// Get the global dual logger
pub fn get_dual_logger() -> Option<&'static DualLogger> {
    DUAL_LOGGER.get()
}

/// Convenience macro for logging
#[macro_export]
macro_rules! dual_log {
    ($level:expr, $target:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::dual_logging::get_dual_logger() {
            logger.log($level, $target, &format!($($arg)*));
        }
    }};
}

/// Log at different levels
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{ dual_log!("ERROR", module_path!(), $($arg)*); }};
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{ dual_log!("WARN", module_path!(), $($arg)*); }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{ dual_log!("INFO", module_path!(), $($arg)*); }};
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{ dual_log!("DEBUG", module_path!(), $($arg)*); }};
}
