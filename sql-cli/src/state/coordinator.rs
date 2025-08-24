//! State coordination trait for Buffer

use crate::buffer::{AppMode, Buffer, FilterState, FuzzyFilterState, SearchState};
use crate::state::events::{StateChange, StateEvent};
use crate::ui::shadow_state::SearchType;
use tracing::{debug, info};

/// Trait for coordinating state changes
pub trait StateCoordinator {
    /// Process a state event and return changes to apply
    fn process_event(&self, event: &StateEvent) -> StateChange;

    /// Apply a state change to the buffer
    fn apply_change(&mut self, change: StateChange);

    /// Emit an event (for notification)
    fn emit_event(&self, event: StateEvent);
}

impl StateCoordinator for Buffer {
    fn process_event(&self, event: &StateEvent) -> StateChange {
        match event {
            StateEvent::ModeChanged { from, to } => self.process_mode_change(from, to),
            StateEvent::SearchStarted { search_type } => self.process_search_start(search_type),
            StateEvent::SearchEnded { search_type } => self.process_search_end(search_type),
            _ => StateChange::default(),
        }
    }

    fn apply_change(&mut self, change: StateChange) {
        debug!("Buffer applying state change: {:?}", change);

        if let Some(mode) = change.mode {
            self.mode = mode;
        }

        if change.clear_all_searches {
            info!("Buffer: Clearing all search states");
            self.search_state = SearchState::default();
            self.filter_state = FilterState::default();
            self.fuzzy_filter_state = FuzzyFilterState::default();
        } else {
            if let Some(search) = change.search_state {
                self.search_state = search;
            }
            if let Some(filter) = change.filter_state {
                self.filter_state = filter;
            }
            if let Some(fuzzy) = change.fuzzy_filter_state {
                self.fuzzy_filter_state = fuzzy;
            }
        }
    }

    fn emit_event(&self, event: StateEvent) {
        // This will be connected to the dispatcher
        debug!("Buffer emitting event: {:?}", event);
    }
}

impl Buffer {
    /// Process mode change and determine required state changes
    fn process_mode_change(&self, from: &AppMode, to: &AppMode) -> StateChange {
        debug!("Processing mode change: {:?} -> {:?}", from, to);

        match (from, to) {
            // Exiting any search mode to Results
            (AppMode::Search, AppMode::Results)
            | (AppMode::Filter, AppMode::Results)
            | (AppMode::FuzzyFilter, AppMode::Results)
            | (AppMode::ColumnSearch, AppMode::Results) => {
                info!("Exiting search mode -> clearing all searches");
                StateChange::clear_searches().and(StateChange::mode(AppMode::Results))
            }

            // Entering search mode from Results
            (AppMode::Results, AppMode::Search) => {
                // Clear other search types
                let mut change = StateChange::mode(AppMode::Search);
                change.filter_state = Some(FilterState::default());
                change.fuzzy_filter_state = Some(FuzzyFilterState::default());
                change
            }

            // Default mode change
            _ => StateChange::mode(to.clone()),
        }
    }

    /// Process search start event
    fn process_search_start(&self, search_type: &SearchType) -> StateChange {
        debug!("Processing search start: {:?}", search_type);

        match search_type {
            SearchType::Vim => {
                let mut change = StateChange::mode(AppMode::Search);
                // Clear other search types
                change.filter_state = Some(FilterState::default());
                change.fuzzy_filter_state = Some(FuzzyFilterState::default());
                change
            }
            SearchType::Column => StateChange::mode(AppMode::ColumnSearch),
            SearchType::Fuzzy => {
                let mut change = StateChange::mode(AppMode::FuzzyFilter);
                // Clear other search types
                change.search_state = Some(SearchState::default());
                change.filter_state = Some(FilterState::default());
                change
            }
            SearchType::Data => {
                let mut change = StateChange::mode(AppMode::Filter);
                // Clear other search types
                change.search_state = Some(SearchState::default());
                change.fuzzy_filter_state = Some(FuzzyFilterState::default());
                change
            }
        }
    }

    /// Process search end event
    fn process_search_end(&self, search_type: &SearchType) -> StateChange {
        info!("Processing search end: {:?}", search_type);

        // Clear the specific search type and return to Results
        match search_type {
            SearchType::Vim => {
                let mut change = StateChange::mode(AppMode::Results);
                change.search_state = Some(SearchState::default());
                change
            }
            SearchType::Column => StateChange::mode(AppMode::Results),
            SearchType::Fuzzy => {
                let mut change = StateChange::mode(AppMode::Results);
                change.fuzzy_filter_state = Some(FuzzyFilterState::default());
                change
            }
            SearchType::Data => {
                let mut change = StateChange::mode(AppMode::Results);
                change.filter_state = Some(FilterState::default());
                change
            }
        }
    }
}
