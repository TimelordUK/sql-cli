use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, Buffer, BufferAPI, BufferManager};
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

    /// Check if search navigation should be handled (for n/N keys)
    pub fn should_handle_search_navigation(state_container: &AppStateContainer) -> bool {
        // Only handle search navigation if there's an active search pattern
        let has_search = !state_container.get_search_pattern().is_empty();

        debug!(
            "StateCoordinator::should_handle_search_navigation: has_search={}",
            has_search
        );

        has_search
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
