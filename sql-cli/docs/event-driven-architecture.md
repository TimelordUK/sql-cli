# Event-Driven Architecture Design

## Overview

This document outlines the vision for refactoring sql-cli from its current tightly-coupled architecture to a clean event-driven system inspired by Redux/Flux patterns. The goal is to completely separate concerns: input handling, state management, business logic, and rendering.

## Current Problems

1. **Tight Coupling** - TUI directly handles keys, modifies state, and renders
2. **Untestable** - Can't test business logic without UI
3. **Hard-coded Keys** - Users can't customize keybindings
4. **Complex Key Handling** - 1000+ lines of nested match statements
5. **State Scattered** - State lives in TUI, buffers, widgets, etc.
6. **Hard to Extend** - Adding features requires touching many files

## Proposed Architecture

```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Input     │────▶│  KeyMapper   │────▶│     Event      │
│  (Terminal) │     │ (Configurable)│     │                │
└─────────────┘     └──────────────┘     └────────────────┘
                                                  │
                                                  ▼
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Render    │◀────│    State     │◀────│   Dispatcher   │
│    (TUI)    │     │  Container   │     │   (Reducer)    │
└─────────────┘     └──────────────┘     └────────────────┘
```

## Core Components

### 1. Event System

```rust
// All possible application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Navigation Events
    Navigate(NavigationTarget),
    ViewportTop,
    ViewportMiddle,
    ViewportBottom,
    ScrollUp(usize),
    ScrollDown(usize),
    
    // Editing Events
    InsertChar(char),
    DeleteChar,
    DeleteWord,
    KillLine,
    Yank,
    ExpandAsterisk,
    
    // Mode Events
    EnterMode(AppMode),
    ExitMode,
    
    // Search Events
    StartSearch(SearchType),
    UpdateSearchQuery(String),
    NextMatch,
    PreviousMatch,
    
    // Query Events
    ExecuteQuery,
    LoadFromCache(String),
    SaveToCache(String),
    
    // Buffer Events
    NewBuffer,
    CloseBuffer,
    SwitchBuffer(usize),
    
    // Data Events
    SortColumn(String, SortOrder),
    FilterData(FilterType),
    ExportData(ExportFormat),
    
    // System Events
    Quit,
    ShowHelp,
    ShowDebug,
    Refresh,
}
```

### 2. KeyMapper (User Configurable)

```rust
pub struct KeyMapper {
    mappings: HashMap<(AppMode, KeyBinding), AppEvent>,
    chord_handler: ChordHandler,
    config_path: PathBuf,
}

impl KeyMapper {
    pub fn from_config(path: &Path) -> Result<Self> {
        // Load from TOML/JSON config
    }
    
    pub fn map_key(&self, key: KeyEvent, mode: AppMode) -> Option<AppEvent> {
        let binding = KeyBinding::from(key);
        self.mappings.get(&(mode, binding)).cloned()
    }
    
    pub fn register_override(&mut self, mode: AppMode, key: KeyBinding, event: AppEvent) {
        self.mappings.insert((mode, key), event);
    }
}
```

**Config Format (keybindings.toml):**
```toml
[command_mode]
"Ctrl+X" = "ExpandAsterisk"
"Ctrl+K" = "KillLine"
"Ctrl+U" = "KillLineBackward"
"Ctrl+R" = "SearchHistory"
"Enter" = "ExecuteQuery"
"Tab" = "AutoComplete"
"/" = { StartSearch = "Forward" }

[results_mode]
"H" = "ViewportTop"
"M" = "ViewportMiddle"
"L" = "ViewportBottom"
"g" = "FirstRow"
"G" = "LastRow"
"/" = { StartSearch = "InResults" }
"y" = "YankCell"
"yy" = "YankRow"

[global]
"F1" = "ShowHelp"
"F5" = "ShowDebug"
"Ctrl+C" = "Quit"
```

### 3. State Container (Redux-style)

```rust
pub struct AppStateContainer {
    state: Arc<RwLock<AppState>>,
    history: Vec<AppEvent>,  // For undo/redo
    subscribers: Vec<Box<dyn Fn(&StateChange)>>,
}

impl AppStateContainer {
    pub fn dispatch(&mut self, event: AppEvent) -> StateChange {
        // Log event for replay/debug
        self.history.push(event.clone());
        
        // Apply event to state (reducer pattern)
        let change = match event {
            AppEvent::Navigate(target) => {
                self.state.write().navigation.go_to(target);
                StateChange::Navigation
            }
            AppEvent::ExpandAsterisk => {
                let expanded = self.expand_select_star();
                self.state.write().query = expanded;
                StateChange::Query
            }
            AppEvent::ExecuteQuery => {
                let result = self.execute_current_query().await?;
                self.state.write().results = Some(result);
                StateChange::Results
            }
            // ... handle all events
        };
        
        // Notify subscribers
        self.notify_subscribers(&change);
        
        change
    }
    
    pub fn snapshot(&self) -> AppStateSnapshot {
        // Immutable snapshot for rendering
        self.state.read().clone().into()
    }
}
```

### 4. Pure Rendering TUI

