# Phase 2 TUI Simplification Plan

## Overview

After completing Phase 1 (low-hanging fruit cleanup), we're entering **Phase 2: Core Function Decomposition**. The goal is to transform the TUI from a massive monolithic handler into a simple dispatcher that orchestrates smaller, focused sub-functions.

## Current State (Post Phase 1)

### ✅ Completed in Phase 1
- **Redux-style pattern established** (TableWidgetManager, RenderState)  
- **Dead code eliminated** (duplicate function key handlers)
- **Search navigation centralized** (SearchManager, VimSearchManager integration)
- **Action system foundation** laid for simple cases
- **Navigation pipeline unified** (hjkl, search all use TableWidgetManager)

### ❌ Remaining Challenges  
- **Massive functions still exist** (`handle_command_input`, `handle_results_input`)
- **Complex action system cases** not yet migrated
- **Vim search still mixed** with regular search logic
- **Widget state management** still ad-hoc (not Redux)

## Phase 2 Strategy: Iterative Function Decomposition

### Core Principle
**"Simplify until TUI becomes a simple dispatcher"**

Each refactoring iteration should:
1. **Identify common behavior patterns** in massive functions
2. **Extract to focused sub-functions** (following `try_handle_*` pattern)
3. **Reduce main function to orchestration only**
4. **Reveal new refactoring opportunities** for next iteration

### Target Architecture
```rust
fn handle_command_input(&mut self, key: KeyEvent) -> Result<bool> {
    // Try specialized handlers first
    if let Some(result) = self.try_handle_buffer_operations(&key)? { return Ok(result); }
    if let Some(result) = self.try_handle_function_keys(&key)? { return Ok(result); }
    if let Some(result) = self.try_handle_history_navigation(&key)? { return Ok(result); }
    if let Some(result) = self.try_handle_text_editing(&key)? { return Ok(result); }
    if let Some(result) = self.try_handle_completion(&key)? { return Ok(result); }
    if let Some(result) = self.try_handle_mode_transitions(&key)? { return Ok(result); }
    
    // Minimal fallback handling
    self.handle_remaining_input(key)
}
```

## Phase 2 Branch Roadmap

### **Branch 1: `tui_function_decomposition_v1`**
**Target:** `handle_command_input` function decomposition

**Current State:** ~200+ lines of mixed responsibilities  
**Goal:** ~50 lines of orchestration + focused sub-functions

**Extraction Candidates:**
- `try_handle_history_navigation` - Ctrl+P/N, Alt+Up/Down history commands
- `try_handle_text_editing` - Kill line, word movement, clipboard operations  
- `try_handle_completion` - Tab completion, suggestion logic
- `try_handle_mode_transitions` - Enter key, mode switching logic

**Success Criteria:**
- Main function reduced to orchestration pattern
- Each sub-function handles single responsibility
- No behavior changes (same functionality)
- Clear separation of concerns

### **Branch 2: `tui_results_decomposition_v1`** 
**Target:** `handle_results_input` function decomposition

**Current State:** ~300+ lines handling all results mode input  
**Goal:** ~75 lines of orchestration + focused sub-functions

**Extraction Candidates:**
- `try_handle_navigation_keys` - hjkl, page up/down, g/G movements
- `try_handle_column_operations` - pin, hide, sort, move operations
- `try_handle_search_operations` - /, ?, n/N search navigation
- `try_handle_yank_operations` - y-prefix chord sequences
- `try_handle_mode_exits` - Escape, q, return to command mode

### **Branch 3: `tui_action_system_completion_v1`**
**Target:** Migrate remaining complex action system cases

**Focus Areas:**
- Text editing operations still in switch statements
- Complex mode transition logic  
- Buffer management direct calls
- Completion system integration

### **Branch 4: `tui_vim_extraction_v1`**
**Target:** Extract vim search as independent component

**Prerequisites:** TUI must be simplified enough to see clean extraction points

**Goals:**
- Create self-contained `VimSearchWidget`
- Remove vim logic from `SearchModesWidget`  
- Apply Redux pattern to vim search state
- Plugin-like architecture (vim as optional component)

### **Branch 5+: Widget Redux Migration**
**Target:** Apply Redux pattern to all remaining widgets

**Candidates:**
- HelpWidget → Redux state management
- StatsWidget → Centralized state
- DebugWidget → State management  
- InputWidget → Redux integration

## Implementation Guidelines

### Function Extraction Pattern
Follow the established `try_handle_*` pattern:

```rust
fn try_handle_category(&mut self, key: &KeyEvent) -> Result<Option<bool>> {
    match key.code {
        // Handle specific keys for this category
        KeyCode::SpecificKey => {
            // Focused logic for this behavior
            Ok(Some(false))
        }
        _ => Ok(None), // Not handled by this category
    }
}
```

### Redux Pattern Application
For state-heavy components, follow TableWidgetManager model:

```rust
pub struct WidgetManager {
    state: WidgetState,
    render_state: RenderState,
}

impl WidgetManager {
    pub fn handle_action(&mut self, action: WidgetAction) {
        // Centralized state updates
        self.render_state.mark_dirty(RenderReason::StateChange);
    }
}
```

## Success Metrics

### Per-Branch Metrics
- **Lines of code reduction** in main functions
- **Cyclomatic complexity** decrease
- **Single responsibility** adherence
- **Zero behavior regression** (tests pass)

### Overall Phase 2 Success
- **TUI becomes simple dispatcher** (~200 total lines in main handlers)
- **Each function has single responsibility** 
- **New refactoring opportunities revealed** for Phase 3
- **Redux pattern ready for widget migration**

## Phase 3 Preview

After Phase 2 completion:
- **Micro-refactoring opportunities** will be visible
- **Widget boundaries** will be clearer  
- **State management patterns** will be established
- **Plugin architecture** will be feasible

The iterative approach ensures each simplification reveals the next logical step, maintaining momentum toward the ultimate goal of a clean, maintainable TUI architecture.