# UI Module Reorganization Plan

## Current State
The `src/ui/` directory contains 37 .rs files all at the same level, making it difficult to understand the architecture.

## Proposed Structure

```
src/ui/
├── mod.rs
├── enhanced_tui.rs         # Main TUI entry point
├── state_coordinator.rs    # Central state coordination (keep at root - it's core)
│
├── key_handling/           # All key-related modules
│   ├── mod.rs
│   ├── dispatcher.rs       (from key_dispatcher.rs)
│   ├── mapper.rs           (from key_mapper.rs)
│   ├── chord_handler.rs    (from key_chord_handler.rs)
│   ├── indicator.rs        (from key_indicator.rs)
│   └── sequence_renderer.rs (from key_sequence_renderer.rs)
│
├── rendering/              # All rendering-related modules
│   ├── mod.rs
│   ├── cell_renderer.rs
│   ├── table_renderer.rs
│   ├── table_render_context.rs
│   ├── tui_renderer.rs
│   ├── render_state.rs
│   └── table_widget_manager.rs
│
├── search/                 # Search-related modules
│   ├── mod.rs
│   ├── vim_search_manager.rs
│   ├── vim_search_adapter.rs
│   ├── search_operations.rs
│   └── shadow_state.rs    # Tracks search state transitions
│
├── input/                  # Input handling
│   ├── mod.rs
│   ├── input_handlers.rs
│   ├── history_input_handler.rs
│   └── action_handlers.rs
│
├── operations/             # Data operations
│   ├── mod.rs
│   ├── simple_operations.rs
│   ├── text_operations.rs
│   ├── data_export_operations.rs
│   └── query_engine_integration.rs
│
├── utils/                  # Utility modules
│   ├── mod.rs
│   ├── column_utils.rs
│   ├── scroll_utils.rs
│   ├── text_utils.rs
│   ├── ui_layout_utils.rs
│   └── viewport_manager.rs
│
├── debug/                  # Debug-related modules
│   ├── mod.rs
│   ├── debug_context.rs
│   ├── enhanced_tui_debug.rs
│   └── enhanced_tui_debug_integration.rs
│
└── legacy/                 # Older modules to be refactored
    ├── mod.rs
    ├── tui_app.rs         # Old TUI implementation
    ├── tui_state.rs       # Old state management
    ├── actions.rs         # Being replaced by action system
    └── enhanced_tui_helpers.rs  # Should be split up

```

## Benefits
1. **Clear subsystems** - Easy to understand what each folder contains
2. **Better discoverability** - Related code is grouped together
3. **Easier navigation** - Less files at root level
4. **Clear architecture** - The structure reflects the design
5. **Simpler imports** - Can use module-level imports

## Migration Strategy

### Phase 1 - Easy Moves (Low Risk)
Start with modules that have clear boundaries and few dependencies:
1. Move all `key_*.rs` files to `key_handling/`
2. Move debug modules to `debug/`
3. Move utility modules to `utils/`

### Phase 2 - Medium Risk
4. Move rendering modules to `rendering/`
5. Move search modules to `search/`
6. Move operation modules to `operations/`

### Phase 3 - Complex Refactoring
7. Move input handling to `input/`
8. Identify and move legacy code to `legacy/`
9. Update all imports and module declarations

## Implementation Notes
- Create new branch: `ui_reorganization`
- Move files incrementally, testing after each group
- Update mod.rs files to properly export modules
- Run `cargo build` and `cargo test` after each move
- Use `git mv` to preserve history

## Example Migration Command
```bash
# Create directory structure
mkdir -p src/ui/{key_handling,rendering,search,input,operations,utils,debug,legacy}

# Move key files (example)
git mv src/ui/key_dispatcher.rs src/ui/key_handling/dispatcher.rs
git mv src/ui/key_mapper.rs src/ui/key_handling/mapper.rs
# ... etc

# Update imports
# Will need to update all files that import these modules
```

## Priority Order
1. **key_handling/** - Clearest grouping, easiest to move
2. **debug/** - Self-contained, few dependencies
3. **utils/** - Generally independent utilities
4. **rendering/** - Clear purpose, moderate dependencies
5. **search/** - Well-defined subsystem
6. **operations/** - May need some refactoring
7. **input/** - More complex dependencies
8. **legacy/** - Requires careful analysis