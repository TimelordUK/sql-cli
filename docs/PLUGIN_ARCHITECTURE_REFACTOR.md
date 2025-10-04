# Plugin Architecture Refactor - Foundation for Production Use

**Date**: 2025-10-04
**Status**: Active Development
**Priority**: P0 - Critical Foundation

## Executive Summary

Two critical refactorings to enable robust plugin usage at work:

1. **SQL CLI as Intelligence Provider**: Replace fragile regex parsing with structured CLI output
2. **Independent Data Model**: Separate table data from buffer representation

These changes form the foundation for advanced features (inline summations, pretty rendering, alternative table designs) without accumulating technical debt.

---

## Part 1: SQL CLI Intelligence Provider

### Current Problems

**Fragile Text Parsing**:
- `get_query_for_expansion()`: 150+ lines of regex for query boundaries
- CTE detection: Manual pattern matching, breaks with WEB CTEs, nested CTEs
- Column extraction: Execute query + parse JSON (works but inefficient)
- Query validation: No validation until execution

**Maintenance Burden**:
- Every edge case requires more regex
- Parser improvements in CLI don't help plugin
- Debugging requires reading Lua logs AND CLI logs
- Duplication of SQL understanding

### Solution: CLI Provides Structured Analysis

The CLI already has a battle-tested recursive descent parser. Make it available to plugin via JSON output.

### New CLI Commands (Rust Implementation)

#### 1. `--analyze-query` (Foundation)

**Purpose**: Get comprehensive query analysis without execution.

```bash
sql-cli data.csv -q "SELECT * FROM trades WHERE price > 100" --analyze-query
```

**JSON Output**:
```json
{
  "valid": true,
  "query_type": "SELECT",
  "has_star": true,
  "star_locations": [
    {"line": 1, "column": 8, "context": "main_query"}
  ],
  "tables": ["trades"],
  "columns": ["price"],
  "ctes": [],
  "subqueries": [],
  "from_clause": {
    "type": "table",
    "name": "trades"
  },
  "where_clause": {
    "present": true,
    "columns_referenced": ["price"]
  },
  "errors": []
}
```

**With CTEs**:
```bash
sql-cli -q "WITH WEB trades AS (...) SELECT * FROM trades" --analyze-query
```

```json
{
  "valid": true,
  "has_star": true,
  "star_locations": [
    {"line": 5, "column": 8, "context": "main_query"}
  ],
  "ctes": [
    {
      "name": "trades",
      "type": "WEB",
      "start_line": 1,
      "end_line": 4,
      "start_offset": 0,
      "end_offset": 245,
      "has_star": false,
      "columns": ["trade_id", "symbol", "price", "qty"],
      "web_config": {
        "url": "http://api.example.com/trades",
        "method": "GET"
      }
    }
  ],
  "tables": ["trades"],
  "errors": []
}
```

#### 2. `--expand-star` (Column Expansion)

**Purpose**: Expand SELECT * to actual column names.

```bash
sql-cli data.csv -q "SELECT * FROM periodic_table" --expand-star
```

**JSON Output**:
```json
{
  "original_query": "SELECT * FROM periodic_table",
  "expanded_query": "SELECT AtomicNumber, Element, Symbol, ... FROM periodic_table",
  "columns": [
    {"name": "AtomicNumber", "type": "INTEGER"},
    {"name": "Element", "type": "TEXT"},
    {"name": "Symbol", "type": "TEXT"}
  ],
  "expansion_count": 28
}
```

**With CTEs**:
```bash
sql-cli -q "WITH WEB trades AS (...) SELECT * FROM trades" --expand-star --data-file trades.csv
```

```json
{
  "original_query": "WITH WEB trades AS (...) SELECT * FROM trades",
  "expanded_query": "WITH WEB trades AS (...) SELECT trade_id, symbol, price, qty FROM trades",
  "columns": [
    {"name": "trade_id", "type": "TEXT"},
    {"name": "symbol", "type": "TEXT"},
    {"name": "price", "type": "FLOAT"},
    {"name": "qty", "type": "INTEGER"}
  ],
  "cte_columns": {
    "trades": ["trade_id", "symbol", "price", "qty"]
  }
}
```

#### 3. `--extract-cte <name>` (CTE Extraction)

**Purpose**: Extract CTE as standalone executable query.

```bash
sql-cli -q "$(cat complex_query.sql)" --extract-cte filtered_trades
```

