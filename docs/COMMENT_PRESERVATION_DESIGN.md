# Comment Preservation in SQL Formatter

## Problem Statement

The `\sf` formatter in the Neovim plugin currently strips all comments when reformatting SQL queries. This is because:

1. **Lexer** ✅ - Comments are tokenized (phase 1 completed in commit `5ee841b`)
2. **Parser** ❌ - Parser uses `next_token()` which skips comments
3. **AST** ❌ - No comment storage in AST nodes
4. **Formatter** ❌ - Cannot emit comments during reconstruction

## Phase 1: Foundation (COMPLETED)

Commit `5ee841b` added comment tokenization:
- `Token::LineComment(String)` - Line comments without `--` prefix
- `Token::BlockComment(String)` - Block comments without `/* */` delimiters
- `next_token_with_comments()` - Preserves comments as tokens
- `tokenize_all_with_comments()` - Returns all tokens including comments
- `next_token()` - Unchanged for backwards compatibility

## Phase 2: Comment Attachment Strategy

### Design Decision: Leading/Trailing Comment Association

Comments should be associated with AST nodes in three positions:

```sql
-- Leading comment (belongs to SELECT)
SELECT
    id,           -- Trailing inline comment (belongs to 'id' select item)
    name,
    /* Block comment before expression */
    CASE
        WHEN x > 0 THEN 'positive'  -- Inline comment
    END as result
FROM table    -- Trailing comment for FROM clause
WHERE id > 10
```

### Comment Storage in AST

Add optional comment fields to major AST nodes:

```rust
// In src/sql/parser/ast.rs

#[derive(Debug, Clone)]
pub struct Comment {
    pub text: String,
    pub is_line_comment: bool,  // true for --, false for /* */
}

#[derive(Debug, Clone)]
pub struct SelectStatement {
    // ... existing fields ...

    // Comment annotations
    pub leading_comments: Vec<Comment>,   // Before SELECT keyword
    pub trailing_comment: Option<Comment>, // After final clause
}

#[derive(Debug, Clone)]
pub enum SelectItem {
    Star {
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
    Column {
        column: ColumnRef,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
    Expression {
        expr: SqlExpression,
        alias: String,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    },
}

// Similar additions to:
// - WhereClause
// - CTE
// - JoinClause
// - SqlExpression variants
```

### Alternative: Simpler Token Stream Approach

Instead of attaching comments to every AST node, use a hybrid approach:

1. **Parse SQL as normal** (skip comments in parser)
2. **Separately tokenize with comments** using `tokenize_all_with_comments()`
3. **Position-based comment mapping** - Track comment positions in original source
4. **Reconstruct during formatting** - Insert comments based on position mapping

```rust
pub struct FormatterWithComments<'a> {
    ast: &'a SelectStatement,
    comment_tokens: Vec<(usize, Comment)>,  // (position, comment)
}
```

**Pros:**
- Minimal AST changes
- Faster to implement
- Parser remains unchanged

**Cons:**
- Position tracking fragile with whitespace changes
- Harder to handle refactoring operations

### Recommendation: Structured Comment Attachment

Use the structured approach with comments in AST nodes because:

1. **Robust** - Survives whitespace/formatting changes
2. **Semantic** - Comments attached to what they describe
3. **Future-proof** - Enables comment-aware refactoring
4. **Clear ownership** - Each node owns its comments

## Phase 3: Parser Modifications

### Parser Changes

```rust
// src/sql/recursive_parser.rs

impl Parser {
    // Collect comments before parsing a construct
    fn collect_leading_comments(&mut self) -> Vec<Comment> {
        let mut comments = Vec::new();
        loop {
            match self.peek_token() {
                Token::LineComment(text) => {
                    self.advance();
                    comments.push(Comment {
                        text: text.clone(),
                        is_line_comment: true,
                    });
                }
                Token::BlockComment(text) => {
                    self.advance();
                    comments.push(Comment {
                        text: text.clone(),
                        is_line_comment: false,
                    });
                }
                _ => break,
            }
        }
        comments
    }

    // Collect trailing inline comment (on same line)
    fn collect_trailing_comment(&mut self) -> Option<Comment> {
        match self.peek_token() {
            Token::LineComment(text) => {
                self.advance();
                Some(Comment {
                    text: text.clone(),
                    is_line_comment: true,
                })
            }
            _ => None,
        }
    }

    pub fn parse_select_statement(&mut self) -> Result<SelectStatement> {
        // Collect comments before SELECT
        let leading_comments = self.collect_leading_comments();

        // ... parse as normal ...

        Ok(SelectStatement {
            // ... existing fields ...
            leading_comments,
            trailing_comment: self.collect_trailing_comment(),
        })
    }
}
```

### Critical Implementation Details

1. **Peek vs Consume**: Use peek to check for comments, advance to consume
2. **Newline awareness**: Trailing comments only on same line
3. **Comment ownership**: First construct after comments owns them
4. **Multiple comments**: Collect all sequential comments

## Phase 4: Formatter Modifications

Update `ast_formatter.rs` to emit comments:

