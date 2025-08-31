# TUI Refactoring Plan

## Current Problems

The enhanced_tui.rs has grown to ~7000 lines and has hit a complexity wall with:
- Too much coupled state
- Difficult to add new features
- Hard to test individual components
- State synchronization issues

## Major Issues

### 1. State Coupling
- Multiple overlapping state representations
- Direct field access instead of encapsulation
- Circular dependencies between components
- Manual state synchronization

### 2. Event Handling
- Massive match statements (500+ lines)
- Duplicated key handling logic
- Mode-specific branches scattered everywhere
- Hard to add new keybindings

### 3. Rendering
- Monolithic render function
- Mixed concerns (data + presentation)
- Hardcoded layout decisions
- No component reusability

## Proposed Architecture

### Phase 1: State Management (Priority: HIGH)
- [x] Extract AppStateContainer (DONE)
- [x] Move to RefCell for interior mutability (DONE)
- [ ] Complete migration from direct field access
- [ ] Remove all Option<AppStateContainer> checks
- [ ] Centralize all state mutations

### Phase 2: Event System (Priority: HIGH)
- [x] Extract KeyDispatcher (DONE)
- [ ] Create ActionHandler for business logic
- [ ] Implement Command pattern for actions
- [ ] Add undo/redo support properly
- [ ] Create event bus for component communication

### Phase 3: Component System (Priority: MEDIUM)
- [ ] Extract DataTable widget
- [ ] Extract CommandBar widget
- [ ] Extract StatusBar widget
- [ ] Create Layout manager
- [ ] Implement proper widget lifecycle

### Phase 4: Rendering Pipeline (Priority: MEDIUM)
- [ ] Create RenderContext abstraction
- [ ] Implement dirty region tracking
- [ ] Add viewport virtualization
- [ ] Optimize for large datasets
- [ ] Add proper scrollbar support

### Phase 5: Testing (Priority: LOW)
- [ ] Add unit tests for each component
- [ ] Create integration test harness
- [ ] Add performance benchmarks
- [ ] Document component APIs

## Immediate TODOs

### Quick Wins (Can do now)
1. [x] Fix history corruption bug
2. [x] Add dual logging system
3. [x] Add configurable cell styles
4. [ ] Clean up unused code
5. [ ] Fix remaining borrow checker issues

### Next Sprint
1. Complete Option<AppStateContainer> removal
2. Finish extracting remaining state into AppStateContainer
3. Create proper Action enum for all operations
4. Implement proper mode stack (not just vec of modes)
5. Add proper error handling (Result types everywhere)

### Technical Debt to Address
1. Remove all `.unwrap()` calls
2. Replace panics with proper error handling
3. Add proper logging throughout
4. Document all public APIs
5. Create examples for each component

## Component Breakdown

### Core Components (must refactor)
- `EnhancedTuiApp` → Split into App + View
- `Buffer` → Extract to separate crate
- `NavigationState` → Merge into AppStateContainer
- Mode handling → Extract ModeManager

### Widgets (can be isolated)
- SearchWidget
- FilterWidget
- HelpWidget
- StatsWidget
- DebugWidget
- HistoryWidget

### Services (already extracted)
- DebugService ✓
- ResultsCache ✓
- HistoryManager (partially)
- YankManager ✓

## Migration Strategy

### Step 1: Freeze Features
- No new features until refactoring complete
- Only bug fixes allowed
- Document all existing behavior

### Step 2: Extract Components
- One component at a time
- Maintain backward compatibility
- Add tests for each extraction

### Step 3: Rebuild Core
- New event loop
- New rendering pipeline
- New state management

### Step 4: Optimize
- Performance profiling
- Memory optimization
- Startup time improvement

## Success Metrics

- [ ] File size: enhanced_tui.rs < 1000 lines
- [ ] Test coverage: > 70%
- [ ] Startup time: < 100ms
- [ ] Memory usage: < 50MB for 1M rows
- [ ] Key response time: < 10ms

## Long-term Vision

### Architecture Goals
- Plugin system for extensions
- Scriptable with Lua/Rhai
- Themeable UI
- Multiple backend support (not just crossterm)
- Client-server split for remote operation

### Feature Goals
- Multi-tab support
- Split panes
- Integrated SQL editor
- Query history with search
- Export to multiple formats
- Visualization support (charts/graphs)

## Notes

The refactoring should be done incrementally without breaking existing functionality. Each phase should be completed and tested before moving to the next. The goal is to make the codebase maintainable and extensible for the long term.

Priority should be given to state management and event handling as these are the biggest blockers for new features.