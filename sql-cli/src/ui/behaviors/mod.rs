// Behavioral traits for TUI functionality
// These traits extract specific behaviors from the main TUI to reduce coupling

pub mod export_behavior;
pub mod status_behavior;

pub use export_behavior::{ExportBehavior, ExportFormat};
pub use status_behavior::StatusBehavior;
