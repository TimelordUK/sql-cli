# Feature Request: Smart Function Completion

## Current Behavior
When using Tab completion for LINQ methods:
- Functions with parameters complete correctly: `.Contains` → `.Contains('|')` with cursor positioned inside quotes
- Parameterless functions complete inconsistently:
  - `Length` completes without parentheses → `.Length` (requires manual `()` addition)
  - `ToLower()` and `ToUpper()` complete with parentheses → `.ToLower()`

## Desired Behavior
All functions should complete intelligently based on their signature:
1. **Functions with parameters**: Add parentheses with cursor positioned for parameter entry
   - Example: `.Contains` → `.Contains('|')` where `|` is cursor
2. **Parameterless functions**: Add empty parentheses with cursor after closing paren
   - Example: `.Length` → `.Length()|` where `|` is cursor

## Implementation Details

### Files to Modify
1. `/src/sql/cursor_aware_parser.rs` - Update method suggestions to be consistent
2. `/src/ui/utils/text_operations.rs` - Update `apply_completion_to_text` to handle function completions

### Suggested Changes

#### In cursor_aware_parser.rs:
```rust
// Define methods with their signatures
let string_methods = vec![
    ("Contains", Some("('')")),      // Has parameters
    ("StartsWith", Some("('')")),    // Has parameters
    ("EndsWith", Some("('')")),      // Has parameters
    ("Length", None),                 // No parameters
    ("ToLower", None),                // No parameters
    ("ToUpper", None),                // No parameters
    ("Trim", None),                   // No parameters
    // ... etc
];
```

#### In completion logic:
```rust
fn complete_method(method_name: &str, has_params: Option<&str>) -> CompletionResult {
    match has_params {
        Some(params) => {
            // Function with parameters - position cursor inside
            let completion = format!("{}{}", method_name, params);
            let cursor_offset = method_name.len() + 2; // After opening quote
            CompletionResult { text: completion, cursor_pos: cursor_offset }
        }
        None => {
            // Parameterless function - position cursor after closing paren
            let completion = format!("{}()", method_name);
            let cursor_offset = completion.len(); // After closing paren
            CompletionResult { text: completion, cursor_pos: cursor_offset }
        }
    }
}
```

## Benefits
1. **Consistency**: All functions complete with proper syntax
2. **Efficiency**: No need to manually add parentheses for parameterless functions
3. **User Experience**: Cursor positioned optimally for continued typing
4. **Reduced Errors**: Prevents forgetting parentheses on function calls

## Examples

### Before:
```sql
WHERE allocationStatus.Len[TAB] → WHERE allocationStatus.Length
-- User must manually add ()

WHERE comment.Cont[TAB] → WHERE comment.Contains('')
-- Works well, cursor inside quotes
```

### After:
```sql
WHERE allocationStatus.Len[TAB] → WHERE allocationStatus.Length()|
-- Parentheses added, cursor after )

WHERE comment.Cont[TAB] → WHERE comment.Contains('|')
-- Still works well, cursor inside quotes
```

## Priority
Low-Medium - Quality of life improvement that would make the SQL editor more intuitive

## Notes
- This is independent of the CommandEditor refactoring
- Would improve the experience for users writing LINQ-style queries
- Could be extended to handle other function types (aggregates, math functions, etc.)