```rust
impl AstFormatter {
    fn format_select(&self, stmt: &SelectStatement, indent_level: usize) -> String {
        let mut result = String::new();
        let indent = self.indent(indent_level);

        // Emit leading comments
        for comment in &stmt.leading_comments {
            self.format_comment(&mut result, comment, &indent);
        }

        // SELECT keyword
        write!(&mut result, "{}{}", indent, self.keyword("SELECT")).unwrap();

        // ... rest of formatting ...

        // Emit trailing comment
        if let Some(ref comment) = stmt.trailing_comment {
            write!(&mut result, " ").unwrap();
            self.format_inline_comment(&mut result, comment);
        }

        result
    }

    fn format_comment(&self, result: &mut String, comment: &Comment, indent: &str) {
        if comment.is_line_comment {
            writeln!(result, "{}-- {}", indent, comment.text).unwrap();
        } else {
            writeln!(result, "{}/* {} */", indent, comment.text).unwrap();
        }
    }

    fn format_inline_comment(&self, result: &mut String, comment: &Comment) {
        if comment.is_line_comment {
            write!(result, "-- {}", comment.text).unwrap();
        } else {
            write!(result, "/* {} */", comment.text).unwrap();
        }
    }
}
```

## Phase 5: Integration

### Parser Integration

The parser needs to use `next_token_with_comments()` instead of `next_token()`:

```rust
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    next_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token_with_comments();  // Changed!
        let next_token = lexer.next_token_with_comments();     // Changed!

        Self {
            lexer,
            current_token,
            next_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = self.lexer.next_token_with_comments();  // Changed!
    }
}
```

### Breaking Change Handling

**Problem**: Changing `Parser::new()` to use comment-aware tokenization affects all parser usage.

**Solution**: Add feature flag or create new parser variant:

```rust
// Option 1: Feature flag (recommended)
impl Parser {
    #[cfg(feature = "preserve-comments")]
    fn get_next_token(&mut self) -> Token {
        self.lexer.next_token_with_comments()
    }

    #[cfg(not(feature = "preserve-comments"))]
    fn get_next_token(&mut self) -> Token {
        self.lexer.next_token()
    }
}

// Option 2: Explicit parser variant
pub struct CommentAwareParser { /* ... */ }
```

**Recommendation**: Use Option 2 initially to avoid breaking existing code, then migrate.

## Implementation Plan

### Step 1: AST Comment Fields (Breaking Changes)
- [ ] Add `Comment` struct to `ast.rs`
- [ ] Add comment fields to `SelectStatement`
- [ ] Add comment fields to `SelectItem` variants
- [ ] Update all AST construction sites

### Step 2: Parser Comment Collection
- [ ] Implement `collect_leading_comments()`
- [ ] Implement `collect_trailing_comment()`
- [ ] Update `parse_select_statement()` to collect comments
- [ ] Update `parse_select_items()` to collect comments
- [ ] Update clause parsing (WHERE, FROM, etc.)

### Step 3: Formatter Comment Emission
- [ ] Implement `format_comment()` helper
- [ ] Implement `format_inline_comment()` helper
- [ ] Update `format_select()` to emit leading/trailing comments
- [ ] Update `format_select_items()` to emit item comments
- [ ] Update clause formatters

### Step 4: Testing
- [ ] Unit tests for comment parsing
- [ ] Unit tests for comment formatting
- [ ] Integration tests with `\sf` command
- [ ] Test edge cases (multiple comments, nested, etc.)

### Step 5: Neovim Plugin Update
- [ ] Remove workaround comment extraction in `formatter.lua`
- [ ] Test with real queries containing comments
- [ ] Update documentation

## Example Test Cases

```sql
-- Test 1: Leading comments
-- This is a leading comment
-- Multiple lines
SELECT id, name FROM users;

-- Test 2: Trailing inline comments
SELECT
    id,        -- Primary key
    name,      -- User name
    email      -- Contact email
FROM users;

-- Test 3: Block comments
SELECT
    /* Complex calculation */
    CASE
        WHEN x > 0 THEN 'positive'
        ELSE 'other'
    END as result
FROM table;

-- Test 4: Mixed comments
-- Header comment
SELECT
    id,  /* inline block */
    name -- inline line
FROM users
WHERE id > 10; -- trailing
```

## Edge Cases to Handle

1. **Comments between tokens**: `SELECT/* */id`
2. **Multiple sequential comments**: Lines of header comments
3. **Comments in expressions**: `x /* comment */ + y`
4. **Nested block comments**: SQL standard doesn't support, but should handle gracefully
5. **Comments after commas**: `SELECT id, /* comment */ name`
6. **Empty comments**: `--` with no text

## Non-Goals

1. **Preserve exact whitespace**: Formatter normalizes whitespace
2. **Preserve comment position within line**: Comments attach to constructs
3. **HTML/markdown in comments**: Treat as plain text

## Success Criteria

1. ✅ `\sf` preserves all comments in reformatted output
2. ✅ Comment placement is semantically meaningful
3. ✅ No existing tests broken
4. ✅ Performance impact < 10%
5. ✅ Works with all SQL features (CTEs, subqueries, window functions)

## Timeline

- **Step 1-2**: 2-3 hours (AST + Parser changes)
- **Step 3**: 1-2 hours (Formatter changes)
- **Step 4**: 1-2 hours (Testing)
- **Step 5**: 30 mins (Plugin update)

**Total**: 5-8 hours of focused work

## References

- Original issue: Commit `39fb7d8` mentions workaround
- Lexer foundation: Commit `5ee841b`
- Related: `/home/me/dev/sql-cli/src/sql/parser/lexer.rs`
- Related: `/home/me/dev/sql-cli/src/sql/parser/ast.rs`
- Related: `/home/me/dev/sql-cli/src/sql/parser/ast_formatter.rs`
