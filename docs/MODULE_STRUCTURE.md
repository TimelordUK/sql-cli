# Module Structure

## Overview
Organizing the codebase into logical modules for better maintainability and clarity.

## Proposed Structure

```
src/
├── lib.rs                    # Public API exports
├── main.rs                   # Binary entry point
│
├── core/                     # Core business logic
│   ├── mod.rs
│   ├── app_state_container.rs
│   ├── buffer_manager.rs
│   ├── service_container.rs
│   └── global_state.rs
│
├── data/                     # Data layer (DataTable/DataView)
│   ├── mod.rs
│   ├── provider.rs          # DataProvider traits
│   ├── table.rs             # DataTable implementation
│   ├── view.rs              # DataView implementation
│   ├── adapters/            # Adapters for existing data sources
│   │   ├── mod.rs
│   │   ├── buffer_adapter.rs
│   │   ├── csv_adapter.rs
│   │   └── api_adapter.rs
│   └── converters/          # Data format converters
│       ├── mod.rs
│       ├── csv_converter.rs
│       └── json_converter.rs
│
├── ui/                      # UI layer
│   ├── mod.rs
│   ├── enhanced_tui.rs     # Main TUI application
│   ├── classic_cli.rs      # Classic CLI mode
│   ├── key_dispatcher.rs   # Key event handling
│   └── renderer.rs         # Rendering logic
│
├── widgets/                 # UI widgets
│   ├── mod.rs
│   ├── debug_widget.rs
│   ├── editor_widget.rs
│   ├── help_widget.rs
│   ├── stats_widget.rs
│   ├── search_modes_widget.rs
│   ├── history_widget.rs
│   └── table_widget.rs
│
├── state/                   # State management
│   ├── mod.rs
│   ├── selection_state.rs
│   ├── filter_state.rs
│   ├── sort_state.rs
│   ├── search_state.rs
│   ├── column_search_state.rs
│   ├── clipboard_state.rs
│   ├── chord_state.rs
│   └── undo_redo_state.rs
│
├── sql/                     # SQL parsing and execution
│   ├── mod.rs
│   ├── parser.rs
│   ├── executor.rs
│   ├── optimizer.rs
│   └── cache.rs
│
├── api/                     # External API interactions
│   ├── mod.rs
│   ├── client.rs
│   ├── models.rs
│   └── endpoints.rs
│
├── utils/                   # Utility functions
│   ├── mod.rs
│   ├── debouncer.rs
│   ├── formatter.rs
│   ├── logger.rs
│   └── paths.rs
│
├── config/                  # Configuration
│   ├── mod.rs
│   ├── settings.rs
│   ├── themes.rs
│   └── keybindings.rs
│
└── tests/                   # Integration tests
    ├── mod.rs
    └── ...
```

## Migration Strategy

### Phase 1: Create Directory Structure (V35)
- Create directories
- Add mod.rs files with re-exports
- No code moves yet

### Phase 2: Move Widgets (V36)
- Move all *_widget.rs files to widgets/
- Update imports

### Phase 3: Move Data Layer (V37)
- Move DataProvider trait to data/provider.rs
- Move datatable* files to data/
- Move converters and adapters

### Phase 4: Move State Components (V38)
- Extract state structs from app_state_container.rs
- Create separate files in state/
- Keep AppStateContainer as orchestrator

### Phase 5: Move UI Components (V39)
- Move enhanced_tui.rs to ui/
- Move classic_cli.rs to ui/
- Move key_dispatcher.rs to ui/

### Phase 6: Move SQL Components (V40)
- Move SQL-related files to sql/
- Organize parser, executor, cache

### Phase 7: Move Utils and Config (V41)
- Move utility files to utils/
- Move config files to config/

## Benefits

1. **Clearer Organization**: Related files grouped together
2. **Easier Navigation**: Find files by functionality
3. **Better Encapsulation**: Modules can have private internals
4. **Scalability**: Easy to add new features in appropriate modules
5. **Testing**: Can test modules in isolation
6. **Documentation**: Each module can have its own README

## Module Visibility Rules

- Each module has a `mod.rs` that controls what's public
- Internal implementation details stay private
- Public API is explicitly exported
- Cross-module dependencies are minimized

## Example: widgets/mod.rs

```rust
// Re-export public widgets
pub mod debug_widget;
pub mod editor_widget;
pub mod help_widget;
pub mod stats_widget;

// Common widget traits (if any)
pub trait Widget {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

// Widget utilities
mod utils;  // Private to widgets module
```

## Example: data/mod.rs

```rust
// Public API for data layer
pub mod provider;
pub use provider::{DataProvider, DataViewProvider};

// DataTable and DataView will be public
pub mod table;
pub mod view;

// Adapters are public for gradual migration
pub mod adapters;

// Internal converters
mod converters;
```

## Gradual Migration

Each phase is a separate PR that:
1. Moves specific files
2. Updates imports
3. Ensures tests pass
4. Maintains backward compatibility

No breaking changes - just reorganization!