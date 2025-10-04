# Neovim Plugin Refactoring: SQL CLI as "Poor Man's Language Server"

**Date**: 2025-10-04
**Status**: Design Proposal
**Priority**: P1 - Architecture

## Problem Statement

The Neovim plugin currently does extensive "hand-crafted" text parsing:
- Regex matching for query boundaries (GO statements, WITH keywords)
- Manual CTE detection and parsing
- SELECT * pattern matching
- Column extraction from query text
- Query context detection

**Issues with current approach:**
1. **Duplication**: SQL CLI already has a proper recursive descent parser
2. **Fragility**: Text parsing breaks with edge cases (standalone `*`, nested CTEs, etc.)
3. **Maintenance burden**: Each new feature adds more complex Lua regex logic
4. **Drift risk**: Parser improvements in CLI don't benefit the plugin

## Vision: SQL CLI as Language Server

Eventually run SQL CLI in server mode as a proper LSP, but in the interim, enhance CLI to act as a **"poor man's language server"** by providing structured query analysis.

## Current Capabilities (Already Available)

The CLI already provides:

### 1. Query Plan (AST)
```bash
sql-cli data.csv -q "SELECT * FROM table" --query-plan
```

**Output**: Full AST with:
- CTEs with names and definitions
- SELECT items (Star vs Column)
- WHERE conditions
- JOINs, ORDER BY, GROUP BY, LIMIT, etc.

### 2. Schema Information
```bash
sql-cli data.csv --schema-json
```

**Output**: JSON schema (already used by plugin)

### 3. Query Validation
Running a query returns success/failure, which validates syntax.

## Proposed Enhancements: New CLI Modes

### 1. Query Analysis Mode (`--analyze-query`)

**Purpose**: Get structured information about a query without executing it.

```bash
sql-cli [data.csv] -q "SELECT * FROM trades" --analyze-query
```

**JSON Output**:
```json
{
  "valid": true,
  "has_star": true,
  "star_locations": [
    {"line": 1, "column": 8, "table_context": "trades"}
  ],
  "ctes": [
    {
      "name": "trades",
      "type": "WEB",
      "start_line": 1,
      "end_line": 5,
      "has_star": false
    }
  ],
  "tables_referenced": ["trades"],
  "columns_selected": ["*"],
  "query_type": "SELECT",
  "error": null
}
```

### 2. CTE Extraction (`--extract-cte <name>`)

**Purpose**: Extract a specific CTE as a standalone query.

```bash
sql-cli -q "$(cat query.sql)" --extract-cte trades
```

**Output**: Just the CTE query, ready to execute:
```sql
SELECT * FROM web_endpoint WHERE trade_date > '2024-01-01'
```

### 3. Column Expansion (`--expand-star`)

**Purpose**: Expand SELECT * to actual columns.

```bash
sql-cli data.csv -q "SELECT * FROM periodic_table LIMIT 1" --expand-star
```

**Output**:
```json
{
  "original": "SELECT * FROM periodic_table",
  "expanded": "SELECT AtomicNumber, Element, Symbol, ... FROM periodic_table",
  "columns": ["AtomicNumber", "Element", "Symbol", ...]
}
```

### 4. Query Context Detection (`--query-at-line <n>`)

**Purpose**: Determine which query/CTE is at a given line in a multi-statement file.

```bash
sql-cli -f queries.sql --query-at-line 45
```

**JSON Output**:
```json
{
  "query_index": 2,
  "query_type": "CTE",
  "cte_name": "filtered_trades",
  "start_line": 40,
  "end_line": 50,
  "parent_query_start": 1,
  "parent_query_end": 100
}
```

### 5. Query Formatting (Already Exists)

```bash
sql-cli --format-sql query.sql
```

**Enhancement**: Add `--format-json` to output as JSON with metadata:
```json
{
  "formatted": "SELECT\n    column1\n  , column2\nFROM table",
  "changes_made": ["indentation", "comma_alignment"]
}
```

## Plugin Refactoring Roadmap

### Phase 1: Replace Text Parsing (P0)

