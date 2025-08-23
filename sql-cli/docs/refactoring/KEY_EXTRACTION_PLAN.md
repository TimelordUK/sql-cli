# Key Extraction Migration Plan

## Goal
Extract key handling from TUI into a clean action-based system while maintaining 100% functionality and testing at each step.

## Current State Analysis
- All key handling is deeply embedded in `enhanced_tui.rs` 
- Keys are handled in multiple places:
  - `handle_key_event()` - main dispatcher
  - Mode-specific handlers (Command, Results, Search, etc.)
  - Chord handling for multi-key sequences
  - Direct mutations of state throughout

## Migration Principles
1. **Incremental** - One category of keys at a time
2. **Non-breaking** - Existing functionality must work at every commit
3. **Testable** - Each extraction phase can be tested independently
4. **Reversible** - Can rollback any phase if issues arise

## Phase 1: Foundation (Branch: extract-key-handling)
**Goal**: Create the action system without changing behavior

### 1.1 Create Action Infrastructure
```rust
// src/ui/actions.rs
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    Home,
    End,
    
    // Will add more as we go...
}

pub struct ActionContext {
    pub mode: AppMode,
    pub selection_mode: SelectionMode,
    pub has_results: bool,
    // Other context needed for decisions
}

pub trait ActionHandler {
    fn can_handle(&self, action: &Action, context: &ActionContext) -> bool;
    fn handle(&mut self, action: Action, context: &ActionContext) -> Result<ActionResult>;
}
```

### 1.2 Create Key-to-Action Mapper
```rust
// src/ui/key_mapper.rs
pub struct KeyMapper {
    mappings: HashMap<(KeyCode, KeyModifiers), Action>,
}

impl KeyMapper {
    pub fn map_key(&self, key: KeyEvent, context: &ActionContext) -> Option<Action> {
        // Map based on key and context
    }
}
```

### 1.3 Integration Point
- Add `key_mapper: KeyMapper` to EnhancedTUI
- In `handle_key_event()`, try mapping first:
  ```rust
  if let Some(action) = self.key_mapper.map_key(key, &context) {
      // New path - will be empty initially
      if self.try_handle_action(action, context)? {
          return Ok(false);
      }
  }
  // Fall through to existing handling
  ```

**Testing**: No behavior change, all tests pass

## Phase 2: Navigation Keys (Branch: extract-navigation-keys)
**Goal**: Extract arrow keys, page up/down, home/end

### 2.1 Add Navigation Actions
```rust
pub enum Action {
    // Existing...
    
    // Navigation with amount
    Navigate(NavigateAction),
}

pub enum NavigateAction {
    Up(usize),      // Support vim-style counts (5j)
    Down(usize),
    Left(usize),
    Right(usize),
    PageUp,
    PageDown,
    Home,
    End,
    FirstColumn,
    LastColumn,
}
```

### 2.2 Extract Navigation Handler
```rust
// src/ui/handlers/navigation.rs
pub struct NavigationHandler<'a> {
    tui: &'a mut EnhancedTUI,
}

impl ActionHandler for NavigationHandler {
    fn handle(&mut self, action: Action, context: &ActionContext) -> Result<ActionResult> {
        match action {
            Action::Navigate(nav) => self.handle_navigation(nav, context),
            _ => Ok(ActionResult::NotHandled),
        }
    }
}
```

### 2.3 Migration Steps
1. Implement NavigationHandler with logic copied from TUI
2. Test with both old and new path active
3. Remove old navigation code from TUI
4. Test thoroughly

**Testing Checklist**:
- [ ] Arrow keys in all modes
- [ ] Page Up/Down
- [ ] Home/End
- [ ] Column navigation with pinned columns
- [ ] Navigation in filtered views

## Phase 3: Mode Switching (Branch: extract-mode-keys)
**Goal**: Extract mode transitions (v, Esc, Enter, etc.)

### 3.1 Add Mode Actions
```rust
pub enum Action {
    // Existing...
    
    // Mode changes
    SwitchMode(AppMode),
    ToggleSelectionMode,
    ExitMode,
    StartSearch,
    StartFilter,
    StartFuzzyFilter,
}
```

