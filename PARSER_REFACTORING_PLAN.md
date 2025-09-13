# SQL Parser Refactoring Plan

## 📊 Current State (Dec 2024)
- **File**: `src/sql/recursive_parser.rs`
- **Size**: 4,572 lines (too large!)
- **Components**:
  - Lexer/Tokenizer: ~441 lines
  - AST Definitions: ~218 lines
  - Parser Implementation: ~3,908 lines
  - 24 parse functions with deep nesting

## 🎯 Goal
Break down the monolithic parser into manageable, focused modules before implementing JOINs and other complex features.

## 📁 Target Module Structure

```
src/sql/parser/
├── mod.rs                 # Public API re-exports
├── lexer.rs              # Token + Lexer (~440 lines)
├── ast.rs                # AST types (~220 lines)
├── error.rs              # Parser errors
├── parser.rs             # Core parser logic
├── expressions/          # Expression parsing
│   ├── mod.rs           # Expression coordinator
│   ├── arithmetic.rs    # Math expressions
│   ├── comparison.rs    # Comparisons, BETWEEN, IN
│   ├── logical.rs       # AND, OR, NOT
│   ├── primary.rs       # Literals, identifiers, functions
│   └── case.rs         # CASE expressions
├── statements/          # Statement parsing
│   ├── mod.rs          # Statement coordinator
│   ├── select.rs       # SELECT parsing
│   ├── cte.rs          # WITH clause, CTEs
│   ├── from.rs         # FROM clause
│   ├── where.rs        # WHERE clause
│   ├── group_by.rs     # GROUP BY, HAVING
│   ├── order_by.rs     # ORDER BY
│   └── joins.rs        # JOIN parsing (future)
└── utils/              # Helpers
    ├── mod.rs
    ├── keywords.rs     # Keyword handling
    ├── identifiers.rs  # Identifier parsing
    └── precedence.rs   # Operator precedence
```

## 📋 Implementation Phases

### ✅ Phase 1: Extract Core Components (COMPLETED - Dec 12, 2024)
- [x] Create `src/sql/parser/` directory structure
- [x] Extract `lexer.rs` with Token enum and Lexer impl (444 lines)
- [x] Extract `ast.rs` with all AST data structures (222 lines)
- [x] Create `mod.rs` to re-export public API (25 lines)
- [x] Create `legacy.rs` for backward compatibility (207 lines)
- [x] Update `recursive_parser.rs` to use new modules
- [x] Verify compilation and basic functionality

**Result**: Reduced main parser from 4,598 to 3,933 lines (665 lines removed, 14.5% reduction)

### ✅ Phase 2: Extract Expression Parsing (COMPLETED)
- [x] Create `expressions/` module structure
- [x] Extract `primary.rs` - literals, identifiers, function calls (~343 lines)
- [x] Extract `arithmetic.rs` - additive, multiplicative expressions (~182 lines)
- [x] Extract `comparison.rs` - comparison operators, BETWEEN, IN, LIKE (~193 lines)
- [x] Add debug/trace logging infrastructure
- [x] Create expression traits (ParsePrimary, ParseArithmetic, ParseComparison)
- [x] Handle method calls within multiplicative precedence
- [x] Extract `logical.rs` - AND, OR, NOT operations (~144 lines)
- [x] Extract `case.rs` - CASE/WHEN expressions (~110 lines)

**Phase 2.1**: Primary expressions extracted (~220 lines)
**Phase 2.2**: Arithmetic expressions extracted (~95 lines)
**Phase 2.3**: Comparison expressions extracted (~80 lines)
**Phase 2.4**: Logical expressions extracted (~144 lines)
**Phase 2.5**: CASE expressions extracted (~110 lines)
**Total reduction**: ~649 lines moved to modular structure

**Bonus**: AND/OR operators now work in CASE WHEN conditions!

### 🔄 Phase 3: Extract Statement Components
- [ ] Create `statements/` module structure
- [ ] Extract `select.rs` - SELECT list parsing
- [ ] Extract `from.rs` - FROM clause and table sources
- [ ] Extract `where.rs` - WHERE clause logic
- [ ] Extract `cte.rs` - WITH clause and CTEs
- [ ] Extract `group_by.rs` - GROUP BY and HAVING
- [ ] Extract `order_by.rs` - ORDER BY parsing
- [ ] Extract `window.rs` - Window function specs

**Expected reduction**: ~1,200 lines

### 🔄 Phase 4: Extract Utilities
- [ ] Create `utils/` module structure
- [ ] Extract keyword recognition logic
- [ ] Extract identifier handling
- [ ] Extract operator precedence rules
- [ ] Create parsing helper functions

**Expected reduction**: ~300 lines

### 🔄 Phase 5: Prepare for JOINs
- [ ] Create `joins.rs` module with clean structure
- [ ] Design comprehensive JOIN AST types
- [ ] Implement JOIN parsing in isolation
- [ ] Add JOIN tests
- [ ] Integrate with main parser

## 🛠️ Refactoring Guidelines

### Do's:
- ✅ Maintain backward compatibility (same public API)
- ✅ Run tests after each extraction
- ✅ Keep modules under 500 lines
- ✅ Use clear, descriptive module names
- ✅ Document module responsibilities
- ✅ Create unit tests for extracted modules

### Don'ts:
- ❌ Don't change parsing logic during extraction
- ❌ Don't break existing functionality
- ❌ Don't create circular dependencies
- ❌ Don't over-abstract too early

## 📈 Progress Tracking

| Phase | Status | Lines Moved | Main File Size | Date Completed |
|-------|--------|------------|----------------|----------------|
| Initial | ❌ | 0 | 4,598 | - |
| Phase 1 | ✅ | 665 | 3,933 | Dec 12, 2024 |
| Phase 2.1 | ✅ | 220 | 3,713 | Dec 13, 2024 |
| Phase 2.2 | ✅ | 95 | 3,618 | Dec 13, 2024 |
| Phase 2.3 | ✅ | 80 | 3,538 | Dec 13, 2024 |
| Phase 2.4 | ✅ | 144 | 3,655 | Dec 13, 2024 |
| Phase 2.5 | ✅ | 110 | 3,633 | Dec 13, 2024 |
| Phase 3 | ⏳ | - | - | - |
| Phase 4 | ⏳ | - | - | - |
| Phase 5 | ⏳ | - | - | - |

## 🎯 Success Metrics

- [ ] No single file over 500 lines
- [ ] All tests passing
- [ ] Clear module boundaries
- [ ] Easy to add new features (like JOINs)
- [ ] Improved code navigation
- [ ] Better testability

## 📝 Notes for Next Session

### Phase 2 Complete! 🎉
All expression parsing has been successfully extracted into modular components.

### Next Priority - Phase 3:
1. Extract formatting code (916 lines!) to separate module
2. Extract statement components (SELECT, FROM, WHERE, etc.)
3. Extract utility functions
4. Continue reducing main parser file size

### Considerations for JOINs:
- Need to extend AST with comprehensive JOIN types
- Consider INNER, LEFT, RIGHT, FULL OUTER joins
- Handle JOIN conditions (ON clause)
- Support USING clause
- Handle multiple joins
- Consider join precedence and associativity

## 🔗 Related Files
- `src/sql/recursive_parser.rs` - Original monolithic parser
- `src/sql/parser/` - New modular structure
- `tests/` - Test files to verify refactoring

## 📚 References
- SQL-92 standard for JOIN syntax
- PostgreSQL JOIN documentation
- SQLite JOIN implementation

---
*Last Updated: December 12, 2024*
*Next Review: Before implementing Phase 2*