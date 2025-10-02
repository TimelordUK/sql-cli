# Plugin Data Model Evolution

**Created**: 2025-10-02
**Status**: Architecture Planning
**Priority**: P2 (Foundation for future features)

## 🎯 Vision

Transform the Neovim plugin from a "buffer parser" to a "data-aware application" with:
- Rich data model (not just text buffers)
- Multiple rendering strategies
- Composable CTEs
- Extensibility via Lua scripts
- Smart caching and incremental updates

## 🔴 Current Architecture (Problems)

### How It Works Now
```
User Query → sql-cli → CSV text → Buffer → Parse buffer for operations
                                           ↓
                                    Re-parse for export
                                           ↓
                                    Re-parse for distinct values
```

### Problems
1. **No data retention** - Everything is text, parsed repeatedly
2. **Buffer coupling** - All features require buffer parsing
3. **Limited composition** - Can't chain queries easily
4. **No metadata** - Lost schema, types, source query
5. **Brittle** - Buffer changes break everything

### Example Pain Points
```lua
-- Current: Parse buffer to get columns
local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
local header = lines[1]
local columns = vim.split(header, ',')  -- Fragile!

-- Current: Can't do this
SELECT * FROM (previous_query_result) WHERE ...
```

## 🟢 Proposed Architecture

### Data Model Layer

```lua
-- State tracked per query execution
QueryResult = {
    -- Core data
    query = string,              -- Original SQL query
    data = {                     -- Structured data (not text)
        columns = {
            { name = "col1", type = "INTEGER", nullable = false },
            { name = "col2", type = "TEXT", nullable = true },
        },
        rows = {
            { col1 = 123, col2 = "foo" },
            { col1 = 456, col2 = "bar" },
        },
    },

    -- Metadata
    source = {
        type = "query" | "file" | "web_cte",
        path = string,           -- File path or URL
        executed_at = timestamp,
    },

    -- Execution info
    execution = {
        time_ms = number,
        row_count = number,
        ctes = {                 -- Extracted CTEs
            { name = "cte1", definition = "...", columns = {...} },
        },
    },

    -- Display state
    display = {
        hidden_columns = { "col3", "col5" },
        sort_column = "col1",
        sort_direction = "DESC",
        filters = {
            { column = "col2", op = "LIKE", value = "%search%" },
        },
    },

    -- Rendering
    render = {
        style = "table" | "csv" | "json" | "custom",
        buffer = bufnr,
        decorations = {          -- Future: annotations, highlights
            { row = 5, type = "warning", message = "..." },
        },
    },
}
```

### New Workflow

```
User Query → sql-cli --output json-structured
           ↓
         Parse ONCE into QueryResult
           ↓
         Store in buffer-local state
           ↓
    ┌──────┴───────┬──────────┬──────────┐
    ▼              ▼          ▼          ▼
  Render      Export     Distinct    Filter
  (table)     (any fmt)   values     (re-query)
    │              │          │          │
    └──────────────┴──────────┴──────────┘
              All use QueryResult
           (no buffer parsing!)
```

## 🛠️ Implementation Phases

### Phase 1: Data Layer Foundation (P2 - Next Month)

**Goal**: Stop parsing buffers, start storing structured data

**Changes**:
```lua
-- New file: nvim-plugin/lua/sql-cli/data_model.lua
local DataModel = {}

function DataModel.new(query, json_output)
    -- Parse sql-cli JSON output into QueryResult structure
end

function DataModel.get_columns(result)
    -- Direct access, no parsing
    return result.data.columns
end

function DataModel.get_row(result, index)
    -- Direct access
    return result.data.rows[index]
end

function DataModel.filter(result, predicate)
    -- Filter in Lua without re-querying
end

return DataModel
```

**Modify sql-cli**:
```bash
# New output format: structured JSON
./target/release/sql-cli -q "SELECT ..." -o json-structured

# Output:
{
    "query": "SELECT ...",
    "columns": [
        {"name": "id", "type": "INTEGER", "nullable": false},
        {"name": "name", "type": "TEXT", "nullable": true}
    ],
    "rows": [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ],
    "metadata": {
        "execution_time_ms": 42,
        "row_count": 2,
        "ctes": [...]
    }
}
```

