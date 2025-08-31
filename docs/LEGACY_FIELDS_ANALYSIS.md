# Legacy Fields Analysis for EnhancedTuiApp

## Fields That Should Be Removed/Migrated to Buffer

### 1. Input-Related Fields (Phase 11 targets)
- `input: Input` - Should be removed, use buffer's InputManager
- `textarea: TextArea<'static>` - Should be removed, use buffer's InputManager
- `edit_mode: EditMode` - Move to buffer (each buffer can have its own edit mode)

### 2. Data Source Fields (Should be in Buffer)
- `csv_client: Option<CsvApiClient>` - Already in buffer, remove from TUI
- `csv_mode: bool` - Already in buffer, remove from TUI
- `csv_table_name: String` - Already in buffer, remove from TUI
- `cache_mode: bool` - Already in buffer, remove from TUI
- `cached_data: Option<Vec<serde_json::Value>>` - Already in buffer, remove from TUI

### 3. Results/Data Fields (Should be in Buffer)
- `filtered_data: Option<Vec<Vec<String>>>` - Should be per-buffer
- `column_widths: Vec<u16>` - Should be per-buffer (each buffer's data has different widths)
- `sort_state: SortState` - Should be per-buffer
- `filter_state: FilterState` - Should be per-buffer
- `search_state: SearchState` - Should be per-buffer

### 4. Navigation State (Should be per-buffer)
- `table_state: TableState` - Should be per-buffer
- `last_results_row: Option<usize>` - Should be per-buffer
- `last_scroll_offset: (usize, usize)` - Should be per-buffer
- `scroll_offset: (usize, usize)` - Should be per-buffer
- `current_column: usize` - Should be per-buffer
- `pinned_columns: Vec<usize>` - Should be per-buffer

## Fields That Should Stay in TUI (Global State)

### 1. Core Infrastructure
- `api_client: ApiClient` - Shared across all buffers
- `buffer_manager: BufferManager` - Manages all buffers
- `query_cache: Option<QueryCache>` - Shared cache

### 2. Global UI State
- `mode: AppMode` - Current global mode
- `show_help: bool` - Global UI state
- `help_scroll: u16` - Global UI state
- `debug_text: String` - Global debug info
- `debug_scroll: u16` - Global debug UI state
- `key_history: Vec<String>` - Global debug tracking

### 3. Parsers and Highlighters (Shared)
- `sql_parser: SqlParser` - Shared parser
- `hybrid_parser: HybridParser` - Shared parser
- `sql_highlighter: SqlHighlighter` - Shared highlighter

### 4. Global Settings
- `config: Config` - Global configuration
- `case_insensitive: bool` - Global setting (though could be per-buffer)

### 5. History and Completion (Global)
- `command_history: CommandHistory` - Shared across all buffers
- `history_state: HistoryState` - Global history UI state
- `completion_state: CompletionState` - Could be per-buffer but currently global

### 6. Clipboard/Selection (Global)
- `selection_mode: SelectionMode` - Global selection mode
- `yank_mode: Option<char>` - Global yank state
- `last_yanked: Option<(String, String)>` - Global clipboard

### 7. Tracking Fields
- `current_buffer_name: Option<String>` - Buffer tracking
- `last_query_source: Option<String>` - Query source tracking
- `input_scroll_offset: u16` - Should probably be per-buffer

## Migration Priority

### High Priority (Blocking buffer independence)
1. Remove `csv_client`, `csv_mode`, `csv_table_name` - Already in buffer
2. Remove `cache_mode`, `cached_data` - Already in buffer
3. Remove `input`, `textarea` - Use buffer's InputManager
4. Move `filtered_data` to buffer
5. Move `table_state`, `scroll_offset`, `current_column` to buffer

### Medium Priority (Improves buffer isolation)
1. Move `sort_state`, `filter_state`, `search_state` to buffer
2. Move `column_widths` to buffer
3. Move `pinned_columns` to buffer
4. Move `edit_mode` to buffer

### Low Priority (Nice to have)
1. Consider per-buffer `completion_state`
2. Consider per-buffer `case_insensitive` setting
3. Move `input_scroll_offset` to buffer

## Current Status Based on Code

Looking at the code, we're in a transitional state:
- Phase 1-6 appear complete (InputManager integration)
- Some fields are marked as "MIGRATED to buffer system" (commented out)
- But legacy fields (`input`, `textarea`, `csv_client`, etc.) are still present and being used
- Many getters/setters are using the buffer but with fallback to legacy fields

## Next Steps

1. **Complete Phase 7-10** if not done
2. **Phase 11: Remove Legacy Fields**
   - Start with fields already duplicated in buffer
   - Update all references to use buffer methods
   - Remove synchronization code
3. **Add remaining state to Buffer**
   - Move navigation state (table_state, scroll_offset, etc.)
   - Move filter/sort state
   - Move column-related state