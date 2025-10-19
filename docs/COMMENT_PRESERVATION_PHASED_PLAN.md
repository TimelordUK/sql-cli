# Phased Comment Preservation Implementation Plan

## Overview

This document outlines a **safe, incremental approach** to adding comment preservation to the SQL parser and formatter, similar to how tree-sitter maintains comments in ASTs for language servers.

The key principle: **Never break existing functionality**. Each phase is independent and can be tested/committed separately.

## Current Status (as of 2025-10-19)

### ✅ Phase 1 Completed (2025-10-18)
1. Lexer has comment tokenization (`Token::LineComment`, `Token::BlockComment`)
2. Dual-mode lexer (`LexerMode::SkipComments`, `LexerMode::PreserveComments`)
3. AST structures have comment fields (breaking change already handled)
4. Comment collection helper methods exist in parser
5. All compilation errors from AST changes are fixed
6. 5 lexer mode tests passing

### ✅ Phase 2 Completed (2025-10-19)
1. Added `ParserMode` enum (Standard, PreserveComments)
2. Added `Parser::with_mode()` constructor
3. `Parser::new()` defaults to Standard mode (backward compatible)
4. Comment collection guarded by mode checks in `parse_select_statement_inner()`
5. 7 parser mode tests passing
6. All 450 existing tests still passing (457 total)

### 🎯 Next: Phase 3 - Formatter Integration
- Make AST formatter emit comments when present
- Estimated time: 1 hour

## The Tree-Sitter Approach

Tree-sitter and language servers handle comments by:
1. **Separate lexical analysis** - Comments are tokens alongside code
2. **Trivia attachment** - Comments attached to nearest syntactic node
3. **Position tracking** - Each token knows its source location
4. **Opt-in processing** - Formatters/linters use comments, compiler ignores them

We'll follow a similar pattern.

---

## Phase 1: Dual-Mode Lexer (Non-Breaking) ✅ READY TO IMPLEMENT

### Goal
Create a lexer wrapper that can operate in two modes without changing existing parser behavior.

### Implementation

```rust
// src/sql/parser/lexer.rs

/// Lexer mode - controls whether comments are preserved or skipped
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerMode {
    /// Standard mode - skip comments (current behavior)
    SkipComments,
    /// Preserve mode - tokenize comments
    PreserveComments,
}

impl Lexer {
    pub fn with_mode(input: &str, mode: LexerMode) -> Self {
        let mut lexer = Self::new(input);
        lexer.mode = mode;
        lexer
    }

    fn mode: LexerMode,  // Add to struct

    // Modify next_token to check mode
    pub fn next_token(&mut self) -> Token {
        match self.mode {
            LexerMode::SkipComments => self.next_token_skip_comments(),
            LexerMode::PreserveComments => self.next_token_with_comments(),
        }
    }

    // Rename current next_token implementation
    fn next_token_skip_comments(&mut self) -> Token {
        // Current implementation
    }

    // Already exists as next_token_with_comments()
}
```

### Testing
```rust
#[test]
fn test_lexer_mode_compatibility() {
    let sql = "SELECT id -- comment\nFROM table";

    // Skip mode (current behavior)
    let mut lexer_skip = Lexer::with_mode(sql, LexerMode::SkipComments);
    assert_eq!(lexer_skip.next_token(), Token::Select);
    assert_eq!(lexer_skip.next_token(), Token::Identifier("id".into()));
    assert_eq!(lexer_skip.next_token(), Token::From);

    // Preserve mode (new behavior)
    let mut lexer_preserve = Lexer::with_mode(sql, LexerMode::PreserveComments);
    assert_eq!(lexer_preserve.next_token(), Token::Select);
    assert_eq!(lexer_preserve.next_token(), Token::Identifier("id".into()));
    assert!(matches!(lexer_preserve.next_token(), Token::LineComment(_)));
    assert_eq!(lexer_preserve.next_token(), Token::From);
}
```

### Verification
- ✅ All existing tests pass (default mode = SkipComments)
- ✅ New tests verify PreserveComments mode works
- ✅ Zero impact on existing parser

**Time estimate: 1 hour**

---

## Phase 2: Opt-In Parser Mode (Non-Breaking) 🔄 NEXT

### Goal
Allow parser to optionally preserve comments without affecting normal usage.

