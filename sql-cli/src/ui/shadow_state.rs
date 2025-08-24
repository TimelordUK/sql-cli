//! Shadow State Manager - Observes state transitions without controlling them
//!
//! This module runs in parallel to the existing state system, observing and
//! logging state changes to help us understand patterns before migrating to
//! centralized state management.

use crate::buffer::AppMode;
use std::collections::VecDeque;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Simplified application state for observation
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Command/query input mode
    Command,
    /// Results navigation mode
    Results,
    /// Any search mode active
    Search { search_type: SearchType },
    /// Help mode
    Help,
    /// Debug view
    Debug,
    /// History search mode
    History,
    /// Jump to row mode
    JumpToRow,
    /// Column statistics view
    ColumnStats,
}

/// Types of search that can be active
#[derive(Debug, Clone, PartialEq)]
pub enum SearchType {
    Vim,    // / search
    Column, // Column name search
    Data,   // Data content search
    Fuzzy,  // Fuzzy filter
}

/// Shadow state manager that observes but doesn't control
pub struct ShadowStateManager {
    /// Current observed state
    state: AppState,

    /// Previous state for transition tracking
    previous_state: Option<AppState>,

    /// History of state transitions
    history: VecDeque<StateTransition>,

    /// Count of transitions observed
    transition_count: usize,

    /// Track if we're in sync with actual state
    discrepancies: Vec<String>,
}

#[derive(Debug, Clone)]
struct StateTransition {
    timestamp: Instant,
    from: AppState,
    to: AppState,
    trigger: String,
}

impl ShadowStateManager {
    pub fn new() -> Self {
        info!(target: "shadow_state", "Shadow state manager initialized");

        Self {
            state: AppState::Command,
            previous_state: None,
            history: VecDeque::with_capacity(100),
            transition_count: 0,
            discrepancies: Vec::new(),
        }
    }

    /// Observe a mode change from the existing system
    pub fn observe_mode_change(&mut self, mode: AppMode, trigger: &str) {
        let new_state = self.mode_to_state(mode.clone());

        // Only log if state actually changed
        if new_state != self.state {
            self.transition_count += 1;

            info!(target: "shadow_state",
                "[#{}] {} -> {} (trigger: {})",
                self.transition_count,
                self.state_display(&self.state),
                self.state_display(&new_state),
                trigger
            );

            // Record transition
            let transition = StateTransition {
                timestamp: Instant::now(),
                from: self.state.clone(),
                to: new_state.clone(),
                trigger: trigger.to_string(),
            };

            self.history.push_back(transition);
            if self.history.len() > 100 {
                self.history.pop_front();
            }

            // Update state
            self.previous_state = Some(self.state.clone());
            self.state = new_state;

            // Log what side effects should happen
            self.log_expected_side_effects();
        } else {
            debug!(target: "shadow_state", 
                "Redundant mode change to {:?} ignored", mode);
        }
    }

    /// Observe search starting
    pub fn observe_search_start(&mut self, search_type: SearchType, trigger: &str) {
        let new_state = AppState::Search {
            search_type: search_type.clone(),
        };

        if !matches!(self.state, AppState::Search { .. }) {
            self.transition_count += 1;

            info!(target: "shadow_state",
                "[#{}] {} -> {:?} search (trigger: {})",
                self.transition_count,
                self.state_display(&self.state),
                search_type,
                trigger
            );

            self.previous_state = Some(self.state.clone());
            self.state = new_state;

            // Note: When we see search start, other searches should be cleared
            warn!(target: "shadow_state",
                "⚠️  Search started - verify other search states were cleared!");
        }
    }

    /// Observe search ending
    pub fn observe_search_end(&mut self, trigger: &str) {
        if matches!(self.state, AppState::Search { .. }) {
            // Return to Results mode (assuming we were in results before search)
            let new_state = AppState::Results;

            info!(target: "shadow_state",
                "[#{}] Exiting search -> {} (trigger: {})",
                self.transition_count,
                self.state_display(&new_state),
                trigger
            );

            self.previous_state = Some(self.state.clone());
            self.state = new_state;

            // Log expected cleanup
            info!(target: "shadow_state", 
                "✓ Expected side effects: Clear search UI, restore navigation keys");
        }
    }

