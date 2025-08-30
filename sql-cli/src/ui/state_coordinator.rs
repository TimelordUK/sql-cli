use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state_container::AppStateContainer;
use crate::buffer::{AppMode, Buffer, BufferAPI, BufferManager};
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
