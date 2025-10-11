# Lexer and Parser Considerations

## Current Implementation

SQL-CLI uses a **bespoke hand-written lexer and recursive descent parser**. This approach has served the project well, providing:

- **Flexibility**: Easy to add custom SQL extensions (WEB CTEs, CODE CTEs, temp tables, etc.)
- **Control**: Direct control over tokenization and parsing behavior
- **Simplicity**: Straightforward code that's easy to understand and modify
- **No dependencies**: Minimal external parser dependencies

## Edge Cases Discovered (2025-01)

### Issue 1: Context-Aware Tokenization - Subtraction vs Negative Numbers

**Problem**: Expressions like `n-1` (without spaces) were incorrectly tokenized.

**Example**:
```sql
SELECT value, value-1 FROM table  -- Fails: "value" followed by "-1" (negative number)
SELECT value, value - 1 FROM table -- Works: "value" "-" "1" (subtraction)
```

**Root Cause**: The lexer eagerly treats `-` followed by a digit as a negative number literal without considering:
1. Previous token context (should be subtraction after identifier/number)
2. Whitespace presence (no space = likely subtraction, space = could be negative)

**Attempted Fix**: Added `prev_token` tracking to determine if `-digit` is subtraction vs negative number.

**New Problem**: This broke legitimate negative numbers in sequences:
```sql
SELECT 456.789 -12.34  -- Now incorrectly parsed as subtraction instead of two numbers
```

**Proper Solution Would Require**:
1. Track whitespace in tokens (add `has_leading_space` flag)
2. Combine context (prev_token) + whitespace to disambiguate
3. More complex lexer state management

**Similar Issues in Other Languages**:
- C++: `vector<vector<int>>` vs `vector<vector<int> >` (pre-C++11)
- JavaScript: Automatic semicolon insertion
- Python: Significant whitespace for indentation

### Issue 2: Reserved Keywords vs Column Names

**Problem**: Adding new keywords for CODE CTEs (`CODE`, `LANGUAGE`, `SOURCE`, `FUNCTION`, `INPUT`) made them reserved, breaking existing queries.

**Example**:
```sql
-- This query broke after adding CODE keyword:
SELECT id, code.IndexOf('2') FROM test_simple_strings
-- Error: "Unexpected token in primary expression: Code"

-- The column 'code' is now tokenized as Token::Code instead of Token::Identifier("code")
```

**Test Failures**:
- `test_indexof_method` - Uses `code` column
- `test_startswith_prefix_check` - Uses `code` column
- `test_multiple_string_methods_in_select` - Uses `code` column
- `test_tokenizer_numbers` - Negative number handling

**Root Cause**: Keywords are globally reserved in the lexer. Column names that match keywords can't be used as identifiers.

**Solutions**:

1. **Context-Sensitive Keywords** (Recommended)
   - Keywords are only reserved in specific parser contexts
   - `CODE` is a keyword after `WITH`, but identifier elsewhere
   - Requires parser-level disambiguation, not lexer-level
   - Example: PostgreSQL does this for many keywords

2. **Quoted Identifiers**
   - Require users to quote conflicting names: `SELECT "code" FROM table`
   - Common in SQL (SQL Server uses `[code]`, others use `"code"`)
   - Less user-friendly

3. **Namespaced Keywords**
   - Use longer keywords: `CODE_CTE` instead of `CODE`
   - Reduces collision risk
   - Makes syntax more verbose

4. **Non-Reserved Keywords**
   - Mark certain keywords as "non-reserved" in lexer
   - Only treat as keyword in specific syntax positions
   - More complex lexer logic

## Off-the-Shelf Alternatives

### Option 1: sqlparser-rs

**Repo**: https://github.com/sqlparser-rs/sqlparser-rs

**Pros**:
- Most popular Rust SQL parser (4.3k stars)
- Supports multiple SQL dialects (PostgreSQL, MySQL, MSSQL, etc.)
- Battle-tested edge case handling
- Good error messages
- Active maintenance

**Cons**:
- Custom SQL extensions (WEB CTEs, CODE CTEs) would require:
  - Forking the library OR
  - Submitting PRs (slow iteration) OR
  - Working around the AST (awkward)
- Less control over tokenization behavior
- Heavier dependency

**Verdict**: Would work for standard SQL, but custom features like WEB/CODE CTEs are core to this project.

### Option 2: Parser Combinators (nom, chumsky)

**Nom**: https://github.com/rust-bakery/nom
**Chumsky**: https://github.com/zesterer/chumsky

**Pros**:
- Composable parser building blocks
- Flexible for custom syntax
- Good error recovery (especially chumsky)
- Type-safe parser construction

**Cons**:
- Steeper learning curve
- More complex codebase
- Error messages can be cryptic for users
- May be slower than hand-written parser

**Verdict**: Good for new projects, but refactoring existing parser is significant work.

### Option 3: Parser Generators (pest, lalrpop)

**Pest**: https://github.com/pest-parser/pest
**LALRPOP**: https://github.com/lalrpop/lalrpop

