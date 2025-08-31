# Enhanced TUI Refactoring Plan

## Current State Analysis
- **File Size**: 8,269 lines (way too large!)
- **Method Count**: 207 methods in single struct
- **Core Issues**:
  - Monolithic structure mixing data, UI, and business logic
  - Difficult to test individual components
  - Buffer refactoring incomplete - data handling still mixed with UI
  - Hard to maintain and extend

## Proposed Architecture

### Core Principle: Separation of Concerns
Split the monolithic EnhancedTuiApp into focused, composable modules:

```
EnhancedTuiApp (Orchestrator)
├── DataManager (Data Layer)
│   ├── BufferManager (manages multiple buffers)
│   ├── ResultsProcessor (transforms query results)
│   └── SchemaManager (tracks table schemas)
│
├── InputSystem (Input Layer)
│   ├── InputHandler (keyboard events)
│   ├── CompletionEngine (tab completion)
│   └── HistoryManager (command history)
│
├── ViewSystem (View Layer)
│   ├── TableRenderer (renders data tables)
│   ├── StatusBar (status information)
│   ├── InputRenderer (input area display)
│   └── ModeIndicator (current mode display)
│
├── StateManager (State Layer)
│   ├── AppState (global app state)
│   ├── ModeManager (mode transitions)
│   └── SelectionTracker (cursor/selection)
│
└── CommandProcessor (Business Logic)
    ├── QueryExecutor (SQL execution)
    ├── SearchFilter (search/filter operations)
    └── ExportManager (data export)
```

## Refactoring Strategy

### Phase 1: Extract Data Operations
**Goal**: Move all data manipulation out of TUI

1. **Create DataManager** (`src/data_manager.rs`)
   - Move results processing logic
   - Move column width calculations
   - Move data filtering/searching logic
   - Move CSV/JSON handling

2. **Enhance BufferManager** (`src/buffer_manager.rs`)
   - Centralize buffer switching logic
   - Handle buffer lifecycle (create/destroy)
   - Manage buffer state persistence

3. **Create ResultsProcessor** (`src/results_processor.rs`)
   - Transform raw query results
   - Handle pagination logic
   - Apply filters and searches
   - Format data for display

### Phase 2: Extract Rendering Logic
**Goal**: Separate UI rendering from state management

1. **Create ViewSystem** (`src/view/mod.rs`)
   - Extract all `render_*` methods
   - Create focused renderer components
   - Implement clean render interfaces

2. **TableRenderer** (`src/view/table_renderer.rs`)
   - Move table drawing logic
   - Handle column width calculations
   - Implement virtualized scrolling

3. **StatusRenderer** (`src/view/status_renderer.rs`)
   - Extract status bar logic
   - Create modular status components
   - Handle help text generation

### Phase 3: Extract Input Handling
**Goal**: Centralize and modularize input processing

1. **Create InputSystem** (`src/input_system/mod.rs`)
   - Extract keyboard event handling
   - Create input command abstraction
   - Implement input routing

2. **CompletionEngine** (`src/input_system/completion.rs`)
   - Move CompletionState to dedicated module
   - Implement per-buffer completion caching
   - Add smart completion features

3. **Enhance HistoryManager** 
   - Already exists but needs integration
   - Add per-buffer history filtering
   - Implement smart history search

### Phase 4: Create State Management
**Goal**: Centralized, predictable state management

1. **StateManager** (`src/state_manager.rs`)
   - Extract all state fields from TUI
   - Implement state transitions
   - Add state validation

2. **ModeManager** (`src/mode_manager.rs`)
   - Handle mode transitions
   - Validate mode changes
   - Implement mode-specific behaviors

### Phase 5: Command Processing
**Goal**: Extract business logic into testable units

1. **CommandProcessor** (`src/commands/mod.rs`)
   - Create command abstraction
   - Implement command pattern
   - Add command history/undo

2. **QueryExecutor** (`src/commands/query.rs`)
   - Extract SQL execution logic
   - Handle different query types
   - Manage query caching

## Implementation Approach

### Step 1: Start with Data Layer (Week 1)
```rust
// src/data_manager.rs
pub struct DataManager {
    buffer_manager: BufferManager,
    results_processor: ResultsProcessor,
    schema_manager: SchemaManager,
}

impl DataManager {
    pub fn process_query_results(&mut self, results: QueryResults) -> ProcessedData {
        // Move data processing logic here
    }
    
    pub fn apply_filter(&mut self, filter: Filter) -> FilteredData {
        // Move filter logic here
    }
}
```

### Step 2: Extract Rendering (Week 2)
```rust
// src/view/table_renderer.rs
pub struct TableRenderer {
    config: TableConfig,
}

impl TableRenderer {
    pub fn render(&self, data: &ProcessedData, area: Rect) -> TableWidget {
        // Move rendering logic here
    }
}
```

### Step 3: Slim Down EnhancedTuiApp
The final EnhancedTuiApp should be a thin orchestrator:

```rust
pub struct EnhancedTuiApp {
    data_manager: DataManager,
    view_system: ViewSystem,
    input_system: InputSystem,
    state_manager: StateManager,
    command_processor: CommandProcessor,
}

impl EnhancedTuiApp {
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        let command = self.input_system.process_event(event)?;
        self.command_processor.execute(command)?;
        self.state_manager.update()?;
        Ok(())
    }
    
    pub fn render(&mut self, frame: &mut Frame) {
        let state = self.state_manager.current_state();
        let data = self.data_manager.current_view();
        self.view_system.render(frame, state, data);
    }
}
```

## Benefits

1. **Testability**: Each component can be unit tested independently
2. **Maintainability**: Clear separation of concerns
3. **Extensibility**: Easy to add new features without touching core
4. **Performance**: Can optimize each layer independently
5. **Reusability**: Components can be reused (e.g., in modern_tui)

## Migration Strategy

1. **Create new modules alongside existing code**
2. **Gradually move functionality to new modules**
3. **Keep existing code working during migration**
4. **Replace old code once new modules are stable**
5. **Use feature flags if needed for gradual rollout**

## Success Metrics

- [ ] EnhancedTuiApp reduced to < 1000 lines
- [ ] Each module < 500 lines
- [ ] 80%+ unit test coverage on new modules
- [ ] No performance regression
- [ ] All existing features preserved
- [ ] Clear module boundaries with defined interfaces

## Next Steps

1. Start with DataManager extraction (highest value, most isolated)
2. Create ResultsProcessor to handle query results
3. Move buffer management logic to dedicated module
4. Begin extracting rendering logic
5. Continue iteratively until complete separation achieved