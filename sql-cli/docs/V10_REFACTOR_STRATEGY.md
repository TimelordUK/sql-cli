# V10 Refactor Strategy

## Current Situation
The `enhanced_tui.rs` file has grown to ~6000 lines with tightly coupled state management. We need to decompose it into manageable, testable widgets while maintaining functionality.

## Strategy: Widget-First with State Manager Integration

### Phase 1: Widget Extraction (Current)
Extract widgets one-by-one using the established pattern:

1. **HistoryWidget** ✅ - Reference implementation complete
2. **StatsWidget** - Next target (relatively isolated)
3. **SearchWidget** - Combines search/filter/fuzzy filter
4. **DebugWidget** - Enhance existing partial extraction
5. **HelpWidget** - Simple, good for practice
6. **EditorWidget** - Complex, tackle last
7. **ResultsWidget** - Core functionality, needs careful planning

### Phase 2: State Manager Integration
After 2-3 widgets are extracted:
1. Integrate StateManager with BufferManager
2. Update widgets to use StateManager for transitions
3. Test state preservation across mode switches

### Phase 3: Input System Refactor
The input system is deeply embedded. Strategy:
1. Create InputWidget abstraction
2. Route all input through InputWidget
3. Separate SQL editing from other input modes
4. Enable multi-line editing in isolated widget

### Phase 4: Buffer System Enhancement
1. Make buffers more independent
2. Each buffer owns its widget instances
3. Enable true multi-buffer editing

## Why History Widget First?

1. **Clear boundaries** - History mode is well-isolated
2. **Manageable scope** - ~400 lines of extraction
3. **Low risk** - Doesn't affect core query execution
4. **Good practice** - Establishes the pattern
5. **Immediate value** - Can be tested independently

## Implementation Approach

### For Each Widget:
1. **Extract State** - Move to dedicated State struct
2. **Extract Logic** - Move event handling to widget
3. **Extract Rendering** - Move all render methods
4. **Define Actions** - Create action enum for communication
5. **Integrate** - Wire up in main TUI
6. **Test** - Create unit tests for widget
7. **Document** - Update documentation

### Integration Pattern:
```rust
// Before (in enhanced_tui.rs)
match mode {
    AppMode::History => {
        // 200+ lines of history handling
    }
}

// After
match mode {
    AppMode::History => {
        match self.history_widget.handle_key(key) {
            HistoryAction::ExecuteCommand(cmd) => self.execute(cmd),
            HistoryAction::Exit => self.exit_mode(),
            _ => {}
        }
    }
}
```

## Success Metrics

1. **File size**: enhanced_tui.rs < 2000 lines
2. **Widget independence**: Each widget compiles/tests alone
3. **State clarity**: Clear ownership of state
4. **Testability**: 80%+ code coverage on widgets
5. **Performance**: No regression in response time

## Risks and Mitigations

### Risk: Breaking existing functionality
**Mitigation**: Extract one widget at a time, test thoroughly

### Risk: State synchronization issues
**Mitigation**: StateManager provides centralized state coordination

### Risk: Performance degradation
**Mitigation**: Profile before/after, optimize hot paths

### Risk: Increased complexity
**Mitigation**: Clear documentation, consistent patterns

## Timeline Estimate

- **Week 1**: HistoryWidget + StatsWidget
- **Week 2**: SearchWidget + StateManager integration
- **Week 3**: DebugWidget + HelpWidget
- **Week 4**: Input system refactor
- **Week 5**: EditorWidget extraction
- **Week 6**: ResultsWidget + Buffer enhancement
- **Week 7**: Testing and optimization
- **Week 8**: Documentation and cleanup

## Current Status

✅ **Completed**:
- StateManager design and implementation
- HistoryWidget extraction
- Widget extraction pattern documented

🔄 **In Progress**:
- HistoryWidget integration with enhanced_tui

📋 **Next Steps**:
1. Complete HistoryWidget integration
2. Test history functionality
3. Begin StatsWidget extraction
4. Integrate StateManager with first 2 widgets

## Notes

- Each widget extraction makes the next one easier
- The pattern becomes clearer with repetition
- Early widgets are learning experiences
- Later widgets benefit from established patterns
- Final result will be much more maintainable