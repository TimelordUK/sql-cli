# SQL CLI Neovim Plugin - Features Roadmap

## ✅ Completed Features

### Table Navigation (v1.0)
- **Cell Navigation**: h/j/k/l for moving between cells
- **Boundary Jumps**: 0 (first column), $ (last column), gg (first row), G (last row)
- **Yank Operations**:
  - `yy` - Yank current cell value
  - `Y` - Yank entire row as CSV
  - `yc` - Yank entire column
- **Tab Navigation**: Tab/Shift-Tab for cell movement
- **Visual Highlighting**: Current cell is highlighted

### Core Features (v0.1)
- Query execution with output window
- SQL formatting (AST-based and fallback)
- Function help system
- Data file detection
- Schema completion
- Visual query selection

## 🚀 Planned Features

### 1. Query History 📜
**Priority: High**

Save and navigate through query execution history.

**Features:**
- Persistent query history (saved to `~/.local/share/sql-cli/history.json`)
- Navigate with `:History` command showing list
- Quick keys: `[q` / `]q` for previous/next query
- Include timestamp, execution time, row count
- `:Favorite` command to bookmark queries
- `:RunLast` to re-execute last query
- Search through history with fuzzy finder

**Implementation Notes:**
- Store in state module
- Save after each successful execution
- Limit history size (configurable, default 1000)
- Include data file used with each query

### 2. Result Search & Filter 🔍
**Priority: High**

Search and filter within query results.

**Features:**
- `/pattern` - Search forward in results
- `?pattern` - Search backward
- `n` / `N` - Next/previous match
- `:Filter <column> <pattern>` - Show only matching rows
- `:Highlight <pattern>` - Highlight all matches
- `:ClearFilter` - Remove filters
- Case-insensitive search option

**Implementation Notes:**
- Use vim's built-in search for simple patterns
- Implement column-specific filtering
- Maintain original results in state

### 3. Export Options 📤
**Priority: Medium**

Multiple export formats for query results.

**Features:**
- `:CopyAsInsert [table_name]` - Generate INSERT statements
- `:CopyAsJson` - Export as JSON array
- `:CopyAsMarkdown` - Format as Markdown table
- `:CopyAsPython` - Python dict/list format
- `:CopyAsHtml` - HTML table
- `:CopyAsLatex` - LaTeX table format
- `:SaveAs <filename>` - Save in detected format by extension

**Implementation Notes:**
- Use current table structure from table_nav
- Support both clipboard and file output
- Preserve data types where possible

### 4. Smart Completions 💡
**Priority: Medium**

Enhanced SQL completion beyond basic schema.

**Features:**
- Column name completion after SELECT/WHERE/GROUP BY
- Function name completion with signatures
- Table name completion from multiple sources
- SQL keyword completion (context-aware)
- Alias completion within query
- Join condition suggestions
- Common query templates

**Implementation Notes:**
- Integrate with nvim-cmp or built-in completion
- Parse current query context
- Cache frequently used items

### 5. Result Transformations 🔄
**Priority: Low**

Interactive result manipulation.

**Features:**
- Click column headers to sort
- `:Hide <column>` - Hide columns
- `:Show <column>` - Show hidden columns
- `:Pivot <row_col> <col_col> <value_col>` - Pivot table
- `:Aggregate <column> <function>` - Quick aggregations
- `:Transpose` - Swap rows and columns
- Column width adjustment

**Implementation Notes:**
- Store transformations in state
- Apply client-side without re-querying
- Visual indicators for sorted columns

### 6. Multi-Query Support 📑
**Priority: Low**

Execute and manage multiple queries.

**Features:**
- Execute all queries in buffer (separated by `;` or `GO`)
- Results in separate tabs
- `:CompareResults` - Side-by-side comparison
- Named result sets
- Query dependencies (use result of query A in query B)
- Batch execution with transaction support

**Implementation Notes:**
- Multiple output buffers
- Tab management for results
- Result set naming and referencing

### 7. Advanced Navigation 🎯
**Priority: Medium**

Enhanced navigation features.

**Features:**
- Go to definition (table/view definitions)
- Peek definition in floating window
- Column usage finder
- Query dependency graph
- Navigate between related queries
- Bookmarks within large result sets

### 8. Performance Features ⚡
**Priority: Low**

Performance monitoring and optimization.

**Features:**
- Query execution time tracking
- Result set size warnings
- Query plan visualization (when available)
- Suggestion for indexes
- Slow query log
- Resource usage monitoring

### 9. Collaboration Features 👥
**Priority: Low**

Team collaboration features.

**Features:**
- Share queries via URL/gist
- Query collections (team favorites)
- Comments on queries
- Query versioning
- Export/import query collections

### 10. Visualization 📊
**Priority: Low**

Basic data visualization.

**Features:**
- Sparklines for numeric columns
- Bar charts for categorical data
- Distribution histograms
- Heatmaps for correlations
- ASCII charts in output buffer

## 🎨 UI/UX Improvements

### Planned Improvements
- Floating window for results (optional)
- Status line integration showing current cell/query info
- Better error messages with suggestions
- Progress indicator for long-running queries
- Customizable color schemes
- Mini-map for large result sets
- Breadcrumb navigation
- Context menu on right-click

## 🔧 Technical Improvements

### Code Quality
- Add comprehensive test suite
- Performance optimizations for large results
- Better error handling and recovery
- Async execution improvements
- Memory usage optimization

### Integration
- LSP integration for SQL
- DAP (Debug Adapter Protocol) support
- Integration with other SQL tools
- Docker/container support
- Cloud database connections

## 📝 Configuration Ideas

### User Preferences
```lua
{
  -- Result display
  max_rows_display = 1000,
  truncate_strings = 100,
  null_display = "NULL",

  -- Performance
  query_timeout = 30,
  cache_results = true,

  -- Features
  auto_complete = true,
  auto_format = false,
  save_history = true,

  -- UI
  float_results = false,
  result_position = "bottom", -- bottom, right, float, tab

  -- Keymaps (customizable)
  keymaps = {
    -- All existing plus new ones
    search_results = "/",
    filter_results = "<leader>sf",
    export_menu = "<leader>se",
    history = "<leader>sh",
  }
}
```

## 🚦 Implementation Priority

1. **Phase 1** (Next Release)
   - Query History
   - Result Search & Filter

2. **Phase 2**
   - Export Options
   - Smart Completions

3. **Phase 3**
   - Result Transformations
   - Advanced Navigation

4. **Phase 4**
   - Multi-Query Support
   - Performance Features

5. **Future**
   - Collaboration Features
   - Visualization

## 📌 Notes

- Focus on features that enhance productivity
- Maintain simplicity - don't overcomplicate
- Keep performance in mind for large datasets
- Ensure all features work in terminal (no GUI dependencies)
- Maintain backward compatibility

## 🤝 Contributing

Ideas and contributions welcome! Features should:
- Solve real workflow problems
- Be intuitive to use
- Follow Vim/Neovim conventions
- Be well-documented
- Include tests where applicable