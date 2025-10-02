# SQL CLI Roadmap 2025

**Last Updated**: 2025-10-02
**Strategic Focus**: Neovim Plugin Enhancement & Developer Experience

## 🎯 Strategic Direction

Based on current usage patterns and ROI analysis:

1. **Neovim Plugin** (80% of effort) - Highest ROI
   - Leverage 30+ years of text editor evolution
   - Easier to develop and maintain than TUI
   - More powerful integration capabilities
   - Better developer experience

2. **Core SQL Engine** (15% of effort) - Stability & Maintenance
   - Current performance adequate for <30K rows
   - Focus on bug fixes and stability
   - Defer major optimizations (CUDA, parallel execution)

3. **TUI** (5% of effort) - Maintenance Only
   - Functional but high maintenance cost
   - Plugin supersedes most TUI use cases
   - Keep stable, no new features

## 📋 Active Priorities (Next 3 Months)

### P0 - Immediate Impact (Plugin UX)

#### 1. Stricter Fuzzy Search
**Status**: Bug - Too permissive
**Impact**: High - Daily workflow friction
**Effort**: Low
**Details**:
- Current fzf search too broad
- Need stricter matching mode option
- Should work out-of-box with fzf
- Investigate fzf exact match flags (`--exact`, `+i`)

#### 2. Column Visibility Management
**Status**: New Feature Request
**Impact**: Medium-High - Data exploration
**Effort**: Medium
**Features**:
- Hide/show columns interactively
- Remember column visibility per query
- Quick toggle for commonly hidden columns
- Integration with `\sx` export (respect hidden columns)

#### 3. FIX Protocol Proxy Enhancement
**Status**: Partially Complete - Needs Documentation
**Impact**: High - Core use case
**Effort**: Low
**Details**:
- Tag-based JSON syntax working
- Group tag aggregation (200|400|800)
- Direct upload from FIX engine
- Document and add examples

### P1 - Short Term Enhancements

#### 4. Plugin Keybinding Refinement
**Status**: Ongoing
**Impact**: Medium
**Effort**: Low
**Details**:
- Review all `\s*` keybindings for conflicts
- Add which-key integration hints
- Document common workflows

#### 5. Smart Expansion Improvements
**Status**: Recently Added - Needs Refinement
**Impact**: Medium
**Effort**: Low
**Details**:
- Handle edge cases in CTE detection
- Better error messages
- Performance optimization for large schemas

### P2 - Medium Term Goals

#### 6. WEB CTE Enhancements
**Status**: Working Well - Room for Improvement
**Impact**: Medium
**Effort**: Medium
**Features**:
- Authentication token management
- Request timeout configuration
- Retry logic
- Better error reporting

#### 7. Export Format Expansion
**Status**: TSV/CSV/HTML Working
**Impact**: Low-Medium
**Effort**: Low
**Additions**:
- Markdown tables
- Excel/XLSX (via library)
- JSON export options

## 🏗️ Strategic Initiatives

### Plugin Data Model Evolution
**Status**: Architecture Planning
**Priority**: P2 (Foundation for future)
**Document**: [PLUGIN_DATA_MODEL_EVOLUTION.md](PLUGIN_DATA_MODEL_EVOLUTION.md)

**Vision**: Transform plugin from "buffer parser" to "data-aware application"

**Key Phases**:
1. **Data Layer** (P2, 1 month) - Stop parsing buffers, store structured data
2. **Multiple Renderers** (P2, 2 months) - Table, compact, summary, tree views
3. **CTE Composition** (P2, 3 months) - Reference previous query results
4. **Lua Scripting** (P3, future) - User-supplied transformations
5. **Smart Decorations** (P3, future) - Annotations, summaries, sparklines

**Benefits**:
- No more fragile buffer parsing
- Composable queries (chain results)
- Multiple visualization styles
- Extensibility via Lua
- Foundation for advanced features

**Quick Win Now**: Store query metadata in `vim.b.sql_cli_result`

## 🔧 Known Issues (Backlog)

### Window Function Ranking Bug
**Status**: Documented
**Priority**: P3 (Low - not blocking)
**Impact**: Low - Rare edge case
**Details**: Window functions don't always start from 0 when ranking

### TUI Performance Issues
**Status**: Known, Deferred
**Priority**: P4 (Very Low)
**Impact**: Low - Plugin is preferred interface
**Details**: TUI is functional but not optimized

## 🚀 Future Exploration (6+ Months)

### CUDA Offloading
**Status**: Research Phase
**Priority**: P4
**ROI**: Uncertain - need >100K row use cases
**Prerequisites**:
- Identify bottleneck operations
- Benchmark GPU vs CPU at scale
- Cost/benefit analysis

### Parallel Query Execution
**Status**: Design Phase
**Priority**: P4
**ROI**: Low at current data volumes
**Details**: Branch-level parallelization for CTEs

### Advanced Type System
**Status**: Conceptual
**Priority**: P5
**Details**: Stronger typing, type inference, schema validation

## 📂 Documentation Reorganization

### Keep in `/docs` (Active/Reference)
- ROADMAP_2025.md (this file)
- PERFORMANCE.md
- NVIM_SMART_COLUMN_COMPLETION.md
- PLUGIN_FEATURES_ROADMAP.md
- FUNCTION_REFERENCE.md
- ARCHITECTURE.md (if exists)

### Move to `/docs/archive/completed/`
- *_COMPLETE.md
- *_IMPLEMENTED.md
- *_STATUS.md (if fully implemented)
- MIGRATION_*.md (completed migrations)

### Move to `/docs/archive/research/`
- CUDA_*.md
- PARALLEL_*.md
- Performance research docs
- Experimental features

### Move to `/docs/archive/tui/`
- TUI_*.md
- Buffer state docs
- TUI-specific architecture
- Key handling docs

### Move to `/docs/reference/`
- AST_FLOW.md
- PARSING_*.md
- Technical deep-dives
- Implementation details

## 🎯 Success Metrics

### Plugin Adoption
- [ ] <5 sec from query idea to results
- [ ] <3 keystrokes for common operations
- [ ] Zero context switching for data exploration

### Developer Experience
- [ ] New feature implementation <1 day
- [ ] Plugin features easier than TUI equivalent
- [ ] Clear feedback on all operations

### Stability
- [ ] No crashes in normal workflow
- [ ] Graceful degradation on errors
- [ ] Fast startup (<100ms for plugin)

## 📝 Notes

**Key Insight**: The Neovim plugin has proven to be the most productive investment. Focus should remain on plugin enhancements that reduce friction in daily data exploration workflows.

**Philosophy**: "Make the common case fast, make the edge cases possible."

**Decision Framework**:
1. Does it improve daily workflow? → P0-P1
2. Does it fix a blocking bug? → P0
3. Is it plugin-related? → Higher priority
4. Is it TUI-related? → Lower priority
5. Does it need >30K rows to matter? → Defer

## 🤝 Contributing

When proposing new features, consider:
- Will this improve the plugin experience?
- What's the maintenance burden?
- Can this be a plugin feature instead of core?
- Does this solve a real daily workflow problem?
