//! State events and changes

use crate::buffer::{AppMode, FilterState, FuzzyFilterState, SearchState};
use crate::ui::state::shadow_state::SearchType;

/// Events that can trigger state changes
#[derive(Debug, Clone)]
pub enum StateEvent {
    /// Mode changed
    ModeChanged { from: AppMode, to: AppMode },

    /// Search started
    SearchStarted { search_type: SearchType },

    /// Search ended (cleared/cancelled)
    SearchEnded { search_type: SearchType },

    /// Search pattern updated
    SearchPatternUpdated {
        search_type: SearchType,
        pattern: String,
    },

    /// Filter activated/deactivated
    FilterToggled {
        filter_type: FilterType,
        active: bool,
    },

    /// Data view updated (query executed)
    DataViewUpdated,

    /// Column operation (hide, pin, sort)
    ColumnOperation { op: ColumnOp },

    /// Viewport changed
    ViewportChanged { row: usize, col: usize },
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Regex,
    Fuzzy,
    Column,
}

#[derive(Debug, Clone)]
pub enum ColumnOp {
    Hide(usize),
    Pin(usize),
    Sort(usize),
    Reset,
}

/// Changes to apply to buffer state
#[derive(Debug, Default, Clone)]
pub struct StateChange {
    pub mode: Option<AppMode>,
    pub search_state: Option<SearchState>,
    pub filter_state: Option<FilterState>,
    pub fuzzy_filter_state: Option<FuzzyFilterState>,
    pub clear_all_searches: bool,
}

impl StateChange {
    /// Create a change that clears all search states
    pub fn clear_searches() -> Self {
        Self {
            search_state: Some(SearchState::default()),
            filter_state: Some(FilterState::default()),
            fuzzy_filter_state: Some(FuzzyFilterState::default()),
            clear_all_searches: true,
            ..Default::default()
        }
    }

    /// Create a mode change
    pub fn mode(mode: AppMode) -> Self {
        Self {
            mode: Some(mode),
            ..Default::default()
        }
    }

    /// Combine with another change
    pub fn and(mut self, other: StateChange) -> Self {
        if other.mode.is_some() {
            self.mode = other.mode;
        }
        if other.search_state.is_some() {
            self.search_state = other.search_state;
        }
        if other.filter_state.is_some() {
            self.filter_state = other.filter_state;
        }
        if other.fuzzy_filter_state.is_some() {
            self.fuzzy_filter_state = other.fuzzy_filter_state;
        }
        if other.clear_all_searches {
            self.clear_all_searches = true;
        }
        self
    }
}