**Current**: Plugin searches for `SELECT *` with regex
**New**: Call `sql-cli -q "..." --analyze-query`, check `has_star` field

**Current**: Plugin manually detects CTEs with pattern matching
**New**: Call `sql-cli -q "..." --analyze-query`, use `ctes` array

**Files to refactor**:
- `nvim-plugin/lua/sql-cli/results.lua` - `get_query_for_expansion()`
- `nvim-plugin/lua/sql-cli/cte_tester_v2.lua` - CTE detection

### Phase 2: Leverage AST for Intelligence (P1)

**Column Expansion** (`\sE`):
- Current: Execute query with LIMIT 1, parse JSON
- New: `sql-cli data.csv -q "..." --expand-star`

**CTE Testing** (`\sC`):
- Current: Extract CTE manually with text slicing
- New: `sql-cli -q "..." --extract-cte <name>`

**Query Validation**:
- New feature: Real-time syntax validation as you type
- Call: `sql-cli -q "..." --analyze-query` (returns `valid: true/false`)

### Phase 3: LSP Mode (Future)

Eventually:
```bash
sql-cli --lsp
```

Full language server protocol support with:
- `textDocument/completion` - Column/table completions
- `textDocument/hover` - Function documentation
- `textDocument/formatting` - SQL formatting
- `textDocument/diagnostic` - Syntax errors
- `textDocument/definition` - Jump to CTE definition

## Implementation Strategy

### Step 1: Add CLI Analysis Commands

**File**: `src/main.rs`

Add new flags:
- `--analyze-query` → JSON output
- `--expand-star` → Column expansion
- `--extract-cte <name>` → CTE extraction
- `--query-at-line <n>` → Context detection

### Step 2: Create Analysis Module

**New File**: `src/analysis/mod.rs`

```rust
pub struct QueryAnalysis {
    pub valid: bool,
    pub has_star: bool,
    pub star_locations: Vec<StarLocation>,
    pub ctes: Vec<CteInfo>,
    pub tables_referenced: Vec<String>,
    pub columns_selected: Vec<String>,
    pub query_type: QueryType,
    pub error: Option<String>,
}

pub fn analyze_query(sql: &str, data_file: Option<&str>) -> QueryAnalysis {
    // Use existing parser
    // Return structured analysis
}
```

### Step 3: Plugin Helper Module

**New File**: `nvim-plugin/lua/sql-cli/cli_analyzer.lua`

```lua
local M = {}

-- Call sql-cli --analyze-query and return parsed JSON
function M.analyze_query(query, data_file)
  local cmd = {'sql-cli'}

  if data_file then
    table.insert(cmd, data_file)
  end

  table.insert(cmd, '-q')
  table.insert(cmd, query)
  table.insert(cmd, '--analyze-query')

  local result = vim.fn.system(cmd)
  return vim.json.decode(result)
end

-- Call sql-cli --expand-star
function M.expand_star(query, data_file)
  local cmd = {'sql-cli'}

  if data_file then
    table.insert(cmd, data_file)
  end

  table.insert(cmd, '-q')
  table.insert(cmd, query)
  table.insert(cmd, '--expand-star')

  local result = vim.fn.system(cmd)
  return vim.json.decode(result)
end

-- Extract CTE
function M.extract_cte(query, cte_name)
  local cmd = {
    'sql-cli',
    '-q', query,
    '--extract-cte', cte_name
  }

  return vim.fn.system(cmd)
end

return M
```

### Step 4: Refactor Existing Functions

**results.lua - expand_star_smart()**:

```lua
-- OLD (100+ lines of regex parsing)
local function get_query_for_expansion(...)
  -- Complex text parsing
end

-- NEW (10 lines)
function M.expand_star_smart(config, state)
  local query = get_buffer_text() -- Simple: get all text
  local data_file = state:get_data_file()

  local analysis = require('sql-cli.cli_analyzer').expand_star(query, data_file)

  if not analysis.columns or #analysis.columns == 0 then
    vim.notify("No columns to expand", vim.log.levels.WARN)
    return
  end

  replace_star_with_columns(analysis.columns)
end
```

