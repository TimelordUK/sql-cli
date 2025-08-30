# TUI Cleanup and Feature Enhancement Plan

## 1. Code Cleanup Tasks

### 1.1 Extract CommandEditor Module
**Status**: Ready for extraction
- Move `CommandEditor` struct to `src/ui/command_editor.rs`
- Already has clean boundaries and minimal coupling
- Only needs minor adjustments for special key filtering

### 1.2 Remove Legacy try_handle_action Code
**Location**: Lines 836-1350+ in enhanced_tui.rs
**Issues**:
- Large switch statement with mix of visitor pattern and legacy code
- Many actions already handled by visitor pattern handlers
- Duplicated logic and unclear flow

**Actions to clean**:
- Actions already handled by visitor pattern (marked in comments):
  - Navigation: NavigationActionHandler
  - Toggle operations: ToggleActionHandler
  - Clear operations: ClearActionHandler
  - Exit operations: ExitActionHandler
  - Column operations: ColumnActionHandler
  - Export operations: ExportActionHandler
  - Yank operations: YankActionHandler
  - UI operations: UIActionHandler
  - Debug/Viewport operations: DebugViewportActionHandler

**Keep for now**:
- Sort(_column_idx)
- HideEmptyColumns
- ExitCurrentMode (complex mode switching logic)
- ExecuteQuery
- A few others with complex state management

### 1.3 Remove Dead Code

#### Dead TODOs (lines 2656-2722)
These are marked as "Phase 3" and "Phase 4" but never called:
- `jump_to_prev_token` / `jump_to_next_token` (Phase 4 - SQL navigation)
- `paste_from_clipboard` (Phase 3 - clipboard operations)
- Ctrl+Y yank from kill ring
- Ctrl+V paste from system clipboard
- Alt+[ / Alt+] SQL token navigation

**Action**: Delete these TODO blocks entirely - they can be re-implemented properly when needed

#### Dead Method (line 6922)
- `debug_generate_key_renderer_info()` - never called
- **Action**: Delete this method

## 2. New Feature Implementation

### 2.1 Trim() String Method
**Purpose**: Remove leading and trailing whitespace from strings
**Example**: `book.Trim() = "Derivs"` matches " Derivs ", "Derivs  ", "  Derivs"

**Implementation locations**:
1. `src/sql/cursor_aware_parser.rs` - Add to string method suggestions
2. `src/data/recursive_where_evaluator.rs` - Implement in evaluate_binary_op
3. Add tests in `tests/method_evaluation_test.rs`

**Code pattern**:
```rust
// In recursive_where_evaluator.rs
"trim" => {
    Some(DataValue::String(s.trim().to_string()))
}
```

### 2.2 Basic Math Operations in SELECT
**Goal**: Support simple arithmetic in SELECT clauses
**Starting with**: `SELECT quantity * price as notional`

#### Phase 1: Basic Binary Operations
**Operators**: `+`, `-`, `*`, `/`
**Types**: Integer and Float only initially

**Implementation plan**:
1. **Parser changes** (`src/sql/recursive_parser.rs`):
   - Add arithmetic operators to Token enum
   - Parse binary expressions in SELECT clause
   - Create BinaryOp AST node for math expressions

2. **AST changes**:
   ```rust
   enum SqlExpression {
       // ... existing variants
       BinaryOp {
           left: Box<SqlExpression>,
           op: ArithmeticOp,
           right: Box<SqlExpression>,
       }
   }
   
   enum ArithmeticOp {
       Add,
       Subtract,
       Multiply,
       Divide,
   }
   ```

3. **Evaluator changes** (`src/data/recursive_where_evaluator.rs`):
   - Add math evaluation support
   - Handle type coercion (int + float = float)
   - Error handling for division by zero

4. **Query executor changes**:
   - Support evaluated expressions in SELECT results
   - Handle aliasing (`as notional`)

#### Phase 2: Extended Operations (Future)
- Parentheses for precedence
- Modulo operator `%`
- Power operator `^`
- Built-in functions: `ROUND()`, `ABS()`, `FLOOR()`, `CEIL()`

### 2.3 Additional String Methods (Quick Wins)
While implementing Trim(), also add:
- `TrimStart()` / `TrimEnd()` - trim only one side
- `PadLeft(n)` / `PadRight(n)` - pad to length
- `Replace(old, new)` - string replacement
- `Substring(start, length)` - extract substring

## 3. Implementation Priority

### Immediate (This Session):
1. ✅ Remove dead code (TODOs and unused method)
2. ✅ Implement Trim() method
3. ✅ Extract CommandEditor to module

### Next Session:
1. Clean up try_handle_action legacy code
2. Start basic math operations (multiplication first)

### Future:
1. Complete math operations
2. Add more string methods
3. Full visitor pattern migration

## 4. Testing Requirements

### For Trim():
- Test with leading spaces: `"  text"` → `"text"`
- Test with trailing spaces: `"text  "` → `"text"`
- Test with both: `"  text  "` → `"text"`
- Test with internal spaces: `"text  text"` → `"text  text"` (unchanged)
- Test with empty string and null values

### For Math Operations:
- Basic operations: `2 * 3 = 6`
- Type mixing: `2 * 3.5 = 7.0`
- Column references: `quantity * price`
- Division by zero handling
- NULL value handling
- Aliasing: `as total_cost`

## 5. Benefits

### Cleanup Benefits:
- Reduced code size (~500+ lines removed)
- Clearer code flow
- Easier maintenance
- Better separation of concerns

### Feature Benefits:
- **Trim()**: Essential for data cleaning in queries
- **Math operations**: Enables calculated fields without post-processing
- Both features align with SQL-like expectations from users