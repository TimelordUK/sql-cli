# SQL-CLI Next Week Development Roadmap

## Priority Tasks for Next Week

### 1. 🐛 Fix Multi-Buffer Support in Neovim Plugin
**Problem**: Plugin may have scoping issues when multiple SQL buffers are open (trades.sql, orders.sql)
- **Investigate**: State management across multiple buffers
- **Test**: Commands running in wrong buffer, output appearing in wrong window
- **Fix**: Ensure buffer-local variables and proper event handler isolation
- **Deliverable**: Each buffer maintains independent state and context

### 2. 📊 Excel Integration
**Goal**: Direct Excel support for import/export
- **Export Features**:
  - Query results → Excel with proper formatting
  - Multiple queries → Different sheets in one workbook
  - Preserve data types, headers, and formatting
  - Include formulas for totals/calculations
- **Import Features**:
  - `SELECT * FROM 'data.xlsx!Sheet1'` syntax
  - Support multiple sheets as different tables
- **Libraries**: Consider Apache POI (via Rust bindings) or calamine crate

### 3. 🔍 FZF Integration for Results Buffer
**Goal**: Fast, interactive filtering of query results in Neovim
- **Implementation**:
  - `<C-f>` to trigger FZF on current results
  - Multi-select with Tab key
  - Column-specific filtering
  - Preserve table formatting after filtering
- **Actions**:
  - `Enter`: Apply filter to buffer
  - `Ctrl-Y`: Yank selected rows
  - `Ctrl-E`: Export selected rows
  - `Ctrl-S`: Generate SQL WHERE clause from selection
- **Benefits**: Navigate large result sets instantly

### 4. 💼 Professional HTML Export Templates
**Goal**: Executive-ready reports for email distribution
- **Design Improvements**:
  - Modern CSS framework (Tailwind/Bootstrap)
  - Corporate branding support (logos, colors)
  - Professional typography and spacing
  - Dark/light mode toggle
  - Print-friendly CSS
- **Interactive Features**:
  - Sortable columns (client-side)
  - Search box for filtering
  - Collapsible sections for multi-query reports
  - Executive summary cards with key metrics
  - Auto-generated charts (Chart.js) for numeric data
- **Email Features**:
  - Inline CSS for email client compatibility
  - Responsive design for mobile viewing
  - Automated insights and anomaly detection
  - Confidentiality footers
- **Export Styles**:
  ```bash
  sql-cli query.sql --export html --export-style executive
  sql-cli query.sql --export html --export-style dashboard
  sql-cli query.sql --export html --export-style email
  ```

## Implementation Priority Order

1. **Day 1-2**: Fix multi-buffer support (critical bug)
2. **Day 2-3**: FZF integration (high user value, relatively quick)
3. **Day 3-4**: Professional HTML templates (immediate business value)
4. **Day 4-5**: Excel integration (more complex, but highly requested)

## Technical Considerations

### Multi-Buffer Fix
- Check `nvim-plugin/lua/sql-cli/executor.lua` for global vs buffer-local state
- Review event handlers and command registration
- Test with multiple file types and switching between buffers

### FZF Integration
- Leverage fzf-lua if available, fallback to system fzf
- Maintain result formatting (borders, alignment)
- Consider performance with very large result sets

### HTML Templates
- Create template system with variable substitution
- Support custom CSS/branding injection
- Ensure email client compatibility (test with Outlook, Gmail)
- Consider generating both inline and external CSS versions

### Excel Integration
- Evaluate Rust crates: `calamine` (read), `rust_xlsxwriter` (write)
- Handle large files efficiently (streaming)
- Preserve Excel data types and formatting
- Support formula generation for aggregates

## Success Metrics

- [ ] Multiple nvim buffers work independently
- [ ] FZF filtering reduces time to find data by 80%
- [ ] HTML exports look professional enough for C-suite
- [ ] Excel round-trip preserves all data and formatting
- [ ] All features have tests and documentation

## Future Ideas (Parking Lot)

- Query result caching in nvim (per buffer)
- Progress indicators for long-running queries
- Auto-complete for table/column names
- Smart JOIN suggestions
- Import from clipboard
- Visual mode for selecting data ranges in TUI
- Yank/paste operations for data cells

## Notes

- Focus on user-facing improvements that provide immediate value
- Maintain backward compatibility with existing workflows
- Add comprehensive tests for each new feature
- Update CLAUDE.md with new capabilities as we add them

---

*Let's make SQL-CLI the go-to tool for data analysis and reporting!* 🚀