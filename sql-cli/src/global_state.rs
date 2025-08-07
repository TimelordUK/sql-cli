use crate::api_client::ApiClient;
use crate::cache::QueryCache;
use crate::config::Config;
use crate::history::CommandHistory;
use crate::hybrid_parser::HybridParser;
use crate::parser::SqlParser;
use crate::sql_highlighter::SqlHighlighter;

/// GlobalState contains all truly application-wide state that is shared across all buffers
/// This includes services, parsers, configuration, and global UI state
pub struct GlobalState {
    // --- Core Services ---
    pub api_client: ApiClient,
    pub sql_parser: SqlParser,
    pub hybrid_parser: HybridParser,
    pub sql_highlighter: SqlHighlighter,
    pub config: Config,
    pub command_history: CommandHistory,
    pub query_cache: Option<QueryCache>,

    // --- Global UI State ---
    pub show_help: bool,
    pub help_scroll: u16,
    pub debug_text: String,
    pub debug_scroll: u16,
    pub input_scroll_offset: u16, // Horizontal scroll for input

    // --- Global Selection/Clipboard ---
    pub selection_mode: SelectionMode,
    pub yank_mode: Option<char>,
    pub last_yanked: Option<(String, String)>, // (description, value)

    // --- Completion & History ---
    pub completion_state: CompletionState,
    pub history_state: HistoryState,

    // --- Global Dialogs ---
    pub jump_to_row_input: String,

    // --- Cache Mode ---
    pub cache_mode: bool,
}

#[derive(Clone, PartialEq)]
pub enum SelectionMode {
    Row,
    Cell,
}

#[derive(Clone)]
pub struct CompletionState {
    pub suggestions: Vec<String>,
    pub current_index: usize,
    pub last_query: String,
    pub last_cursor_pos: usize,
}

#[derive(Clone)]
pub struct HistoryState {
    pub search_query: String,
    pub matches: Vec<(usize, String)>, // (index, command)
    pub selected_index: usize,
}

impl GlobalState {
    pub fn new(api_url: &str, config: Config) -> Self {
        Self {
            api_client: ApiClient::new(api_url),
            sql_parser: SqlParser::new(),
            hybrid_parser: HybridParser::new(),
            sql_highlighter: SqlHighlighter::new(),
            command_history: CommandHistory::new().unwrap_or_default(),
            query_cache: None,
            config,

            show_help: false,
            help_scroll: 0,
            debug_text: String::new(),
            debug_scroll: 0,
            input_scroll_offset: 0,

            selection_mode: SelectionMode::Row,
            yank_mode: None,
            last_yanked: None,

            completion_state: CompletionState {
                suggestions: Vec::new(),
                current_index: 0,
                last_query: String::new(),
                last_cursor_pos: 0,
            },

            history_state: HistoryState {
                search_query: String::new(),
                matches: Vec::new(),
                selected_index: 0,
            },

            jump_to_row_input: String::new(),

            cache_mode: false,
        }
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll = 0; // Reset scroll when opening help
        }
    }

    /// Clear debug text
    pub fn clear_debug(&mut self) {
        self.debug_text.clear();
        self.debug_scroll = 0;
    }

    /// Add line to debug text
    pub fn add_debug_line(&mut self, line: String) {
        if !self.debug_text.is_empty() {
            self.debug_text.push('\n');
        }
        self.debug_text.push_str(&line);
    }

    /// Toggle selection mode between Row and Cell
    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = match self.selection_mode {
            SelectionMode::Row => SelectionMode::Cell,
            SelectionMode::Cell => SelectionMode::Row,
        };
    }

    /// Check if in cell selection mode
    pub fn is_cell_mode(&self) -> bool {
        matches!(self.selection_mode, SelectionMode::Cell)
    }

    /// Toggle cache mode
    pub fn toggle_cache_mode(&mut self) {
        self.cache_mode = !self.cache_mode;
    }

    /// Initialize query cache if not already present
    pub fn init_cache(&mut self, _cache_dir: Option<std::path::PathBuf>) {
        if self.query_cache.is_none() {
            self.query_cache = QueryCache::new().ok();
        }
    }
}
