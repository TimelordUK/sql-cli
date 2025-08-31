# Widget Extraction Pattern

## Overview
This document describes the pattern for extracting widgets from the monolithic `enhanced_tui.rs` into self-contained, reusable widget modules.

## Goals
1. **Decouple state** from the main TUI application
2. **Create reusable widgets** that can be tested independently
3. **Establish clear boundaries** between widgets
4. **Enable state persistence** across mode switches
5. **Simplify the main TUI file** to focus on orchestration

## The Pattern

### 1. Widget Structure
Each widget should follow this structure:

```rust
// widget_name.rs
pub struct WidgetNameState {
    // All state specific to this widget
}

pub struct WidgetName {
    // Internal dependencies (e.g., command_history)
    state: WidgetNameState,
    // Any other widget-specific resources
}

pub enum WidgetNameAction {
    // Actions that the widget can emit
    None,
    Exit,
    ExecuteCommand(String),
    // ... specific to widget
}

impl WidgetName {
    pub fn new(dependencies) -> Self { }
    pub fn initialize(&mut self) { }
    pub fn handle_key(&mut self, key: KeyEvent) -> WidgetNameAction { }
    pub fn render(&self, f: &mut Frame, area: Rect) { }
    pub fn get_state(&self) -> &WidgetNameState { }
    pub fn set_state(&mut self, state: WidgetNameState) { }
}
```

### 2. State Management
- **State struct**: Contains all mutable state for the widget
- **get_state/set_state**: Enable state persistence when switching modes
- **StateManager integration**: Widget state can be captured/restored

### 3. Action Pattern
- Widgets return **Actions** instead of directly modifying app state
- Main TUI interprets actions and coordinates between widgets
- This maintains loose coupling

### 4. Integration in Enhanced TUI

```rust
// In enhanced_tui.rs
pub struct EnhancedTuiApp {
    // ... other fields ...
    history_widget: HistoryWidget,
    state_manager: StateManager,
}

// In mode handling
AppMode::History => {
    match self.history_widget.handle_key(key) {
        HistoryAction::ExecuteCommand(cmd) => {
            self.set_input_text(cmd);
            self.execute_query();
            self.exit_mode();
        }
        HistoryAction::Exit => {
            self.exit_mode();
        }
        // ... handle other actions
    }
}

// In render
AppMode::History => {
    self.history_widget.render(f, area);
}
```

## Extraction Process

### Step 1: Identify Widget Boundaries
- Find all state related to the widget
- Identify all render methods
- Locate key handling logic
- Find any widget-specific helper methods

### Step 2: Create Widget Module
1. Create new file `src/widget_name.rs`
2. Define state struct with all widget state
3. Define widget struct with dependencies
4. Define action enum for widget outputs

### Step 3: Move Logic
1. Move state fields to widget state struct
2. Move render methods to widget
3. Move key handling to widget
4. Convert direct modifications to action returns

### Step 4: Integrate with Main TUI
1. Add widget as field in EnhancedTuiApp
2. Route key events to widget in appropriate mode
3. Handle widget actions in main event loop
4. Call widget render in render method

### Step 5: Add State Persistence
1. Implement get_state/set_state methods
2. Integrate with StateManager for mode transitions
3. Test state preservation across mode switches

## Widgets to Extract (Priority Order)

1. **HistoryWidget** ✅ - Completed as reference implementation
2. **StatsWidget** - Column statistics display
3. **DebugWidget** - Already partially extracted, needs completion
4. **SearchWidget** - Search/filter functionality
5. **HelpWidget** - Help text display
6. **EditorWidget** - SQL editor (most complex)
7. **ResultsWidget** - Results table display

## Benefits

1. **Testability**: Widgets can be unit tested in isolation
2. **Reusability**: Widgets can be used in other TUI applications
3. **Maintainability**: Clear boundaries make code easier to understand
4. **Performance**: Only active widget needs to process events
5. **State Management**: Clean state transitions with StateManager

## Example: HistoryWidget

The HistoryWidget demonstrates the pattern:
- **State isolation**: `HistoryState` contains all history-specific state
- **Action pattern**: Returns `HistoryAction` enum values
- **Self-contained rendering**: All render logic in the widget
- **Clean integration**: Main TUI just routes events and handles actions

## Next Steps

1. Continue with StatsWidget extraction
2. Update each widget to use StateManager
3. Create widget tests
4. Document widget APIs
5. Consider creating a Widget trait for common behavior