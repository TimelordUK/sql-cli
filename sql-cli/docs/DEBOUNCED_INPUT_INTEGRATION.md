# DebouncedInput Widget Integration Guide

## Overview
The `DebouncedInput` widget provides a unified way to handle text input with debouncing across all search/filter modes. This prevents performance issues when searching through large datasets (e.g., 20k rows) by delaying the actual search until the user stops typing.

## How It Works

1. **User types** → Input updates immediately in UI
2. **Debouncer triggered** → Starts countdown (default 300ms)
3. **User keeps typing** → Debouncer resets
4. **User stops typing** → After 300ms, search executes
5. **Visual feedback** → Shows "(typing...)" indicator while pending

## Integration Pattern

### 1. Add DebouncedInput to Your Struct

```rust
use crate::widgets::debounced_input::{DebouncedInput, DebouncedInputBuilder};

pub struct EnhancedTuiApp {
    // For vim search (/)
    vim_search_input: DebouncedInput,
    // For column search (\)
    column_search_input: DebouncedInput,
    // For filter
    filter_input: DebouncedInput,
    // For fuzzy filter
    fuzzy_filter_input: DebouncedInput,
    // ... other fields
}
```

### 2. Initialize with Appropriate Config

```rust
impl EnhancedTuiApp {
    pub fn new() -> Self {
        Self {
            // Vim search with 300ms debounce
            vim_search_input: DebouncedInputBuilder::new()
                .debounce_ms(300)
                .title("Search")
                .style(Style::default().fg(Color::Yellow))
                .build(),
                
            // Column search with 200ms debounce (faster for column names)
            column_search_input: DebouncedInputBuilder::new()
                .debounce_ms(200)
                .title("Column Search")
                .style(Style::default().fg(Color::Green))
                .build(),
                
            // Filter with 500ms debounce (more expensive operation)
            filter_input: DebouncedInputBuilder::new()
                .debounce_ms(500)
                .title("Filter")
                .style(Style::default().fg(Color::Cyan))
                .build(),
                
            // Fuzzy filter with 400ms debounce
            fuzzy_filter_input: DebouncedInputBuilder::new()
                .debounce_ms(400)
                .title("Fuzzy Filter")
                .style(Style::default().fg(Color::Magenta))
                .build(),
            // ...
        }
    }
}
```

### 3. Handle Input in Event Loop

```rust
fn handle_search_mode_input(&mut self, key: KeyEvent) -> Result<bool> {
    // Use the appropriate input based on current mode
    let input = match self.buffer().get_mode() {
        AppMode::Search => &mut self.vim_search_input,
        AppMode::ColumnSearch => &mut self.column_search_input,
        AppMode::Filter => &mut self.filter_input,
        AppMode::FuzzyFilter => &mut self.fuzzy_filter_input,
        _ => return Ok(false),
    };
    
    match input.handle_key(key) {
        DebouncedInputAction::Continue => {
            // User is still typing, no action needed
        }
        DebouncedInputAction::InputChanged(pattern) => {
            // Input changed but debounced, update UI only
            self.buffer_mut().set_status_message(
                format!("Searching for: {} (typing...)", pattern)
            );
        }
        DebouncedInputAction::ExecuteDebounced(pattern) => {
            // Should not happen here, handled in check phase
        }
        DebouncedInputAction::Confirm(pattern) => {
            // User pressed Enter, execute search immediately
            self.execute_search(pattern);
            input.deactivate();
            self.buffer_mut().set_mode(AppMode::Results);
        }
        DebouncedInputAction::Cancel => {
            // User pressed Esc
            input.deactivate();
            self.buffer_mut().set_mode(AppMode::Results);
            self.buffer_mut().set_status_message("Search cancelled".to_string());
        }
        DebouncedInputAction::PassThrough => {
            // Let parent handle (e.g., Ctrl+C)
            return Ok(true);
        }
    }
    
    Ok(false)
}
```

### 4. Check for Debounced Actions in Main Loop

```rust
fn run(&mut self) -> Result<()> {
    loop {
        // Check all debounced inputs
        self.check_debounced_actions();
        
        // Handle events with timeout to allow debounce checks
        if event::poll(Duration::from_millis(50))? {
            // Handle key events...
        }
    }
}

fn check_debounced_actions(&mut self) {
    // Check vim search
    if let Some(pattern) = self.vim_search_input.check_debounce() {
        self.execute_vim_search(pattern);
    }
    
    // Check column search
    if let Some(pattern) = self.column_search_input.check_debounce() {
        self.execute_column_search(pattern);
    }
    
    // Check filter
    if let Some(pattern) = self.filter_input.check_debounce() {
        self.apply_filter(pattern);
    }
    
    // Check fuzzy filter
    if let Some(pattern) = self.fuzzy_filter_input.check_debounce() {
        self.apply_fuzzy_filter(pattern);
    }
}
```

### 5. Render the Active Input

```rust
fn render_input_area(&self, f: &mut Frame, area: Rect) {
    match self.buffer().get_mode() {
        AppMode::Search => {
            self.vim_search_input.render(f, area);
        }
        AppMode::ColumnSearch => {
            self.column_search_input.render(f, area);
        }
        AppMode::Filter => {
            self.filter_input.render(f, area);
        }
        AppMode::FuzzyFilter => {
            self.fuzzy_filter_input.render(f, area);
        }
        _ => {
            // Render normal command input
        }
    }
}
```

## Benefits

1. **Unified Logic**: All search modes use the same debouncing mechanism
2. **Performance**: No more blocking UI on every keystroke with large datasets
3. **Configurable**: Each mode can have different debounce delays
4. **Visual Feedback**: Users see "(typing...)" indicator while debouncing
5. **Maintainable**: Single source of truth for debounced input handling

## Migration Path

1. **Phase 1**: Integrate vim search (/) first as it has the biggest performance impact
2. **Phase 2**: Migrate column search (\) and filter modes
3. **Phase 3**: Migrate fuzzy filter
4. **Phase 4**: Remove old SearchModesWidget and consolidate

## Testing

```bash
# Create a large test file
seq 1 20000 | awk '{print "row"$1",data"$1",value"$1}' > large_test.csv

# Test with the application
./target/release/sql-cli large_test.csv

# Try searching with / - should no longer feel sluggish
# Try other search modes - all should have smooth typing
```

## Configuration

The debounce delays can be made configurable via the config file:

```toml
[search]
vim_search_debounce_ms = 300
column_search_debounce_ms = 200
filter_debounce_ms = 500
fuzzy_filter_debounce_ms = 400
```

This ensures consistent performance across all machines while allowing users to tune for their specific needs.