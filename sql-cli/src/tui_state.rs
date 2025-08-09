#[derive(Clone)]
pub enum SelectionMode {
    Normal,
    Column,
    Block,
    Visual,
}

#[derive(Clone)]
pub struct FilterState {
    pub active: bool,
    pub pattern: String,
    pub cursor_pos: usize,
}

#[derive(Clone)]
pub struct FuzzyFilterState {
    pub active: bool,
    pub pattern: String,
    pub cursor_pos: usize,
}

#[derive(Clone)]
pub struct ColumnSearchState {
    pub active: bool,
    pub column_index: usize,
    pub pattern: String,
    pub cursor_pos: usize,
    pub results: Vec<usize>,
    pub current_match: usize,
}

#[derive(Clone)]
pub struct SearchState {
    pub pattern: String,
    pub current_match: Option<(usize, usize)>, // (row, col)
    pub matches: Vec<(usize, usize)>,
    pub match_index: usize,
}

#[derive(Clone)]
pub struct CompletionState {
    pub items: Vec<String>,
    pub selected_index: usize,
    pub active: bool,
    pub prefix_len: usize,
}

pub struct HistoryState {
    pub active: bool,
    pub input: String,
    pub cursor_pos: usize,
}

#[derive(Clone)]
pub struct TuiState {
    pub filter_state: FilterState,
    pub fuzzy_filter_state: FuzzyFilterState,
    pub column_search_state: ColumnSearchState,
    pub search_state: SearchState,
    pub completion_state: CompletionState,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            active: false,
            pattern: String::new(),
            cursor_pos: 0,
        }
    }
}

impl Default for FuzzyFilterState {
    fn default() -> Self {
        Self {
            active: false,
            pattern: String::new(),
            cursor_pos: 0,
        }
    }
}

impl Default for ColumnSearchState {
    fn default() -> Self {
        Self {
            active: false,
            column_index: 0,
            pattern: String::new(),
            cursor_pos: 0,
            results: Vec::new(),
            current_match: 0,
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            current_match: None,
            matches: Vec::new(),
            match_index: 0,
        }
    }
}

impl Default for CompletionState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            active: false,
            prefix_len: 0,
        }
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            active: false,
            input: String::new(),
            cursor_pos: 0,
        }
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            filter_state: FilterState::default(),
            fuzzy_filter_state: FuzzyFilterState::default(),
            column_search_state: ColumnSearchState::default(),
            search_state: SearchState::default(),
            completion_state: CompletionState::default(),
        }
    }
}