**Output**: Plain SQL (not JSON)
```sql
SELECT *
FROM trades
WHERE trade_date > '2024-01-01'
  AND status = 'filled'
```

#### 4. `--query-at-position <line>:<col>` (Context Detection)

**Purpose**: Determine query context at cursor position.

```bash
sql-cli -q "$(cat multi_query.sql)" --query-at-position 45:10
```

**JSON Output**:
```json
{
  "context_type": "CTE",
  "cte_name": "filtered_trades",
  "cte_index": 1,
  "query_bounds": {
    "start_line": 40,
    "end_line": 50,
    "start_offset": 800,
    "end_offset": 1200
  },
  "parent_query_bounds": {
    "start_line": 1,
    "end_line": 100
  },
  "can_execute_independently": true
}
```

#### 5. `--format-sql --output json` (Enhanced Formatting)

**Purpose**: Format SQL with metadata.

```bash
sql-cli -q "select * from trades where price>100" --format-sql --output json
```

**JSON Output**:
```json
{
  "formatted": "SELECT\n    *\nFROM trades\nWHERE price > 100",
  "changes": [
    "normalized_keywords",
    "added_indentation",
    "added_spacing_around_operators"
  ],
  "style": "comma_leading"
}
```

### Rust Implementation Plan

#### Step 1: Create Analysis Module

**New File**: `src/analysis/mod.rs`

```rust
use serde::{Serialize, Deserialize};
use crate::sql::ast::SelectStatement;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryAnalysis {
    pub valid: bool,
    pub query_type: String,
    pub has_star: bool,
    pub star_locations: Vec<StarLocation>,
    pub tables: Vec<String>,
    pub columns: Vec<String>,
    pub ctes: Vec<CteAnalysis>,
    pub subqueries: Vec<SubqueryAnalysis>,
    pub from_clause: Option<FromClauseInfo>,
    pub where_clause: Option<WhereClauseInfo>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StarLocation {
    pub line: usize,
    pub column: usize,
    pub context: String,  // "main_query", "cte_name", "subquery"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CteAnalysis {
    pub name: String,
    pub cte_type: String,  // "Standard", "WEB", "Recursive"
    pub start_line: usize,
    pub end_line: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub has_star: bool,
    pub columns: Vec<String>,
    pub web_config: Option<WebCteConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebCteConfig {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnExpansion {
    pub original_query: String,
    pub expanded_query: String,
    pub columns: Vec<ColumnInfo>,
    pub expansion_count: usize,
    pub cte_columns: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

pub fn analyze_query(sql: &str, data_file: Option<&str>) -> Result<QueryAnalysis> {
    // Use existing parser
    let ast = parse_sql(sql)?;

    // Analyze AST
    let mut analysis = QueryAnalysis {
        valid: true,
        query_type: determine_query_type(&ast),
        has_star: false,
        star_locations: vec![],
        tables: vec![],
        columns: vec![],
        ctes: vec![],
        subqueries: vec![],
        from_clause: None,
        where_clause: None,
        errors: vec![],
    };

    // Extract CTEs
    for cte in &ast.ctes {
        analysis.ctes.push(analyze_cte(cte)?);
    }

    // Check for SELECT *
    for item in &ast.select_items {
        if matches!(item, SelectItem::Star) {
            analysis.has_star = true;
            analysis.star_locations.push(StarLocation {
                line: 1, // TODO: Track line numbers in parser
                column: 8,
                context: "main_query".to_string(),
            });
        }
    }

    // Extract tables
    if let Some(table) = &ast.from_table {
        analysis.tables.push(table.clone());
    }

    Ok(analysis)
}

pub fn expand_star(sql: &str, data_file: Option<&str>) -> Result<ColumnExpansion> {
    // Parse query
    let ast = parse_sql(sql)?;

    // Load data to get schema
    let data_table = if let Some(file) = data_file {
        load_data_file(file)?
    } else {
        // Try to execute CTEs to get schema
        execute_for_schema(&ast)?
    };

    // Get column names
    let columns: Vec<ColumnInfo> = data_table
        .columns
        .iter()
        .map(|col| ColumnInfo {
            name: col.name.clone(),
            data_type: format!("{:?}", col.data_type),
        })
        .collect();

    // Replace * with column names
    let expanded = replace_star_in_ast(&ast, &columns);

    Ok(ColumnExpansion {
        original_query: sql.to_string(),
        expanded_query: expanded,
        columns,
        expansion_count: columns.len(),
        cte_columns: HashMap::new(), // TODO: Expand CTE stars
    })
}

pub fn extract_cte(sql: &str, cte_name: &str) -> Result<String> {
    let ast = parse_sql(sql)?;

    for cte in &ast.ctes {
        if cte.name == cte_name {
            return Ok(format_cte_as_query(cte));
        }
    }

    Err(format!("CTE '{}' not found", cte_name).into())
}
```

