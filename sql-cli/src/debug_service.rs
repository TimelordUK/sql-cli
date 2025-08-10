use chrono::Local;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Trait for components that can provide debug information
pub trait DebugProvider {
    /// Get the component's name for identification in logs
    fn component_name(&self) -> &str;

    /// Generate debug information about current state
    fn debug_info(&self) -> String;

    /// Generate a compact summary for the status line
    fn debug_summary(&self) -> Option<String> {
        None
    }
}

/// Service for collecting and managing debug information across the application
pub struct DebugService {
    /// Collected debug entries
    entries: Arc<Mutex<Vec<DebugEntry>>>,

    /// Maximum number of entries to keep
    max_entries: usize,

    /// Whether debug collection is enabled
    enabled: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug)]
pub struct DebugEntry {
    pub timestamp: String,
    pub component: String,
    pub level: DebugLevel,
    pub message: String,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DebugLevel {
    Info,
    Warning,
    Error,
    Trace,
}

impl DebugService {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max_entries,
            enabled: Arc::new(Mutex::new(false)),
        }
    }

    /// Clone the service (for sharing between components)
    pub fn clone_service(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            max_entries: self.max_entries,
            enabled: Arc::clone(&self.enabled),
        }
    }

    /// Enable or disable debug collection
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut e) = self.enabled.lock() {
            *e = enabled;
        }
    }

    /// Check if debug collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.lock().map(|e| *e).unwrap_or(false)
    }

    /// Log a debug message
    pub fn log(
        &self,
        component: &str,
        level: DebugLevel,
        message: String,
        context: Option<String>,
    ) {
        if !self.is_enabled() {
            return;
        }

        let entry = DebugEntry {
            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
            component: component.to_string(),
            level,
            message,
            context,
        };

        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);

            // Trim to max size
            if entries.len() > self.max_entries {
                let remove_count = entries.len() - self.max_entries;
                entries.drain(0..remove_count);
            }
        }
    }

    /// Log an info message
    pub fn info(&self, component: &str, message: String) {
        self.log(component, DebugLevel::Info, message, None);
    }

    /// Log a warning message
    pub fn warn(&self, component: &str, message: String) {
        self.log(component, DebugLevel::Warning, message, None);
    }

    /// Log an error message
    pub fn error(&self, component: &str, message: String) {
        self.log(component, DebugLevel::Error, message, None);
    }

    /// Log a trace message with context
    pub fn trace(&self, component: &str, message: String, context: String) {
        self.log(component, DebugLevel::Trace, message, Some(context));
    }

    /// Get all debug entries
    pub fn get_entries(&self) -> Vec<DebugEntry> {
        self.entries.lock().map(|e| e.clone()).unwrap_or_default()
    }

    /// Get recent entries (last n)
    pub fn get_recent_entries(&self, count: usize) -> Vec<DebugEntry> {
        if let Ok(entries) = self.entries.lock() {
            let start = entries.len().saturating_sub(count);
            entries[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Clear all debug entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    /// Generate a formatted debug dump
    pub fn generate_dump(&self) -> String {
        let mut dump = String::new();
        dump.push_str("=== DEBUG SERVICE LOG ===\n\n");

        if let Ok(entries) = self.entries.lock() {
            if entries.is_empty() {
                dump.push_str("No debug entries collected.\n");
            } else {
                dump.push_str(&format!(
                    "Total entries: {} (max: {})\n\n",
                    entries.len(),
                    self.max_entries
                ));

                for entry in entries.iter() {
                    let level_str = match entry.level {
                        DebugLevel::Info => "INFO ",
                        DebugLevel::Warning => "WARN ",
                        DebugLevel::Error => "ERROR",
                        DebugLevel::Trace => "TRACE",
                    };

                    dump.push_str(&format!(
                        "[{}] {} [{}] {}\n",
                        entry.timestamp, level_str, entry.component, entry.message
                    ));

                    if let Some(ref ctx) = entry.context {
                        dump.push_str(&format!("  Context: {}\n", ctx));
                    }
                }
            }
        }

        dump.push_str("\n=== END DEBUG LOG ===\n");
        dump
    }

    /// Generate a summary of debug entries by component
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("=== DEBUG SUMMARY ===\n\n");

        if let Ok(entries) = self.entries.lock() {
            use std::collections::HashMap;
            let mut component_counts: HashMap<String, (usize, usize, usize)> = HashMap::new();

            for entry in entries.iter() {
                let counts = component_counts
                    .entry(entry.component.clone())
                    .or_insert((0, 0, 0));
                match entry.level {
                    DebugLevel::Error => counts.0 += 1,
                    DebugLevel::Warning => counts.1 += 1,
                    _ => counts.2 += 1,
                }
            }

            for (component, (errors, warnings, others)) in component_counts {
                summary.push_str(&format!(
                    "{}: {} errors, {} warnings, {} info/trace\n",
                    component, errors, warnings, others
                ));
            }
        }

        summary
    }
}

/// Macro for easy debug logging
#[macro_export]
macro_rules! debug_log {
    ($service:expr, $component:expr, $($arg:tt)*) => {
        $service.info($component, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! debug_trace {
    ($service:expr, $component:expr, $msg:expr, $ctx:expr) => {
        $service.trace($component, $msg.to_string(), $ctx.to_string())
    };
}
