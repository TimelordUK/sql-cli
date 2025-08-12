use chrono::Local;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
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
            let message = message.trim();
            if !message.is_empty() {
                // The compact format is: "LEVEL target: message"
                // First, try to extract the level
                let (level, rest) = if message.starts_with("TRACE ") {
                    (Level::TRACE, &message[6..])
                } else if message.starts_with("DEBUG ") {
                    (Level::DEBUG, &message[6..])
                } else if message.starts_with("INFO ") {
                    (Level::INFO, &message[5..])
                } else if message.starts_with("WARN ") {
                    (Level::WARN, &message[5..])
                } else if message.starts_with("ERROR ") {
                    (Level::ERROR, &message[6..])
                } else {
                    // If no level prefix, just store the whole message
                    self.buffer
                        .push(LogEntry::new(Level::INFO, "general", message.to_string()));
                    return Ok(buf.len());
                };

                // Now parse "target: message" from rest
                let (target, msg) = if let Some(colon_pos) = rest.find(':') {
                    let potential_target = &rest[..colon_pos];
                    // Check if this looks like a target (no spaces)
                    if !potential_target.contains(' ') {
                        (potential_target, rest[colon_pos + 1..].trim())
                    } else {
                        ("general", rest)
                    }
                } else {
                    ("general", rest)
                };

                self.buffer
                    .push(LogEntry::new(level, target, msg.to_string()));
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

/// Dual writer that writes to both ring buffer and file
pub struct DualWriter {
    buffer: LogRingBuffer,
    dual_logger: &'static crate::dual_logging::DualLogger,
}

impl DualWriter {
    pub fn new(
        buffer: LogRingBuffer,
        dual_logger: &'static crate::dual_logging::DualLogger,
    ) -> Self {
        Self {
            buffer,
            dual_logger,
        }
    }
}

impl std::io::Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Parse the log message
        if let Ok(message) = std::str::from_utf8(buf) {
            let message = message.trim();
            if !message.is_empty() {
                // The compact format is: "LEVEL target: message"
                // First, try to extract the level
                let (level, rest) = if message.starts_with("TRACE ") {
                    (Level::TRACE, &message[6..])
                } else if message.starts_with("DEBUG ") {
                    (Level::DEBUG, &message[6..])
                } else if message.starts_with("INFO ") {
                    (Level::INFO, &message[5..])
                } else if message.starts_with("WARN ") {
                    (Level::WARN, &message[5..])
                } else if message.starts_with("ERROR ") {
                    (Level::ERROR, &message[6..])
                } else {
                    // Skip lines that are just timestamps or empty
                    if message.starts_with("2025-") || message.starts_with("2024-") {
                        return Ok(buf.len());
                    }
                    // If no level prefix, just store the whole message
                    let entry = LogEntry::new(Level::INFO, "general", message.to_string());
                    self.buffer.push(entry.clone());
                    self.dual_logger.log("INFO", "general", message);
                    return Ok(buf.len());
                };

                // Now parse "target: message" from rest
                let (target, msg) = if let Some(colon_pos) = rest.find(':') {
                    let potential_target = &rest[..colon_pos];
                    // Check if this looks like a target (no spaces)
                    if !potential_target.contains(' ') {
                        (potential_target, rest[colon_pos + 1..].trim())
                    } else {
                        ("general", rest)
                    }
                } else {
                    ("general", rest)
                };

                // Write to ring buffer for F5 display
                self.buffer
                    .push(LogEntry::new(level, target, msg.to_string()));

                // Write to file
                let level_str = match level {
                    Level::TRACE => "TRACE",
                    Level::DEBUG => "DEBUG",
                    Level::INFO => "INFO",
                    Level::WARN => "WARN",
                    Level::ERROR => "ERROR",
                };
                self.dual_logger.log(level_str, target, msg);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.dual_logger.flush();
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for DualWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        Self {
            buffer: self.buffer.clone(),
            dual_logger: self.dual_logger,
        }
    }
}

impl Clone for DualWriter {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            dual_logger: self.dual_logger,
        }
    }
}

/// Global log buffer accessible throughout the application
static LOG_BUFFER: OnceLock<LogRingBuffer> = OnceLock::new();

/// Initialize the global log buffer
pub fn init_log_buffer() -> LogRingBuffer {
    let buffer = LogRingBuffer::new();
    LOG_BUFFER.set(buffer.clone()).ok();
    buffer
}

/// Get the global log buffer
pub fn get_log_buffer() -> Option<LogRingBuffer> {
    LOG_BUFFER.get().cloned()
}

/// Initialize tracing with dual logging (ring buffer + file)
pub fn init_tracing_with_dual_logging() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Initialize the dual logger (ring buffer + file)
    let dual_logger = crate::dual_logging::init_dual_logger();

    // Initialize the ring buffer for F5 display
    let buffer = init_log_buffer();

    // Create a custom writer that writes to both ring buffer and file
    let dual_writer = DualWriter::new(buffer.clone(), dual_logger);

    // Create a subscriber with our dual writer
    let fmt_layer = fmt::layer()
        .with_writer(dual_writer)
        .with_target(true)
        .with_level(true)
        .with_ansi(false)
        .without_time() // We add our own timestamps
        .compact();

    // Set up env filter - default to TRACE for everything to catch all logs
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    // Log initial message
    tracing::info!(target: "EnhancedTuiApp", "Logging system initialized with dual output");
}

/// Initialize tracing with our custom ring buffer writer (legacy)
pub fn init_tracing() -> LogRingBuffer {
    init_tracing_with_dual_logging();
    get_log_buffer().unwrap_or_else(|| LogRingBuffer::new())
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