#### Step 2: Update Main.rs with New Flags

**File**: `src/main.rs`

```rust
use crate::analysis::{analyze_query, expand_star, extract_cte};

// In parse_args()
if let Some(query) = matches.value_of("query") {
    if matches.is_present("analyze-query") {
        let analysis = analyze_query(query, data_file.as_deref())?;
        println!("{}", serde_json::to_string_pretty(&analysis)?);
        return Ok(());
    }

    if matches.is_present("expand-star") {
        let expansion = expand_star(query, data_file.as_deref())?;
        println!("{}", serde_json::to_string_pretty(&expansion)?);
        return Ok(());
    }

    if let Some(cte_name) = matches.value_of("extract-cte") {
        let cte_query = extract_cte(query, cte_name)?;
        println!("{}", cte_query);
        return Ok(());
    }

    // ... existing query execution
}
```

#### Step 3: Track Line Numbers in Parser

**Enhancement Needed**: `src/sql/recursive_parser.rs`

```rust
// Add to Token
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,      // NEW
    pub column: usize,    // NEW
}

// Update tokenizer to track position
impl Tokenizer {
    pub fn tokenize(&mut self, sql: &str) -> Result<Vec<Token>> {
        let mut line = 1;
        let mut column = 1;

        for ch in sql.chars() {
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }

            // Track line/column in tokens
            // ...
        }
    }
}

// Add to AST nodes
pub struct SelectStatement {
    // ... existing fields
    pub start_line: usize,    // NEW
    pub end_line: usize,      // NEW
    pub start_offset: usize,  // NEW
    pub end_offset: usize,    // NEW
}
```

---

## Part 2: Independent Data Model

### Current Problems

**Buffer Peeking**:
- Reading cell values requires buffer manipulation
- Table state coupled to buffer representation
- Can't re-render without rebuilding buffer
- No way to decorate or enhance display

**Limitations**:
- Can't show inline summations
- Can't swap table rendering styles
- Can't add visual decorations (highlights, borders, etc.)
- Performance issues with large result sets

### Solution: Independent Data Model

Separate **data** from **view**:
- Data model holds actual table state
- Buffer is just a rendering target
- Can re-render at any time with any style
- Can decorate without affecting data

### Data Model Architecture

#### Core Data Structure

**New File**: `nvim-plugin/lua/sql-cli/data_model.lua`