    /// Check if we're in search mode
    pub fn is_search_active(&self) -> bool {
        matches!(self.state, AppState::Search { .. })
    }

    /// Get current search type if active
    pub fn get_search_type(&self) -> Option<SearchType> {
        if let AppState::Search { ref search_type } = self.state {
            Some(search_type.clone())
        } else {
            None
        }
    }

    /// Get display string for status line
    pub fn status_display(&self) -> String {
        format!("[Shadow: {}]", self.state_display(&self.state))
    }

    /// Get debug info about recent transitions
    pub fn debug_info(&self) -> String {
        let mut info = format!(
            "Shadow State Debug (transitions: {})\n",
            self.transition_count
        );
        info.push_str(&format!("Current: {:?}\n", self.state));

        if !self.history.is_empty() {
            info.push_str("\nRecent transitions:\n");
            for transition in self.history.iter().rev().take(5) {
                info.push_str(&format!(
                    "  {:?} ago: {} -> {} ({})\n",
                    transition.timestamp.elapsed(),
                    self.state_display(&transition.from),
                    self.state_display(&transition.to),
                    transition.trigger
                ));
            }
        }

        if !self.discrepancies.is_empty() {
            info.push_str("\n⚠️  Discrepancies detected:\n");
            for disc in self.discrepancies.iter().rev().take(3) {
                info.push_str(&format!("  - {}\n", disc));
            }
        }

        info
    }

    /// Report a discrepancy between shadow and actual state
    pub fn report_discrepancy(&mut self, expected: &str, actual: &str) {
        let msg = format!("Expected: {}, Actual: {}", expected, actual);
        warn!(target: "shadow_state", "Discrepancy: {}", msg);
        self.discrepancies.push(msg);
    }

    // ============= Comprehensive Read Methods =============
    // These methods make shadow state easy to query and will eventually
    // replace all buffer().get_mode() calls

    /// Get the current state
    pub fn get_state(&self) -> &AppState {
        &self.state
    }

    /// Get the current mode (converts state to AppMode for compatibility)
    pub fn get_mode(&self) -> AppMode {
        match &self.state {
            AppState::Command => AppMode::Command,
            AppState::Results => AppMode::Results,
            AppState::Search { search_type } => match search_type {
                SearchType::Column => AppMode::ColumnSearch,
                SearchType::Data => AppMode::Search,
                SearchType::Fuzzy => AppMode::FuzzyFilter,
                SearchType::Vim => AppMode::Search, // Vim search uses Search mode
            },
            AppState::Help => AppMode::Help,
            AppState::Debug => AppMode::Debug,
            AppState::History => AppMode::History,
            AppState::JumpToRow => AppMode::JumpToRow,
            AppState::ColumnStats => AppMode::ColumnStats,
        }
    }

    /// Check if currently in Results mode
    pub fn is_in_results_mode(&self) -> bool {
        matches!(self.state, AppState::Results)
    }

    /// Check if currently in Command mode
    pub fn is_in_command_mode(&self) -> bool {
        matches!(self.state, AppState::Command)
    }

    /// Check if currently in any Search mode
    pub fn is_in_search_mode(&self) -> bool {
        matches!(self.state, AppState::Search { .. })
    }

    /// Check if currently in Help mode
    pub fn is_in_help_mode(&self) -> bool {
        matches!(self.state, AppState::Help)
    }

    /// Check if currently in Debug mode
    pub fn is_in_debug_mode(&self) -> bool {
        matches!(self.state, AppState::Debug)
    }

    /// Check if currently in History mode
    pub fn is_in_history_mode(&self) -> bool {
        matches!(self.state, AppState::History)
    }

    /// Check if currently in JumpToRow mode
    pub fn is_in_jump_mode(&self) -> bool {
        matches!(self.state, AppState::JumpToRow)
    }

