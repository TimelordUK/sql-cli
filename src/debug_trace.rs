#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl DebugLevel {
    pub fn enabled(&self, level: DebugLevel) -> bool {
        *self >= level
    }
}

#[derive(Clone)]
pub struct DebugContext {
    level: DebugLevel,
    component_filter: Option<Vec<String>>,
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new(DebugLevel::Off)
    }
}

impl DebugContext {
    pub fn new(level: DebugLevel) -> Self {
        Self {
            level,
            component_filter: None,
        }
    }

    pub fn with_filter(mut self, components: Vec<String>) -> Self {
        self.component_filter = Some(components);
        self
    }

    #[inline(always)]
    pub fn is_enabled(&self, level: DebugLevel) -> bool {
        self.level.enabled(level)
    }

    #[inline(always)]
    pub fn should_log(&self, component: &str, level: DebugLevel) -> bool {
        if !self.is_enabled(level) {
            return false;
        }

        if let Some(ref filter) = self.component_filter {
            filter.iter().any(|c| component.starts_with(c))
        } else {
            true
        }
    }
}

pub struct QueryTrace {
    context: DebugContext,
    traces: Vec<TraceEntry>,
}

#[derive(Clone)]
pub struct TraceEntry {
    pub component: String,
    pub phase: String,
    pub message: String,
    pub timing_us: Option<u64>,
}

impl QueryTrace {
    pub fn new(context: DebugContext) -> Self {
        Self {
            context,
            traces: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.context.level != DebugLevel::Off
    }

    #[inline(always)]
    pub fn log<C, P, M>(&mut self, component: C, phase: P, message: M)
    where
        C: Into<String>,
        P: Into<String>,
        M: Into<String>,
    {
        if self.is_enabled() {
            let component = component.into();
            if self.context.should_log(&component, DebugLevel::Debug) {
                self.traces.push(TraceEntry {
                    component,
                    phase: phase.into(),
                    message: message.into(),
                    timing_us: None,
                });
            }
        }
    }

    #[inline(always)]
    pub fn log_timed<C, P, M>(&mut self, component: C, phase: P, message: M, timing_us: u64)
    where
        C: Into<String>,
        P: Into<String>,
        M: Into<String>,
    {
        if self.is_enabled() {
            let component = component.into();
            if self.context.should_log(&component, DebugLevel::Debug) {
                self.traces.push(TraceEntry {
                    component,
                    phase: phase.into(),
                    message: message.into(),
                    timing_us: Some(timing_us),
                });
            }
        }
    }

    pub fn format_output(&self) -> String {
        if self.traces.is_empty() {
            return String::new();
        }

        let mut output = String::from("\n=== QUERY EXECUTION TRACE ===\n");
        let mut current_component = String::new();

        for entry in &self.traces {
            if entry.component != current_component {
                output.push_str(&format!("\n[{}]\n", entry.component));
                current_component = entry.component.clone();
            }

            if let Some(timing) = entry.timing_us {
                output.push_str(&format!(
                    "  {}: {} ({:.3}ms)\n",
                    entry.phase,
                    entry.message,
                    timing as f64 / 1000.0
                ));
            } else {
                output.push_str(&format!("  {}: {}\n", entry.phase, entry.message));
            }
        }

        output.push_str("\n=== END TRACE ===\n");
        output
    }

    pub fn get_traces(&self) -> &[TraceEntry] {
        &self.traces
    }
}

#[macro_export]
macro_rules! trace_log {
    ($trace:expr, $component:expr, $phase:expr, $($arg:tt)*) => {
        if $trace.is_enabled() {
            $trace.log($component, $phase, format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! trace_timed {
    ($trace:expr, $component:expr, $phase:expr, $timing:expr, $($arg:tt)*) => {
        if $trace.is_enabled() {
            $trace.log_timed($component, $phase, format!($($arg)*), $timing);
        }
    };
}

pub struct ScopedTimer {
    start: std::time::Instant,
}

impl ScopedTimer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}