```lua
local M = {}

-- Table data model (independent of buffer)
local DataModel = {}
DataModel.__index = DataModel

function DataModel.new(query_result)
  local self = setmetatable({}, DataModel)

  -- Core data (from CLI JSON)
  self.columns = query_result.columns or {}
  self.rows = query_result.data or {}
  self.metadata = query_result.metadata or {}

  -- Display state
  self.visible_columns = {} -- Column indices to show
  self.column_order = {}    -- Custom column ordering
  self.column_widths = {}   -- Calculated widths

  -- Sort state
  self.sort_column = nil
  self.sort_direction = nil

  -- Filter state
  self.filter_text = ""
  self.filtered_rows = {}   -- Indices of visible rows

  -- Decorations
  self.decorations = {
    inline_sums = {},       -- Column -> sum value
    highlights = {},        -- Row/col -> highlight group
    borders = {},           -- Custom border styles
  }

  -- Statistics (computed once, cached)
  self.stats = {
    row_count = #self.rows,
    null_counts = {},
    sum_values = {},
    distinct_counts = {},
  }

  self:initialize()
  return self
end

function DataModel:initialize()
  -- Set default visible columns (all)
  for i = 1, #self.columns do
    table.insert(self.visible_columns, i)
    self.column_order[i] = i
  end

  -- Set default filtered rows (all)
  for i = 1, #self.rows do
    table.insert(self.filtered_rows, i)
  end

  -- Compute initial stats
  self:compute_stats()
end

function DataModel:compute_stats()
  -- Compute statistics for numeric columns
  for i, col in ipairs(self.columns) do
    local sum = 0
    local null_count = 0
    local distinct = {}

    for _, row in ipairs(self.rows) do
      local value = row[i]
      if value == nil or value == "" then
        null_count = null_count + 1
      else
        if type(value) == "number" then
          sum = sum + value
        end
        distinct[value] = true
      end
    end

    self.stats.sum_values[col] = sum
    self.stats.null_counts[col] = null_count
    self.stats.distinct_counts[col] = vim.tbl_count(distinct)
  end
end

function DataModel:get_cell(row_idx, col_idx)
  -- Get cell value by ACTUAL row/col index
  if row_idx < 1 or row_idx > #self.rows then
    return nil
  end
  if col_idx < 1 or col_idx > #self.columns then
    return nil
  end
  return self.rows[row_idx][col_idx]
end

function DataModel:get_visible_cell(visible_row, visible_col)
  -- Get cell value by VISIBLE indices (after filter/reorder)
  local actual_row = self.filtered_rows[visible_row]
  local actual_col = self.column_order[visible_col]
  return self:get_cell(actual_row, actual_col)
end

function DataModel:apply_filter(filter_text)
  self.filter_text = filter_text
  self.filtered_rows = {}

  -- Apply filter to all rows
  for i, row in ipairs(self.rows) do
    local matches = false
    for _, cell in ipairs(row) do
      if cell and tostring(cell):lower():find(filter_text:lower(), 1, true) then
        matches = true
        break
      end
    end
    if matches then
      table.insert(self.filtered_rows, i)
    end
  end
end

function DataModel:sort_by_column(col_idx, direction)
  self.sort_column = col_idx
  self.sort_direction = direction or "asc"

  -- Sort filtered_rows by column value
  table.sort(self.filtered_rows, function(a_idx, b_idx)
    local a = self.rows[a_idx][col_idx]
    local b = self.rows[b_idx][col_idx]

    if self.sort_direction == "asc" then
      return (a or "") < (b or "")
    else
      return (a or "") > (b or "")
    end
  end)
end

function DataModel:reorder_columns(new_order)
  -- new_order is array of column indices
  self.column_order = new_order
end

function DataModel:hide_column(col_idx)
  -- Remove from visible_columns
  for i, idx in ipairs(self.visible_columns) do
    if idx == col_idx then
      table.remove(self.visible_columns, i)
      break
    end
  end
end

function DataModel:show_column(col_idx)
  if not vim.tbl_contains(self.visible_columns, col_idx) then
    table.insert(self.visible_columns, col_idx)
  end
end

function DataModel:add_inline_sum(col_idx)
  local col_name = self.columns[col_idx]
  self.decorations.inline_sums[col_idx] = self.stats.sum_values[col_name] or 0
end

function DataModel:get_visible_data()
  -- Return data in display order for rendering
  local visible = {
    columns = {},
    rows = {},
  }

  -- Get visible column names in order
  for _, col_idx in ipairs(self.visible_columns) do
    local actual_idx = self.column_order[col_idx]
    table.insert(visible.columns, self.columns[actual_idx])
  end

  -- Get filtered rows with visible columns in order
  for _, row_idx in ipairs(self.filtered_rows) do
    local row = {}
    for _, col_idx in ipairs(self.visible_columns) do
      local actual_idx = self.column_order[col_idx]
      table.insert(row, self.rows[row_idx][actual_idx])
    end
    table.insert(visible.rows, row)
  end

  return visible
end

M.DataModel = DataModel
return M
```

#### Renderer Module

**New File**: `nvim-plugin/lua/sql-cli/renderer.lua`