### 3.2 Extract Mode Handler
Similar pattern to navigation

**Testing Checklist**:
- [ ] 'v' toggles selection mode
- [ ] Esc exits modes properly
- [ ] Enter commits in appropriate modes
- [ ] Mode transition side effects work

## Phase 4: Editing Keys (Branch: extract-editing-keys)
**Goal**: Extract text editing, undo/redo

### 4.1 Add Editing Actions
```rust
pub enum Action {
    // Existing...
    
    // Editing
    InsertChar(char),
    Backspace,
    Delete,
    CutLine,
    Undo,
    Redo,
}
```

**Testing Checklist**:
- [ ] Text input in command mode
- [ ] Backspace/Delete
- [ ] Ctrl+U (clear line)
- [ ] Undo/Redo

## Phase 5: Clipboard/Yank (Branch: extract-clipboard-keys)
**Goal**: Extract yank operations

### 5.1 Add Clipboard Actions
```rust
pub enum Action {
    // Existing...
    
    // Clipboard
    YankCell,
    YankRow,
    YankColumn,
    YankAll,
    Paste,
}
```

**Testing Checklist**:
- [ ] 'y' in Cell mode yanks cell
- [ ] 'yy' chord yanks row
- [ ] Paste operations
- [ ] Clipboard integration

## Phase 6: Complex Operations (Branch: extract-complex-keys)
**Goal**: Extract sorting, filtering, pinning

### 6.1 Add Complex Actions
```rust
pub enum Action {
    // Existing...
    
    // Data operations
    Sort(Option<usize>),  // None = current column
    TogglePin,
    ClearPins,
    Filter(String),
    ClearFilter,
    ExecuteQuery,
}
```

## Phase 7: Chord Support (Branch: extract-chord-keys)
**Goal**: Extract multi-key sequences

### 7.1 Chord Handler
```rust
pub struct ChordHandler {
    pending: Vec<KeyEvent>,
    timeout: Duration,
}
```

## Phase 8: Vim Motions (Branch: add-vim-motions)
**Goal**: Add count support (5j, 10>, etc.)

### 8.1 Motion Parser
```rust
pub struct MotionParser {
    count_buffer: String,
}

impl MotionParser {
    pub fn parse_key(&mut self, key: KeyEvent) -> Option<Motion> {
        // Handle numeric prefix collection
    }
}
```

## Testing Strategy

### After Each Phase:
1. **Unit Tests**: Test new handlers in isolation
2. **Integration Tests**: Test with mock TUI
3. **Manual Testing**: Run through test scenarios
4. **Regression Testing**: Full test suite must pass
5. **Performance Testing**: Ensure no degradation

### Test Files to Create:
```
tests/
  actions/
    test_navigation.rs
    test_mode_switching.rs
    test_editing.rs
    test_clipboard.rs
    test_complex_ops.rs
```

## Rollback Strategy

Each phase is a separate branch that can be:
1. Tested independently
2. Merged only when stable
3. Reverted if issues found in production

## Success Metrics

- [ ] All existing functionality preserved
- [ ] Key handling code reduced by 50%+ in TUI
- [ ] New features easier to add
- [ ] Key bindings customizable
- [ ] Ready for Redux pattern

## Timeline Estimate

- Phase 1 (Foundation): 2-3 hours
- Phase 2 (Navigation): 3-4 hours  
- Phase 3 (Mode): 2-3 hours
- Phase 4 (Editing): 2-3 hours
- Phase 5 (Clipboard): 2-3 hours
- Phase 6 (Complex): 3-4 hours
- Phase 7 (Chords): 2-3 hours
- Phase 8 (Vim): 2-3 hours

**Total**: 2-3 days of focused work, with testing

## Next Steps

1. Start with Phase 1 on current branch
2. Create test file structure
3. Implement Action enum and infrastructure
4. Add integration point without changing behavior
5. Verify all tests pass
6. Move to Phase 2

This incremental approach ensures we can stop at any phase and still have a working, improved system.