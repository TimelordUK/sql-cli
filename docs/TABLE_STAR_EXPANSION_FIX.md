# Table-Scoped Star Expansion Fix

**Date**: 2025-10-21
**Issue**: `SELECT user.*, item.name` doesn't work - ignores table prefix
**Status**: Identified, ready to fix

## Problem

While the infrastructure for table-scoped `*` expansion exists, it's not being used:

```sql
-- This should work but doesn't:
SELECT user.*, item.name, item.weight
FROM user
JOIN item ON user.id = item.user_id
```

Currently this expands `user.*` to ALL columns from the joined table, not just user columns.

## Root Cause

The star expansion code in `src/data/query_engine.rs` **ignores the `table_prefix` field**:

```rust
// Current code (WRONG):
SelectItem::Star { .. } => {  // <-- ignores table_prefix!
    // Expand * to all columns from source table
    for col_name in source_table.column_names() {
        expanded_items.push(SelectItem::Column {
            column: ColumnRef::unquoted(col_name.to_string()),
            leading_comments: vec![],
            trailing_comment: None,
        });
    }
}
```

## What Works ✅

1. **Parser**: Correctly parses `table.*` and sets `table_prefix: Some("table")`
2. **AST**: Has `SelectItem::Star { table_prefix: Option<String> }`
3. **DataColumn**: Has `qualified_name` and `source_table` fields
4. **Joins**: Preserve `qualified_name` and `source_table` on all columns

## What's Missing ❌

The expansion logic needs to:
1. Check if `table_prefix` is `Some(prefix)`
2. If so, filter columns to only those from that table
3. Match using `source_table` or `qualified_name` fields

## The Fix

### Location
`src/data/query_engine.rs` - star expansion in `apply_select_items()` and `apply_select_with_row_expansion()`

### Required Changes

```rust
// BEFORE (current):
SelectItem::Star { .. } => {
    for col_name in source_table.column_names() {
        expanded_items.push(SelectItem::Column {
            column: ColumnRef::unquoted(col_name.to_string()),
            leading_comments: vec![],
            trailing_comment: None,
        });
    }
}

// AFTER (fixed):
SelectItem::Star { table_prefix, .. } => {
    if let Some(prefix) = table_prefix {
        // Scoped expansion: user.* expands only user's columns
        for (col_idx, col) in source_table.columns.iter().enumerate() {
            // Check if this column belongs to the specified table
            let matches_table = if let Some(ref source) = col.source_table {
                source == prefix
            } else if let Some(ref qualified) = col.qualified_name {
                // Try to match qualified name pattern "table.column"
                qualified.starts_with(&format!("{}.", prefix))
            } else {
                // No source info - can't determine, skip
                false
            };

            if matches_table {
                expanded_items.push(SelectItem::Column {
                    column: ColumnRef::unquoted(col.name.clone()),
                    leading_comments: vec![],
                    trailing_comment: None,
                });
            }
        }
    } else {
        // Unscoped expansion: * expands to all columns
        for col_name in source_table.column_names() {
            expanded_items.push(SelectItem::Column {
                column: ColumnRef::unquoted(col_name.to_string()),
                leading_comments: vec![],
                trailing_comment: None,
            });
        }
    }
}
```

## Files to Modify

1. **`src/data/query_engine.rs`**:
   - Function: `apply_select_items()` (around line 1750)
   - Function: `apply_select_with_row_expansion()` (around line 2000)

Both functions have identical star expansion logic that needs updating.

## Test Cases

### Test 1: Basic Join with Table Star
```sql
WITH
    WEB users AS (URL 'file://data/users.csv'),
    WEB items AS (URL 'file://data/items.csv')
SELECT users.*, items.name, items.weight
FROM users
JOIN items ON users.id = items.user_id
```

**Expected**: All columns from users + name and weight from items
**Current**: All columns from joined table (users + items) + duplicated name and weight

### Test 2: Multiple Table Stars
```sql
SELECT users.*, items.*
FROM users
JOIN items ON users.id = items.user_id
```

**Expected**: All columns from users, then all columns from items
**Current**: All columns twice

### Test 3: Mixed with Unscoped Star
```sql
SELECT *, users.id as user_id
FROM users
```

**Expected**: All columns + duplicate id column aliased
**Current**: Works (unscoped star works fine)

### Test 4: CTE with Qualified Names
```sql
WITH enriched AS (
    SELECT users.*, items.name as item_name
    FROM users JOIN items ON users.id = items.user_id
)
SELECT * FROM enriched
```

**Expected**: Should preserve column names from CTE
**Current**: Unknown (needs testing after fix)

## Implementation Steps

1. **Create helper function** to check if column belongs to table:
   ```rust
   fn column_matches_table(col: &DataColumn, table_name: &str) -> bool {
       if let Some(ref source) = col.source_table {
           return source == table_name || source.ends_with(&format!(".{}", table_name));
       }
       if let Some(ref qualified) = col.qualified_name {
           return qualified.starts_with(&format!("{}.", table_name));
       }
       false
   }
   ```

2. **Update `apply_select_items()`** to use table_prefix
3. **Update `apply_select_with_row_expansion()`** to use table_prefix
4. **Add tests** for all test cases above
5. **Update Python tests** to verify behavior

## Related Issues

This fix will enable:
- **Better column control in joins** - select exactly which table's columns you want
- **Clearer queries** - `user.*` is more explicit than listing all user columns
- **Foundation for other features**:
  - Table aliasing in SELECT (`SELECT u.* FROM users u`)
  - Schema qualification (`SELECT schema.table.* FROM ...`)
  - Better error messages when columns are ambiguous

## Backward Compatibility

✅ **No breaking changes**:
- Unscoped `*` continues to work as before
- Only adds new functionality for `table.*` syntax
- Parser already accepts this syntax (it just didn't work)

## Success Criteria

- [ ] `user.*` expands only user's columns in joins
- [ ] `item.*` expands only item's columns in joins
- [ ] `*` (unscoped) continues to expand all columns
- [ ] Multiple table stars work in same query
- [ ] Works with LEFT JOIN, INNER JOIN, CROSS JOIN
- [ ] Error message if table prefix doesn't exist
- [ ] All existing tests pass
- [ ] Python integration tests pass

## Notes

The user mentioned: "i think problem is we are losing somewhere the granularity of exactly which table holds which columns"

This is partially correct - we're NOT losing it (joins preserve source_table and qualified_name), but we're **not using it** when expanding stars. The metadata is there, we just need to check it!
