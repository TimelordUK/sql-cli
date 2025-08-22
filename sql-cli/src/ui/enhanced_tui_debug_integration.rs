use crate::debug::Priority;
/// Integration of new debug registry system with existing toggle_debug_mode
/// This file provides a gradual migration path from the old debug system to the new trait-based system
use crate::ui::enhanced_tui::EnhancedTuiApp;

impl EnhancedTuiApp {
    /// Generate debug info using the new registry system
    /// This can be called alongside the existing debug generation to compare outputs
    pub fn generate_registry_debug_info(&mut self) -> String {
        // First, register all current providers
        self.register_debug_providers();

        // Generate the report
        let mut debug_report = self.debug_registry.generate_debug_report();

        // Add any sections that aren't yet migrated to providers
        // These will be gradually moved to their own providers

        // Add parser info (not yet migrated)
        if let Some(buffer) = self.buffer_manager.current() {
            let query = buffer.input_manager.get_text();
            if !query.is_empty() {
                let parser_info = self.debug_generate_parser_info(&query);
                debug_report.insert_str(0, &parser_info);
            }
        }

        // Add navigation timing (not yet migrated)
        if !self.navigation_timings.is_empty() {
            debug_report.push_str("\n========== NAVIGATION TIMING ==========\n");
            debug_report.push_str(&format!(
                "Last {} navigation timings:\n",
                self.navigation_timings.len()
            ));
            for timing in &self.navigation_timings {
                debug_report.push_str(&format!("  {}\n", timing));
            }
            // Calculate average
            if self.navigation_timings.len() > 0 {
                let total_ms: f64 = self
                    .navigation_timings
                    .iter()
                    .filter_map(|s| self.debug_extract_timing(s))
                    .sum();
                let avg_ms = total_ms / self.navigation_timings.len() as f64;
                debug_report.push_str(&format!("Average navigation time: {:.3}ms\n", avg_ms));
            }
        }

        // Add render timing (not yet migrated)
        if !self.render_timings.is_empty() {
            debug_report.push_str("\n========== RENDER TIMING ==========\n");
            debug_report.push_str(&format!(
                "Last {} render timings:\n",
                self.render_timings.len()
            ));
            for timing in &self.render_timings {
                debug_report.push_str(&format!("  {}\n", timing));
            }
            // Calculate average render time
            if self.render_timings.len() > 0 {
                let total_ms: f64 = self
                    .render_timings
                    .iter()
                    .filter_map(|s| self.debug_extract_timing(s))
                    .sum();
                let avg_ms = total_ms / self.render_timings.len() as f64;
                debug_report.push_str(&format!("Average render time: {:.3}ms\n", avg_ms));
            }
        }

        // Add trace logs if available
        debug_report.push_str(&self.debug_generate_trace_logs());

        // Add state change logs
        debug_report.push_str(&self.debug_generate_state_logs());

        debug_report
    }

    /// Toggle debug mode with optional use of new registry system
    /// Set use_registry to true to use the new system, false for legacy
    pub fn toggle_debug_mode_with_registry(&mut self, use_registry: bool) {
        if use_registry {
            // Use new registry-based system
            let should_exit_debug = {
                if let Some(buffer) = self.buffer_manager.current() {
                    buffer.mode == crate::buffer::AppMode::Debug
                } else {
                    false
                }
            };

            if should_exit_debug {
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    buffer.mode = crate::buffer::AppMode::Command;
                }
            } else {
                // Enter debug mode with registry-generated content
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    buffer.mode = crate::buffer::AppMode::Debug;
                }

                // Generate debug info using registry
                let debug_info = self.generate_registry_debug_info();

                // Update the debug widget with new content
                self.debug_widget.set_content(debug_info);
            }
        } else {
            // Use existing implementation
            self.toggle_debug_mode();
        }
    }

    /// Check if we should use the new debug system
    /// This can be controlled by config or environment variable in the future
    pub fn should_use_registry_debug(&self) -> bool {
        // For now, default to false to maintain stability
        // Can be changed to true when ready to fully migrate
        false
    }
}
