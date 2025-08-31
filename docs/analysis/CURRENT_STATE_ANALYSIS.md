# Current State Analysis: Mapping the Chaos

## Executive Summary

Our TUI has **complex nested state** that's currently scattered across multiple components. Before designing the centralized state manager, we need to precisely map:
- All modes and substates
- Where state currently lives
- State transition triggers  
- Side effects that need coordination

## Primary Mode Hierarchy

### 1. **INPUT MODE** (Command/Query entry)
**Primary State**: User typing SQL queries
```
AppMode::Command
├── Normal typing
├── Tab completion active
├── History search (Ctrl+R)
└── Cursor positioning
```

**Current State Locations**:
- `Buffer.mode` - Primary mode flag
- `Buffer.input_text` - Query text 
- `Buffer.cursor_pos` - Cursor position
- `CompletionState` - Tab completion
- `HistorySearchState` - History search

### 2. **RESULTS MODE** (Navigation & Operations)
**Primary State**: User navigating/operating on query results
```
AppMode::Results
├── Normal navigation (hjkl, arrows)
├── Search substates:
│   ├── VIM search (/ key) - text search in data
│   ├── Column search (search key) - column name search  
│   └── Fuzzy filter - live data filtering
├── Operations:
│   ├── Column operations (pin, hide, sort)
│   ├── Selection mode
│   └── Edit mode (future)
└── View states:
    ├── Normal view
    ├── Debug view (F5)
    └── Pretty query view
```

**Current State Locations**:
- `Buffer.mode` - Primary mode
- `VimSearchManager` - Vim search state
- `ColumnSearchState` - Column search 
- `FilterState` - Fuzzy filtering
- `NavigationState` - Cursor/viewport position
- `SortState` - Column sorting
- `SelectionState` - Row/cell selection

## State Transition Triggers

### Entering Results Mode
```
Trigger: User presses Enter on query
Effects needed:
├── Execute query → dataview
├── Switch to Results mode  
├── Clear all search states
├── Reset viewport to (0,0)
├── Update status line
└── Set navigation context
```

### Search Mode Transitions
```
Trigger: User presses '/' (vim search)
Effects needed:
├── Enter VimSearch::Typing state
├── Show search input in status
├── Capture keystrokes for pattern
└── Clear other search states

Trigger: User presses Escape in search  
Effects needed:
├── Exit search state
├── Restore normal navigation
├── Update key mapping context
├── Clear search display
└── Reset status line
```

### Mode Restoration
```
Trigger: User runs new query while in search
Effects needed:
├── Clear ALL search states  
├── Reset to normal Results mode
├── Update viewport
├── Restore navigation keys
└── Update status line
```

## Current State Scatter Points

### 1. **Search State** (The 'N' key bug source)
```rust
// Scattered across 3+ locations:
buffer.get_search_pattern()                    // Regular search
vim_search_manager.is_active()                 // Vim search  
state_container.column_search().is_active      // Column search
// Plus fuzzy filter, history search...
```

### 2. **Navigation Context**
```rust
// Scattered across:
buffer.get_mode()                              // Primary mode
navigation_state.selected_row/column           // Position
viewport_manager.viewport_position             // Scroll state  
selection_state.mode                           // Selection type
```

### 3. **UI Coordination**
```rust
// Status line needs:
mode + search_state + navigation + results + help_text

// Key mapping needs:
mode + search_active + has_results + selection_mode

// Viewport needs:
navigation_state + search_matches + selection
```

## Side Effects That Need Coordination

### When Exiting Search Mode
1. **Key Mapping**: 'N' should map to toggle_line_numbers, not search navigation
2. **Status Line**: Remove search pattern display, show normal mode info  
3. **Viewport**: May need to restore previous position
4. **Input State**: Clear search input buffers

### When Entering Results Mode  
1. **Navigation**: Initialize cursor position
2. **Viewport**: Set to show results from top
3. **Search States**: Clear all previous search state
4. **Key Context**: Enable results navigation keys
5. **Status**: Show results info (row count, etc.)

### When Switching Between Search Types
1. **Previous Search**: Clear state from previous search type
2. **Key Mapping**: Update context for new search type
3. **Status Display**: Show appropriate search UI
4. **Input Capture**: Redirect keystrokes appropriately

## The Core Problem

**Multiple sources of truth** lead to **inconsistent state**:
- Action system checks 3+ search state sources
- Key mapping depends on scattered boolean flags  
- UI components each manage their own state
- State transitions don't automatically coordinate side effects

## Requirements for State Manager

### Must Handle
1. **Hierarchical States**: Mode → Submode → Operation state
2. **Automatic Side Effects**: Status line, viewport, key context updates
3. **State Validation**: Prevent impossible state combinations
4. **Transition Safety**: Ensure clean exits from all substates
5. **Debugging**: Central logging of all state changes

### Must Avoid
1. **Big Bang Migration**: Change everything at once
2. **Over-Engineering**: Complex state machines for simple cases  
3. **Performance Issues**: State checks are in hot paths
4. **Breaking Changes**: Maintain current TUI behavior during migration

## Next Steps

1. **Design Precise State Model**: Define enum hierarchy for all states
2. **Identify Transition Points**: Map all places that trigger state changes
3. **Plan Migration Order**: Start with search state (highest pain point)
4. **Build Incrementally**: One state type at a time, maintain compatibility

This analysis shows why the 'N' key bug exists - we have at least 3 different "search active" states that aren't coordinated!