# Data Model Architecture - Neovim Plugin Refactoring

## Goal
Replace fragile text-based table parsing with a robust data model that separates:
1. **Data** - Structured rows/columns from SQL CLI
2. **View** - What's visible and how it's displayed
3. **Rendering** - Converting data to buffer text

## Current State (Text-Based)
```
SQL CLI → Text Output → Buffer → Parse ASCII borders → Navigate
                                    ↑ FRAGILE
```

Problems:
- Parsing ASCII borders is slow and fragile
- Can't handle 3k+ rows efficiently (parses entire buffer)
- No column metadata (types, alignment)
- Hard to add features (hide columns, themes, totals)

## New Architecture (Data-Driven)

```
SQL CLI (--output-json-structured)
  ↓
Data Model (Lua)
  ├── rows: [["val1", "val2"], ...]
  ├── columns: [{name, type, width}, ...]
  └── metadata: {total_rows, query_time}
  ↓
Viewport (Lua)
  ├── current_pos: {row: 1, col: 1}
  ├── visible_range: {top_row: 1, num_rows: 50}
  └── column_state: {hidden: [], order: [0,1,2]}
  ↓
Renderer (Lua)
  └── Renders ONLY visible rows to buffer
```

## Data Model Schema

### CLI Output Format (JSON)
```json
{
  "columns": [
    {
      "name": "id",
      "type": "INTEGER",
      "max_width": 10,
      "alignment": "right"
    },
    {
      "name": "name",
      "type": "TEXT",
      "max_width": 30,
      "alignment": "left"
    }
  ],
  "rows": [
    ["1", "Hydrogen"],
    ["2", "Helium"]
  ],
  "metadata": {
    "total_rows": 2,
    "query_time_ms": 15,
    "source": "chemistry.csv"
  }
}
```

### Lua Data Model
```lua
-- nvim-plugin/lua/sql-cli/data_model.lua
local DataModel = {
  columns = {},      -- Column metadata
  rows = {},         -- Raw data rows
  metadata = {},     -- Query metadata

  -- Computed
  total_rows = 0,
  total_cols = 0,
}

function DataModel:new(json_data)
  -- Parse JSON from CLI
  -- Store structured data
end

function DataModel:get_cell(row, col)
  -- Get cell value by logical position
end

function DataModel:get_row(row)
  -- Get entire row
end

function DataModel:get_column_values(col)
  -- Get all values in a column (for stats, export)
end
```

### Viewport
```lua
-- nvim-plugin/lua/sql-cli/viewport.lua
local Viewport = {
  data_model = nil,   -- Reference to data model

  -- Position
  current_row = 1,
  current_col = 1,

  -- Visible range (for large datasets)
  top_row = 1,
  visible_rows = 50,   -- Render only these rows

  -- Column state
  column_order = {},   -- [0, 1, 2] (logical col indices)
  hidden_columns = {}, -- [3, 5] (hidden col indices)
  column_widths = {},  -- Custom widths or nil for auto

  -- Theme
  theme = "ascii",     -- ascii, unicode, markdown, minimal
}

function Viewport:move_cursor(dr, dc)
  -- Move cursor with bounds checking
  -- Update visible range if needed (keep cursor centered)
end

function Viewport:center_on_cursor()
  -- Adjust top_row to center current row
end

function Viewport:get_visible_data()
  -- Return only data that should be rendered
  -- Respects hidden columns, column order
end
```

### Renderer
```lua
-- nvim-plugin/lua/sql-cli/renderer.lua
local Renderer = {
  viewport = nil,
  themes = {
    ascii = { ... },
    unicode = { ... },
    markdown = { ... },
  }
}

function Renderer:render_to_buffer(bufnr)
  -- Get visible data from viewport
  -- Render to buffer using theme
  -- Use extmarks for current cell highlight
end

function Renderer:update_highlight()
  -- Update current cell highlight using extmarks
  -- No need to re-render entire buffer
end
```

## Implementation Phases

### Phase 1: Foundation (Keep existing system working)
- [x] Create this design doc
- [ ] Add `--output-json-structured` flag to sql-cli
- [ ] Create `data_model.lua` with JSON parsing
- [ ] Create `viewport.lua` with basic state
- [ ] Create `renderer.lua` with ASCII theme only
- [ ] **Toggle**: Add `use_data_model = false` config option
  - `false`: Use old text-based navigation (default, stable)
  - `true`: Use new data model (experimental)

**Goal**: Both systems work, can switch between them

### Phase 2: Feature Parity
- [ ] Implement all navigation in data model mode
- [ ] Implement all yank operations
- [ ] Implement multi-table support
- [ ] Make data model the default (`use_data_model = true`)

### Phase 3: Performance
- [ ] Virtual scrolling (render only visible 50-100 rows)
- [ ] Lazy column width calculation
- [ ] Efficient cell highlighting with extmarks

### Phase 4: Advanced Features
- [ ] Column operations (hide, reorder, resize)
- [ ] Multiple themes
- [ ] Totals row (virtual text)
- [ ] Column statistics overlay
- [ ] Conditional formatting

## Rollout Strategy

1. **Add flag, don't use it yet**
   - sql-cli gets `--output-json-structured` flag
   - Returns structured JSON
   - Plugin ignores it, uses text output (backward compatible)

2. **Build new system alongside old**
   - Create data_model.lua, viewport.lua, renderer.lua
   - Add config: `table_navigation.use_data_model = false`
   - Old code path still default

3. **Test with real queries**
   - Enable `use_data_model = true` in config
   - Test with chemistry.sql, large datasets
   - Fix bugs without breaking production use

4. **Gradual migration**
   - Once stable, make data model the default
   - Keep old code for one release as fallback
   - Eventually remove old text-parsing code

## CLI Changes Needed

### New Flag: `--output-json-structured`
```bash
./sql-cli data.csv -q "SELECT * FROM data" --output-json-structured
```

Output:
```json
{
  "columns": [
    {"name": "id", "type": "INTEGER", "max_width": 5, "alignment": "right"},
    {"name": "name", "type": "TEXT", "max_width": 20, "alignment": "left"}
  ],
  "rows": [
    ["1", "Hydrogen"],
    ["2", "Helium"]
  ],
  "metadata": {
    "total_rows": 2,
    "query_time_ms": 15
  }
}
```

### Implementation Location
- Add to `src/non_interactive.rs` alongside existing `--output csv/json/table`
- Create `output_json_structured()` function
- Use existing DataTable metadata for column info

## Benefits

✅ **Performance**: Render only 50-100 visible rows, handle 100k+ row datasets
✅ **Reliability**: No fragile text parsing, direct data access
✅ **Features**: Easy to add column hiding, reordering, themes, totals
✅ **Maintainability**: Clean separation of concerns
✅ **Backward Compatible**: Toggle between old/new during transition

## Testing Plan

1. **Unit tests**: data_model.lua operations
2. **Integration tests**: Full flow with real queries
3. **Performance tests**: 1k, 10k, 100k row datasets
4. **Feature parity**: All existing features work with data model

## Next Steps

1. Implement `--output-json-structured` in sql-cli
2. Create `data_model.lua` skeleton
3. Create simple proof-of-concept with chemistry.sql
4. Get feedback, iterate

---

**Status**: Design phase
**Owner**: SQL CLI Team
**Created**: 2025-10-04
