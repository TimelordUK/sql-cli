# Future Improvements

## Parser Feedback Enhancement
**Observation:** Debug mode shows valuable parser errors that could help users in real-time.

**Example:**
```
AST TREE:
❌ PARSE ERROR ❌
Expected table name after FROM
```

**Proposed Solution:**
- Show parser errors in status line while typing
- Update in real-time as user types
- Color code: Yellow for incomplete, Red for errors, Green for valid
- Example status: `[Parser: Expected table name after FROM]`

## Testing Infrastructure
As the editor becomes more sophisticated, we need:

### Unit Tests for InputManager
- Test history navigation
- Test mode switching
- Test cursor positioning
- Test kill ring operations

### Integration Tests for Buffer
- Test multi-buffer scenarios
- Test state persistence
- Test undo/redo with buffers

### Parser Feedback Tests
- Test error detection
- Test suggestion generation
- Test completion accuracy

## Smart Status Line
Dynamic status line that shows context-aware information:

1. **While typing SQL:**
   - Parser status (valid/invalid/incomplete)
   - Suggestions available (Tab to complete)
   - Syntax errors

2. **In Results mode:**
   - Row count, filtered count
   - Current position
   - Active filters/search

3. **During operations:**
   - Progress indicators
   - Operation status
   - Performance metrics

## Code Organization Benefits
Once input abstraction is complete:
- Easier to add new features
- Testable components
- Clear separation of concerns
- Plugin architecture potential

## Completion Improvements
With proper abstraction:
- Context-aware completions
- Column name suggestions after SELECT
- Table suggestions after FROM
- Function suggestions
- Snippet expansion (e.g., `sel<Tab>` → `SELECT * FROM`)

## Multi-Buffer Enhancements
- Split view (see two queries side by side)
- Query diff tool
- Copy results between buffers
- Named workspaces

## Performance Monitoring
- Query execution time in status
- Memory usage for large datasets
- Index suggestions
- Query optimization hints