**Pros**:
- Declarative grammar specification
- Automatic parser generation
- Well-defined grammar file (good documentation)

**Cons**:
- Additional build step
- Less runtime flexibility
- Grammar debugging can be difficult
- Learning PEG/LR(1) formalisms

**Verdict**: Great for clean-slate design, overkill for incremental improvements.

## Recommendation: Keep Bespoke Parser

**Why the current approach is good**:

1. **Custom SQL dialect is core to the project**
   - WEB CTEs for HTTP data fetching
   - CODE CTEs for programmable transformations
   - Temp tables (#table syntax)
   - Template variable injection
   - These are NOT standard SQL features

2. **Easy to extend**
   - Adding new keywords: 5 lines in lexer
   - Adding new syntax: one new parse method
   - Quick iteration on language design

3. **Simple and maintainable**
   - ~700 lines of lexer code
   - Recursive descent parser is straightforward
   - No external parser DSL to learn

4. **Edge cases are manageable**
   - Most issues have known solutions (context-sensitive keywords, whitespace tracking)
   - Only implement when actually needed
   - Don't over-engineer for hypothetical problems

5. **Performance is good**
   - Hand-written parsers are typically faster than generated ones
   - No runtime overhead from parser combinators

**When to reconsider**:

1. **If standard SQL compliance becomes critical**
   - Need to support complex joins, subqueries, CTEs, window functions at parity with PostgreSQL
   - Currently the project has 90% coverage of common SQL, which is sufficient

2. **If custom extensions become too numerous**
   - 10+ custom keywords causing constant collisions
   - Complex syntax interactions that are hard to maintain

3. **If error messages become a major pain point**
   - Parser combinators (chumsky) excel at error recovery and helpful messages
   - Current parser gives adequate errors for typical use

4. **If external contributors struggle with parser code**
   - PEG grammar might be easier for newcomers to understand
   - But: most contributions won't touch the parser

## Recommended Fixes for Current Issues

### For Issue 1 (Subtraction without spaces)

**Short-term**: Require spaces around `-` operator in expressions.
- Document in user guide: "Use `n - 1`, not `n-1`"
- Most SQL writers use spaces anyway

**Long-term** (if user demand is high):
1. Add `has_leading_whitespace: bool` to Token enum
2. Track whitespace in lexer advance()
3. Combine prev_token + whitespace for disambiguation:
   ```rust
   Some('-') if next_is_digit() => {
       if prev_token.is_identifier_like() && !has_leading_whitespace {
           Token::Minus  // value-1 → subtraction
       } else {
           Token::NumberLiteral(...)  // SELECT -1, or value -1 → negative
       }
   }
   ```

### For Issue 2 (Reserved keywords)

**Short-term**: Use longer/namespaced keywords for CODE CTEs
- `WITH CODE_BLOCK name AS ...` instead of `WITH CODE name AS ...`
- `LANGUAGE` → `CODE_LANGUAGE`
- `SOURCE` → `CODE_SOURCE`
- Avoids collisions with common column names

**Medium-term**: Context-sensitive keyword parsing
- Lexer returns `Token::Identifier("code")` for all occurrences
- Parser checks if identifier matches keyword in current context:
  ```rust
  fn parse_cte(&mut self) -> Result<CTE> {
      if self.peek_identifier_matches("CODE") {
          return self.parse_code_cte();
      }
      // ...
  }
  ```
- More parser logic, but better UX

**Long-term**: Implement SQL-standard quoted identifiers
- PostgreSQL/ANSI: `SELECT "code" FROM table`
- SQL Server style: `SELECT [code] FROM table` (already partially supported)
- Document and encourage quoting for keyword conflicts

## Trade-off Summary

| Approach | Flexibility | Maintenance | Edge Cases | User Friendliness |
|----------|-------------|-------------|------------|-------------------|
| **Bespoke (current)** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **sqlparser-rs** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Parser Combinators** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Parser Generators** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |

**Conclusion**: The bespoke parser is the right choice for this project. Edge cases should be handled incrementally as user pain points emerge, not preemptively.

## References

- [PostgreSQL Reserved Keywords](https://www.postgresql.org/docs/current/sql-keywords-appendix.html) - Good example of reserved vs non-reserved
- [SQLite Parser Implementation](https://www.sqlite.org/src/file/src/parse.y) - Hand-written parser that's very robust
- [Crafting Interpreters - Scanning](https://craftinginterpreters.com/scanning.html) - Great primer on lexer design
- [Context-Sensitive Lexing in SQL](https://use-the-index-luke.com/sql/where-clause/obfuscation/smart-logic) - Why SQL is hard to parse

## Decision Log

- **2025-01-11**: Decided to keep bespoke parser after analyzing trade-offs
- **2025-01-11**: Rolled back context-aware `-` tokenization due to edge cases
- **2025-01-11**: Rolled back CODE CTE keywords due to column name conflicts
- **Next**: Implement CODE CTEs with namespaced keywords (`CODE_BLOCK`, etc.)
