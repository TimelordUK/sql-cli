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

        // Also clear column search state
        state_container.clear_column_search();

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

    // ========== FILTER MANAGEMENT ==========

    /// Apply text filter and coordinate all state updates
    /// Returns the number of matching rows
    pub fn apply_text_filter_with_refs(
        state_container: &mut AppStateContainer,
        pattern: &str,
    ) -> usize {
        let case_insensitive = state_container.is_case_insensitive();

        debug!(
            "StateCoordinator::apply_text_filter_with_refs: Applying text filter with pattern '{}', case_sensitive: {}",
            pattern, !case_insensitive
        );

        // Apply filter to DataView and get results
        let rows_after = if let Some(dataview) = state_container.get_buffer_dataview_mut() {
            let rows_before = dataview.row_count();
            dataview.apply_text_filter(pattern, !case_insensitive);
            let rows_after = dataview.row_count();
            debug!(
                "Text filter: {} rows before, {} rows after",
                rows_before, rows_after
            );
            rows_after
        } else {
            debug!("No DataView available for text filtering");
            0
        };

        // Update status message
        let status = if pattern.is_empty() {
            "Filter cleared".to_string()
        } else {
            format!("Filter applied: '{}' - {} matches", pattern, rows_after)
        };
        state_container.set_status_message(status);

        debug!(
            "StateCoordinator: Text filter applied - {} matches for pattern '{}'",
            rows_after, pattern
        );

        rows_after
    }

    /// Apply fuzzy filter and coordinate all state updates
    /// Returns (match_count, filter_indices)
    pub fn apply_fuzzy_filter_with_refs(
        state_container: &mut AppStateContainer,
        viewport_manager: &RefCell<Option<ViewportManager>>,
    ) -> (usize, Vec<usize>) {
        let pattern = state_container.get_fuzzy_filter_pattern();
        let case_insensitive = state_container.is_case_insensitive();

        debug!(
            "StateCoordinator::apply_fuzzy_filter_with_refs: Applying fuzzy filter with pattern '{}', case_insensitive: {}",
            pattern, case_insensitive
        );

        // Apply filter to DataView and get results
        let (match_count, indices) =
            if let Some(dataview) = state_container.get_buffer_dataview_mut() {
                dataview.apply_fuzzy_filter(&pattern, case_insensitive);
                let match_count = dataview.row_count();
                let indices = dataview.get_fuzzy_filter_indices();
                (match_count, indices)
            } else {
                (0, Vec::new())
            };

        // Update state based on filter results
        if pattern.is_empty() {
            state_container.set_fuzzy_filter_active(false);
            state_container.set_status_message("Fuzzy filter cleared".to_string());
        } else {
            state_container.set_fuzzy_filter_active(true);
            state_container.set_status_message(format!("Fuzzy filter: {} matches", match_count));

            // Reset navigation to first match if we have results
            if match_count > 0 {
                // Get current column offset to preserve horizontal scroll
                let col_offset = state_container.get_scroll_offset().1;

                // Reset to first row of filtered results
                state_container.set_selected_row(Some(0));
                state_container.set_scroll_offset((0, col_offset));
                state_container.set_table_selected_row(Some(0));

                // Update navigation state
                let mut nav = state_container.navigation_mut();
                nav.selected_row = 0;
                nav.scroll_offset.0 = 0;

                // Update ViewportManager if present
                if let Ok(mut vm_borrow) = viewport_manager.try_borrow_mut() {
                    if let Some(ref mut vm) = *vm_borrow {
                        vm.set_crosshair_row(0);
                        vm.set_scroll_offset(0, col_offset);
                        debug!(
                            "StateCoordinator: Reset viewport to first match (row 0) with {} total matches",
                            match_count
                        );
                    }
                }
            }
        }

        debug!(
            "StateCoordinator: Fuzzy filter applied - {} matches, pattern: '{}'",
            match_count, pattern
        );

        (match_count, indices)
    }

    // ========== TABLE STATE MANAGEMENT ==========

    /// Reset all table-related state to initial values
    /// This is typically called when switching data sources or after queries
    pub fn reset_table_state_with_refs(
        state_container: &mut AppStateContainer,
        viewport_manager: &RefCell<Option<ViewportManager>>,
    ) {
        debug!("StateCoordinator::reset_table_state_with_refs: Resetting all table state");

        // Reset navigation state
        state_container.navigation_mut().reset();
        state_container.set_table_selected_row(Some(0));
        state_container.reset_navigation_state();

        // Reset ViewportManager if it exists
        if let Ok(mut vm_borrow) = viewport_manager.try_borrow_mut() {
            if let Some(ref mut vm) = *vm_borrow {
                vm.reset_crosshair();
                debug!("StateCoordinator: Reset ViewportManager crosshair position");
            }
        }

        // Clear filter state
        state_container.filter_mut().clear();

        // Clear search state
        {
            let mut search = state_container.search_mut();
            search.pattern.clear();
            search.current_match = 0;
            search.matches.clear();
            search.is_active = false;
        }

        // Clear fuzzy filter state
        state_container.clear_fuzzy_filter_state();

        // Clear column search state (added for completeness)
        state_container.clear_column_search();

        debug!("StateCoordinator: Table state reset complete");
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

    // ========== QUERY MANAGEMENT ==========

    /// Set SQL query and update all related state
    /// This centralizes the complex logic of setting up SQL query state
    pub fn set_sql_query_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        parser: &mut HybridParser,
        table_name: &str,
        raw_table_name: &str,
        config: &Config,
    ) -> String {
        debug!(
            "StateCoordinator::set_sql_query_with_refs: Setting query for table '{}'",
            table_name
        );

        // Create the initial SQL query
        let auto_query = format!("SELECT * FROM {}", table_name);

        // Update the hybrid parser with the table information
        if let Some(dataview) = state_container
            .buffers()
            .current()
            .and_then(|b| b.get_dataview())
        {
            let columns = dataview.column_names();
            parser.update_single_table(table_name.to_string(), columns);

            // Set status message
            let display_msg = if raw_table_name != table_name {
                format!(
                    "Loaded '{}' as table '{}' with {} columns. Query pre-populated.",
                    raw_table_name,
                    table_name,
                    dataview.column_count()
                )
            } else {
                format!(
                    "Loaded table '{}' with {} columns. Query pre-populated.",
                    table_name,
                    dataview.column_count()
                )
            };
            state_container.set_status_message(display_msg);
        }

        // Set initial mode based on config
        let initial_mode = match config.behavior.start_mode.to_lowercase().as_str() {
            "results" => AppMode::Results,
            "command" => AppMode::Command,
            _ => AppMode::Results, // Default to results if invalid config
        };

        // Sync mode across all state containers
        Self::sync_mode_with_refs(
            state_container,
            shadow_state,
            initial_mode.clone(),
            "initial_load_from_config",
        );

        debug!(
            "StateCoordinator: SQL query set to '{}', mode set to {:?}",
            auto_query, initial_mode
        );

        auto_query
    }

    /// Handle query execution and all related state changes
    /// Returns true if application should exit
    pub fn handle_execute_query_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        query: &str,
    ) -> Result<bool, anyhow::Error> {
        debug!(
            "StateCoordinator::handle_execute_query_with_refs: Processing query '{}'",
            query
        );

        let trimmed = query.trim();

        if trimmed.is_empty() {
            state_container
                .set_status_message("Empty query - please enter a SQL command".to_string());
            return Ok(false);
        }

        // Check for special commands
        if trimmed == ":help" {
            state_container.set_help_visible(true);
            Self::sync_mode_with_refs(
                state_container,
                shadow_state,
                AppMode::Help,
                "help_requested",
            );
            state_container.set_status_message("Help Mode - Press ESC to return".to_string());
            Ok(false)
        } else if trimmed == ":exit" || trimmed == ":quit" || trimmed == ":q" {
            Ok(true) // Signal exit
        } else if trimmed == ":tui" {
            state_container.set_status_message("Already in TUI mode".to_string());
            Ok(false)
        } else {
            // Regular SQL query - execution handled by TUI
            state_container.set_status_message(format!("Processing query: '{}'", trimmed));
            Ok(false)
        }
    }

    // ========== QUERY EXECUTION SYNCHRONIZATION ==========

    /// Switch to Results mode after successful query execution
    pub fn switch_to_results_after_query(&mut self) {
        self.sync_mode(AppMode::Results, "execute_query_success");
    }

    /// Static version for delegation
    pub fn switch_to_results_after_query_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
    ) {
        Self::sync_mode_with_refs(
            state_container,
            shadow_state,
            AppMode::Results,
            "execute_query_success",
        );
    }

    // ========== SEARCH STATE TRANSITIONS ==========

    /// Apply filter search with proper state coordination
    pub fn apply_filter_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        pattern: &str,
    ) {
        debug!(
            "StateCoordinator::apply_filter_search_with_refs: Applying filter with pattern '{}'",
            pattern
        );

        // Update filter pattern in multiple places for consistency
        state_container.set_filter_pattern(pattern.to_string());
        state_container
            .filter_mut()
            .set_pattern(pattern.to_string());

        // Log the state before and after
        let before_count = state_container
            .get_buffer_dataview()
            .map(|v| v.source().row_count())
            .unwrap_or(0);

        debug!(
            "StateCoordinator: Filter search - case_insensitive={}, rows_before={}",
            state_container.is_case_insensitive(),
            before_count
        );

        // Note: The actual apply_filter() call will be done by TUI
        // as it has the implementation

        debug!(
            "StateCoordinator: Filter pattern set to '{}', mode={:?}",
            pattern,
            shadow_state.borrow().get_mode()
        );
    }

    /// Apply fuzzy filter search with proper state coordination
    pub fn apply_fuzzy_filter_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        pattern: &str,
    ) {
        debug!(
            "StateCoordinator::apply_fuzzy_filter_search_with_refs: Applying fuzzy filter with pattern '{}'",
            pattern
        );

        let before_count = state_container
            .get_buffer_dataview()
            .map(|v| v.source().row_count())
            .unwrap_or(0);

        // Set the fuzzy filter pattern
        state_container.set_fuzzy_filter_pattern(pattern.to_string());

        debug!(
            "StateCoordinator: Fuzzy filter - rows_before={}, pattern='{}'",
            before_count, pattern
        );

        // Note: The actual apply_fuzzy_filter() call will be done by TUI
        // After applying, we can check the results

        debug!(
            "StateCoordinator: Fuzzy filter pattern set, mode={:?}",
            shadow_state.borrow().get_mode()
        );
    }

    /// Apply column search with proper state coordination
    pub fn apply_column_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        pattern: &str,
    ) {
        debug!(
            "StateCoordinator::apply_column_search_with_refs: Starting column search with pattern '{}'",
            pattern
        );

        // Start column search through AppStateContainer
        state_container.start_column_search(pattern.to_string());

        // Ensure we stay in ColumnSearch mode
        let current_mode = shadow_state.borrow().get_mode();
        if current_mode != AppMode::ColumnSearch {
            debug!(
                "StateCoordinator: WARNING - Mode was {:?}, restoring to ColumnSearch",
                current_mode
            );
            Self::sync_mode_with_refs(
                state_container,
                shadow_state,
                AppMode::ColumnSearch,
                "column_search_mode_restore",
            );
        }

        debug!(
            "StateCoordinator: Column search started with pattern '{}'",
            pattern
        );
    }

    // ========== HISTORY SEARCH COORDINATION ==========

    /// Start history search with proper state transitions
    pub fn start_history_search_with_refs(
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::shadow_state::ShadowStateManager>,
        current_input: String,
    ) -> (String, usize) {
        debug!("StateCoordinator::start_history_search_with_refs: Starting history search");

        let mut input_to_use = current_input;

        // If in Results mode, switch to Command mode first
        if shadow_state.borrow().is_in_results_mode() {
            let last_query = state_container.get_last_query();
            if !last_query.is_empty() {
                input_to_use = last_query.clone();
                debug!(
                    "StateCoordinator: Using last query for history search: '{}'",
                    last_query
                );
            }

            // Transition to Command mode
            state_container.set_mode(AppMode::Command);
            shadow_state
                .borrow_mut()
                .observe_mode_change(AppMode::Command, "history_search_from_results");
            state_container.set_table_selected_row(None);
        }

        // Start history search with the input
        state_container.start_history_search(input_to_use.clone());

        // Note: update_history_matches_in_container() will be called by TUI
        // as it has the schema context implementation

        // Get match count for status
        let match_count = state_container.history_search().matches.len();
        state_container.set_status_message(format!("History search: {} matches", match_count));

        // Switch to History mode
        state_container.set_mode(AppMode::History);
        shadow_state
            .borrow_mut()
            .observe_mode_change(AppMode::History, "history_search_started");

        debug!(
            "StateCoordinator: History search started with {} matches, mode=History",
            match_count
        );

        (input_to_use, match_count)
    }

    // ========== NAVIGATION COORDINATION ==========

    /// Coordinate goto first row with vim search state
    pub fn goto_first_row_with_refs(
        state_container: &mut AppStateContainer,
        vim_search_adapter: Option<&RefCell<crate::ui::vim_search_adapter::VimSearchAdapter>>,
        viewport_manager: Option<&RefCell<Option<ViewportManager>>>,
    ) {
        debug!("StateCoordinator::goto_first_row_with_refs: Going to first row");

        // Set position to first row
        state_container.set_table_selected_row(Some(0));
        state_container.set_scroll_offset((0, 0));

        // If vim search is active and navigating, reset to first match
        if let Some(adapter) = vim_search_adapter {
            let is_navigating = adapter.borrow().is_navigating();

            if is_navigating {
                if let Some(viewport_ref) = viewport_manager {
                    let mut vim_search_mut = adapter.borrow_mut();
                    let mut viewport_borrow = viewport_ref.borrow_mut();
                    if let Some(ref mut viewport) = *viewport_borrow {
                        if let Some(first_match) = vim_search_mut.reset_to_first_match(viewport) {
                            debug!(
                                "StateCoordinator: Reset vim search to first match at ({}, {})",
                                first_match.row, first_match.col
                            );
                        }
                    }
                }
            }
        }
    }

    /// Coordinate goto last row
    pub fn goto_last_row_with_refs(state_container: &mut AppStateContainer) {
        debug!("StateCoordinator::goto_last_row_with_refs: Going to last row");

        // Get total rows from dataview if available
        if let Some(dataview) = state_container.get_buffer_dataview() {
            let last_row = dataview.row_count().saturating_sub(1);
            state_container.set_table_selected_row(Some(last_row));

            // Adjust scroll to show last row
            // This is simplified - actual viewport calculation would be more complex
            let scroll_row = last_row.saturating_sub(20); // Assume ~20 visible rows
            state_container.set_scroll_offset((scroll_row, 0));
        }
    }

    /// Coordinate goto specific row
    pub fn goto_row_with_refs(state_container: &mut AppStateContainer, row: usize) {
        debug!("StateCoordinator::goto_row_with_refs: Going to row {}", row);

        // Validate row is within bounds
        if let Some(dataview) = state_container.get_buffer_dataview() {
            let max_row = dataview.row_count().saturating_sub(1);
            let target_row = row.min(max_row);

            state_container.set_table_selected_row(Some(target_row));

            // Adjust scroll if needed
            let current_scroll = state_container.get_scroll_offset().0;
            if target_row < current_scroll || target_row > current_scroll + 20 {
                // Center the target row in viewport
                let new_scroll = target_row.saturating_sub(10);
                state_container.set_scroll_offset((new_scroll, 0));
            }
        }
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
