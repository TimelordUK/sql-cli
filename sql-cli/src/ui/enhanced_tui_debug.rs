use crate::debug::{
    BufferDebugProvider, BufferManagerDebugProvider, DataViewDebugProvider, MemoryDebugProvider,
};
use crate::ui::enhanced_tui::EnhancedTuiApp;
use std::sync::Arc;

impl EnhancedTuiApp {
    /// Register all available debug providers with the debug registry
    pub fn register_debug_providers(&mut self) {
        // Clear any existing providers first
        self.debug_registry.clear();

        // Register memory tracker
        let memory_provider = Arc::new(MemoryDebugProvider::new(self.memory_tracker.clone()));
        self.debug_registry.register(memory_provider);

        // Register BufferManager provider
        let buffers: Vec<Arc<dyn crate::buffer::BufferAPI>> = self
            .buffer_manager
            .all_buffers()
            .iter()
            .map(|b| Arc::new(b.clone()) as Arc<dyn crate::buffer::BufferAPI>)
            .collect();

        let buffer_manager_provider = Arc::new(BufferManagerDebugProvider::new(
            buffers,
            self.buffer_manager.current_index(),
        ));
        self.debug_registry.register(buffer_manager_provider);

        // Register current buffer provider
        if let Some(buffer) = self.buffer_manager.current() {
            let buffer_provider = Arc::new(BufferDebugProvider::new(
                Arc::new(buffer.clone()) as Arc<dyn crate::buffer::BufferAPI>
            ));
            self.debug_registry.register(buffer_provider);

            // Register DataView provider if available
            if let Some(dataview) = buffer.dataview.as_ref() {
                let dataview_provider =
                    Arc::new(DataViewDebugProvider::new(Arc::new(dataview.clone())));
                self.debug_registry.register(dataview_provider);
            }
        }

        // Note: ViewportManager cannot be registered directly as it doesn't implement Clone
        // and we can't get ownership. This would need architectural changes to support.
        // For now, ViewportManager debug info is gathered directly in the main debug function.

        // Record a memory snapshot after registering providers
        self.memory_tracker.record_snapshot();
    }

    /// Generate debug report using the registry
    pub fn generate_debug_report(&self) -> String {
        self.debug_registry.generate_debug_report()
    }

    /// Get list of registered debug providers
    pub fn list_debug_providers(&self) -> Vec<String> {
        self.debug_registry.list_providers()
    }
}
