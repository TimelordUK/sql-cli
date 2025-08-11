# Master Refactoring Roadmap

## Overview
This document outlines the complete refactoring journey from our current state to a fully reactive, mathematically-capable SQL CLI with proper state management and DataTable/DataView architecture.

## Phase 1: Complete State Migration to AppStateContainer ✅ (70% Complete)
**Timeline: 1-2 weeks**
**Approach: Small incremental branches (V-branches), test, merge, repeat**

### Completed ✅
- [x] V1-V10: Initial widget extraction and modularization
- [x] V11-V16: AppStateContainer foundation
- [x] V17: ResultsState migration
- [x] V18-V24: SearchState, FilterState migrations
- [x] V25: NavigationState migration with dual-lock modes

### Remaining State Migrations
- [ ] V26: SortState migration
- [ ] V27: SelectionState migration  
- [ ] V28: ClipboardState migration
- [ ] V29: ColumnSearchState migration
- [ ] V30: ChordState migration
- [ ] V31: Final cleanup - remove all state from enhanced_tui.rs

**Success Criteria:**
- All state lives in AppStateContainer
- TUI is purely orchestration layer
- No state coupling in TUI

## Phase 2: DataTable/DataView Refactoring 
**Timeline: 2-3 weeks**
**Approach: Implement C#-inspired DataTable/DataView pattern**

Reference: `docs/DATATABLE_DATAVIEW_ARCHITECTURE.md`, `docs/DataView.md`

### Core Components
1. **DataTable** (Immutable Data Store)
   - Holds raw query results
   - Column definitions and types
   - No presentation logic
   - Shared across views

2. **DataView** (Presentation Layer)
   - References DataTable (no data duplication)
   - Sorting, filtering, column visibility
   - Virtual scrolling
   - Multiple views of same data

3. **Benefits**
   - Memory efficiency (single data copy)
   - Multiple simultaneous views
   - Clean separation of data/presentation
   - Prepared for reactive updates

### Implementation Steps
- [ ] V32: Create DataTable structure
- [ ] V33: Migrate QueryResponse to DataTable
- [ ] V34: Create DataView interface
- [ ] V35: Implement basic DataView (sorting/filtering)
- [ ] V36: Virtual scrolling in DataView
- [ ] V37: Multiple DataView support
- [ ] V38: Remove old buffer/results structures

## Phase 3: ReactJS/Redux Style State Management
**Timeline: 2-3 weeks**
**Approach: Event-driven reactive architecture**

### Core Concepts
1. **Actions** (User/System Events)
   ```rust
   enum Action {
       SetResults(DataTable),
       ApplyFilter(FilterSpec),
       SortColumn(usize, SortOrder),
       NavigateTo(usize, usize),
   }
   ```

2. **Reducers** (Pure State Transformations)
   ```rust
   fn reduce(state: AppState, action: Action) -> AppState
   ```

3. **Subscriptions** (Widget Updates)
   - Widgets subscribe to state slices
   - Automatic re-render on change
   - No manual state synchronization

### Implementation Steps
- [ ] V39: Define Action enum hierarchy
- [ ] V40: Implement reducer pattern
- [ ] V41: Create subscription system
- [ ] V42: Convert first widget to reactive
- [ ] V43-V50: Migrate all widgets to reactive
- [ ] V51: Remove all manual state updates
- [ ] V52: Add state time-travel debugging

## Phase 4: Mathematical Expression Support in SQL
**Timeline: 2-3 weeks**
**Approach: Extend parser and evaluator for computed columns**

Reference: `docs/SQL_MATH_EXTENSIONS.md`

### Features
1. **Basic Arithmetic**
   ```sql
   SELECT quantity * price AS total FROM trades
   SELECT (bid + ask) / 2 AS mid_price FROM quotes
   ```

2. **Functions**
   ```sql
   SELECT ROUND(price, 2), ABS(pnl), SQRT(variance) FROM results
   ```

3. **Aggregations with Math**
   ```sql
   SELECT SUM(quantity * price) / SUM(quantity) AS vwap
   ```

### Implementation Steps
- [ ] V53: Extend AST for expressions
- [ ] V54: Add expression parser
- [ ] V55: Implement expression evaluator
- [ ] V56: Type checking for expressions
- [ ] V57: Function library (ROUND, ABS, etc.)
- [ ] V58: Aggregate function support
- [ ] V59: Performance optimization
- [ ] V60: Expression caching

## Timeline Summary
- **Week 1-2**: Complete state migration (V26-V31)
- **Week 3-5**: DataTable/DataView refactoring (V32-V38)
- **Week 6-8**: ReactJS/Redux architecture (V39-V52)
- **Week 9-11**: Mathematical expressions (V53-V60)

**Total Timeline: ~2.5-3 months**

## Key Principles
1. **Incremental**: Small, testable changes
2. **Backwards Compatible**: Keep existing functionality working
3. **Test-Driven**: Each V-branch includes tests
4. **Merge Often**: Main branch always stable
5. **Document**: Update docs with each phase

## Success Metrics
- [ ] Zero state in TUI layer
- [ ] < 100ms response time for all operations
- [ ] Support for 1M+ row datasets
- [ ] Full mathematical expression support
- [ ] Time-travel debugging capability
- [ ] Multiple simultaneous data views

## Dependencies
- Current refactoring documents in `docs/refactoring/`
- DataTable/DataView design in `docs/DATATABLE_DATAVIEW_ARCHITECTURE.md`
- Math extensions spec in `docs/SQL_MATH_EXTENSIONS.md`
- State management patterns in `docs/STATE_MANAGEMENT_REFACTOR.md`

## Risk Mitigation
- Small incremental changes reduce risk
- Each V-branch can be reverted independently
- Comprehensive testing at each step
- Regular merges to main keep codebase stable
- Backwards compatibility maintained throughout

## Next Immediate Step
Start V26 branch for SortState migration following the same pattern as V25:
1. Move SortState to AppStateContainer
2. Update TUI to use AppStateContainer methods
3. Test thoroughly
4. Merge to main
5. Continue with V27