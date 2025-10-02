# SQL CLI - Project Summary

**Last Updated**: 2025-10-02
**Version**: 1.56.0
**Strategic Focus**: Neovim Plugin Excellence

## 🎯 What Is This?

Fast in-memory SQL query tool with dual interface:
- **Neovim Plugin** (Primary) - Data exploration in your editor
- **CLI/TUI** (Secondary) - Standalone terminal tool

**Core Value**: Query CSV/JSON files and HTTP endpoints with SQL, instantly.

## 📊 Current State (October 2025)

### ✅ What Works Great

**SQL Engine**:
- ✅ SELECT, WHERE, JOIN, GROUP BY, ORDER BY, LIMIT
- ✅ 100+ SQL functions (math, string, date, conversion)
- ✅ Window functions (ROW_NUMBER, RANK, LAG, LEAD, etc.)
- ✅ CTEs (Common Table Expressions)
- ✅ WEB CTEs (query HTTP APIs as tables)
- ✅ Performance: 8ms @ 100K rows for simple queries

**Neovim Plugin**:
- ✅ Smart column expansion (`\sE`) - discovers columns from CTEs
- ✅ Distinct value analysis (`\srD`) - instant cardinality
- ✅ Export (TSV/CSV/HTML) to clipboard or browser
- ✅ SQL refactoring tools (CASE generation, banding)
- ✅ Fuzzy search through results
- ✅ Cross-buffer operation

**Data Sources**:
- ✅ CSV files
- ✅ JSON files
- ✅ HTTP endpoints (WEB CTEs)
- ✅ FIX protocol proxy integration

### 🔧 Active Development

**This Week (P0)**:
- Fix fuzzy search being too permissive
- Document FIX proxy usage with examples

**This Month (P1)**:
- Column hide/show feature
- Smart expansion edge case improvements

**Next 3 Months (P2)**:
- Plugin data model (stop parsing buffers)
- Multiple rendering styles
- WEB CTE authentication

### 🚫 Explicitly Not Doing

- ❌ Major TUI enhancements (plugin superseded it)
- ❌ CUDA/GPU acceleration (no performance need)
- ❌ Parallel query execution (not a bottleneck)
- ❌ Type system overhaul (works well enough)

## 📈 Performance Characteristics

**Tested @ 100K rows**:
- Simple SELECT: **8ms**
- JOINs: **<40ms**
- GROUP BY: **~500ms**
- Window functions: **~1.2s**

**Sufficient for**: <30K row datasets (primary use case)

## 🗺️ Architecture

```
┌─────────────────────────────────────────────┐
│           Neovim Plugin (Lua)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Commands │  │ Keybinds │  │ Renderers│ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
│       │             │              │       │
│       └─────────────┼──────────────┘       │
│                     ▼                       │
│            ┌─────────────────┐             │
│            │   State Layer   │ ← Future    │
│            │  (QueryResult)  │   data model│
│            └────────┬────────┘             │
└─────────────────────┼──────────────────────┘
                      ▼
        ┌─────────────────────────┐
        │   sql-cli (Rust Core)   │
        │  ┌──────────────────┐   │
        │  │  SQL Parser      │   │
        │  │  Query Executor  │   │
        │  │  Data Sources    │   │
        │  │  Function Reg.   │   │
        │  └──────────────────┘   │
        └─────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
    CSV Files    JSON Files    HTTP APIs
```

## 📚 Documentation Structure

**Start Here**:
- [START_HERE.md](START_HERE.md) - Onboarding guide (15 min)
- [ROADMAP_2025.md](ROADMAP_2025.md) - Strategy & priorities
- [PRIORITIZED_TASKS.md](PRIORITIZED_TASKS.md) - Current work

**Active Development**:
- [PLUGIN_DATA_MODEL_EVOLUTION.md](PLUGIN_DATA_MODEL_EVOLUTION.md) - Architecture evolution
- [NVIM_SMART_COLUMN_COMPLETION.md](NVIM_SMART_COLUMN_COMPLETION.md) - Column features
- [PERFORMANCE.md](PERFORMANCE.md) - Benchmarks

