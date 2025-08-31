# Column Search Mode Fix

## Problem
Column search mode was not working properly - after typing a search pattern and the debounced search executed:
1. The search would find matching columns and show the status message
2. But Tab/Shift-Tab navigation wouldn't work
3. The mode would exit back to Results

## Root Cause
The issue was that `enter_search_mode` was being called TWICE when pressing the backslash key:
1. First by the dispatcher action "start_column_search" 
2. Then again by a duplicate direct key handler for `KeyCode::Char('\\')`

The second call with empty SQL was causing the mode to get confused.

## Solution
Removed all duplicate key handlers in `handle_results_input`:
- Removed duplicate handler for `\` (column search)
- Removed duplicate handler for `/` (search) 
- Removed duplicate handler for `F` (filter)
- Removed duplicate handler for `f` (fuzzy filter)

Now all search mode entries are handled consistently through the dispatcher.

## Additional Improvements
1. Added defensive logic to ensure mode stays in ColumnSearch after debounced search
2. Enhanced debug logging to track mode transitions
3. Updated status messages to always show "Tab/Shift-Tab to navigate" hint
4. Added synchronization between widget state and buffer state

## Testing
Run: `./test_column_search.sh` to create a test file and see instructions for testing the flow.

The column search should now:
- Stay in ColumnSearch mode after typing
- Allow Tab/Shift-Tab to navigate between matching columns
- Show current match position (e.g., "Column 2/5: orderid")
- Only exit on Enter (to jump to column) or Esc (to cancel)
EOF < /dev/null