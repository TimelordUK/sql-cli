# Testing CompletionState Migration

## What to Test

### 1. Basic Tab Completion
```bash
./target/release/sql-cli test_completion.csv
```

**Test scenarios:**
- Type `SELECT ` and press Tab - should show column suggestions
- Type `SELECT Na` and Tab - should complete to "Name"
- Type `SELECT * WHERE D` and Tab - should complete to "Department"
- Press Tab multiple times - should cycle through suggestions

### 2. What to Look for in Logs

**Success indicators:**
- Status line shows: `"Completed: Name (1/4 - Tab for next)"` when cycling
- Tab cycles through all available columns
- Completion works for both columns and methods (`.Contains()`, etc.)

**Debug output (if debug enabled):**
```
[Completion] Set 4 completion suggestions
[Completion] Cycling to suggestion 2/4: Age
[Completion] Cleared 4 suggestions
```

### 3. Check AppStateContainer is Working

Press **F5** (debug dump) and look for:
```
=== COMPLETION STATE ===
Active: true
Suggestions: ["Name", "Age", "Department", "Salary"]
Current Index: 0
Last Query: "SELECT "
Last Cursor: 7
Total Completions: 3
```

### 4. Potential Issues to Watch For

**If you see these warnings:**
- `"[WARNING] CompletionState migration: state_container not available"` - means fallback to local state
- No completion suggestions appearing - check if AppStateContainer initialized properly
- Completion not cycling - check if state is being maintained between Tab presses

### 5. Test Method Completion

Type a column name followed by a dot:
```sql
SELECT * WHERE Name.
```
Press Tab - should suggest: `Contains()`, `StartsWith()`, `EndsWith()`, etc.

### 6. Test Context Tracking

1. Type `SELECT Na` and press Tab (completes to "Name")
2. Press Tab again WITHOUT typing - should cycle to next suggestion
3. Type any character - should reset completion context
4. Press Tab - should get fresh suggestions

### 7. Performance Check

The completion should be instant. If there's any lag when pressing Tab, that's a regression.

## Expected Behavior

✅ **Working correctly if:**
- Tab completes partial column names
- Multiple tabs cycle through options
- Status shows "(1/4 - Tab for next)" format
- Typing resets completion state
- Method completions work after dot

❌ **Not working if:**
- Tab does nothing
- Can't cycle through suggestions
- Status line doesn't show completion info
- AppStateContainer warnings in console

## Quick Debug Commands

If issues, run with debug output:
```bash
RUST_LOG=debug ./target/release/sql-cli test_completion.csv 2>completion_debug.log
```

Then check the log:
```bash
grep -i completion completion_debug.log
```

Should see state management messages from AppStateContainer.