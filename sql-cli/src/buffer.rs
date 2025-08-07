use crate::api_client::QueryResponse;
use crate::csv_datasource::CsvApiClient;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::TableState;
use regex::Regex;
use std::path::PathBuf;
use tui_input::Input;
use tui_textarea::TextArea;

// Re-define the types we need (these should eventually be moved to a common module)
#[derive(Clone, Debug, PartialEq)]
pub enum AppMode {
    Command,
    Results,
    Search,
    Filter,
    FuzzyFilter,
    ColumnSearch,
    Help,
    History,
    Debug,
    PrettyQuery,
    CacheList,
    JumpToRow,
    ColumnStats,
}

#[derive(Clone, PartialEq)]
pub enum EditMode {
    SingleLine,
    MultiLine,
}

#[derive(Clone, PartialEq, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
    None,
}

#[derive(Clone)]
pub struct SortState {
    pub column: Option<usize>,
    pub order: SortOrder,
}

#[derive(Clone)]
pub struct FilterState {
    pub pattern: String,
    pub regex: Option<Regex>,
    pub active: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            regex: None,
            active: false,
        }
    }
}

pub struct FuzzyFilterState {
    pub pattern: String,
    pub active: bool,
    pub matcher: SkimMatcherV2,
    pub filtered_indices: Vec<usize>,
}

impl Clone for FuzzyFilterState {
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.clone(),
            active: self.active,
            matcher: SkimMatcherV2::default(), // Create new matcher
            filtered_indices: self.filtered_indices.clone(),
        }
    }
}

impl Default for FuzzyFilterState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            active: false,
            matcher: SkimMatcherV2::default(),
            filtered_indices: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct SearchState {
    pub pattern: String,
    pub current_match: Option<(usize, usize)>,
    pub matches: Vec<(usize, usize)>,
    pub match_index: usize,
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

#[derive(Clone)]
pub struct ColumnSearchState {
    pub pattern: String,
    pub matching_columns: Vec<usize>,
    pub current_match: usize,
}

impl Default for ColumnSearchState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            matching_columns: Vec::new(),
            current_match: 0,
        }
    }
}

pub type ColumnStatistics = std::collections::BTreeMap<String, String>;

/// Represents a single buffer/tab with its own independent state
#[derive(Clone)]
pub struct Buffer {
    /// Unique identifier for this buffer
    pub id: usize,

    /// File path if loaded from file
    pub file_path: Option<PathBuf>,

    /// Display name (filename or "untitled")
    pub name: String,

    /// Whether this buffer has unsaved changes
    pub modified: bool,

    // --- Data State ---
    pub csv_client: Option<CsvApiClient>,
    pub csv_mode: bool,
    pub csv_table_name: String,
    pub results: Option<QueryResponse>,
    pub cached_data: Option<Vec<serde_json::Value>>,

    // --- UI State ---
    pub mode: AppMode,
    pub edit_mode: EditMode,
    pub input: Input,
    pub textarea: TextArea<'static>,
    pub table_state: TableState,
    pub last_results_row: Option<usize>,
    pub last_scroll_offset: (usize, usize),

    // --- Query State ---
    pub last_query: String,
    pub status_message: String,

    // --- Filter/Search State ---
    pub sort_state: SortState,
    pub filter_state: FilterState,
    pub fuzzy_filter_state: FuzzyFilterState,
    pub search_state: SearchState,
    pub column_search_state: ColumnSearchState,
    pub filtered_data: Option<Vec<Vec<String>>>,

    // --- View State ---
    pub column_widths: Vec<u16>,
    pub scroll_offset: (usize, usize),
    pub current_column: usize,
    pub pinned_columns: Vec<usize>,
    pub column_stats: Option<ColumnStatistics>,
    pub compact_mode: bool,
    pub viewport_lock: bool,
    pub viewport_lock_row: Option<usize>,
    pub show_row_numbers: bool,
    pub case_insensitive: bool,

