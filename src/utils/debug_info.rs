use crate::buffer::{BufferAPI, SortOrder, SortState};
use crate::hybrid_parser::HybridParser;
use chrono::Local;
use serde_json::Value;

/// Handles debug information generation and management
pub struct DebugInfo;

impl DebugInfo {
    /// Generate full debug information including parser state, buffer state, etc.
    pub fn generate_full_debug_simple(
        buffer: &dyn BufferAPI,
        buffer_count: usize,
        buffer_index: usize,
        buffer_names: Vec<String>,
        hybrid_parser: &HybridParser,
        sort_state: &SortState,
        input_text: &str,
        cursor_pos: usize,
        visual_cursor: usize,
        api_url: &str,
    ) -> String {
        let mut debug_info = String::new();

        // Get parser debug info
        debug_info.push_str(&hybrid_parser.get_detailed_debug_info(input_text, cursor_pos));

        // Add input state information
        let input_state = format!(
            "\n========== INPUT STATE ==========\n\
            Input Value Length: {}\n\
            Cursor Position: {}\n\
            Visual Cursor: {}\n\
            Input Mode: Command\n",
            input_text.len(),
            cursor_pos,
            visual_cursor
        );
        debug_info.push_str(&input_state);

        // Add dataset information
        let dataset_info = if let Some(dataview) = buffer.get_dataview() {
            let table_name = dataview.source().name.clone();
            let columns = dataview.column_names();
            format!(
                "\n========== DATASET INFO ==========\n\
                Table Name: {}\n\
                Visible Columns ({}): {}\n\
                Hidden Columns: {}\n",
                table_name,
                columns.len(),
                columns.join(", "),
                dataview.get_hidden_column_names().len()
            )
        } else {
            "\n========== DATASET INFO ==========\nNo DataView available\n".to_string()
        };
        debug_info.push_str(&dataset_info);

        // Add current data statistics
        let data_stats = if let Some(dataview) = buffer.get_dataview() {
            let total_rows = dataview.source().row_count();
            let filtered_rows = dataview.row_count();
            format!(
                "\n========== CURRENT DATA ==========\n\
                Total Rows Loaded: {}\n\
                Filtered Rows: {}\n\
                Has Filter: {}\n\
                Current Column: {}\n\
                Sort State: {}\n",
                total_rows,
                filtered_rows,
                dataview.has_filter(),
                buffer.get_current_column(),
                match sort_state {
                    SortState {
                        column: Some(col),
                        order,
                    } => format!(
                        "Column {} - {}",
                        col,
                        match order {
                            SortOrder::Ascending => "Ascending",
                            SortOrder::Descending => "Descending",
                            SortOrder::None => "None",
                        }
                    ),
                    _ => "None".to_string(),
                }
            )
        } else {
            "\n========== CURRENT DATA ==========\nNo DataView available\n".to_string()
        };
        debug_info.push_str(&data_stats);

        // Add status line info
        let status_line_info = format!(
            "\n========== STATUS LINE INFO ==========\n\
            Current Mode: {:?}\n\
            Case Insensitive: {}\n\
            Compact Mode: {}\n\
            Viewport Lock: {}\n\
            Data Source: {}\n",
            buffer.get_mode(),
            buffer.is_case_insensitive(),
            buffer.is_compact_mode(),
            buffer.is_viewport_lock(),
            buffer.get_last_query_source().unwrap_or("None".to_string()),
        );
        debug_info.push_str(&status_line_info);

        // Add buffer manager debug info
        debug_info.push_str("\n========== BUFFER MANAGER STATE ==========\n");
        debug_info.push_str(&format!("Buffer Manager: INITIALIZED\n"));
        debug_info.push_str(&format!("Number of Buffers: {}\n", buffer_count));
        debug_info.push_str(&format!("Current Buffer Index: {}\n", buffer_index));
        debug_info.push_str(&format!("Has Multiple Buffers: {}\n", buffer_count > 1));

        // Add info about all buffers
        for (i, name) in buffer_names.iter().enumerate() {
            let is_current = i == buffer_index;
            debug_info.push_str(&format!(
                "Buffer {}: {} {}\n",
                i + 1,
                name,
                if is_current { "[CURRENT]" } else { "" }
            ));
        }

        debug_info
    }

