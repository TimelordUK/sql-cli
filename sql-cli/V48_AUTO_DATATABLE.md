# V48 Enhancement: Automatic DataTable Creation

## Current Behavior
- DataTable is created when `buffer.set_results()` is called
- This happens during normal query execution
- F6 manually forces DataTable creation if missing

## Issue
When loading CSV/JSON files, the initial auto-query might not always trigger DataTable creation, requiring F6 to be pressed.

## Solution
Ensure DataTable is ALWAYS created when we have results:

1. **On CSV/JSON Load**: Create DataTable immediately after auto-query
2. **On Query Execution**: Already working (set_results creates DataTable)
3. **Remove F6 Requirement**: DataTable should always exist when results exist

## Implementation
Add a check after loading CSV/JSON to ensure DataTable exists:
```rust
// After executing auto-query
if buffer.get_results().is_some() && !buffer.has_datatable() {
    // Force DataTable creation
    let results = buffer.get_results().unwrap().clone();
    buffer.set_results(Some(results));
}
```

## Benefits
- No need to press F6
- DataTable is always available for rendering
- Better user experience
- Consistent performance from the start