# Documentation Index

**Quick Navigation for Active Development**

## 🎯 Start Here

- **[ROADMAP_2025.md](ROADMAP_2025.md)** - Strategic priorities and focus areas
- **[PERFORMANCE.md](PERFORMANCE.md)** - Benchmark results and performance characteristics
- **[FUNCTION_REFERENCE.md](FUNCTION_REFERENCE.md)** - Complete SQL function documentation

## 🔌 Neovim Plugin (Primary Focus)

- **[NVIM_SMART_COLUMN_COMPLETION.md](NVIM_SMART_COLUMN_COMPLETION.md)** - Smart column expansion design
- **[PLUGIN_FEATURES_ROADMAP.md](PLUGIN_FEATURES_ROADMAP.md)** - Plugin feature planning
- **[NVIM_REFACTORING_DESIGN.md](NVIM_REFACTORING_DESIGN.md)** - SQL refactoring tools

## 📚 Reference Material

### Technical Architecture
- [archive/reference/](archive/reference/) - AST, parsing, and implementation details

### Completed Work
- [archive/completed/](archive/completed/) - Finished features and migrations

### Research & Future Work
- [archive/research/](archive/research/) - CUDA, parallel execution, experimental features

### TUI (Maintenance Mode)
- [archive/tui/](archive/tui/) - TUI-specific documentation

## 🔍 Quick Lookups

### By Topic

**Data Sources**
- WEB CTE documentation (search for "WEB CTE")
- CSV/JSON loading (search for "datasource")
- FIX protocol proxy (ROADMAP P0-3)

**Features**
- Window functions → FUNCTION_REFERENCE.md
- Aggregates → FUNCTION_REFERENCE.md
- CTEs → Search "CTE" in active docs
- Fuzzy search → ROADMAP P0-1

**Plugin Operations**
- Column expansion (`\sE`) → NVIM_SMART_COLUMN_COMPLETION.md
- Distinct values (`\srD`) → ROADMAP (recently added)
- Export (`\sx`) → Plugin docs
- Execute (`\sx`) → Plugin docs

## 📊 Doc Statistics

- Total: ~213 docs
- Active: ~20 docs
- Archived: ~193 docs
- Focus: 80% plugin, 15% engine, 5% TUI

## 🧹 Archive Organization

```
docs/
├── archive/
│   ├── completed/      # Finished features, migrations, status docs
│   ├── research/       # CUDA, parallel, experimental
│   ├── tui/           # TUI-specific (maintenance mode)
│   └── reference/      # Technical deep dives
├── ROADMAP_2025.md    # Strategic direction
├── PERFORMANCE.md     # Benchmarks
└── [Active docs]      # Current development focus
```

## 💡 Using This Index

1. **Starting new work?** → Check ROADMAP_2025.md for priorities
2. **Need technical details?** → Check archive/reference/
3. **Looking for a feature?** → Check if it's in archive/completed/
4. **Plugin development?** → Check NVIM_*.md docs
5. **Performance question?** → PERFORMANCE.md

---

**Philosophy**: Keep active docs minimal. Archive the rest. Focus on plugin excellence.
