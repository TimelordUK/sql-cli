# State Hierarchy Design Decision

## The Question
Should viewport/cursor locks be tracked as separate states or sub-states in our shadow state manager?

## Current Situation

### What We Have
- **ViewportManager** owns cursor lock and viewport lock state
- **Shadow State** tracks high-level modes (Command, Results, Search, etc.)
- Locks only make sense in Results mode

### The Options

## Option 1: Sub-states within Results Mode
```rust
enum AppState {
    Command,
    Results(ResultsSubState),
    Search { search_type: SearchType },
    // ...
}

enum ResultsSubState {
    Normal,
    CursorLocked,
    ViewportLocked,
    BothLocked,
}
```

**Pros:**
- Accurately represents that locks are Results-specific
- Single source of truth for application state
- Clear state hierarchy

**Cons:**
- Duplicates state that ViewportManager already tracks
- More complex state transitions
- Risk of synchronization issues

## Option 2: Orthogonal State Dimensions
```rust
struct ShadowState {
    mode: AppState,
    cursor_lock: bool,
    viewport_lock: bool,
}
```

**Pros:**
- Simpler, flatter structure
- Easier to track independent toggles
- No duplication with ViewportManager

**Cons:**
- Loses the semantic connection that locks are Results-specific
- Multiple pieces of state to coordinate

## Option 3: Observer Pattern Only (Current)
Keep locks in ViewportManager, shadow state just observes major modes.

**Pros:**
- No duplication
- Clear ownership (ViewportManager owns navigation state)
- Simple shadow state

**Cons:**
- Shadow state doesn't see full picture
- Can't track state combinations like "Results with cursor locked"

## Recommendation

For now, **Option 3** (current approach) is probably best because:

1. **Shadow state is temporary** - It's a learning tool before centralized state management
2. **Avoid premature abstraction** - We're still learning the patterns
3. **Single responsibility** - ViewportManager owns navigation, shadow observes modes

However, we should **log lock changes** for learning:

```rust
// In toggle_cursor_lock action handler
info!(target: "shadow_state", "Cursor lock toggled: {} (in {:?} mode)", 
      is_locked, self.buffer().get_mode());
```

## Future Centralized State

When we move to centralized state management, consider:

```rust
struct ApplicationState {
    mode: AppMode,
    navigation: NavigationState,
    search: SearchState,
    // ...
}

struct NavigationState {
    cursor_position: (usize, usize),
    viewport_offset: (usize, usize),
    cursor_locked: bool,
    viewport_locked: bool,
    // ...
}
```

This gives clear ownership while maintaining accessibility.

## Decision Log

- **Current**: Keep locks in ViewportManager, log changes for learning
- **Future**: Move to hierarchical state with clear domain boundaries
- **Rationale**: Avoid duplication while learning actual usage patterns