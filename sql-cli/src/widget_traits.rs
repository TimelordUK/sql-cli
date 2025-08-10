/// Trait for widgets that can provide debug information
///
/// All widgets in the TUI should implement this trait to provide
/// consistent debug output for the F5 debug view.
pub trait DebugInfoProvider {
    /// Generate a formatted string containing debug information about the widget's state
    ///
    /// The output should be human-readable and include:
    /// - Widget name/type as a header
    /// - Current state (active/inactive, mode, etc.)
    /// - Any cached or saved data
    /// - Configuration or settings
    /// - Any error states or warnings
    fn debug_info(&self) -> String;

    /// Optional: Get a short one-line summary of the widget state
    /// Useful for compact debug views
    fn debug_summary(&self) -> String {
        "No summary available".to_string()
    }
}

/// Extension trait for collecting debug info from multiple widgets
pub trait DebugInfoCollector {
    /// Collect debug info from all widgets
    fn collect_widget_debug_info(&self) -> String;
}
