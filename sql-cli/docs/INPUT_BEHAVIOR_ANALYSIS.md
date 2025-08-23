# Input Behavior Analysis

## Current State
The input/text manipulation functions in EnhancedTui are scattered and tightly coupled. Most delegate to BufferManager but the synchronization logic is embedded in the TUI.

## Functions Identified for Extraction

### Pure Text Manipulation (Can be extracted)
These functions manipulate text and cursor position without needing TUI internals:

1. **Word Movement**
   - `move_cursor_word_backward()` - Move cursor back one word
   - `move_cursor_word_forward()` - Move cursor forward one word
   - `jump_to_prev_token()` - Jump to previous SQL token
   - `jump_to_next_token()` - Jump to next SQL token

2. **Text Deletion/Killing**
   - `kill_line()` - Delete from cursor to end of line (Ctrl+K)
   - `kill_line_backward()` - Delete from cursor to start of line (Ctrl+U)
   - `delete_word_backward()` - Delete word before cursor (Ctrl+W)
   - `delete_word_forward()` - Delete word after cursor (Alt+D)

3. **Basic Input Operations**
   - `get_input_text()` - Get current input text
   - `set_input_text(text)` - Set input text
   - `set_input_text_with_cursor(text, cursor)` - Set text and cursor position
   - `get_input_cursor()` - Get cursor position
   - `clear_input()` - Clear input text

### Search/Filter Text Management (Partially extractable)
These manage search patterns but are coupled to mode management:

4. **Search Pattern Management**
   - `handle_column_search_input()` - Manages column search text input
   - `update_column_search()` - Updates column search results
   - Search pattern building (character add/remove)
   - Filter pattern management

## Current Implementation Pattern

```rust
// Current pattern - tightly coupled
fn kill_line(&mut self) {
    if let Some(buffer) = self.buffer_manager.current_mut() {
        buffer.kill_line();  // Delegates to buffer
        
        // Sync for rendering if single-line mode
        if buffer.get_edit_mode() == EditMode::SingleLine {
            let text = buffer.get_input_text();
            let cursor = buffer.get_input_cursor_position();
            self.set_input_text_with_cursor(text, cursor);
            self.cursor_manager.set_position(cursor);
        }
    }
}
```

## Proposed Refactoring Strategy

### Phase 1: Create TextOperation Results
Create result types that describe what happened:

```rust
pub struct TextOperationResult {
    pub new_text: String,
    pub new_cursor_position: usize,
    pub killed_text: Option<String>,  // For kill ring
    pub description: String,
}

pub struct SearchTextResult {
    pub pattern: String,
    pub cursor_position: usize,
    pub action: SearchAction,
}

pub enum SearchAction {
    AddChar(char),
    RemoveChar,
    Clear,
    Execute,
}
```

### Phase 2: Extract Pure Text Operations
Move text manipulation to a separate module that returns results:

```rust
pub trait TextManipulation {
    fn kill_line(&self, text: &str, cursor: usize) -> TextOperationResult;
    fn kill_line_backward(&self, text: &str, cursor: usize) -> TextOperationResult;
    fn delete_word_forward(&self, text: &str, cursor: usize) -> TextOperationResult;
    fn delete_word_backward(&self, text: &str, cursor: usize) -> TextOperationResult;
    fn move_word_forward(&self, text: &str, cursor: usize) -> usize;
    fn move_word_backward(&self, text: &str, cursor: usize) -> usize;
}
```

### Phase 3: Create InputBehavior Trait
Trait that uses the pure text operations:

```rust
pub trait InputBehavior: TextManipulation {
    // Required methods for TUI access
    fn buffer_manager(&mut self) -> &mut BufferManager;
    fn cursor_manager(&mut self) -> &mut CursorManager;
    
    // High-level input operations using TextManipulation
    fn kill_line(&mut self) {
        let (text, cursor) = self.get_current_input();
        let result = TextManipulation::kill_line(&text, cursor);
        self.apply_text_result(result);
    }
    
    // Helper to apply results
    fn apply_text_result(&mut self, result: TextOperationResult) {
        // Update buffer
        // Update cursor
        // Handle kill ring if needed
    }
}
```

## Benefits of This Approach

1. **Separation of Concerns**
   - Pure text manipulation logic separate from TUI
   - Easy to test text operations in isolation
   - Reusable for other input fields

2. **Reduced Coupling**
   - Text operations don't know about TUI internals
   - TUI just applies results
   - Clear data flow

3. **Better Testing**
   - Can unit test text manipulation without TUI
   - Can test cursor movement logic independently
   - Can verify kill ring behavior

## Implementation Order

1. **Start with Word Movement** (simplest)
   - Pure cursor position calculation
   - No text modification
   - Good proof of concept

2. **Then Kill/Delete Operations**
   - Text modification with results
   - Kill ring integration
   - More complex but clear benefits

3. **Finally Search/Filter Patterns**
   - Most complex due to mode interaction
   - May need mode-specific traits
   - Build on lessons from first two

## Challenges to Address

1. **Kill Ring Integration**
   - Currently managed by Buffer
   - Need to return killed text in result
   - Buffer applies to kill ring

2. **Mode-Specific Behavior**
   - Different modes use input differently
   - May need mode-aware text operations
   - Consider mode-specific traits

3. **Undo/Redo Integration**
   - Currently saves state before operations
   - Need to include in result or handle separately
   - Consider command pattern

## Next Steps

1. Create `TextOperationResult` struct
2. Implement pure `TextManipulation` functions
3. Test pure functions independently
4. Create `InputBehavior` trait
5. Refactor TUI to use trait
6. Extract to separate module