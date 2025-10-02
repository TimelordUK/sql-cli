# Prioritized Task List

**Created**: 2025-10-02
**Focus**: High-impact plugin improvements

## 🔴 P0 - Start This Week

### 1. Fix Fuzzy Search Strictness
**File**: Check `nvim-plugin/lua/sql-cli/results.lua` or wherever fzf is called
**Issue**: Search too permissive, returns irrelevant results
**Solution**:
```lua
-- Current (too broad)
fzf_opts = { '--no-sort', '--tac' }

-- Try adding:
fzf_opts = { '--no-sort', '--tac', '--exact' }  -- Exact substring match
-- OR
fzf_opts = { '--no-sort', '--tac', '-i' }       -- Case insensitive exact
```
**Test**: Search for specific values, should not return partial matches
**Time**: 30 minutes

### 2. Document FIX Proxy Integration
**File**: Create `examples/fix_proxy_example.sql`
**Content**:
```sql
-- Example of FIX protocol proxy with tag aggregation
WITH WEB fix_messages AS (
    URL 'http://localhost:8080/fix/messages'
    METHOD POST
    HEADERS ('Authorization': 'Bearer TOKEN')
    BODY '{
        "tags": ["35", "49", "56", "11", "38"],  -- MsgType, Sender, Target, ClOrdID, Qty
        "group_tags": ["453", "448"]              -- Party IDs (aggregated with |)
    }'
    FORMAT JSON
    JSON_PATH 'messages'
)
SELECT
    MsgType,
    SenderCompID,
    COUNT(*) as message_count,
    SUM(CAST(OrderQty as INTEGER)) as total_qty
FROM fix_messages
GROUP BY MsgType, SenderCompID
ORDER BY message_count DESC;
```
**Time**: 1 hour

## 🟠 P1 - This Month

### 3. Column Hide/Show Feature
**Design Considerations**:
- Where to store hidden columns? (buffer-local variable?)
- How to toggle? (new keymap `\sh`?)
- Should it persist across query re-runs?
- Integration with export (respect hidden columns)

**Implementation Plan**:
1. Add buffer-local state for hidden columns
2. Modify display logic to skip hidden columns
3. Add keybinding to toggle column under cursor
4. Add command to show all columns
5. Update export to respect hidden state

**Files to modify**:
- `nvim-plugin/lua/sql-cli/state.lua` - Add hidden_columns table
- `nvim-plugin/lua/sql-cli/results.lua` - Filter display
- `nvim-plugin/lua/sql-cli/init.lua` - Add keybindings
- `nvim-plugin/lua/sql-cli/export.lua` - Respect hidden columns

**Time**: 4-6 hours

### 4. Smart Expansion Edge Cases
**Known Issues**:
- Nested CTEs sometimes fail to detect
- Multi-statement queries need better parsing
- Error messages not helpful

**Improvements**:
- Better CTE boundary detection
- Handle multiple CTEs in single query
- Clear error when query fails with explanation

**Time**: 2-3 hours

## 🟡 P2 - Next Month

### 5. WEB CTE Auth Management
**Features**:
- Store auth tokens per endpoint
- Token refresh logic
- Timeout configuration
- Better error messages for 401/403

**Files**:
- `src/sql/parser/web_cte_parser.rs`
- New: `src/auth/token_manager.rs`

**Time**: 6-8 hours

### 6. Export Format Additions
**Formats to add**:
- Markdown table (easy - just formatting)
- JSON (already have data, just serialize)
- Excel/XLSX (use rust_xlsxwriter crate)

**Time**: 3-4 hours per format

## 🟢 P3 - Future / Nice to Have

### 7. Window Function Ranking Bug
**Issue**: Doesn't always start from 0
**Impact**: Low - rare edge case
**Investigation needed**: Which window functions? Under what conditions?

### 8. Plugin Performance Profiling
**Goal**: <100ms startup, <1s for common operations
**Tools**: Neovim profiler, Lua timing

### 9. Which-Key Integration
**Add** descriptive labels for all `\s*` keybindings
**Benefits**: Discoverability, less documentation needed

## 📋 Quick Wins (< 30 min each)

- [ ] Add `:SqlCliVersion` command to show version
- [ ] Add `:SqlCliHelp` to open README
- [ ] Improve error message when sql-cli not in PATH
- [ ] Add example queries to nvim-plugin/examples/
- [ ] Document all keybindings in one place

## 🎯 This Week's Focus

**Monday-Wednesday**: P0-1 (Fuzzy search fix)
**Thursday**: P0-2 (FIX proxy docs)
**Friday**: Plan P1-3 (Column hide/show)

## 📊 Effort vs Impact Matrix

```
High Impact │ P0-1 Fuzzy │ P1-3 Hide/Show │
            │ P0-2 FIX   │                │
────────────┼────────────┼────────────────┤
            │ P2-6 Export│ P3-8 Perf      │
Low Impact  │ P2-5 Auth  │ P3-9 Which-Key │
            └────────────┴────────────────┘
              Low Effort    High Effort
```

## 🚫 Explicitly Deferred

- CUDA offloading (no performance need yet)
- Parallel query execution (current data size doesn't justify)
- TUI enhancements (plugin is preferred)
- Type system overhaul (working well enough)
- Advanced caching (not a bottleneck)

## ✅ Success Criteria

**This Quarter**:
- [ ] Fuzzy search works reliably
- [ ] FIX proxy fully documented with examples
- [ ] Can hide/show columns interactively
- [ ] All common operations <3 keystrokes
- [ ] Zero crashes in normal workflow

**Measurements**:
- Time from question to insight: <30 seconds
- Plugin startup: <100ms
- Query execution feedback: immediate
- Export any format: <2 keystrokes