**cte_tester_v2.lua - test_cte_at_cursor()**:

```lua
-- OLD (200+ lines of CTE detection)
local function find_cte_at_cursor(...)
  -- Complex regex and line counting
end

-- NEW (20 lines)
function M.test_cte_at_cursor()
  local query = get_buffer_text()
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1]

  local analysis = require('sql-cli.cli_analyzer').analyze_query(query)

  if not analysis.ctes or #analysis.ctes == 0 then
    vim.notify("No CTEs found", vim.log.levels.WARN)
    return
  end

  -- Find CTE at cursor line
  local target_cte = nil
  for _, cte in ipairs(analysis.ctes) do
    if cursor_line >= cte.start_line and cursor_line <= cte.end_line then
      target_cte = cte
      break
    end
  end

  if not target_cte then
    vim.notify("No CTE at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract and execute
  local cte_query = require('sql-cli.cli_analyzer').extract_cte(query, target_cte.name)
  execute_query(cte_query)
end
```

## Benefits

### 1. **Reliability**
- Use battle-tested parser instead of fragile regex
- Parser handles all SQL edge cases
- Single source of truth for SQL understanding

### 2. **Maintainability**
- Plugin code dramatically simpler (100+ lines → 10-20 lines)
- Parser improvements automatically benefit plugin
- No regex debugging

### 3. **Extensibility**
- Easy to add new analysis features in Rust
- Plugin just calls CLI and uses JSON response
- Can add LSP mode without plugin changes

### 4. **Performance**
- Parser is fast (Rust)
- Can cache analysis results
- No difference vs current approach (plugin already shells out)

### 5. **Testability**
- Can test CLI analysis with simple bash commands
- No need for complex Neovim test harness
- Easier to debug

## Example Usage

### Before (Current):
```lua
-- 150 lines of text parsing
local function get_query_for_expansion()
  local lines = vim.api.nvim_buf_get_lines(...)

  -- Find WITH keyword
  for i = current_line, 1, -1 do
    if lines[i]:match("^%s*WITH%s+") then
      start_line = i
      break
    end
  end

  -- Find GO statement
  for i = current_line, #lines do
    if lines[i]:match("^%s*GO%s*$") then
      end_line = i - 1
      break
    end
  end

  -- Extract query text
  -- ... 50 more lines
end
```

### After (Proposed):
```lua
-- 5 lines
local function get_query_for_expansion()
  local all_text = table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), '\n')
  return all_text
end
```

All the intelligence is in the CLI.

## Migration Plan

1. **Week 1**: Implement `--analyze-query` in CLI
2. **Week 2**: Implement `--expand-star` and `--extract-cte` in CLI
3. **Week 3**: Create `cli_analyzer.lua` helper module
4. **Week 4**: Refactor `expand_star_smart()` to use CLI
5. **Week 5**: Refactor `test_cte_at_cursor()` to use CLI
6. **Week 6**: Remove old parsing code, update docs

## Open Questions

1. **Performance**: Is shelling out for every analysis too slow?
   - Can add caching in plugin
   - Can eventually use LSP mode for persistent process

2. **Error handling**: How to handle CLI errors gracefully?
   - CLI returns JSON with `error` field
   - Plugin shows user-friendly message

3. **Backward compatibility**: Support older CLI versions?
   - Check CLI version, fall back to old parsing if needed

## Next Steps

1. Review this design with user
2. Implement `--analyze-query` flag in CLI
3. Create proof-of-concept with one refactored function
4. Measure performance impact
5. Full migration if successful

## Related Documentation

- [ROADMAP_2025.md](ROADMAP_2025.md) - Overall roadmap
- [NVIM_PLUGIN_LOGGING.md](NVIM_PLUGIN_LOGGING.md) - Logging system
- [EXPAND_STAR_FIX.md](EXPAND_STAR_FIX.md) - Current expand star implementation
- [DEBUGGING_WITH_LOGS.md](DEBUGGING_WITH_LOGS.md) - Debugging guide