**Reference**:
- [FUNCTION_REFERENCE.md](FUNCTION_REFERENCE.md) - SQL functions
- [INDEX.md](INDEX.md) - Find anything
- `archive/` - Historical docs (~193 docs)

## 🎨 Design Philosophy

**"Make the common case fast"**

1. **Plugin over TUI** - Leverage 30 years of editor evolution
2. **Simple over complex** - Unless complexity is hidden
3. **Fast feedback** - Milliseconds, not seconds
4. **Clear errors** - Tell users what went wrong
5. **Composability** - Small pieces, loosely joined

**Decision Framework**:
```
Does it improve daily workflow?     → P0/P1
Is it plugin-related?               → Higher priority
Is it TUI-related?                  → Lower priority
Needs >30K rows to matter?          → Defer
```

## 🚀 Key Innovations

### 1. WEB CTEs
Query HTTP APIs as SQL tables:
```sql
WITH WEB users AS (
    URL 'https://api.example.com/users'
    FORMAT JSON
    JSON_PATH 'users'
)
SELECT * FROM users WHERE active = true;
```

### 2. Smart Column Expansion
`\sE` on `SELECT *` discovers actual columns from query execution

### 3. Instant Cardinality
`\srD` on column name shows top 100 distinct values with counts

### 4. FIX Protocol Integration
Tag-based JSON syntax for FIX message analysis with group aggregation

## 📊 Usage Patterns

**Typical Workflow**:
```
1. Write SQL in .sql buffer
2. \sx to execute (see results)
3. \sE to expand SELECT *
4. \srD to analyze column cardinality
5. Refine query based on insights
6. \sc to copy results (TSV to clipboard)
7. Paste into spreadsheet or docs
```

**Power User**:
- Chain CTEs to build complex analysis
- Reference previous query results
- Custom projections via Lua (planned)
- Multiple rendering styles (planned)

## 🎯 Success Metrics

**Developer Experience**:
- Query to insight: **<30 seconds**
- Common operations: **<3 keystrokes**
- Plugin startup: **<100ms**
- No context switching

**Stability**:
- Zero crashes in normal workflow
- Graceful error handling
- Fast feedback on all operations

## 🔮 Future Vision (6-12 months)

### Data Model Evolution
- Structured data storage (no buffer parsing)
- Multiple renderers (table/compact/summary)
- CTE composition (chain queries)
- Lua scripting for custom transforms

### Advanced Features
- Live queries with auto-refresh
- Visual query diff
- Collaborative CTE sharing
- Query history with undo/redo
- Sparklines and decorations
- Custom aggregations in Lua

### Extensibility
- User-supplied Lua transforms
- Custom renderer plugins
- Domain-specific functions
- Integration with external tools

## 📈 Project Stats

- **Lines of Code**: ~50K (Rust + Lua)
- **Documentation**: ~20 active docs, ~193 archived
- **SQL Functions**: 100+
- **Test Coverage**: Integration + unit tests
- **Performance**: Adequate for <30K rows
- **Development Focus**: 80% plugin, 15% core, 5% TUI

## 🤝 Contributing

1. Read [START_HERE.md](START_HERE.md)
2. Check [PRIORITIZED_TASKS.md](PRIORITIZED_TASKS.md) for P0/P1 items
3. Pick a "Quick Win" or P0 task
4. Focus on plugin improvements
5. Test in real workflow before PR

**Good First Issues**:
- P0-1: Fuzzy search strictness
- P0-2: FIX proxy documentation
- Quick wins: See PRIORITIZED_TASKS.md

## 📞 Resources

- **Repository**: [github.com/TimelordUK/sql-cli](https://github.com/TimelordUK/sql-cli)
- **Documentation**: `/docs` directory
- **Examples**: `/examples` directory
- **Tests**: `/tests` directory

---

**TL;DR**: Fast SQL tool. Great Neovim plugin. Focus on developer experience. Plugin-first architecture. Data model evolution coming soon. 🚀
