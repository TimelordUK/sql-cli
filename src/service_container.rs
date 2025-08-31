use crate::app_state_container::AppStateContainer;
use crate::debug_service::DebugService;
use std::sync::Arc;

/// Container for shared services that widgets can access
/// This provides dependency injection for widgets
pub struct ServiceContainer {
    /// Debug service for logging and diagnostics
    pub debug_service: DebugService,

    /// Reference to the application state container
    pub state_container: Arc<AppStateContainer>,
}

impl ServiceContainer {
    pub fn new(state_container: Arc<AppStateContainer>) -> Self {
        Self {
            debug_service: DebugService::new(1000), // Keep last 1000 debug entries
            state_container,
        }
    }

    /// Clone the service container (for sharing with widgets)
    pub fn clone_for_widget(&self) -> Self {
        Self {
            debug_service: self.debug_service.clone_service(),
            state_container: Arc::clone(&self.state_container),
        }
    }

    /// Enable debug mode
    pub fn enable_debug(&self) {
        self.debug_service.set_enabled(true);
        self.debug_service
            .info("ServiceContainer", "Debug mode enabled".to_string());
    }

    /// Disable debug mode
    pub fn disable_debug(&self) {
        self.debug_service
            .info("ServiceContainer", "Debug mode disabled".to_string());
        self.debug_service.set_enabled(false);
    }

    /// Toggle debug mode
    pub fn toggle_debug(&self) {
        if self.debug_service.is_enabled() {
            self.disable_debug();
        } else {
            self.enable_debug();
        }
    }

    /// Generate a comprehensive debug dump
    pub fn generate_debug_dump(&self) -> String {
        let mut dump = String::new();

        // Add state container dump
        dump.push_str(&self.state_container.debug_dump());
        dump.push_str("\n");

        // Add debug service log
        dump.push_str(&self.debug_service.generate_dump());
        dump.push_str("\n");

        // Add debug summary
        dump.push_str(&self.debug_service.generate_summary());

        dump
    }
}
