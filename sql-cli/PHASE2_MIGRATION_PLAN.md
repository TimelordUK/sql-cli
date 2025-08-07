# Phase 2 Buffer Migration Plan

## Field Categorization

### ✅ Already Migrated to Buffer (Phase 1)
- `edit_mode` - How the current buffer is being edited
- `case_insensitive` - Per-buffer setting for comparisons
- `last_results_row` - Last selected row in results
- `last_scroll_offset` - Last scroll position
- `last_query_source` - Source of last query (cache/api/file)

### 🔴 Should Move to Buffer (Phase 2)
These are buffer-specific and should move:

#### Core Query/Results State
- `input` - The SQL input for this buffer
- `textarea` - Multi-line editor for this buffer
- `results` - Query results for this buffer
- ✅ `table_state` - Table selection state **[DONE - Wrapper added, tested, working]**
- ✅ `mode` - Current mode (Command/Results/etc) for this buffer **[DONE - Using existing wrapper methods]**
- ✅ `status_message` - Status message for current buffer **[DONE - Fully migrated]**

#### Filtering/Search State
- ✅ `filter_state` - Active filters on this buffer's data **[DONE - Wrapper added, tested, working]**
- `fuzzy_filter_state` - Fuzzy filter state
- `search_state` - Search within results
- `column_search_state` - Column search state
- `filtered_data` - Filtered view of results

#### Display State
- ✅ `column_widths` - Calculated widths for this buffer's columns **[DONE - Wrapper added, migrated, tested]**
- ✅ `scroll_offset` - Current scroll position (row, col) **[DONE - Wrapper added, tested, working]**
- ✅ `current_column` - Currently selected column **[DONE - Wrapper added, migrated, tested]**
- ✅ `pinned_columns` - Which columns are pinned in this buffer **[DONE - Wrapper added, migrated, tested]**
- `column_stats` - Statistics for selected column
- ✅ `compact_mode` - Per-buffer display preference **[DONE - Wrapper added, migrated, tested]**
- ✅ `show_row_numbers` - Per-buffer display preference **[DONE - Wrapper added, migrated, tested]**
- ✅ `viewport_lock` - Viewport locking for this buffer **[DONE - Wrapper added, migrated, tested]**
- ✅ `viewport_lock_row` - The locked row **[DONE - Wrapper added, migrated, tested]**

#### CSV/Data State
- `csv_client` - CSV data source for this buffer
- ✅ `csv_mode` - Whether this buffer is in CSV mode **[DONE - Wrapper added, migrated, tested]**
- ✅ `csv_table_name` - Table name for CSV data **[DONE - Wrapper added, migrated, tested]**
- `cached_data` - Cached JSON data for this buffer

#### Edit State
- ✅ `undo_stack` - Undo history for this buffer **[DONE - Wrapper added, migrated, tested]**
- ✅ `redo_stack` - Redo history for this buffer **[DONE - Wrapper added, migrated, tested]**
- ✅ `kill_ring` - Kill ring for this buffer **[DONE - Wrapper added, migrated, tested]**

### 🟢 Should Stay Global (in GlobalState)
These are truly application-wide:

#### Core Services
- `api_client` - Shared API client
- `sql_parser` - Shared parser instance
- `hybrid_parser` - Shared parser instance
- `sql_highlighter` - Shared syntax highlighter
- `config` - Application configuration
- `command_history` - Global command history
- `query_cache` - Global query cache

#### UI State
- `show_help` - Global help display toggle
- `help_scroll` - Help page scroll position
- `debug_text` - Debug output text
- `debug_scroll` - Debug view scroll position
- `input_scroll_offset` - Horizontal scroll for input (might move to buffer?)

#### Global Features
- `selection_mode` - Row vs Cell selection mode (global preference)
- `yank_mode` - Multi-key yank tracking
- `last_yanked` - Last yanked content (could be global or per-buffer)
- `completion_state` - Autocomplete state (might be per-buffer?)
- `history_state` - History search state
- `jump_to_row_input` - Jump dialog input

#### Buffer Management
- `buffer_manager` - The buffer manager itself
- `current_buffer_name` - Display name of current buffer

### 🟡 Needs Discussion
These could go either way:
- ✅ `cache_mode` - Could be global or per-buffer **[DONE - Migrated to buffer system]**
- `last_visible_rows` - Viewport tracking, probably per-buffer

## Migration Status

### Completed Migrations
- ✅ **status_message** - Fully migrated to buffer system, field removed from TUI struct
  - All 149 references migrated
  - Wrapper methods work with buffer system
  - Field completely removed from EnhancedTuiApp

### Completed Migrations (Continued)
- ✅ **scroll_offset** - Successfully migrated to buffer system
  - All 26 references migrated to use wrapper methods
  - Get/set wrapper methods working with buffer system
  - Field still present for backward compatibility (can be removed later)
  - Tested and working with TUI

- ✅ **table_state** - Successfully migrated to buffer system
  - All 31 references migrated to use wrapper methods
  - Using get_table_state() and get_table_state_mut() accessors
  - Table selection state ready for per-buffer management
  - Tested and working with TUI

- ✅ **filter_state** - Successfully migrated to buffer system
  - All 26 references migrated to use wrapper methods
  - Using get_filter_state() and get_filter_state_mut() accessors
  - Filter state ready for per-buffer management
  - Tested and working with TUI

### Pending
- All other fields listed above

## Migration Steps

### Step 1: Create GlobalState struct ✅
- Move truly global fields into a new GlobalState struct
- Keep in TUI but behind a single field

### Step 2: Move Query/Results State
- Move `input`, `textarea`, `results`, `table_state`
- Move `mode`, `status_message`
- Add compatibility wrappers

### Step 3: Move Filter/Search State
- Move all filter states
- Move search states
- Move `filtered_data`

### Step 4: Move Display State
- Move column widths, scroll offset
- Move pinned columns, column stats
- Move viewport settings

### Step 5: Move CSV/Data State
- Move CSV client and mode
- Move cached data

### Step 6: Move Edit State
- Move undo/redo stacks
- Move kill ring

### Step 7: Clean Up
- Remove redundant fields from TUI
- Remove compatibility wrappers where possible
- Update all references

## Testing Strategy

After each step:
1. Build and run basic queries
2. Test the specific functionality moved
3. Test F5 debug to ensure buffer state is correct
4. Run through test_all_fixes.sh scenarios
5. Commit with clear message about what was moved

## Notes

- Each step should be a separate commit
- Compatibility wrappers allow gradual migration
- Test thoroughly after each step
- Don't try to do too much at once