### Implementation

```rust
// src/sql/recursive_parser.rs

#[derive(Debug, Clone, Copy)]
pub enum ParserMode {
    /// Standard parsing - skip comments (current behavior)
    Standard,
    /// Preserve comments in AST
    PreserveComments,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self::with_mode(input, ParserMode::Standard)
    }

    pub fn with_mode(input: &str, mode: ParserMode) -> Self {
        let lexer_mode = match mode {
            ParserMode::Standard => LexerMode::SkipComments,
            ParserMode::PreserveComments => LexerMode::PreserveComments,
        };

        let mut lexer = Lexer::with_mode(input, lexer_mode);
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            mode,  // Store mode
            // ... rest of fields
        }
    }

    fn parse_select_statement_inner(&mut self) -> Result<SelectStatement, String> {
        // Collect leading comments ONLY in PreserveComments mode
        let leading_comments = if self.mode == ParserMode::PreserveComments {
            self.collect_leading_comments()
        } else {
            vec![]
        };

        // ... rest of parsing ...

        // Collect trailing comment ONLY in PreserveComments mode
        let trailing_comment = if self.mode == ParserMode::PreserveComments {
            self.collect_trailing_comment()
        } else {
            None
        };

        Ok(SelectStatement {
            // ... all fields ...
            leading_comments,
            trailing_comment,
        })
    }
}
```

### Key Points
- **Default behavior unchanged**: `Parser::new()` uses Standard mode
- **Opt-in**: Only `Parser::with_mode(..., PreserveComments)` collects comments
- **Guard all comment collection**: Check mode before calling collection methods
- **Minimal overhead**: Empty vecs/None in Standard mode (essentially free)

### Testing
```rust
#[test]
fn test_parser_mode_compatibility() {
    let sql = "-- Leading\nSELECT * FROM t";

    // Standard mode (current behavior)
    let mut parser_std = Parser::new(sql);
    let stmt = parser_std.parse().unwrap();
    assert!(stmt.leading_comments.is_empty());

    // Preserve mode (new behavior)
    let mut parser_preserve = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser_preserve.parse().unwrap();
    assert_eq!(stmt.leading_comments.len(), 1);
    assert_eq!(stmt.leading_comments[0].text.trim(), "Leading");
}
```

### Verification
- ✅ All existing parser tests pass (Standard mode)
- ✅ New tests verify PreserveComments mode
- ✅ Zero performance impact in Standard mode

**Time estimate: 1-2 hours**

---

## Phase 3: Formatter Integration (Non-Breaking) 🔜 LATER

### Goal
Make the AST formatter emit comments when they're present.

### Implementation

```rust
// src/sql/parser/ast_formatter.rs

impl AstFormatter {
    fn format_select(&self, stmt: &SelectStatement, indent_level: usize) -> String {
        let mut result = String::new();
        let indent = self.indent(indent_level);

        // Emit leading comments if present
        for comment in &stmt.leading_comments {
            self.format_comment(&mut result, comment, &indent);
        }

        // SELECT keyword
        write!(&mut result, "{}{}", indent, self.keyword("SELECT")).unwrap();

        // ... rest of formatting ...

        // Emit trailing comment if present
        if let Some(ref comment) = stmt.trailing_comment {
            write!(&mut result, "  ").unwrap();
            self.format_inline_comment(&mut result, comment);
        }

        result
    }

    fn format_comment(&self, result: &mut String, comment: &Comment, indent: &str) {
        if comment.is_line_comment {
            writeln!(result, "{}-- {}", indent, comment.text).unwrap();
        } else {
            // Multi-line block comments
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

### Testing
```rust
#[test]
fn test_format_with_comments() {
    let sql = "-- Important query\nSELECT * FROM users -- fetch all";

    // Parse with comment preservation
    let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser.parse().unwrap();

    // Format and verify comments preserved
    let formatted = format_select_statement(&stmt);
    assert!(formatted.contains("-- Important query"));
    assert!(formatted.contains("-- fetch all"));
}
```

### Verification
- ✅ Formatter handles empty comment fields gracefully (Standard mode)
- ✅ Comments are emitted when present (PreserveComments mode)
- ✅ Formatting is correct both with and without comments

**Time estimate: 1 hour**

---

## Phase 4: Neovim Plugin Integration 🎯 GOAL

### Goal
Use comment-preserving parser in `\sf` formatter.

### Implementation

```rust
// src/cli/refactoring.rs or wherever \sf is handled

