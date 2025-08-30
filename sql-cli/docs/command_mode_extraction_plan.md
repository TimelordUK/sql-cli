# Command Mode Extraction Plan

## Overview
This document outlines a phased approach to extract command mode functionality from `enhanced_tui.rs` into its own module. The command mode is currently deeply integrated with the main TUI, sharing state and behavior with other modes.

## Current State Analysis

### Command Mode Dependencies

#### 1. State Dependencies
Command mode currently accesses multiple state containers:
- **AppStateContainer** - Primary state holder
  - `get_input_text()` / `set_input_text()`
  - `get_input_cursor_position()` / `set_input_cursor_position()`
  - `get_edit_mode()` / `set_edit_mode()`
  - `execute_query()` - Query execution
  - `get_completions()` - SQL completions
  - Mode transitions
  
- **ShadowStateManager** - State synchronization
  - Mode tracking and observation
  - Vim state management
  - Search state coordination
  
- **BufferManager** - Text buffer operations
  - Command history
  - Query buffer management
  - Input text storage
  
- **CursorManager** - Cursor positioning
  - Input cursor tracking
  - Multi-line navigation
  
- **StateCoordinator** - State synchronization
  - Cross-component state updates
  - Event propagation

#### 2. Behavioral Dependencies
Command mode handles multiple input types:
- **Text editing** (InputBehavior trait)
  - Character insertion/deletion
  - Word navigation
  - Line operations (kill-line, etc.)
  
- **History navigation**
  - Ctrl+P/N for previous/next
  - History search
  - Command recall
  
- **SQL-specific operations**
  - Tab completion
  - Syntax highlighting
  - Query execution (Enter)
  - Asterisk expansion
  
- **Mode transitions**
  - Escape to Results
  - Arrow keys for navigation
  - Function keys (F1-F12)

#### 3. Rendering Dependencies
- Syntax highlighting (SQL keywords)
- Cursor position calculation
- Horizontal scrolling for long queries
- Multi-line display (currently disabled)
- Status line updates

## Extraction Phases

### Phase 1: Create CommandEditor Struct (In-place refactoring)
**Goal**: Consolidate command mode logic without moving files

1. Create `CommandEditor` struct within enhanced_tui.rs:
```rust
struct CommandEditor {
    // References to shared state
    state_container: Rc<RefCell<AppStateContainer>>,
    shadow_state: Rc<RefCell<ShadowStateManager>>,
    buffer_manager: BufferManager,
    cursor_manager: CursorManager,
    
    // Command-specific state
    history_handler: HistoryInputHandler,
    completion_state: CompletionState,
    scroll_offset: usize,
}
```

2. Move command-specific methods to CommandEditor:
   - `handle_command_input()`
   - `try_handle_text_editing()`
   - `try_handle_history_navigation()`
   - `try_handle_buffer_operations()` (command-specific parts)
   - Query execution logic
   - Completion handling

3. Keep shared interfaces in EnhancedTui:
   - `get_input_text()` / `set_input_text()`
   - Mode transition triggers
   - Rendering calls

### Phase 2: Extract Common Input Traits
**Goal**: Define clear interfaces for command mode

1. Create new trait `CommandInputHandler`:
```rust
trait CommandInputHandler {
    fn handle_input(&mut self, key: KeyEvent) -> Result<InputResult>;
    fn get_text(&self) -> &str;
    fn set_text(&mut self, text: String);
    fn get_cursor(&self) -> usize;
    fn set_cursor(&mut self, pos: usize);
    fn execute_query(&mut self) -> Result<()>;
}
```

2. Create `InputResult` enum:
```rust
enum InputResult {
    Handled,
    NotHandled,
    ModeChange(AppMode),
    ExecuteQuery,
    Exit,
}
```

3. Implement trait for CommandEditor

### Phase 3: Move to Separate Module
**Goal**: Physical separation while maintaining integration

