# VimSearchAdapter: Lessons from a Failed Redux Pattern

## What Was Supposed to Happen

VimSearchAdapter was meant to be our first proper Redux-style component:
- **Listen to events** - Subscribe to key press events
- **Dispatch actions** - Send search actions to state manager
- **No direct dependencies** - Work only through state container
- **Clean separation** - UI logic separate from search logic

## What Actually Happened

### 1. Direct Dependencies Everywhere
```rust
pub fn handle_key(
    &mut self,
    key: KeyCode,
    dataview: &DataView,        // Direct DataView dependency!
    viewport: &mut ViewportManager,  // Direct ViewportManager!
    buffer: &dyn BufferAPI,     // Direct Buffer dependency!
) -> bool
```

### 2. No Event Subscription
Instead of subscribing to events:
```rust
// What we should have:
impl StateSubscriber for VimSearchAdapter {
    fn on_event(&mut self, event: StateEvent) {
        match event {
            StateEvent::KeyPress(key) => self.handle_key_event(key),
            // ...
        }
    }
}
```

We got direct method calls from TUI:
```rust
// What we actually have:
if self.vim_search_adapter.borrow().is_active() {
    self.vim_search_adapter.borrow_mut().exit_navigation();
}
```

### 3. Adapter Became Another Layer
Instead of being an adapter that decouples, it became another layer that adds complexity:
```
TUI → VimSearchAdapter → VimSearchManager
         ↓                    ↓
      Buffer              DataView
      ViewportManager
```

## Why It Failed

### 1. Started with Wrong Approach
- Added as a RefCell field in TUI instead of independent component
- Immediately given direct access to UI components
- No clear action/event system to work with

### 2. Incremental Coupling
- Each feature added more direct dependencies
- "Just pass the buffer for now" → permanent coupling
- "We'll refactor later" → technical debt

### 3. Missing Infrastructure
- No proper event bus/dispatcher
- No action system for state changes
- No clear state management pattern

## The Pattern We Keep Repeating

1. **Start with good intentions** - "Let's make this decoupled"
2. **Take shortcuts** - "Just pass Buffer directly for now"
3. **Add more coupling** - "Also needs DataView and ViewportManager"
4. **Never refactor** - "It works, let's move on"
5. **End up with mess** - Multiple layers of coupling

## What We Should Have Done

### 1. Built Infrastructure First
```rust
// Event system
enum SearchEvent {
    Start,
    UpdatePattern(String),
    NextMatch,
    PreviousMatch,
    Exit,
}

// Action dispatcher
impl AppStateContainer {
    pub fn dispatch_search(&mut self, event: SearchEvent) {
        // Handle search state changes
    }
}
```

### 2. Made Adapter Truly Independent
```rust
struct VimSearchAdapter {
    state: Rc<RefCell<AppStateContainer>>,
}

impl VimSearchAdapter {
    pub fn handle_key(&mut self, key: KeyCode) {
        let mut state = self.state.borrow_mut();
        match key {
            KeyCode::Char('n') => state.dispatch_search(SearchEvent::NextMatch),
            // ...
        }
    }
}
```

### 3. Kept UI Separate
- TUI only renders based on state
- Adapter only changes state
- No direct UI manipulation

## The Real Problem

**We keep adding abstraction layers without fixing the fundamental coupling.**

Each new layer (Adapter, Manager, etc.) still takes the same dependencies:
- Buffer
- DataView  
- ViewportManager

This just moves the coupling around instead of eliminating it.

## Solution Going Forward

1. **Stop adding layers** - Fix the root coupling issues
2. **Route through AppStateContainer** - Everything goes through state
3. **No direct UI access** - Components change state, UI renders state
4. **Event-driven** - Components subscribe to events, not direct calls

## Key Takeaway

**Adding an "Adapter" doesn't decouple anything if the adapter itself is tightly coupled.**

The VimSearchAdapter became just another tightly coupled component in the chain, proving that the name "Adapter" doesn't make something an adapter - the architecture does.