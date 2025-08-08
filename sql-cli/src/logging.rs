use chrono::Local;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 1000;

/// A log entry with timestamp and message
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: Level, target: &str, message: String) -> Self {
        Self {
            timestamp: Local::now().format("%H:%M:%S.%3f").to_string(),
            level: level.to_string().to_uppercase(),
            target: target.to_string(),
            message,
        }
    }

    /// Format for display in debug view
    pub fn format_for_display(&self) -> String {
        format!(
            "[{}] {} [{}] {}",
            self.timestamp, self.level, self.target, self.message
        )
    }
}

/// Thread-safe ring buffer for log entries
#[derive(Clone)]
pub struct LogRingBuffer {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl LogRingBuffer {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().rev().take(count).rev().cloned().collect()
    }

    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.len()
    }
}

/// Custom writer that captures logs to our ring buffer
pub struct RingBufferWriter {
    buffer: LogRingBuffer,
}

impl RingBufferWriter {
    pub fn new(buffer: LogRingBuffer) -> Self {
        Self { buffer }
    }
}

impl std::io::Write for RingBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Parse the log message and add to ring buffer
        if let Ok(message) = std::str::from_utf8(buf) {
            // Simple parsing - in production you'd want more robust parsing
            let message = message.trim();
            if !message.is_empty() {
                // Extract level and target from the formatted message
                // Format is typically: "2024-01-01T12:00:00.000Z  INFO target: message"
                let parts: Vec<&str> = message.splitn(4, ' ').collect();
                if parts.len() >= 3 {
                    let level_str = parts[1].trim();
                    let level = match level_str {
                        "TRACE" => Level::TRACE,
                        "DEBUG" => Level::DEBUG,
                        "INFO" => Level::INFO,
                        "WARN" => Level::WARN,
                        "ERROR" => Level::ERROR,
                        _ => Level::INFO,
                    };

                    let target_and_msg = parts[2..].join(" ");
                    let (target, msg) = if let Some(colon_pos) = target_and_msg.find(':') {
                        let target = &target_and_msg[..colon_pos];
                        let msg = target_and_msg[colon_pos + 1..].trim();
                        (target, msg)
                    } else {
                        ("unknown", target_and_msg.as_str())
                    };

                    self.buffer
                        .push(LogEntry::new(level, target, msg.to_string()));
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for RingBufferWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl Clone for RingBufferWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

/// Global log buffer accessible throughout the application
static mut LOG_BUFFER: Option<LogRingBuffer> = None;

/// Initialize the global log buffer
pub fn init_log_buffer() -> LogRingBuffer {
    let buffer = LogRingBuffer::new();
    unsafe {
        LOG_BUFFER = Some(buffer.clone());
    }
    buffer
}

/// Get the global log buffer
pub fn get_log_buffer() -> Option<LogRingBuffer> {
    unsafe { LOG_BUFFER.clone() }
}

/// Initialize tracing with our custom ring buffer writer
pub fn init_tracing() -> LogRingBuffer {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let buffer = init_log_buffer();
    let writer = RingBufferWriter::new(buffer.clone());

    // Create a subscriber with our custom writer
    let fmt_layer = fmt::layer()
        .with_writer(writer)
        .with_target(true)
        .with_level(true)
        .with_ansi(false)
        .compact();

    // Set up env filter - default to INFO, but allow override with RUST_LOG
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("sql_cli=debug,enhanced_tui=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    buffer
}

/// Convenience macros for common operations
#[macro_export]
macro_rules! trace_operation {
    ($op:expr) => {
        tracing::debug!(target: "operation", "{}", $op);
    };
}

#[macro_export]
macro_rules! trace_query {
    ($query:expr) => {
        tracing::info!(target: "query", "Executing: {}", $query);
    };
}

#[macro_export]
macro_rules! trace_buffer_switch {
    ($from:expr, $to:expr) => {
        tracing::debug!(target: "buffer", "Switching from buffer {} to {}", $from, $to);
    };
}

#[macro_export]
macro_rules! trace_key {
    ($key:expr) => {
        tracing::trace!(target: "input", "Key: {:?}", $key);
    };
}
