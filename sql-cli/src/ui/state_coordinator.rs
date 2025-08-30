use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, Buffer, BufferAPI, BufferManager};
use crate::config::config::Config;
use crate::data::data_view::DataView;
use crate::sql::hybrid_parser::HybridParser;
use crate::ui::viewport_manager::ViewportManager;
use crate::widgets::search_modes_widget::SearchMode;

use tracing::{debug, error};

/// StateCoordinator manages and synchronizes all state components in the TUI
/// This centralizes state management and reduces coupling in the main TUI
pub struct StateCoordinator {
    /// Core application state
    pub state_container: AppStateContainer,

    /// Shadow state for tracking mode transitions
    pub shadow_state: Rc<RefCell<crate::ui::shadow_state::ShadowStateManager>>,

    /// Viewport manager for display state
    pub viewport_manager: Rc<RefCell<Option<ViewportManager>>>,

    /// SQL parser with schema information
    pub hybrid_parser: HybridParser,
}

impl StateCoordinator {
    // ========== STATIC METHODS FOR DELEGATION ==========
    // These methods work with references and can be called without owning the components
    // This allows incremental migration from EnhancedTuiApp

    /// Static version of sync_mode that works with references
    pub fn sync_mode_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        mode: AppMode,
        trigger: &str,
    ) {
        debug!(
            "StateCoordinator::sync_mode_with_refs: Setting mode to {:?} with trigger '{}'",
            mode, trigger
        );

        // Set in AppStateContainer
        state_container.set_mode(mode.clone());

        // Set in current buffer
        if let Some(buffer) = state_container.buffers_mut().current_mut() {
            buffer.set_mode(mode.clone());
        }

        // Observe in shadow state
        shadow_state.borrow_mut().observe_mode_change(mode, trigger);
    }

    /// Static version of update_parser_for_current_buffer
    pub fn update_parser_with_refs(state_container: &AppStateContainer, parser: &mut HybridParser) {
        if let Some(dataview) = state_container.get_buffer_dataview() {
            let table_name = dataview.source().name.clone();
            let columns = dataview.source().column_names();

            debug!(
                "StateCoordinator: Updating parser with {} columns for table '{}'",
                columns.len(),
                table_name
            );
            parser.update_single_table(table_name, columns);
        }
    }

    // ========== CONSTRUCTORS ==========

    pub fn new(
        state_container: AppStateContainer,
        shadow_state: Rc<RefCell<crate::ui::shadow_state::ShadowStateManager>>,
        viewport_manager: Rc<RefCell<Option<ViewportManager>>>,
        hybrid_parser: HybridParser,
    ) -> Self {
        Self {
            state_container,
            shadow_state,
            viewport_manager,
            hybrid_parser,
        }
    }

    // ========== MODE SYNCHRONIZATION ==========

    /// Synchronize mode across all state containers
    /// This ensures AppStateContainer, Buffer, and ShadowState are all in sync
    pub fn sync_mode(&mut self, mode: AppMode, trigger: &str) {
        debug!(
            "StateCoordinator::sync_mode: Setting mode to {:?} with trigger '{}'",
            mode, trigger
        );

        // Set in AppStateContainer
        self.state_container.set_mode(mode.clone());

        // Set in current buffer
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            buffer.set_mode(mode.clone());
        }

        // Observe in shadow state
        self.shadow_state
            .borrow_mut()
            .observe_mode_change(mode, trigger);
    }

    /// Alternative mode setter that goes through shadow state
    pub fn set_mode_via_shadow_state(&mut self, mode: AppMode, trigger: &str) {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            debug!(
                "StateCoordinator::set_mode_via_shadow_state: Setting mode to {:?} with trigger '{}'",
                mode, trigger
            );
            self.shadow_state
                .borrow_mut()
                .set_mode(mode, buffer, trigger);
        } else {
            error!(
                "StateCoordinator::set_mode_via_shadow_state: No buffer available! Cannot set mode to {:?}",
                mode
            );
        }
    }

    // ========== BUFFER SYNCHRONIZATION ==========

    /// Synchronize all state after buffer switch
    /// This should be called after any buffer switch operation
    pub fn sync_after_buffer_switch(&mut self) {
        // For now, just update the parser
        // TODO: Add viewport sync when we refactor viewport management
        self.update_parser_for_current_buffer();
    }

    /// Update parser schema from current buffer's DataView
    pub fn update_parser_for_current_buffer(&mut self) {
        // Update parser schema from DataView
        if let Some(dataview) = self.state_container.get_buffer_dataview() {
            let table_name = dataview.source().name.clone();
            let columns = dataview.source().column_names();

            debug!(
                "StateCoordinator: Updating parser with {} columns for table '{}'",
                columns.len(),
                table_name
            );
            self.hybrid_parser.update_single_table(table_name, columns);
        }
    }

    // ========== SEARCH MODE SYNCHRONIZATION ==========

    /// Enter a search mode with proper state synchronization
    pub fn enter_search_mode(&mut self, mode: SearchMode) -> String {
        debug!("StateCoordinator::enter_search_mode: {:?}", mode);

        // Determine the trigger for this search mode
        let trigger = match mode {
            SearchMode::ColumnSearch => "backslash_column_search",
            SearchMode::Search => "data_search_started",
            SearchMode::FuzzyFilter => "fuzzy_filter_started",
            SearchMode::Filter => "filter_started",
        };

        // Sync mode across all state containers
        self.sync_mode(mode.to_app_mode(), trigger);

        // Also observe the search mode start in shadow state for search-specific tracking
        let search_type = match mode {
            SearchMode::ColumnSearch => crate::ui::shadow_state::SearchType::Column,
            SearchMode::Search => crate::ui::shadow_state::SearchType::Data,
            SearchMode::FuzzyFilter | SearchMode::Filter => {
                crate::ui::shadow_state::SearchType::Fuzzy
            }
        };
        self.shadow_state
            .borrow_mut()
            .observe_search_start(search_type, trigger);

        trigger.to_string()
    }

    // ========== SEARCH CANCELLATION ==========

    /// Cancel search and properly restore state
    /// This handles all the complex state synchronization when Escape is pressed during search
    pub fn cancel_search(&mut self) -> (Option<String>, Option<usize>) {
        debug!("StateCoordinator::cancel_search: Canceling search and restoring state");

        // Clear search state in state container
        self.state_container.clear_search();

        // Observe search end in shadow state
        self.shadow_state
            .borrow_mut()
            .observe_search_end("search_cancelled");

        // Switch back to Results mode with proper synchronization
        self.sync_mode(AppMode::Results, "search_cancelled");

        // Return saved SQL and cursor position for restoration
        // This would come from search widget's saved state
        (None, None)
    }

    /// Static version for delegation pattern with vim search adapter
    pub fn cancel_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        vim_search_adapter: Option<&RefCell<crate::ui::vim_search_adapter::VimSearchAdapter>>,
    ) {
        debug!("StateCoordinator::cancel_search_with_refs: Canceling search and clearing all search state");

        // Clear vim search adapter if provided
        if let Some(adapter) = vim_search_adapter {
            debug!("Clearing vim search adapter state");
            adapter.borrow_mut().clear();
        }

        // Clear search pattern in state container
        state_container.set_search_pattern(String::new());
        state_container.clear_search();

        // Observe search end in shadow state
        shadow_state
            .borrow_mut()
            .observe_search_end("search_cancelled");

        // Sync back to Results mode
        Self::sync_mode_with_refs(
            state_container,
            shadow_state,
            AppMode::Results,
            "vim_search_cancelled",
        );
    }

    /// Check if 'n' key should navigate to next search match
    /// Returns true only if there's an active search (not cancelled with Escape)
    pub fn should_handle_next_match(
        state_container: &AppStateContainer,
        vim_search_adapter: Option<&RefCell<crate::ui::vim_search_adapter::VimSearchAdapter>>,
    ) -> bool {
        // 'n' should only work if there's a search pattern AND it hasn't been cancelled
        let has_search = !state_container.get_search_pattern().is_empty();
        let pattern = state_container.get_search_pattern();

        // Check if vim search is active or navigating
        // After Escape, this will be false
        let vim_active = if let Some(adapter) = vim_search_adapter {
            let adapter_ref = adapter.borrow();
            adapter_ref.is_active() || adapter_ref.is_navigating()
        } else {
            false
        };

        debug!(
            "StateCoordinator::should_handle_next_match: pattern='{}', vim_active={}, result={}",
            pattern,
            vim_active,
            has_search && vim_active
        );

        // Only handle if search exists AND hasn't been cancelled with Escape
        has_search && vim_active
    }

    /// Check if 'N' key should navigate to previous search match
    /// Returns true only if there's an active search (not cancelled with Escape)
    pub fn should_handle_previous_match(
        state_container: &AppStateContainer,
        vim_search_adapter: Option<&RefCell<crate::ui::vim_search_adapter::VimSearchAdapter>>,
    ) -> bool {
        // 'N' should only work if there's a search pattern AND it hasn't been cancelled
        let has_search = !state_container.get_search_pattern().is_empty();
        let pattern = state_container.get_search_pattern();

        // Check if vim search is active or navigating
        // After Escape, this will be false
        let vim_active = if let Some(adapter) = vim_search_adapter {
            let adapter_ref = adapter.borrow();
            adapter_ref.is_active() || adapter_ref.is_navigating()
        } else {
            false
        };

        debug!(
            "StateCoordinator::should_handle_previous_match: pattern='{}', vim_active={}, result={}",
            pattern, vim_active, has_search && vim_active
        );

        // Only handle if search exists AND hasn't been cancelled with Escape
        has_search && vim_active
    }

    /// Complete a search operation (after Apply/Enter is pressed)
    /// This keeps the pattern for n/N navigation but marks search as complete
    pub fn complete_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        vim_search_adapter: Option<&RefCell<crate::ui::vim_search_adapter::VimSearchAdapter>>,
        mode: AppMode,
        trigger: &str,
    ) {
        debug!(
            "StateCoordinator::complete_search_with_refs: Completing search, switching to {:?}",
            mode
        );

        // Note: We intentionally DO NOT clear the search pattern here
        // The pattern remains available for n/N navigation

        // Mark vim search adapter as not actively searching
        // but keep the matches for navigation
        if let Some(adapter) = vim_search_adapter {
            debug!("Marking vim search as complete but keeping matches");
            adapter.borrow_mut().mark_search_complete();
        }

        // Observe search completion in shadow state
        shadow_state
            .borrow_mut()
            .observe_search_end("search_completed");

        // Switch to the target mode (usually Results)
        Self::sync_mode_with_refs(state_container, shadow_state, mode, trigger);
    }

    // ========== DATAVIEW MANAGEMENT ==========

    /// Add a new DataView and coordinate all necessary state updates
    /// This centralizes the complex logic of adding a new data source
    pub fn add_dataview_with_refs(
        state_container: &mut AppStateContainer,
        viewport_manager: &RefCell<Option<ViewportManager>>,
        dataview: DataView,
        source_name: &str,
        config: &Config,
    ) -> Result<(), anyhow::Error> {
        debug!(
            "StateCoordinator::add_dataview_with_refs: Adding DataView for '{}'",
            source_name
        );

        // Create a new buffer with the DataView
        let buffer_id = state_container.buffers().all_buffers().len() + 1;
        let mut buffer = crate::buffer::Buffer::new(buffer_id);

        // Set the DataView directly
        buffer.set_dataview(Some(dataview.clone()));

        // Use just the filename for the buffer name, not the full path
        let buffer_name = std::path::Path::new(source_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(source_name)
            .to_string();
        buffer.set_name(buffer_name.clone());

        // Apply config settings to the buffer
        buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        buffer.set_compact_mode(config.display.compact_mode);
        buffer.set_show_row_numbers(config.display.show_row_numbers);

        debug!(
            "StateCoordinator: Created buffer '{}' with {} rows, {} columns",
            buffer_name,
            dataview.row_count(),
            dataview.column_count()
        );

        // Add the buffer and switch to it
        state_container.buffers_mut().add_buffer(buffer);
        let new_index = state_container.buffers().all_buffers().len() - 1;
        state_container.buffers_mut().switch_to(new_index);

        // Update state container with the DataView
        state_container.set_dataview(Some(dataview.clone()));

        // Update viewport manager with the new DataView
        // Replace the entire ViewportManager with a new one for the DataView
        *viewport_manager.borrow_mut() = Some(ViewportManager::new(Arc::new(dataview.clone())));

        debug!("StateCoordinator: Created new ViewportManager for DataView");

        // Update navigation state with data dimensions
        let row_count = dataview.row_count();
        let column_count = dataview.column_count();
        state_container.update_data_size(row_count, column_count);

        debug!(
            "StateCoordinator: DataView '{}' successfully added and all state synchronized",
            buffer_name
        );

        Ok(())
    }

    // ========== QUERY EXECUTION SYNCHRONIZATION ==========

    /// Switch to Results mode after successful query execution
    pub fn switch_to_results_after_query(&mut self) {
        self.sync_mode(AppMode::Results, "execute_query_success");
    }

    // ========== STATE ACCESS ==========

    /// Get reference to AppStateContainer
    pub fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    /// Get mutable reference to AppStateContainer
    pub fn state_container_mut(&mut self) -> &mut AppStateContainer {
        &mut self.state_container
    }

    /// Get reference to current buffer
    pub fn current_buffer(&self) -> Option<&Buffer> {
        self.state_container.buffers().current()
    }

    /// Get mutable reference to current buffer
    pub fn current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.state_container.buffers_mut().current_mut()
    }

    /// Get reference to buffer manager
    pub fn buffers(&self) -> &BufferManager {
        self.state_container.buffers()
    }

    /// Get mutable reference to buffer manager
    pub fn buffers_mut(&mut self) -> &mut BufferManager {
        self.state_container.buffers_mut()
    }

    /// Get reference to hybrid parser
    pub fn parser(&self) -> &HybridParser {
        &self.hybrid_parser
    }

    /// Get mutable reference to hybrid parser
    pub fn parser_mut(&mut self) -> &mut HybridParser {
        &mut self.hybrid_parser
    }
}
