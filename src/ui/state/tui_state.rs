#[derive(Clone)]
pub enum SelectionMode {
    Normal,
    Column,
    Block,
    Visual,
}

#[derive(Clone, Default)]
pub struct FilterState {
    pub active: bool,
    pub pattern: String,
    pub cursor_pos: usize,
}

#[derive(Clone, Default)]
pub struct FuzzyFilterState {
    pub active: bool,
    pub pattern: String,
    pub cursor_pos: usize,
}

#[derive(Clone, Default)]
pub struct ColumnSearchState {
    pub active: bool,
    pub column_index: usize,
    pub pattern: String,
    pub cursor_pos: usize,
    pub results: Vec<usize>,
    pub current_match: usize,
}

#[derive(Clone, Default)]
pub struct SearchState {
    pub pattern: String,
    pub current_match: Option<(usize, usize)>, // (row, col)
    pub matches: Vec<(usize, usize)>,
    pub match_index: usize,
}

#[derive(Clone, Default)]
pub struct CompletionState {
    pub items: Vec<String>,
    pub selected_index: usize,
    pub active: bool,
    pub prefix_len: usize,
}

#[derive(Default)]
pub struct HistoryState {
    pub active: bool,
    pub input: String,
    pub cursor_pos: usize,
}

#[derive(Clone, Default)]
pub struct TuiState {
    pub filter_state: FilterState,
    pub fuzzy_filter_state: FuzzyFilterState,
    pub column_search_state: ColumnSearchState,
    pub search_state: SearchState,
    pub completion_state: CompletionState,
}