**Benefits**:
- All features access data directly
- No more fragile CSV parsing
- Foundation for advanced features

**Effort**: 6-8 hours

---

### Phase 2: Multiple Renderers (P2 - 2 months)

**Goal**: Separate data from display

**Renderers**:
```lua
-- nvim-plugin/lua/sql-cli/renderers/table.lua
function TableRenderer.render(query_result, opts)
    -- Current table style
end

-- nvim-plugin/lua/sql-cli/renderers/compact.lua
function CompactRenderer.render(query_result, opts)
    -- Denser format for large datasets
end

-- nvim-plugin/lua/sql-cli/renderers/tree.lua
function TreeRenderer.render(query_result, opts)
    -- Hierarchical view for nested data
end

-- nvim-plugin/lua/sql-cli/renderers/summary.lua
function SummaryRenderer.render(query_result, opts)
    -- Show aggregates at top, details below
end
```

**User Experience**:
```vim
:SqlCliRenderAs compact      " Change current buffer rendering
:SqlCliRenderAs summary      " Show summary view
\sr                          " Toggle through renderers
```

**Effort**: 4-6 hours per renderer

---

### Phase 3: CTE Composition (P2 - 3 months)

**Goal**: Reference previous query results in new queries

**Usage**:
```sql
-- First query
SELECT * FROM trades WHERE date = '2025-10-02';

-- Store as CTE (keybinding: \sS "save as CTE")
-- Prompts for name: "todays_trades"

-- Second query (can reference it!)
WITH todays_trades AS (
    -- Plugin automatically injects the data or re-executes query
)
SELECT
    symbol,
    COUNT(*) as trade_count,
    SUM(quantity) as total_qty
FROM todays_trades
GROUP BY symbol;
```

**Implementation**:
```lua
-- Store CTE reference
state.saved_ctes["todays_trades"] = {
    query = original_query,
    result = query_result,
    timestamp = timestamp,
}

-- When executing new query
if query contains "todays_trades" then
    -- Option 1: Inject data as VALUES clause
    -- Option 2: Re-execute source query
    -- Option 3: Cache in temp table
end
```

**Effort**: 8-12 hours

---

### Phase 4: Lua Scripting Engine (P3 - Future)

**Goal**: User-supplied transformations without Rust changes

**Example**:
```lua
-- User script: highlight_outliers.lua
function transform(row, context)
    local mean = context.stats.mean_price
    local stddev = context.stats.stddev_price

    if math.abs(row.price - mean) > 2 * stddev then
        return {
            row = row,
            highlight = "WarningMsg",
            annotation = "Outlier: " .. row.price
        }
    end
    return { row = row }
end
```

**CLI Integration**:
```bash
# Execute with Lua transform
./target/release/sql-cli -q "SELECT ..." --lua-transform highlight_outliers.lua
```

**Rust Side**:
```rust
// Embed mlua
use mlua::Lua;

fn apply_lua_transform(data: &mut QueryResult, script: &str) -> Result<()> {
    let lua = Lua::new();
    lua.load(script).exec()?;
    // Call transform function for each row
}
```

**Benefits**:
- Complex projections without Rust coding
- User extensibility
- Rapid prototyping
- Domain-specific transforms

**Effort**: 12-16 hours

---

### Phase 5: Smart Decorations (P3 - Future)

**Goal**: Annotate results with derived information