```lua
local M = {}

-- Render data model to buffer
function M.render_to_buffer(bufnr, data_model, style)
  style = style or "default"

  local lines = {}

  if style == "default" then
    lines = M.render_default(data_model)
  elseif style == "compact" then
    lines = M.render_compact(data_model)
  elseif style == "decorated" then
    lines = M.render_decorated(data_model)
  end

  -- Write to buffer
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)

  -- Apply decorations (highlights, etc.)
  M.apply_decorations(bufnr, data_model)
end

function M.render_default(data_model)
  local visible = data_model:get_visible_data()
  local lines = {}

  -- Header
  table.insert(lines, "# " .. table.concat(visible.columns, " | "))
  table.insert(lines, "")

  -- Rows
  for _, row in ipairs(visible.rows) do
    table.insert(lines, table.concat(row, " | "))
  end

  -- Inline sums if present
  if vim.tbl_count(data_model.decorations.inline_sums) > 0 then
    table.insert(lines, "")
    table.insert(lines, "--- Sums ---")
    for col_idx, sum in pairs(data_model.decorations.inline_sums) do
      local col_name = visible.columns[col_idx]
      table.insert(lines, string.format("%s: %s", col_name, sum))
    end
  end

  return lines
end

function M.render_compact(data_model)
  -- Minimal spacing, no decorations
  local visible = data_model:get_visible_data()
  local lines = {}

  table.insert(lines, table.concat(visible.columns, "|"))
  for _, row in ipairs(visible.rows) do
    table.insert(lines, table.concat(row, "|"))
  end

  return lines
end

function M.render_decorated(data_model)
  -- Full decorations: borders, colors, inline stats
  local visible = data_model:get_visible_data()
  local lines = {}

  -- Calculate column widths
  local widths = {}
  for i, col in ipairs(visible.columns) do
    widths[i] = #col
  end
  for _, row in ipairs(visible.rows) do
    for i, cell in ipairs(row) do
      widths[i] = math.max(widths[i], #tostring(cell))
    end
  end

  -- Top border
  local border = "┌"
  for i, w in ipairs(widths) do
    border = border .. string.rep("─", w + 2)
    if i < #widths then
      border = border .. "┬"
    end
  end
  border = border .. "┐"
  table.insert(lines, border)

  -- Header
  local header = "│"
  for i, col in ipairs(visible.columns) do
    header = header .. " " .. col .. string.rep(" ", widths[i] - #col) .. " │"
  end
  table.insert(lines, header)

  -- Separator
  local sep = "├"
  for i, w in ipairs(widths) do
    sep = sep .. string.rep("─", w + 2)
    if i < #widths then
      sep = sep .. "┼"
    end
  end
  sep = sep .. "┤"
  table.insert(lines, sep)

  -- Rows
  for _, row in ipairs(visible.rows) do
    local line = "│"
    for i, cell in ipairs(row) do
      local s = tostring(cell)
      line = line .. " " .. s .. string.rep(" ", widths[i] - #s) .. " │"
    end
    table.insert(lines, line)
  end

  -- Bottom border
  local bottom = "└"
  for i, w in ipairs(widths) do
    bottom = bottom .. string.rep("─", w + 2)
    if i < #widths then
      bottom = bottom .. "┴"
    end
  end
  bottom = bottom .. "┘"
  table.insert(lines, bottom)

  -- Inline sums below table
  if vim.tbl_count(data_model.decorations.inline_sums) > 0 then
    table.insert(lines, "")
    for col_idx, sum in pairs(data_model.decorations.inline_sums) do
      local col_name = visible.columns[col_idx]
      table.insert(lines, string.format("  Σ %s = %s", col_name, sum))
    end
  end

  return lines
end

function M.apply_decorations(bufnr, data_model)
  local ns_id = vim.api.nvim_create_namespace('sql_cli_decorations')

  -- Clear existing decorations
  vim.api.nvim_buf_clear_namespace(bufnr, ns_id, 0, -1)

  -- Apply highlights
  for pos, hl_group in pairs(data_model.decorations.highlights) do
    local row, col = pos.row, pos.col
    vim.api.nvim_buf_add_highlight(bufnr, ns_id, hl_group, row, col, -1)
  end
end

M.render_to_buffer = render_to_buffer
return M
```

#### Integration with Executor

**File**: `nvim-plugin/lua/sql-cli/executor.lua`

```lua
local data_model = require('sql-cli.data_model')
local renderer = require('sql-cli.renderer')

-- When query results come back
function M.handle_query_result(result)
  -- Create data model from JSON result
  local model = data_model.DataModel.new(result)

  -- Store in state
  M.current_data_model = model

  -- Render to buffer
  local bufnr = M.get_result_buffer()
  renderer.render_to_buffer(bufnr, model, M.config.render_style)
end

-- Re-render without re-executing query
function M.rerender()
  if not M.current_data_model then
    vim.notify("No data to render", vim.log.levels.WARN)
    return
  end

  local bufnr = M.get_result_buffer()
  renderer.render_to_buffer(bufnr, M.current_data_model, M.config.render_style)
end

-- Add inline sum decoration
function M.add_inline_sum_at_cursor()
  if not M.current_data_model then
    return
  end

  local col_idx = M.get_column_at_cursor()
  M.current_data_model:add_inline_sum(col_idx)
  M.rerender()
end

-- Toggle rendering style
function M.toggle_render_style()
  local styles = {"default", "compact", "decorated"}
  local current_idx = vim.tbl_contains(styles, M.config.render_style)
  M.config.render_style = styles[(current_idx % #styles) + 1]

  vim.notify("Render style: " .. M.config.render_style, vim.log.levels.INFO)
  M.rerender()
end
```