    /// Check if currently in ColumnStats mode
    pub fn is_in_column_stats_mode(&self) -> bool {
        matches!(self.state, AppState::ColumnStats)
    }

    /// Check if in column search specifically
    pub fn is_in_column_search(&self) -> bool {
        matches!(
            self.state,
            AppState::Search {
                search_type: SearchType::Column
            }
        )
    }

    /// Check if in data search specifically
    pub fn is_in_data_search(&self) -> bool {
        matches!(
            self.state,
            AppState::Search {
                search_type: SearchType::Data
            }
        )
    }

    /// Check if in fuzzy filter mode specifically
    pub fn is_in_fuzzy_filter(&self) -> bool {
        matches!(
            self.state,
            AppState::Search {
                search_type: SearchType::Fuzzy
            }
        )
    }

    /// Check if in vim search mode specifically
    pub fn is_in_vim_search(&self) -> bool {
        matches!(
            self.state,
            AppState::Search {
                search_type: SearchType::Vim
            }
        )
    }

    /// Get the previous state if any
    pub fn get_previous_state(&self) -> Option<&AppState> {
        self.previous_state.as_ref()
    }

    /// Check if we can navigate (in Results mode)
    pub fn can_navigate(&self) -> bool {
        self.is_in_results_mode()
    }

    /// Check if we can edit (in Command mode or search modes)
    pub fn can_edit(&self) -> bool {
        self.is_in_command_mode() || self.is_in_search_mode()
    }

    /// Get transition count (useful for debugging)
    pub fn get_transition_count(&self) -> usize {
        self.transition_count
    }

    /// Get the last transition if any
    pub fn get_last_transition(&self) -> Option<&StateTransition> {
        self.history.back()
    }

    // Helper methods

    fn mode_to_state(&self, mode: AppMode) -> AppState {
        match mode {
            AppMode::Command => AppState::Command,
            AppMode::Results => AppState::Results,
            AppMode::Search | AppMode::ColumnSearch => {
                // Try to preserve search type if we're already in search
                if let AppState::Search { ref search_type } = self.state {
                    AppState::Search {
                        search_type: search_type.clone(),
                    }
                } else {
                    // Guess based on mode
                    let search_type = match mode {
                        AppMode::ColumnSearch => SearchType::Column,
                        _ => SearchType::Data,
                    };
                    AppState::Search { search_type }
                }
            }
            AppMode::Help => AppState::Help,
            AppMode::Debug | AppMode::PrettyQuery => AppState::Debug,
            AppMode::History => AppState::History,
            AppMode::JumpToRow => AppState::JumpToRow,
            AppMode::ColumnStats => AppState::ColumnStats,
            _ => self.state.clone(), // Preserve current for unknown modes
        }
    }

    fn state_display(&self, state: &AppState) -> String {
        match state {
            AppState::Command => "COMMAND".to_string(),
            AppState::Results => "RESULTS".to_string(),
            AppState::Search { search_type } => format!("SEARCH({:?})", search_type),
            AppState::Help => "HELP".to_string(),
            AppState::Debug => "DEBUG".to_string(),
            AppState::History => "HISTORY".to_string(),
            AppState::JumpToRow => "JUMP_TO_ROW".to_string(),
            AppState::ColumnStats => "COLUMN_STATS".to_string(),
        }
    }

    fn log_expected_side_effects(&self) {
        match (&self.previous_state, &self.state) {
            (Some(AppState::Command), AppState::Results) => {
                debug!(target: "shadow_state", 
                    "Expected side effects: Clear searches, reset viewport, enable nav keys");
            }
            (Some(AppState::Results), AppState::Search { .. }) => {
                debug!(target: "shadow_state",
                    "Expected side effects: Clear other searches, setup search UI");
            }
            (Some(AppState::Search { .. }), AppState::Results) => {
                debug!(target: "shadow_state",
                    "Expected side effects: Clear search UI, restore nav keys");
            }
            _ => {}
        }
    }
}

impl Default for ShadowStateManager {
    fn default() -> Self {
        Self::new()
    }
}
