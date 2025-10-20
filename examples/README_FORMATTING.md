# SQL Formatting Examples

This directory contains example SQL files demonstrating various features of the `sql-cli` formatter.

## Comment Preservation (NEW!)

As of version 1.61.0, the SQL formatter can preserve comments when formatting queries.

### Quick Start

**Format a query with comments preserved:**
```bash
cat examples/reformat_comments.sql | ./target/release/sql-cli --format --preserve-comments
```

**Format without preserving comments (default):**
```bash
cat examples/reformat_comments.sql | ./target/release/sql-cli --format
```

### Demo Script

Run the interactive demo:
```bash
./scripts/test_comment_formatting.sh
```

This will show before/after examples of:
- Line comments (`-- comment`)
- Block comments (`/* comment */`)
- Mixed comment styles
- Complex query reformatting with comments

### Example File

**File:** `examples/reformat_comments.sql`

Contains 10 different examples demonstrating:
1. Simple queries with leading comments
2. Multi-line queries with documentation
3. Queries needing reformatting
4. Complex analytics queries with comments
5. Aggregation with comments
6. Block comment style
7. Mixed comment styles
8. Data quality checks with comments
9. Join queries with documentation

### Usage in Neovim

When using the Neovim plugin, comments are **automatically preserved** when you use `\sf` to format a query:

1. Open a SQL file in Neovim
2. Position cursor on a query with comments
3. Press `\sf`
4. Comments are preserved in the formatted output!

**Example:**

Before `\sf`:
```sql
-- Important user query
-- Filters active users only
SELECT id,name,email FROM users WHERE active=true
```

After `\sf`:
```sql
-- Important user query
-- Filters active users only
SELECT id, name, email
FROM users
WHERE active = TRUE
```

### CLI Options

The formatter supports several options that work with comment preservation:

```bash
# Preserve comments (NEW!)
--preserve-comments

# Use lowercase keywords
--lowercase

# Compact formatting
--compact

# Use tabs for indentation
--tabs
```

**Example with multiple options:**
```bash
cat query.sql | sql-cli --format --preserve-comments --lowercase --compact
```

### What Gets Preserved

✅ **Currently Supported:**
- Leading comments before SELECT statement
- Trailing comments at end of statement
- Line comments (`-- comment`)
- Block comments (`/* comment */`)
- Multiple consecutive comments
- Multi-line block comments

⚠️ **Not Yet Supported:**
- Inline comments within SELECT items
- Comments within WHERE/JOIN clauses
- Comments between columns in SELECT list

These are planned for future enhancement (see Phase 5 in `docs/COMMENT_PRESERVATION_PHASED_PLAN.md`).

### Examples

#### Example 1: Simple Line Comments

**Input:**
```sql
-- Fetch active users
-- Filter by status
SELECT id, name FROM users WHERE active = true
```

**Output (with --preserve-comments):**
```sql
-- Fetch active users
-- Filter by status
SELECT id, name
FROM users
WHERE active = TRUE
```

#### Example 2: Block Comments

**Input:**
```sql
/*
 * Important Query
 * Author: Data Team
 * Purpose: Weekly reporting
 */
SELECT COUNT(*) FROM orders
```

**Output (with --preserve-comments):**
```sql
/* Important Query
 * Author: Data Team
 * Purpose: Weekly reporting */
SELECT COUNT(*)
FROM orders
```

#### Example 3: Complex Reformatting

**Input:**
```sql
-- High-value customer analysis
SELECT u.id,u.name,COUNT(*) as orders,SUM(o.amount) as total FROM users u WHERE u.active=true
```

**Output (with --preserve-comments):**
```sql
-- High-value customer analysis
SELECT
    u.id,
    u.name,
    COUNT('*') AS orders,
    SUM(o.amount) AS total
FROM users
WHERE u.active = TRUE
```

### Testing

Run the test suite to verify comment preservation:
```bash
cargo test comment_preservation --lib
```

All tests should pass:
- `test_standard_mode_skips_comments`
- `test_preserve_mode_captures_leading_comments`
- `test_formatter_emits_comments`
- `test_full_round_trip_with_comments`
- `test_backward_compatibility`
- `test_block_comment_preservation`

### Documentation

For more details, see:
- `docs/COMMENT_PRESERVATION_COMPLETE.md` - Full implementation guide
- `docs/COMMENT_PRESERVATION_PHASED_PLAN.md` - Development phases
- `src/sql/parser/comment_preservation_tests.rs` - Test suite

### Troubleshooting

**Comments are being stripped:**
- Make sure you're using the `--preserve-comments` flag
- In Neovim, this is automatic (no flag needed)

**Formatter errors:**
- Some complex SQL features may not be fully supported yet
- Try simplifying the query or file an issue

**Neovim plugin not preserving comments:**
- Make sure you have the latest version of the plugin
- Rebuild the CLI: `cargo build --release`
- The plugin should automatically pass `--preserve-comments`

### Performance

Comment preservation has **minimal performance impact**:
- When `--preserve-comments` is NOT used: Zero overhead
- When `--preserve-comments` IS used: Negligible overhead (~1-2%)
- All 457 tests complete in ~6 seconds regardless of mode

### Contributing

Want to help improve comment preservation? Check out Phase 5 plans in:
`docs/COMMENT_PRESERVATION_PHASED_PLAN.md`

Potential improvements include:
- Inline comments within SELECT items
- Comments within WHERE/JOIN clauses
- CTE-level comments
- Comment positioning hints