    /// Generate complete debug context for current state
    pub fn generate_debug_context(buffer: &dyn BufferAPI) -> String {
        let mut context = String::new();
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        context.push_str(&format!("=== TUI Debug Context - {} ===\n\n", timestamp));

        // Current query info
        context.push_str("CURRENT QUERY:\n");
        let query = buffer.get_query();
        let last_query = buffer.get_last_query();
        let current_query = if !query.is_empty() {
            &query
        } else {
            &last_query
        };
        context.push_str(&format!("{}\n\n", current_query));

        // Buffer state
        context.push_str("BUFFER STATE:\n");
        context.push_str(&format!("- ID: {}\n", buffer.get_id()));
        context.push_str(&format!(
            "- File: {}\n",
            buffer
                .get_file_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "memory".to_string())
        ));
        context.push_str(&format!("- Mode: {:?}\n", buffer.get_mode()));
        context.push_str(&format!(
            "- Case Insensitive: {}\n",
            buffer.is_case_insensitive()
        ));

        // Results info
        if let Some(datatable) = buffer.get_datatable() {
            context.push_str(&format!("\nRESULTS INFO:\n"));
            context.push_str(&format!("- Total rows: {}\n", datatable.row_count()));
            context.push_str(&format!("- Columns: {}\n", datatable.column_count()));
            context.push_str(&format!(
                "- Column names: {}\n",
                datatable.column_names().join(", ")
            ));

            // Filter info
            if buffer.is_filter_active() {
                context.push_str(&format!("\nFILTER:\n"));
                context.push_str(&format!("- Pattern: {}\n", buffer.get_filter_pattern()));
                if let Some(dataview) = buffer.get_dataview() {
                    context.push_str(&format!("- Filtered rows: {}\n", dataview.row_count()));
                }
            }

            if buffer.is_fuzzy_filter_active() {
                context.push_str(&format!("\nFUZZY FILTER:\n"));
                context.push_str(&format!(
                    "- Pattern: {}\n",
                    buffer.get_fuzzy_filter_pattern()
                ));
                let indices = buffer.get_fuzzy_filter_indices();
                context.push_str(&format!("- Matched rows: {}\n", indices.len()));
            }
        }

        // Navigation state
        context.push_str("\nNAVIGATION:\n");
        context.push_str(&format!("- Current row: {:?}\n", buffer.get_selected_row()));
        context.push_str(&format!(
            "- Current column: {}\n",
            buffer.get_current_column()
        ));
        context.push_str(&format!(
            "- Scroll offset: ({}, {})\n",
            buffer.get_scroll_offset().0,
            buffer.get_scroll_offset().1
        ));

