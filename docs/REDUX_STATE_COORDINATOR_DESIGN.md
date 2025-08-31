# Redux-Style State Coordinator Design

## Problem Statement
The N key toggle (line numbers) stops working after entering and exiting search mode because the VimSearchManager is not properly reset. This is a symptom of a larger problem: components don't know when state transitions happen that affect them.

## Current Issue with N Key
1. User presses N -> Line numbers toggle works
2. User enters search mode (/)
3. VimSearchManager becomes active
4. User exits search (Esc)
5. VimSearchManager is NOT reset/cleared
6. N key is still captured by VimSearchManager instead of toggling line numbers

## Root Cause
- No centralized state transition notification system
- Components (VimSearchManager) don't know when they should reset
- State is scattered with no coordination mechanism

## Redux-Like Solution Design

### Core Concepts

#### 1. Actions (Events)
```rust
pub enum StateAction {
    // Mode transitions
    EnterSearchMode { search_type: SearchType },
    ExitSearchMode,
    SwitchToResults,
    SwitchToCommand,
    
    // Search-specific
    UpdateSearchPattern { pattern: String },
    ClearAllSearches,
    
    // UI actions
    ToggleLineNumbers,
    ToggleColumnStats,
}
```

#### 2. State Transitions with Side Effects
```rust
pub struct StateTransition {
    pub action: StateAction,
    pub side_effects: Vec<SideEffect>,
}

pub enum SideEffect {
    ResetVimSearch,
    ClearColumnSearch,
    ClearFuzzyFilter,
    RestoreNavigationKeys,
    UpdateViewport,
}
```

#### 3. Component Subscriptions
Components register interest in specific side effects:

```rust
pub trait StateSubscriber {
    fn handle_side_effect(&mut self, effect: &SideEffect);
}

impl StateSubscriber for VimSearchManager {
    fn handle_side_effect(&mut self, effect: &SideEffect) {
        match effect {
            SideEffect::ResetVimSearch => {
                self.clear();
                self.deactivate();
            }
            _ => {}
        }
    }
}
```

### Implementation Plan

#### Phase 1: Minimal Coordinator
```rust
pub struct StateCoordinator {
    subscribers: Vec<Box<dyn StateSubscriber>>,
    shadow_state: ShadowStateManager,
}

impl StateCoordinator {
    pub fn dispatch(&mut self, action: StateAction) {
        // Determine side effects based on action
        let side_effects = self.determine_side_effects(&action);
        
        // Update shadow state
        self.shadow_state.process_action(&action);
        
        // Notify all subscribers
        for effect in &side_effects {
            for subscriber in &mut self.subscribers {
                subscriber.handle_side_effect(effect);
            }
        }
    }
    
    fn determine_side_effects(&self, action: &StateAction) -> Vec<SideEffect> {
        match action {
            StateAction::ExitSearchMode => vec![
                SideEffect::ResetVimSearch,
                SideEffect::ClearColumnSearch,
                SideEffect::RestoreNavigationKeys,
            ],
            StateAction::EnterSearchMode { search_type } => {
                // Clear other search types
                let mut effects = vec![];
                match search_type {
                    SearchType::Vim => {
                        effects.push(SideEffect::ClearColumnSearch);
                        effects.push(SideEffect::ClearFuzzyFilter);
                    }
                    SearchType::Column => {
                        effects.push(SideEffect::ResetVimSearch);
                        effects.push(SideEffect::ClearFuzzyFilter);
                    }
                    _ => {}
                }
                effects
            }
            _ => vec![],
        }
    }
}
```

#### Phase 2: Avoiding Borrow Issues
Instead of components owning references, use a message-passing approach:

```rust
// Components return commands instead of mutating directly
pub enum ComponentCommand {
    VimSearch(VimSearchCommand),
    Viewport(ViewportCommand),
    Filter(FilterCommand),
}

pub enum VimSearchCommand {
    Clear,
    SetPattern(String),
    Deactivate,
}

// In TUI main loop
let commands = state_coordinator.dispatch(action);
for cmd in commands {
    match cmd {
        ComponentCommand::VimSearch(vim_cmd) => {
            self.vim_search_manager.execute(vim_cmd);
        }
        // ... handle other commands
    }
}
```

### Fixing the N Key Issue

#### Current Flow (Broken)
```
1. User presses /
2. VimSearchManager.activate()
3. User presses Esc
4. Mode changes to Results
5. VimSearchManager still active! <-- BUG
6. N key goes to VimSearchManager instead of line numbers
```

#### New Flow (Fixed)
```
1. User presses /
2. dispatch(EnterSearchMode { search_type: Vim })
3. User presses Esc
4. dispatch(ExitSearchMode)
5. Side effect: ResetVimSearch
6. VimSearchManager.clear() and deactivate()
7. N key now toggles line numbers correctly
```

### Migration Strategy

#### Step 1: Add StateCoordinator to TUI
```rust
// In enhanced_tui.rs
struct EnhancedTui {
    state_coordinator: StateCoordinator,
    // ... existing fields
}
```

#### Step 2: Replace Direct Mode Changes
```rust
// OLD
self.buffer.borrow_mut().set_mode(AppMode::Results);

// NEW
self.state_coordinator.dispatch(StateAction::SwitchToResults);
```

#### Step 3: Components React to Side Effects
```rust
// VimSearchManager gets notified automatically
// No need for manual reset calls scattered throughout code
```

## Benefits

1. **Solves N Key Issue**: VimSearchManager properly resets on mode exit
2. **Centralized Coordination**: All state transitions in one place
3. **No Scattered Reset Calls**: Components clean themselves up via side effects
4. **Testable**: Can test state transitions without UI
5. **Debuggable**: All transitions logged in one place
6. **Extensible**: Easy to add new side effects and subscribers

## Implementation Checklist

### Week 1: Core Infrastructure
- [ ] Create StateAction enum
- [ ] Create SideEffect enum
- [ ] Implement StateCoordinator
- [ ] Add to EnhancedTui

### Week 2: Fix N Key Issue
- [ ] Make VimSearchManager a StateSubscriber
- [ ] Add ExitSearchMode action with ResetVimSearch side effect
- [ ] Replace manual VimSearchManager resets with dispatch calls
- [ ] Test N key works after search mode

### Week 3: Expand Coverage
- [ ] Migrate other search managers
- [ ] Add viewport side effects
- [ ] Convert filter operations

## Success Criteria

1. N key toggle works consistently after search mode
2. No manual reset calls needed in TUI code
3. All state transitions go through coordinator
4. Components automatically clean up on mode changes
5. State transitions are fully logged and traceable