# Refactoring Plan for handle_command_input

## Current State
- **586 lines** in a single method
- Handles all keyboard input in Command mode
- Mix of high-level and low-level logic
- Difficult to test and maintain

## Proposed Structure

### Phase 1: Create Helper Methods (In-Place Refactoring)

```rust
impl EnhancedTuiApp {
    // Main handler - orchestrates all the helpers
    fn handle_command_input(&mut self, key: KeyEvent) -> Result<bool> {
        let normalized_key = self.normalize_and_log_key(key)?;
        
        // Try handlers in order of priority
        if let Some(result) = self.try_action_system(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_editor_widget(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_history_operations(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_buffer_management(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_text_editing(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_function_keys(normalized_key)? {
            return Ok(result);
        }
        
        if let Some(result) = self.try_query_execution(normalized_key)? {
            return Ok(result);
        }
        
        // Default: handle as regular character input
        self.handle_default_input(normalized_key)
    }
    
    // Helper methods for each behavior group
    fn normalize_and_log_key(&mut self, key: KeyEvent) -> Result<KeyEvent> { ... }
    fn try_action_system(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_editor_widget(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_history_operations(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_buffer_management(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_text_editing(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_function_keys(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn try_query_execution(&mut self, key: KeyEvent) -> Result<Option<bool>> { ... }
    fn handle_default_input(&mut self, key: KeyEvent) -> Result<bool> { ... }
}
```

### Phase 2: Create Command Input Handler Trait (Future)

```rust
trait CommandInputHandler {
    fn can_handle(&self, key: &KeyEvent, context: &CommandContext) -> bool;
    fn handle(&mut self, key: KeyEvent, app: &mut EnhancedTuiApp) -> Result<InputResult>;
}

enum InputResult {
    Handled,
    Exit,
    NotHandled,
}

// Implement for each behavior group
struct HistoryHandler;
struct BufferManagementHandler;
struct TextEditingHandler;
// etc.
```

## Implementation Order

1. **Start with simplest extractions:**
   - Function key handlers (mostly independent)
   - Buffer management (clear boundaries)
   - History operations (well-defined)

2. **Then handle complex ones:**
   - Text editing (many small operations)
   - Editor widget integration
   - Action system integration

3. **Finally:**
   - Query execution (core functionality)
   - Default input handling

## Benefits

1. **Testability**: Each helper can be tested independently
2. **Readability**: Clear separation of concerns
3. **Maintainability**: Easy to find and modify specific behaviors
4. **Extensibility**: New handlers can be added easily
5. **Gradual Migration**: Can be done incrementally without breaking functionality

## Example Extraction: Function Keys

```rust
fn try_function_keys(&mut self, key: KeyEvent) -> Result<Option<bool>> {
    match key.code {
        KeyCode::F(1) => {
            self.toggle_help_mode();
            Ok(Some(false))
        }
        KeyCode::F(3) => {
            self.show_pretty_query();
            Ok(Some(false))
        }
        KeyCode::F(5) => {
            self.toggle_debug_mode();
            Ok(Some(false))
        }
        KeyCode::F(8) => {
            self.toggle_case_sensitivity();
            Ok(Some(false))
        }
        KeyCode::F(9) => {
            self.handle_kill_line_alt();
            Ok(Some(false))
        }
        KeyCode::F(10) => {
            self.handle_kill_line_backward_alt();
            Ok(Some(false))
        }
        KeyCode::F(12) => {
            self.toggle_key_indicator();
            Ok(Some(false))
        }
        _ => Ok(None) // Not a function key we handle
    }
}
```

## Next Steps

1. Create helper methods one by one
2. Test after each extraction
3. Keep the main method as a clean orchestrator
4. Document each helper's responsibility
5. Consider future trait-based approach for phase 2