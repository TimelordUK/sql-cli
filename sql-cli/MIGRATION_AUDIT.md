# Buffer Migration Audit

This document tracks the migration of fields from EnhancedTuiApp to Buffer via BufferAPI.

## Migration Categories

### ✅ Already in Buffer (via BufferAPI)
These fields already exist in Buffer and have BufferAPI methods:

- [x] `mode: AppMode` → `get_mode()`, `set_mode()`
- [x] `status_message: String` → `get_status_message()`, `set_status_message()`
- [x] `results: Option<QueryResponse>` → `get_results()`, `set_results()`
- [x] `table_state: TableState` → `get_selected_row()`, `set_selected_row()`
- [x] `input: Input` → `get_input_value()`, `set_input_value()`, etc.
- [x] `current_column: usize` → `get_current_column()`, `set_current_column()`
- [x] `scroll_offset: (usize, usize)` → `get_scroll_offset()`, `set_scroll_offset()`
- [x] `pinned_columns: Vec<usize>` → `get_pinned_columns()`, `add_pinned_column()`, etc.
- [x] `compact_mode: bool` → `is_compact_mode()`, `set_compact_mode()`
- [x] `show_row_numbers: bool` → `is_show_row_numbers()`, `set_show_row_numbers()`
- [x] `filter_state: FilterState` → `get_filter_pattern()`, `is_filter_active()`, etc.
- [x] `search_state: SearchState` → `get_search_pattern()`, `get_search_matches()`, etc.
- [x] `sort_state: SortState` → `get_sort_column()`, `get_sort_order()`, etc.
- [x] `csv_client: Option<CsvApiClient>` → `get_csv_client()`, `get_csv_client_mut()`
- [x] `csv_mode: bool` → `is_csv_mode()`
- [x] `csv_table_name: String` → `get_table_name()`
- [x] `filtered_data: Option<Vec<Vec<String>>>` → `get_filtered_data()`, `set_filtered_data()`

### ✅ Phase 1: Simple Fields (COMPLETED)
These are straightforward to migrate and test:

- [x] `edit_mode: EditMode` - How the editor is being used (single/multi-line)
- [x] `last_results_row: Option<usize>` - Position preservation
- [x] `last_scroll_offset: (usize, usize)` - Position preservation
- [x] `case_insensitive: bool` - Search/filter behavior
- [x] `last_query_source: Option<String>` - Track where query came from

### 🔄 Phase 2: Buffer-Specific Complex State
These need to move but require more care:

- [ ] `textarea: TextArea<'static>` - Multi-line editor state
- [ ] `fuzzy_filter_state: FuzzyFilterState` - Fuzzy search state
- [ ] `column_search_state: ColumnSearchState` - Column search functionality
- [ ] `column_widths: Vec<u16>` - Display formatting
- [ ] `column_stats: Option<ColumnStatistics>` - Statistics for current column
- [ ] `cached_data: Option<Vec<serde_json::Value>>` - Cached query results
- [ ] `cache_mode: bool` - Whether caching is active
- [ ] `undo_stack: Vec<(String, usize)>` - Undo history
- [ ] `redo_stack: Vec<(String, usize)>` - Redo history
- [ ] `kill_ring: String` - Vim-style yank buffer
- [ ] `viewport_lock: bool` - Display behavior
- [ ] `viewport_lock_row: Option<usize>` - Display behavior
- [ ] `jump_to_row_input: String` - Jump command input
- [ ] `last_visible_rows: usize` - Display tracking

### 🚫 Should Stay in TUI (Global State)
These are truly global and shouldn't be in buffers:

- `api_client: ApiClient` - Global API connection
- `sql_parser: SqlParser` - Global parser
- `hybrid_parser: HybridParser` - Global parser
- `config: Config` - User configuration
- `command_history: CommandHistory` - Global command history
- `sql_highlighter: SqlHighlighter` - Syntax highlighting
- `query_cache: Option<QueryCache>` - Global cache
- `show_help: bool` - UI state
- `help_scroll: u16` - UI state
- `debug_text: String` - Debug UI
- `debug_scroll: u16` - Debug UI
- `input_scroll_offset: u16` - UI display
- `selection_mode: SelectionMode` - Global UI preference
- `yank_mode: Option<char>` - Global clipboard state
- `last_yanked: Option<(String, String)>` - Global clipboard
- `completion_state: CompletionState` - UI helper
- `history_state: HistoryState` - UI helper
- `buffer_manager: Option<BufferManager>` - The manager itself
- `current_buffer_name: Option<String>` - Current buffer tracking

## Migration Strategy

### Step 1: Create Compatibility Layer
For each field we want to migrate, create a method that:
1. Checks if buffer_manager exists
2. If yes, use BufferAPI
3. If no, use direct field access

Example:
```rust
fn get_edit_mode(&self) -> EditMode {
    if let Some(buffer) = self.get_current_buffer() {
        buffer.get_edit_mode()
    } else {
        self.edit_mode
    }
}
```

### Step 2: Update All References
Search for all direct field accesses and replace with compatibility methods:
- `self.edit_mode` → `self.get_edit_mode()`
- `self.edit_mode = x` → `self.set_edit_mode(x)`

### Step 3: Test After Each Migration
Run the TUI and verify:
1. Basic navigation works
2. Editing works
3. Filtering/searching works
4. No crashes or unexpected behavior

### Step 4: Move Field to Buffer
Once all references use the compatibility layer:
1. Add field to Buffer struct
2. Add to BufferAPI trait
3. Implement in Buffer
4. Remove from EnhancedTuiApp

## Progress Tracking

### Phase 1 Checklist (Simple Fields)
- [ ] Add `edit_mode` to BufferAPI
- [ ] Create `get_edit_mode()` / `set_edit_mode()` compatibility methods
- [ ] Update all `self.edit_mode` references
- [ ] Test TUI functionality
- [ ] Move field to Buffer
- [ ] Repeat for other Phase 1 fields

### Testing Checklist
After each field migration:
- [ ] Can open a CSV file
- [ ] Can type and execute queries
- [ ] Can navigate results
- [ ] Can filter results
- [ ] Can search in results
- [ ] Can yank/copy data
- [ ] No crashes or panics

## Notes

- We can merge to master at any point - the compatibility layer ensures nothing breaks
- Each field migration is independent - we can do them one at a time
- If something goes wrong, we can easily revert a single field migration
- The goal is gradual, safe migration, not speed