**Features**:
```lua
-- Auto-generate summary row
┌──────────┬─────────┬──────────┐
│ Symbol   │ Qty     │ Price    │
├──────────┼─────────┼──────────┤
│ AAPL     │ 100     │ 150.00   │
│ GOOGL    │ 50      │ 2800.00  │
│ MSFT     │ 200     │ 380.00   │
├──────────┼─────────┼──────────┤
│ TOTAL    │ 350     │ Avg: 610 │  ← Auto-generated
└──────────┴─────────┴──────────┘

-- Inline calculations
┌──────────┬─────────┬──────────┬─────────┐
│ Symbol   │ Qty     │ Price    │ Value   │ ← Calculated column
├──────────┼─────────┼──────────┼─────────┤
│ AAPL     │ 100     │ 150.00   │ 15,000  │
│ GOOGL    │ 50      │ 2800.00  │ 140,000 │
└──────────┴─────────┴──────────┴─────────┘

-- Sparklines for trends
┌──────────┬─────────┬──────────────────┐
│ Symbol   │ Price   │ 7-Day Trend      │
├──────────┼─────────┼──────────────────┤
│ AAPL     │ 150.00  │ ▁▂▃▅▆█▇ (↑ 5%)  │
│ GOOGL    │ 2800.00 │ ▇█▆▅▃▂▁ (↓ 3%)  │
└──────────┴─────────┴──────────────────┘
```

**Implementation**:
- Query additional data sources
- Store in QueryResult.decorations
- Render with decorations

**Effort**: Variable (4-8 hours per decoration type)

---

## 📐 Architecture Principles

### 1. Separation of Concerns
```
Data Layer     ─→  QueryResult (pure data)
               ↓
Business Logic ─→  Filtering, sorting, transformations
               ↓
Rendering      ─→  Table, CSV, JSON, custom
               ↓
Display        ─→  Buffer, floating window, etc.
```

### 2. Immutability
```lua
-- Don't modify QueryResult
-- Create new derived versions
local filtered = DataModel.filter(result, predicate)
local sorted = DataModel.sort(filtered, column)
```

### 3. Lazy Evaluation
```lua
-- Don't render until needed
local result = execute_query(query)  -- Just data
result:render_as("table")            -- Render on demand
result:render_as("summary")          -- Different view of same data
```

### 4. Extensibility Points
- Custom renderers
- Lua transforms
- Decoration plugins
- Export formats

## 🎯 Success Metrics

### Phase 1 Complete When:
- [ ] No buffer parsing in core operations
- [ ] All features use QueryResult
- [ ] Can access any data point in O(1)
- [ ] Export works from data, not buffer

### Phase 2 Complete When:
- [ ] Can switch renderers without re-query
- [ ] 3+ renderer types available
- [ ] Renderer selection persists

### Phase 3 Complete When:
- [ ] Can reference previous results in queries
- [ ] CTE composition feels natural
- [ ] No performance degradation

### Phase 4 Complete When:
- [ ] User Lua scripts work
- [ ] Documentation for script API
- [ ] Example scripts provided

## 🔄 Migration Strategy

### Incremental Adoption
1. Add data model layer alongside current code
2. Migrate features one by one
3. Deprecate buffer parsing gradually
4. Remove old code when confident

### Backwards Compatibility
- Keep current keybindings working
- Add new features as opt-in
- Smooth transition for users

## 🚀 Quick Wins Before Full Implementation

**Can do now** (use existing architecture):
1. Cache last query result in buffer variable
2. Store column list separately
3. Tag buffers with metadata

**Example**:
```lua
-- After query execution
vim.b.sql_cli_result = {
    columns = columns,
    query = query,
    row_count = row_count,
}

-- Later features can access
local cols = vim.b.sql_cli_result.columns  -- No parsing!
```

## 📝 Related Documents

- [ROADMAP_2025.md](ROADMAP_2025.md) - Overall strategy
- [PRIORITIZED_TASKS.md](PRIORITIZED_TASKS.md) - Current tasks
- [NVIM_SMART_COLUMN_COMPLETION.md](NVIM_SMART_COLUMN_COMPLETION.md) - Column features

## 🤔 Open Questions

1. **Data size limits**: How much data to keep in memory?
2. **Caching strategy**: When to invalidate cached CTEs?
3. **Lua vs Rust**: Which transforms belong where?
4. **Streaming**: Support for very large result sets?
5. **Multi-buffer**: Share data across buffers?

## 💡 Future Possibilities

- **Live queries**: Auto-refresh on data change
- **Query diff**: Compare two result sets visually
- **Collaborative**: Share CTEs across sessions
- **Persistence**: Save QueryResults to disk
- **Undo/Redo**: Query execution history

---

**Next Action**: Start Phase 1 after completing P0/P1 priorities