### New Capabilities Enabled

With independent data model:

1. **Instant re-render**: Change style without re-executing query
2. **Inline summations**: `\sa` on column shows sum at bottom
3. **Hide/show columns**: `\ch` hides current column, `\cs` shows all
4. **Reorder columns**: Drag/drop or command
5. **Multiple render styles**: Default, compact, decorated, custom
6. **Decorations**: Highlights, borders, annotations
7. **No buffer peeking**: All data access through model

### New Keybindings

```lua
-- Rendering
vim.keymap.set('n', '<leader>sr', ':SqlCliRerender<CR>', {desc = 'Re-render table'})
vim.keymap.set('n', '<leader>st', ':SqlCliToggleStyle<CR>', {desc = 'Toggle render style'})

-- Decorations
vim.keymap.set('n', '<leader>sa', ':SqlCliInlineSum<CR>', {desc = 'Add inline sum for column'})
vim.keymap.set('n', '<leader>sh', ':SqlCliHighlightColumn<CR>', {desc = 'Highlight column'})

-- Column operations
vim.keymap.set('n', '<leader>ch', ':SqlCliHideColumn<CR>', {desc = 'Hide column at cursor'})
vim.keymap.set('n', '<leader>cs', ':SqlCliShowAllColumns<CR>', {desc = 'Show all columns'})
vim.keymap.set('n', '<leader>co', ':SqlCliReorderColumns<CR>', {desc = 'Reorder columns'})
```

---

## Implementation Timeline

### Week 1: CLI Analysis Foundation
- [ ] Create `src/analysis/mod.rs` with analysis structures
- [ ] Implement `--analyze-query` flag
- [ ] Add line number tracking to parser
- [ ] Test with example queries

### Week 2: CLI Column Expansion
- [ ] Implement `--expand-star` flag
- [ ] Handle CTEs in expansion
- [ ] Implement `--extract-cte` flag
- [ ] Test with WEB CTEs

### Week 3: Plugin CLI Integration
- [ ] Create `nvim-plugin/lua/sql-cli/cli_analyzer.lua`
- [ ] Refactor `expand_star_smart()` to use `--expand-star`
- [ ] Refactor `test_cte_at_cursor()` to use `--extract-cte`
- [ ] Remove old regex parsing code

### Week 4: Data Model Foundation
- [ ] Create `nvim-plugin/lua/sql-cli/data_model.lua`
- [ ] Create `nvim-plugin/lua/sql-cli/renderer.lua`
- [ ] Integrate with executor
- [ ] Basic rendering working

### Week 5: Advanced Rendering
- [ ] Implement compact and decorated styles
- [ ] Add inline summations
- [ ] Add column hide/show
- [ ] Test with large result sets

### Week 6: Polish and Testing
- [ ] Remove buffer peeking code
- [ ] Performance testing
- [ ] Documentation
- [ ] User testing at work

---

## Success Metrics

### Part 1: CLI Analysis
- ✅ Zero regex parsing in plugin for SQL understanding
- ✅ `\sE` works 100% reliably with all query types
- ✅ `\sC` works 100% reliably with nested CTEs
- ✅ Plugin code reduced by 60%+

### Part 2: Data Model
- ✅ Can re-render without re-executing query
- ✅ Can show inline sums on numeric columns
- ✅ Can switch render styles instantly
- ✅ Zero buffer peeking for cell values

---

## Next Steps

1. **Start with `--analyze-query`**: Foundation for everything
2. **Add line tracking to parser**: Critical for accurate analysis
3. **Proof of concept**: Refactor one function to use CLI analysis
4. **Validate approach**: Ensure performance is acceptable
5. **Full migration**: Complete both refactorings

This establishes the architectural foundation for production use without accumulating more technical debt.
