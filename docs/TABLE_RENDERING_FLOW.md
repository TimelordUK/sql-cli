# Table Rendering Flow in SQL-CLI

## Overview
The table rendering system uses a coordinated approach between TableWidgetManager, RenderState, and the actual table_renderer.

## Key Components

### 1. TableWidgetManager (`src/ui/table_widget_manager.rs`)
- **Purpose**: Centralized state management for table position and rendering
- **Key State**:
  - `position`: Current cursor position (row, column)
  - `render_state`: RenderState instance that tracks dirty flags
  - `viewport_manager`: Manages visible portion of data
  - `dataview`: The actual data to display

### 2. RenderState (`src/ui/render_state.rs`)
- **Purpose**: Tracks when UI needs re-rendering
- **Key Methods**:
  - `mark_dirty()`: Flags that render is needed
  - `needs_render()`: Checks if render should happen
  - `rendered()`: Clears dirty flag after render

### 3. Table Renderer (`src/ui/table_renderer.rs`)
- **Purpose**: Pure rendering function that draws the table
- **Key Function**: `render_table(f, area, context)`

## The Flow

### Step 1: User Action (e.g., pressing 'j' to move down)
```rust
// In enhanced_tui.rs - key handler detects 'j' key
fn handle_results_input() {
    case 'j' => self.next_row()
}
```

### Step 2: Navigation Updates State
```rust
// next_row() updates navigation state
fn next_row(&mut self) {
    // Update navigation in state_container
    self.state_container.navigation_mut().selected_row += 1;
    
    // CRITICAL: Update TableWidgetManager
    self.table_widget_manager.borrow_mut()
        .navigate_to(new_row, current_col);
}
```

### Step 3: TableWidgetManager Marks Dirty
```rust
// In table_widget_manager.rs
pub fn navigate_to(&mut self, row: usize, column: usize) {
    if self.position.row != row || self.position.column != column {
        self.position = TablePosition { row, column };
        
        // This is the key! Mark render state as dirty
        self.render_state.on_navigation_change();
        info!("TableWidgetManager: State marked dirty, will trigger re-render");
    }
}
```

### Step 4: RenderState Tracks Dirty Flag
```rust
// In render_state.rs (called by on_navigation_change)
pub fn mark_dirty(&mut self, reason: RenderReason) {
    self.dirty = true;
    self.dirty_reason = Some(RenderReason::NavigationChange);
}
```

### Step 5: Main Loop Checks for Render
```rust
// Back in enhanced_tui.rs main event loop
fn run(&mut self) {
    loop {
        // After handling key event...
        
        // Check if TableWidgetManager needs render
        if self.table_widget_manager.borrow().needs_render() {
            info!("TableWidgetManager needs render after key event");
            
            // RENDER HAPPENS HERE!
            terminal.draw(|f| self.ui(f))?;
            
            // Clear the dirty flag
            self.table_widget_manager.borrow_mut().rendered();
        }
    }
}
```

### Step 6: UI Method Builds Context and Renders
```rust
// In enhanced_tui.rs
fn ui(&mut self, f: &mut Frame) {
    // ... layout calculation ...
    
    // When rendering table area:
    if let Some(provider) = self.get_data_provider() {
        self.render_table_with_provider(f, results_area, provider.as_ref());
    }
}

fn render_table_with_provider(&self, f: &mut Frame, area: Rect, provider: &dyn DataProvider) {
    // Build context with current state
    let context = self.build_table_context(area, provider);
    
    // Call the pure renderer
    crate::ui::table_renderer::render_table(f, area, &context);
}
```

### Step 7: Pure Renderer Draws the Table
```rust
// In table_renderer.rs - pure function, no state
pub fn render_table(f: &mut Frame, area: Rect, ctx: &TableRenderContext) {
    // Build table widget from context
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL));
    
    // Actually render to frame
    f.render_widget(table, area);
}
```

## Unused Abstraction: check_and_render

The TableWidgetManager has a `check_and_render` method that encapsulates the pattern:
```rust
pub fn check_and_render<F>(&mut self, mut render_fn: F) -> bool {
    if self.needs_render() {
        // Log detailed info
        render_fn(&self.position, &self.render_state);
        self.rendered();
        return true;
    }
    false
}
```

**BUT THIS IS NOT CURRENTLY USED!** 

Instead, the code manually does:
```rust
if self.table_widget_manager.borrow().needs_render() {
    terminal.draw(|f| self.ui(f))?;
    self.table_widget_manager.borrow_mut().rendered();
}
```

This could be refactored to use the check_and_render method for better encapsulation.

## Key Insights

1. **Separation of Concerns**:
   - TableWidgetManager owns position state
   - RenderState tracks dirty flags
   - table_renderer is pure rendering logic

2. **The Coordination**:
   - User actions update TableWidgetManager
   - TableWidgetManager marks RenderState dirty
   - Main loop checks `needs_render()`
   - If true, calls `terminal.draw()` which invokes `ui()`
   - `ui()` eventually calls the pure `render_table()`
   - After render, calls `rendered()` to clear dirty flag

3. **Why This Pattern**:
   - Prevents unnecessary renders (only when dirty)
   - Centralizes state changes (through TableWidgetManager)
   - Keeps rendering pure (table_renderer has no state)
   - Enables debouncing and rate limiting

## The Log Messages You See

```
[15:10:42.050] TableWidgetManager: Position changed, marking dirty for re-render
[15:10:42.050] TableWidgetManager: State marked dirty, will trigger re-render
[15:10:42.050] TableWidgetManager needs render after key event
```

These show:
1. Position changed detected
2. Dirty flag set
3. Main loop detected dirty flag and triggered render

This is the complete flow from key press to pixels on screen!