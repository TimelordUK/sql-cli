//! Adapter to make VimSearchManager work with StateDispatcher

use crate::buffer::{AppMode, Buffer};
use crate::state::{StateEvent, StateSubscriber};
use crate::ui::shadow_state::SearchType;
use crate::ui::vim_search_manager::VimSearchManager;
use tracing::{debug, info};

/// Adapter that connects VimSearchManager to the state dispatcher
pub struct VimSearchAdapter {
    manager: VimSearchManager,
    is_active: bool,
}

impl VimSearchAdapter {
    pub fn new(manager: VimSearchManager) -> Self {
        Self {
            manager,
            is_active: false,
        }
    }

    /// Check if vim search should handle a key based on Buffer state
    pub fn should_handle_key(&self, buffer: &Buffer) -> bool {
        // Check Buffer's state, not internal state
        let in_search_mode = buffer.mode == AppMode::Search;
        let has_pattern = !buffer.search_state.pattern.is_empty();

        debug!(
            "VimSearchAdapter: should_handle_key? mode={:?}, pattern='{}', active={}",
            buffer.mode, buffer.search_state.pattern, self.is_active
        );

        // Only handle keys if we're in search mode OR have an active pattern
        in_search_mode || (self.is_active && has_pattern)
    }

    /// Clear the search manager when search ends
    pub fn clear(&mut self) {
        info!("VimSearchAdapter: Clearing vim search");
        self.manager.clear();
        self.is_active = false;
    }

    /// Get the inner manager
    pub fn manager(&self) -> &VimSearchManager {
        &self.manager
    }

    /// Get mutable reference to inner manager
    pub fn manager_mut(&mut self) -> &mut VimSearchManager {
        &mut self.manager
    }
}

impl StateSubscriber for VimSearchAdapter {
    fn on_state_event(&mut self, event: &StateEvent, buffer: &Buffer) {
        match event {
            StateEvent::SearchStarted { search_type } => {
                if matches!(search_type, SearchType::Vim) {
                    info!("VimSearchAdapter: Activating for vim search");
                    self.is_active = true;
                    self.manager.start_search();
                }
            }

            StateEvent::SearchEnded { search_type } => {
                if matches!(search_type, SearchType::Vim) {
                    info!("VimSearchAdapter: Search ended, clearing");
                    self.clear();
                }
            }

            StateEvent::ModeChanged { from: _, to } => {
                // If we exit to Results mode and search is empty, clear
                if *to == AppMode::Results && buffer.search_state.pattern.is_empty() {
                    if self.is_active {
                        info!(
                            "VimSearchAdapter: Mode changed to Results with empty search, clearing"
                        );
                        self.clear();
                    }
                }

                // If we enter Search mode, activate
                if *to == AppMode::Search {
                    info!("VimSearchAdapter: Mode changed to Search, activating");
                    self.is_active = true;
                    if !self.manager.is_active() {
                        self.manager.start_search();
                    }
                }
            }

            _ => {}
        }
    }

    fn name(&self) -> &str {
        "VimSearchAdapter"
    }
}
