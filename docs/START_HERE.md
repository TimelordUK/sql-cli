# 🚀 Start Here - SQL CLI Development Guide

**Last Updated**: 2025-10-02

## 📍 You Are Here

This is a SQL query tool with:
- **Rust core** - Fast in-memory SQL engine
- **Neovim plugin** - Primary interface (80% of development)
- **TUI** - Legacy interface (maintenance only)

## 🎯 Quick Start for Contributors

### 1. Understand the Strategy
Read: **[ROADMAP_2025.md](ROADMAP_2025.md)** (5 min read)

**TL;DR**: We focus on the Neovim plugin because it's:
- Easier to develop
- More powerful (30 years of text editor evolution)
- Better developer experience
- Where most value is created

### 2. Check Current Priorities
Read: **[PRIORITIZED_TASKS.md](PRIORITIZED_TASKS.md)** (2 min read)

**This Week**:
- P0: Fix fuzzy search (too permissive)
- P0: Document FIX proxy integration

**This Month**:
- P1: Add column hide/show feature
- P1: Improve smart expansion edge cases

### 3. Find What You Need
See: **[INDEX.md](INDEX.md)**

Common lookups:
- Plugin development → `NVIM_*.md` docs
- Function reference → `FUNCTION_REFERENCE.md`
- Performance → `PERFORMANCE.md`
- Old/completed work → `archive/`

## 🔧 Development Workflow

### Plugin Development (Most Common)

```bash
# 1. Edit plugin code
nvim nvim-plugin/lua/sql-cli/results.lua

# 2. Test in Neovim
# :source % to reload
# \sx to execute query
# \sE to expand columns
# \srD to show distinct values

# 3. Build if you changed Rust
cargo build --release

# 4. Commit
git add -A
git commit -m "feat(nvim): your feature"
git push
```

### Core Engine Changes (Less Common)

```bash
# 1. Edit Rust code
nvim src/sql/recursive_parser.rs

# 2. Build and test
cargo build --release
cargo test

# 3. Test CLI
./target/release/sql-cli -q "SELECT 1+1" -o csv

# 4. Format and commit
cargo fmt
git commit -m "feat(core): your feature"
```

## 📚 Documentation Structure

```
docs/
├── START_HERE.md          ← You are here
├── ROADMAP_2025.md        ← Strategy & priorities
├── PRIORITIZED_TASKS.md   ← What to work on
├── INDEX.md               ← Find anything
│
├── NVIM_*.md              ← Plugin docs (active)
├── PERFORMANCE.md         ← Benchmarks
├── FUNCTION_REFERENCE.md  ← SQL functions
│
└── archive/               ← Historical docs
    ├── completed/         ← Finished features
    ├── research/          ← Future ideas (CUDA, etc)
    ├── tui/              ← TUI docs (maintenance)
    └── reference/         ← Technical deep dives
```

## 🎨 Design Philosophy

**"Make the common case fast"**

- Plugin over TUI (easier, more powerful)
- Simple over complex (unless complexity is hidden)
- Fast feedback (milliseconds, not seconds)
- Clear errors (tell user what went wrong)

**Decision Framework**:
1. Does it improve daily workflow? → High priority
2. Is it plugin-related? → Higher priority
3. Is it TUI-related? → Lower priority
4. Needs >30K rows? → Defer

## 🐛 Common Tasks

### Adding a New SQL Function
1. Edit `src/sql/functions/[category].rs`
2. Implement `SqlFunction` trait
3. Register in `src/sql/functions/mod.rs`
4. Rebuild: `cargo build --release`
5. Test: `./target/release/sql-cli --list-functions`

### Adding a Plugin Keybinding
1. Edit `nvim-plugin/lua/sql-cli/init.lua`
2. Add vim.keymap.set() call
3. Reload in Neovim: `:source %`
4. Test the keybinding

### Fixing a Bug
1. Check if it's already documented (grep docs)
2. Reproduce in minimal case
3. Fix in appropriate file
4. Test thoroughly
5. Update CHANGELOG.md if user-facing

## ⚡ Performance Guidelines

**Current Performance** (see PERFORMANCE.md):
- SELECT: 8ms @ 100K rows
- JOINs: <40ms @ 100K rows
- GROUP BY: ~500ms @ 100K rows
- Window functions: ~1.2s @ 100K rows

**Optimization Rule**: Don't optimize unless:
- It affects <30K row queries (our common case)
- Users are reporting issues
- It's a plugin performance issue

## 🚫 Don't Waste Time On

- TUI enhancements (plugin is better)
- CUDA/GPU (no need at current scale)
- Parallel execution (not a bottleneck)
- Micro-optimizations (measure first)

## ✅ Good First Issues

1. **Fuzzy search strictness** (P0) - Add `--exact` flag to fzf
2. **FIX proxy docs** (P0) - Write examples
3. **Add quick wins** - See PRIORITIZED_TASKS.md
4. **Improve error messages** - Always helpful
5. **Add examples** - More SQL query examples

## 🤝 Getting Help

1. Check docs: `grep -r "your topic" docs/`
2. Check examples: Look in `examples/` or `tests/`
3. Check git history: `git log --grep="keyword"`
4. Read the code: It's well-commented

## 📊 Success Metrics

You're successful when:
- Common operations take <3 keystrokes
- Query to insight in <30 seconds
- Plugin feels instant (<100ms)
- No context switching needed
- Code is obvious in 6 months

## 🎯 Your First Contribution

**Recommended Path**:
1. Read ROADMAP_2025.md (understand strategy)
2. Read PRIORITIZED_TASKS.md (see P0 tasks)
3. Pick something from "Quick Wins"
4. Make the change
5. Test in Neovim
6. Submit PR

**Example**: Fix fuzzy search
- Time: 30 minutes
- Impact: High (daily use)
- Difficulty: Easy
- Files: 1-2

---

**Welcome aboard! Focus on the plugin, make it delightful.** 🚀