1. Create `src/ui/command_editor/mod.rs`
2. Move CommandEditor struct and implementation
3. Create sub-modules:
   - `command_editor/input.rs` - Input handling
   - `command_editor/history.rs` - History navigation
   - `command_editor/completion.rs` - SQL completions
   - `command_editor/render.rs` - Rendering helpers

4. Update enhanced_tui.rs to use CommandEditor:
```rust
pub struct EnhancedTuiApp {
    // ... other fields ...
    command_editor: CommandEditor,
}
```

### Phase 4: State Abstraction
**Goal**: Reduce direct state dependencies

1. Create `CommandState` wrapper:
```rust
struct CommandState {
    text: String,
    cursor: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    completions: Vec<String>,
    completion_index: Option<usize>,
}
```

2. Use StateCoordinator for state updates:
   - Define command-specific events
   - Route through StateCoordinator
   - Remove direct state manipulation

3. Create `CommandStateProvider` trait:
```rust
trait CommandStateProvider {
    fn get_command_state(&self) -> &CommandState;
    fn update_command_state<F>(&mut self, f: F) 
        where F: FnOnce(&mut CommandState);
}
```

### Phase 5: Rendering Separation
**Goal**: Independent rendering logic

1. Create `CommandRenderer`:
```rust
struct CommandRenderer {
    syntax_highlighter: SqlSyntaxHighlighter,
    scroll_manager: HorizontalScrollManager,
}
```

2. Implement rendering trait:
```rust
impl WidgetRenderer for CommandRenderer {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &CommandState);
}
```

3. Move rendering logic from enhanced_tui

### Phase 6: Final Integration
**Goal**: Clean, modular architecture

1. Update enhanced_tui to delegate:
```rust
match mode {
    AppMode::Command => {
        let result = self.command_editor.handle_input(key)?;
        self.handle_command_result(result)
    }
    // ... other modes
}
```

2. Clean up interfaces:
   - Remove command-specific code from enhanced_tui
   - Ensure all communication through traits
   - Document public APIs

## Implementation Strategy

### Phase-by-Phase Approach
1. **Each phase should be a separate PR**
2. **Maintain full functionality between phases**
3. **Add tests for new interfaces**
4. **Update documentation progressively**

### Testing Strategy
- Unit tests for CommandEditor
- Integration tests for mode transitions
- Regression tests for existing functionality
- Performance tests for large queries

### Risk Mitigation
- Keep original code as fallback initially
- Feature flag for new implementation
- Extensive testing at each phase
- Gradual rollout

## Benefits of This Approach

1. **Modularity**: Clear separation of concerns
2. **Testability**: Easier to test command mode in isolation
3. **Maintainability**: Reduced complexity in enhanced_tui
4. **Reusability**: Command editor could be used elsewhere
5. **Performance**: Potential for optimizations
6. **Future Extensions**: Easier to add features like:
   - Multi-line editing
   - Advanced completions
   - Syntax checking
   - Query formatting

## Current Blockers

1. **Tight Coupling**: Command mode shares many helpers with other modes
2. **State Management**: Direct manipulation of multiple state containers
3. **Event Handling**: Complex event routing through multiple handlers
4. **Rendering**: Integrated with main TUI rendering loop

## Next Steps

1. **Phase 1 Implementation** (Immediate)
   - Create CommandEditor struct in enhanced_tui.rs
   - Move command-specific methods
   - Test thoroughly

2. **Feedback and Iteration**
   - Review Phase 1 implementation
   - Adjust plan based on findings
   - Continue with Phase 2

3. **Documentation**
   - Document new interfaces
   - Update architecture diagrams
   - Create migration guide

## Notes

- The StateCoordinator added recently will be crucial for this refactoring
- We can leverage the existing InputBehavior trait
- The new modular structure (input/, state/, etc.) provides a good pattern to follow
- Consider using the same event-driven approach as the rest of the application