pub fn format_query_preserving_comments(query: &str) -> Result<String, String> {
    // Use comment-preserving mode
    let mut parser = Parser::with_mode(query, ParserMode::PreserveComments);
    match parser.parse() {
        Ok(stmt) => Ok(format_select_statement(&stmt)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
```

```lua
-- nvim-plugin/lua/sql-cli/formatter.lua

-- Can now remove comment extraction workaround!
function M.format_query(query)
    -- Just call the formatter - comments are preserved automatically
    local result = vim.fn.system({
        sql_cli_path,
        '--format-preserving-comments',
        query
    })
    return result
end
```

### Testing
Manual testing in Neovim:
1. Write SQL with comments
2. Run `\sf`
3. Verify comments preserved

**Time estimate: 30 minutes**

---

## Phase 5: Incremental Enhancement 🚀 FUTURE

Once the foundation is solid, gradually add comment preservation to more constructs:

### 5.1: SELECT Item Comments
```sql
SELECT
    id,        -- Primary key
    name,      -- User name
    email      -- Contact
FROM users
```

### 5.2: WHERE Clause Comments
```sql
WHERE
    -- Check active users
    status = 'active'
    -- And verified emails
    AND verified = true
```

### 5.3: CTE Comments
```sql
-- User metrics
WITH user_stats AS (
    SELECT ...
)
SELECT * FROM user_stats
```

Each enhancement:
- Is optional
- Has its own tests
- Can be done independently
- Doesn't break existing functionality

---

## Implementation Order

### Week 1 (Today - Next Session)
- [ ] **Phase 1**: Dual-mode lexer (1 hour)
- [ ] **Phase 2**: Opt-in parser mode (1-2 hours)
- [ ] Test thoroughly
- [ ] Commit: "feat: Add opt-in comment preservation mode to parser"

### Week 2
- [ ] **Phase 3**: Formatter integration (1 hour)
- [ ] Test end-to-end
- [ ] Commit: "feat: Formatter emits comments when present in AST"

### Week 3
- [ ] **Phase 4**: Neovim plugin integration (30 min)
- [ ] Manual testing with real queries
- [ ] Remove old workarounds
- [ ] Commit: "feat: Preserve comments in \\sf formatter"

### Future
- [ ] **Phase 5**: Incremental enhancements as needed

---

## Safety Guarantees

Each phase maintains these guarantees:

1. **Backward compatibility**: All existing code paths work unchanged
2. **Opt-in changes**: New behavior only when explicitly requested
3. **Test coverage**: Each phase has dedicated tests
4. **Independent commits**: Each phase can be merged separately
5. **Rollback safety**: Can revert any phase without breaking others

---

## Success Criteria

### Minimal Success (Phase 1-4)
- ✅ `\sf` in Neovim preserves top-level comments
- ✅ No existing tests broken
- ✅ No performance regression in standard mode

### Full Success (Phase 5)
- ✅ All comment types preserved
- ✅ Comments attached to correct AST nodes
- ✅ Complex queries format correctly with comments

---

## Technical Decisions

### Why Opt-In Mode?
- **Safety**: Existing code unaffected
- **Testing**: Can thoroughly test before making default
- **Migration**: Gradual rollout to users
- **Performance**: Zero overhead when not needed

### Why Not Separate Comment AST?
- Would require parallel data structure
- More complex to maintain
- Harder to keep in sync
- Our approach integrates naturally

### Why This Phasing?
- Each phase is independently testable
- Can stop at any phase with working system
- Low risk - nothing breaks along the way
- Matches how tree-sitter evolved

---

## Next Session Action Items

1. **Implement Phase 1** (Dual-mode lexer)
   - Add `LexerMode` enum
   - Add `with_mode()` constructor
   - Update `next_token()` to check mode
   - Write tests

2. **Test Phase 1**
   - Run all existing tests
   - Run new mode tests
   - Verify zero impact

3. **Commit Phase 1**
   - Clear commit message
   - Document the change
   - Note: foundation for comment preservation

Then we pause, verify everything works, and plan Phase 2 for next time.

**Estimated time for Phase 1: 1-1.5 hours**
