use crate::buffer::{AppMode, BufferAPI};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ModeState {
    pub mode: AppMode,
    pub context: StateContext,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct StateContext {
    pub input_text: String,
    pub cursor_position: usize,
    pub scroll_offset: (usize, usize),

    pub selected_row: Option<usize>,
    pub selected_column: usize,

    pub search_pattern: Option<String>,
    pub filter_pattern: Option<String>,
    pub fuzzy_filter_pattern: Option<String>,
    pub column_search_pattern: Option<String>,

    pub table_scroll: (usize, usize),
    pub column_widths: Vec<u16>,

    pub custom_data: HashMap<String, Value>,
}

pub struct StateManager {
    mode_stack: Vec<ModeState>,
    current_context: StateContext,
    max_stack_size: usize,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            mode_stack: Vec::new(),
            current_context: StateContext::default(),
            max_stack_size: 10,
        }
    }

    pub fn push_mode(&mut self, new_mode: AppMode, buffer: &mut dyn BufferAPI) {
        let current_state = ModeState {
            mode: buffer.get_mode(),
            context: self.capture_context(buffer),
            timestamp: Instant::now(),
        };

        self.mode_stack.push(current_state);
        if self.mode_stack.len() > self.max_stack_size {
            self.mode_stack.remove(0);
        }

        self.transition_to_mode(new_mode, buffer);
    }

    pub fn pop_mode(&mut self, buffer: &mut dyn BufferAPI) -> bool {
        if let Some(previous_state) = self.mode_stack.pop() {
            self.restore_context(&previous_state.context, buffer);
            buffer.set_mode(previous_state.mode);
            true
        } else {
            buffer.set_mode(AppMode::Command);
            false
        }
    }

    pub fn peek_previous_mode(&self) -> Option<AppMode> {
        self.mode_stack.last().map(|state| state.mode.clone())
    }

    pub fn save_current_state(&mut self, buffer: &dyn BufferAPI) {
        self.current_context = self.capture_context(buffer);
    }

    pub fn restore_current_state(&self, buffer: &mut dyn BufferAPI) {
        self.restore_context(&self.current_context, buffer);
    }

    pub fn get_stack_depth(&self) -> usize {
        self.mode_stack.len()
    }

    pub fn clear_stack(&mut self) {
        self.mode_stack.clear();
    }

    fn capture_context(&self, buffer: &dyn BufferAPI) -> StateContext {
        StateContext {
            input_text: buffer.get_input_text(),
            cursor_position: buffer.get_input_cursor_position(),
            scroll_offset: buffer.get_scroll_offset(),
            selected_row: buffer.get_selected_row(),
            selected_column: buffer.get_current_column(),
            search_pattern: if !buffer.get_search_pattern().is_empty() {
                Some(buffer.get_search_pattern())
            } else {
                None
            },
            filter_pattern: if !buffer.get_filter_pattern().is_empty() {
                Some(buffer.get_filter_pattern())
            } else {
                None
            },
            fuzzy_filter_pattern: if !buffer.get_fuzzy_filter_pattern().is_empty() {
                Some(buffer.get_fuzzy_filter_pattern())
            } else {
                None
            },
            column_search_pattern: {
                // Column search migrated to AppStateContainer
                None
            },
            table_scroll: buffer.get_scroll_offset(),
            column_widths: Vec::new(), // TODO: Implement when column width tracking is added
            custom_data: HashMap::new(),
        }
    }

    fn restore_context(&self, context: &StateContext, buffer: &mut dyn BufferAPI) {
        buffer.set_input_text(context.input_text.clone());
        buffer.set_input_cursor_position(context.cursor_position);
        buffer.set_scroll_offset(context.scroll_offset);

        if let Some(row) = context.selected_row {
            buffer.set_selected_row(Some(row));
        }
        buffer.set_current_column(context.selected_column);

        if let Some(pattern) = &context.search_pattern {
            buffer.set_search_pattern(pattern.clone());
        }
        if let Some(pattern) = &context.filter_pattern {
            buffer.set_filter_pattern(pattern.clone());
        }
        if let Some(pattern) = &context.fuzzy_filter_pattern {
            buffer.set_fuzzy_filter_pattern(pattern.clone());
        }
        if let Some(pattern) = &context.column_search_pattern {
            // Column search migrated to AppStateContainer
            // buffer.set_column_search_pattern(pattern.clone());
        }
    }

    fn transition_to_mode(&self, mode: AppMode, buffer: &mut dyn BufferAPI) {
        buffer.set_mode(mode.clone());

        match mode {
            AppMode::Search => {
                buffer.set_search_pattern(String::new());
            }
            AppMode::Filter => {
                buffer.set_filter_pattern(String::new());
            }
            AppMode::FuzzyFilter => {
                buffer.set_fuzzy_filter_pattern(String::new());
                buffer.set_fuzzy_filter_active(false);
            }
            AppMode::ColumnSearch => {
                // Column search migrated to AppStateContainer
                // buffer.set_column_search_pattern(String::new());
            }
            _ => {}
        }
    }

    pub fn format_debug_info(&self) -> String {
        let mut info = String::from("========== STATE MANAGER ==========\n");
        info.push_str(&format!("Stack Depth: {}\n", self.mode_stack.len()));

        if !self.mode_stack.is_empty() {
            info.push_str("\nMode Stack (oldest to newest):\n");
            for (i, state) in self.mode_stack.iter().enumerate() {
                info.push_str(&format!("  [{}] {:?}\n", i, state.mode));
            }
        }

        info.push_str(&format!("\nCurrent Mode Context:\n"));
        info.push_str(&format!(
            "  Input Length: {}\n",
            self.current_context.input_text.len()
        ));
        info.push_str(&format!(
            "  Cursor: {}\n",
            self.current_context.cursor_position
        ));
        info.push_str(&format!(
            "  Selected Row: {:?}\n",
            self.current_context.selected_row
        ));
        info.push_str(&format!(
            "  Selected Column: {}\n",
            self.current_context.selected_column
        ));

        info
    }
}