```rust
impl EnhancedTuiApp {
    pub async fn run(mut self) -> Result<()> {
        let mut key_mapper = KeyMapper::from_config("~/.config/sql-cli/keys.toml")?;
        let mut state_container = AppStateContainer::new();
        
        loop {
            // 1. Handle Input
            if let Ok(key) = event::read()? {
                // 2. Map to Event
                let mode = state_container.current_mode();
                if let Some(event) = key_mapper.map_key(key, mode) {
                    // 3. Dispatch Event
                    let change = state_container.dispatch(event).await?;
                    
                    // 4. Render if needed
                    if change.needs_render() {
                        let snapshot = state_container.snapshot();
                        self.render_from_state(&snapshot)?;
                    }
                }
            }
        }
    }
    
    fn render_from_state(&mut self, state: &AppStateSnapshot) -> Result<()> {
        // Pure rendering - no business logic
        self.terminal.draw(|f| {
            match state.mode {
                AppMode::Command => self.render_command_mode(f, state),
                AppMode::Results => self.render_results_mode(f, state),
                AppMode::Help => self.render_help(f, state),
                // ...
            }
        })?;
        Ok(())
    }
}
```

### 5. Event Bus for Widgets

```rust
pub trait Widget {
    fn handle_event(&mut self, event: &AppEvent) -> Option<AppEvent>;
    fn render(&self, f: &mut Frame, area: Rect, state: &WidgetState);
}

pub struct EventBus {
    widgets: Vec<Box<dyn Widget>>,
}

impl EventBus {
    pub fn dispatch(&mut self, event: AppEvent) -> Vec<AppEvent> {
        // Allow widgets to handle/transform events
        let mut new_events = vec![];
        for widget in &mut self.widgets {
            if let Some(new_event) = widget.handle_event(&event) {
                new_events.push(new_event);
            }
        }
        new_events
    }
}
```

## Migration Roadmap

### Phase 0: Complete State Migration (Current - 1-2 days)
- [x] FilterState, SearchState, SortState, SelectionState
- [x] ViewportLockState, TabCompleteState, HistorySearchState, HelpState  
- [ ] NavigationState, UndoRedoState, ClipboardState, SearchStates

### Phase 1: DataView Layer (1-2 weeks)
- Abstract all data operations into DataView trait
- Implement for CSV, JSON, API, Cache sources
- Unified data pipeline with transformations
- Virtual scrolling and lazy loading

### Phase 2: Event System Core (1 week)
- Define all AppEvents
- Create EventDispatcher
- Implement StateChange notifications
- Add event logging/replay infrastructure

### Phase 3: KeyMapper Implementation (1 week)
- Build configurable mapping system
- Create default keybinding configs
- Support user overrides
- Implement chord handling via events

### Phase 4: State Container Redux (2 weeks)
- Move all business logic from TUI to reducers
- Implement state snapshots
- Add undo/redo via event replay
- Create middleware system for async operations

### Phase 5: Widget Abstraction (1 week)
- Define Widget trait
- Convert existing rendering to widgets
- Implement event bubbling
- Create widget composition system

### Phase 6: TUI Simplification (1 week)
- Remove all key handling from TUI
- Convert to pure rendering
- Implement render diffing for performance
- Add render debugging tools

## Benefits

### For Users
- **Customizable Keys** - Full control over keybindings
- **Consistent Behavior** - All state changes go through one path
- **Better Performance** - Only re-render what changed
- **Undo/Redo** - Event sourcing enables time travel
- **Scriptable** - Can replay event sequences

### For Developers
- **Testable** - Test state changes without UI
- **Debuggable** - See every event and state change
- **Maintainable** - Clear separation of concerns
- **Extensible** - Easy to add new features
- **Reusable** - Widgets and state can be used elsewhere

### For Future
- **Multiple UIs** - Web UI, GUI, CLI all share state
- **Plugins** - Events make plugin system possible
- **Macros** - Record and replay event sequences
- **Remote Control** - Send events over network
- **AI Integration** - AI can generate events

## Implementation Priority

1. **High Priority**
   - Event definition
   - Basic dispatcher
   - KeyMapper with config

2. **Medium Priority** 
   - State snapshots
   - Widget abstraction
   - Event replay

3. **Low Priority**
   - Render diffing
   - Plugin system
   - Remote control

## Testing Strategy

```rust
#[test]
fn test_expand_asterisk() {
    let mut container = AppStateContainer::new();
    container.set_query("SELECT * FROM users");
    container.set_schema(vec!["id", "name", "email"]);
    
    let change = container.dispatch(AppEvent::ExpandAsterisk);
    
    assert_eq!(change, StateChange::Query);
    assert_eq!(container.get_query(), "SELECT id, name, email FROM users");
}
```

## Configuration Examples

### Power User Config
```toml
[command_mode]
";" = "ExecuteQuery"  # Vim-style command
"Ctrl+Space" = "AutoComplete"
":w" = { SaveToCache = "default" }
":q" = "Quit"

[chord_sequences]
"gg" = "FirstRow"
"dd" = "DeleteLine"
"yy" = "YankLine"
```

### Emacs User Config
```toml
[command_mode]
"Ctrl+X Ctrl+S" = "SaveQuery"
"Ctrl+X Ctrl+C" = "Quit"
"Meta+X" = "ShowCommandPalette"
```

## Conclusion

This architecture provides a clean, testable, and extensible foundation for sql-cli. The event-driven approach separates concerns, enables user customization, and sets up the codebase for future features like plugins, macros, and alternative UIs.

The migration will take approximately 6-8 weeks but will result in a much more maintainable and feature-rich application.

## Next Steps

1. Complete current state migration (v30+)
2. Design DataView interface
3. Create proof-of-concept for one widget
4. Build event system prototype
5. Implement basic KeyMapper

---

*Last Updated: 2025-01-11*
*Status: Planning Phase*
*Owner: sql-cli team*