        context
    }

    /// Generate a complete test case string that can be pasted into a test file
    pub fn generate_test_case(buffer: &dyn BufferAPI) -> String {
        let query = buffer.get_query();
        let last_query = buffer.get_last_query();
        let current_query = if !query.is_empty() {
            &query
        } else {
            &last_query
        };

        let mut test_case = String::new();
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        // Header comment with session info
        test_case.push_str(&format!(
            "// Test case generated from TUI session at {}\n",
            timestamp
        ));
        test_case.push_str(&format!(
            "// Buffer: {} (ID: {})\n",
            buffer
                .get_file_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "memory".to_string()),
            buffer.get_id()
        ));

        if let Some(datatable) = buffer.get_datatable() {
            test_case.push_str(&format!(
                "// Results: {} rows, {} columns\n",
                datatable.row_count(),
                datatable.column_count()
            ));
        }

        test_case.push_str("\n#[test]\n");
        test_case.push_str("fn test_yanked_from_tui_session() -> anyhow::Result<()> {\n");
        test_case.push_str("    let mut harness = QueryReplayHarness::new();\n\n");

        test_case.push_str("    harness.add_query(CapturedQuery {\n");
        test_case.push_str(&format!(
            "        description: \"Captured from TUI session {}\".to_string(),\n",
            timestamp
        ));

        // Add data file path
        if let Some(file_path) = buffer.get_file_path() {
            test_case.push_str(&format!(
                "        data_file: \"{}\".to_string(),\n",
                file_path.to_string_lossy()
            ));
        } else {
            test_case.push_str("        data_file: \"data/trades.json\".to_string(),\n");
        }

        // Add query
        test_case.push_str(&format!(
            "        query: \"{}\".to_string(),\n",
            current_query.replace('"', "\\\"")
        ));

        // Add expected results
        if let Some(datatable) = buffer.get_datatable() {
            test_case.push_str(&format!(
                "        expected_row_count: {},\n",
                datatable.row_count()
            ));

            // Add column names
            test_case.push_str("        expected_columns: vec![\n");
            for column_name in datatable.column_names() {
                test_case.push_str(&format!("            \"{}\".to_string(), \n", column_name));
            }
            test_case.push_str("        ],\n");
        } else {
            test_case.push_str("        expected_row_count: 0,\n");
            test_case.push_str("        expected_columns: vec![],\n");
            test_case.push_str("        expected_first_row: None,\n");
        }

        test_case.push_str(&format!(
            "        case_insensitive: {},\n",
            buffer.is_case_insensitive()
        ));
        test_case.push_str("    });\n\n");

        test_case.push_str("    // Run the test\n");
        test_case.push_str("    harness.run_all_tests()?;\n\n");
        test_case.push_str("    println!(\"✅ Yanked query test passed!\");\n");
        test_case.push_str("    Ok(())\n");
        test_case.push_str("}\n");

        test_case
    }

    /// Convert a serde_json::Value to Rust code representation
    fn value_to_rust_code(value: &Value) -> String {
        match value {
            Value::String(s) => format!(
                "serde_json::Value::String(\"{}\".to_string())",
                s.replace('"', "\\\"")
            ),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    format!("serde_json::Value::Number(serde_json::Number::from({}))", i)
                } else if let Some(f) = n.as_f64() {
                    format!(
                        "serde_json::Value::Number(serde_json::Number::from_f64({}).unwrap())",
                        f
                    )
                } else {
                    format!(
                        "serde_json::Value::Number(serde_json::Number::from_str(\"{}\").unwrap())",
                        n
                    )
                }
            }
            Value::Bool(b) => format!("serde_json::Value::Bool({})", b),
            Value::Null => "serde_json::Value::Null".to_string(),
            _ => format!("serde_json::json!({})", value),
        }
    }

    /// Generate buffer state summary for status messages
    pub fn generate_buffer_summary(buffer: &dyn BufferAPI) -> String {
        let mut summary = Vec::new();

        summary.push(format!("Buffer #{}", buffer.get_id()));

        if let Some(path) = buffer.get_file_path() {
            summary.push(format!(
                "File: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }

        if let Some(datatable) = buffer.get_datatable() {
            summary.push(format!("{} rows", datatable.row_count()));

            if buffer.is_filter_active() {
                if let Some(dataview) = buffer.get_dataview() {
                    summary.push(format!("{} filtered", dataview.row_count()));
                }
            }

            if buffer.is_fuzzy_filter_active() {
                let indices = buffer.get_fuzzy_filter_indices();
                summary.push(format!("{} fuzzy matches", indices.len()));
            }
        }

        summary.join(" | ")
    }

    /// Generate query execution debug info
    pub fn generate_query_debug(query: &str, error: Option<&str>) -> String {
        let mut debug = String::new();
        let timestamp = Local::now().format("%H:%M:%S%.3f");

        debug.push_str(&format!("[{}] Query execution:\n", timestamp));
        debug.push_str(&format!("Query: {}\n", query));

        if let Some(err) = error {
            debug.push_str(&format!("Error: {}\n", err));
        } else {
            debug.push_str("Status: Success\n");
        }

        debug
    }
}

/// Manages debug view scrolling and navigation
pub struct DebugView {
    pub content: String,
    pub scroll_offset: u16,
}

impl DebugView {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.scroll_offset = 0; // Reset scroll when content changes
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.get_max_scroll();
        if (self.scroll_offset as usize) < max_scroll {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    pub fn page_down(&mut self) {
        let max_scroll = self.get_max_scroll();
        self.scroll_offset = (self.scroll_offset + 10).min(max_scroll as u16);
    }

    pub fn go_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn go_to_bottom(&mut self) {
        self.scroll_offset = self.get_max_scroll() as u16;
    }

    pub fn get_max_scroll(&self) -> usize {
        let line_count = self.content.lines().count();
        line_count.saturating_sub(10) // Assuming 10 visible lines
    }

    pub fn get_visible_lines(&self, height: usize) -> Vec<String> {
        self.content
            .lines()
            .skip(self.scroll_offset as usize)
            .take(height)
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for DebugView {
    fn default() -> Self {
        Self::new()
    }
}
