use std::time::{Duration, Instant};

/// A simple debouncer that tracks when an action should be triggered
/// after a period of inactivity
#[derive(Debug, Clone)]
pub struct Debouncer {
    /// The duration to wait after the last event before triggering
    delay: Duration,
    /// When the last event occurred
    last_event: Option<Instant>,
    /// Whether we have a pending trigger
    pending: bool,
}

impl Debouncer {
    /// Create a new debouncer with the specified delay in milliseconds
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            last_event: None,
            pending: false,
        }
    }

    /// Register that an event occurred
    pub fn trigger(&mut self) {
        self.last_event = Some(Instant::now());
        self.pending = true;
    }

    /// Check if enough time has passed to execute the debounced action
    /// Returns true if the action should be executed
    pub fn should_execute(&mut self) -> bool {
        if !self.pending {
            return false;
        }

        if let Some(last) = self.last_event {
            if last.elapsed() >= self.delay {
                self.pending = false;
                self.last_event = None;
                return true;
            }
        }
        false
    }

    /// Get the time remaining before the action will trigger
    /// Returns None if no action is pending
    pub fn time_remaining(&self) -> Option<Duration> {
        if !self.pending {
            return None;
        }

        self.last_event.map(|last| {
            let elapsed = last.elapsed();
            if elapsed >= self.delay {
                Duration::from_millis(0)
            } else {
                self.delay - elapsed
            }
        })
    }

    /// Reset the debouncer, canceling any pending action
    pub fn reset(&mut self) {
        self.last_event = None;
        self.pending = false;
    }

    /// Check if there's a pending action
    pub fn is_pending(&self) -> bool {
        self.pending
    }
}

/// Trait for widgets that support debounced input
pub trait DebouncedInput {
    /// Called when input changes but should be debounced
    fn on_input_changed(&mut self);

    /// Called when the debounce timer expires and action should be taken
    fn on_debounced_execute(&mut self);

    /// Check if debouncing should trigger execution
    fn check_debounce(&mut self) -> bool;
}