    // --- Misc State ---
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
    pub kill_ring: String,
    pub last_visible_rows: usize,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new(id: usize) -> Self {
        Self {
            id,
            file_path: None,
            name: format!("[Buffer {}]", id),
            modified: false,

            csv_client: None,
            csv_mode: false,
            csv_table_name: String::new(),
            results: None,
            cached_data: None,

            mode: AppMode::Command,
            edit_mode: EditMode::SingleLine,
            input: Input::default(),
            textarea: TextArea::default(),
            table_state: TableState::default(),
            last_results_row: None,
            last_scroll_offset: (0, 0),

            last_query: String::new(),
            status_message: String::new(),

            sort_state: SortState {
                column: None,
                order: SortOrder::None,
            },
            filter_state: FilterState::default(),
            fuzzy_filter_state: FuzzyFilterState::default(),
            search_state: SearchState::default(),
            column_search_state: ColumnSearchState::default(),
            filtered_data: None,

            column_widths: Vec::new(),
            scroll_offset: (0, 0),
            current_column: 0,
            pinned_columns: Vec::new(),
            column_stats: None,
            compact_mode: false,
            viewport_lock: false,
            viewport_lock_row: None,
            show_row_numbers: false,
            case_insensitive: false,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 30,
        }
    }

    /// Create a buffer from a CSV file
    pub fn from_csv(
        id: usize,
        path: PathBuf,
        csv_client: CsvApiClient,
        table_name: String,
    ) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.csv")
            .to_string();

        let mut buffer = Self::new(id);
        buffer.file_path = Some(path);
        buffer.name = name;
        buffer.csv_client = Some(csv_client);
        buffer.csv_mode = true;
        buffer.csv_table_name = table_name;

        buffer
    }

    /// Create a buffer from a JSON file
    pub fn from_json(
        id: usize,
        path: PathBuf,
        csv_client: CsvApiClient,
        table_name: String,
    ) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.json")
            .to_string();

        let mut buffer = Self::new(id);
        buffer.file_path = Some(path);
        buffer.name = name;
        buffer.csv_client = Some(csv_client);
        buffer.csv_mode = true;
        buffer.csv_table_name = table_name;

        buffer
    }

    /// Get display name for tab bar
    pub fn display_name(&self) -> String {
        if self.modified {
            format!("{}*", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Get short name for tab bar (truncated if needed)
    pub fn short_name(&self, max_len: usize) -> String {
        let display = self.display_name();
        if display.len() <= max_len {
            display
        } else {
            format!("{}...", &display[..max_len.saturating_sub(3)])
        }
    }

    /// Check if buffer has a specific file open
    pub fn has_file(&self, path: &PathBuf) -> bool {
        self.file_path.as_ref() == Some(path)
    }
}

/// Manages multiple buffers and switching between them
pub struct BufferManager {
    buffers: Vec<Buffer>,
    current_buffer_index: usize,
    next_buffer_id: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            current_buffer_index: 0,
            next_buffer_id: 1,
        }
    }

    /// Add a new buffer and make it current
    pub fn add_buffer(&mut self, mut buffer: Buffer) -> usize {
        buffer.id = self.next_buffer_id;
        self.next_buffer_id += 1;

        let index = self.buffers.len();
        self.buffers.push(buffer);
        self.current_buffer_index = index;
        index
    }

    /// Get current buffer
    pub fn current(&self) -> Option<&Buffer> {
        self.buffers.get(self.current_buffer_index)
    }

    /// Get current buffer mutably
    pub fn current_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.current_buffer_index)
    }

    /// Switch to next buffer
    pub fn next_buffer(&mut self) {
        if !self.buffers.is_empty() {
            self.current_buffer_index = (self.current_buffer_index + 1) % self.buffers.len();
        }
    }

    /// Switch to previous buffer
    pub fn prev_buffer(&mut self) {
        if !self.buffers.is_empty() {
            if self.current_buffer_index == 0 {
                self.current_buffer_index = self.buffers.len() - 1;
            } else {
                self.current_buffer_index -= 1;
            }
        }
    }

    /// Switch to buffer by index
    pub fn switch_to(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.current_buffer_index = index;
        }
    }

    /// Close current buffer
    pub fn close_current(&mut self) -> bool {
        if self.buffers.len() <= 1 {
            return false; // Don't close last buffer
        }

        self.buffers.remove(self.current_buffer_index);

        // Adjust current index if needed
        if self.current_buffer_index >= self.buffers.len() {
            self.current_buffer_index = self.buffers.len() - 1;
        }

        true
    }

    /// Find buffer by file path
    pub fn find_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.buffers.iter().position(|b| b.has_file(path))
    }

    /// Get all buffers for display
    pub fn all_buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    /// Get current buffer index
    pub fn current_index(&self) -> usize {
        self.current_buffer_index
    }

    /// Check if we have multiple buffers
    pub fn has_multiple(&self) -> bool {
        self.buffers.len() > 1
    }
}
