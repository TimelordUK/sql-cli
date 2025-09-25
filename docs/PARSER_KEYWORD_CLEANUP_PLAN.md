# Parser Keyword Cleanup Plan

## Current State Analysis

The recursive parser currently has many hardcoded string comparisons for keywords, causing:
- Performance overhead from repeated `to_uppercase()` calls
- String allocations in hot paths
- Inconsistent keyword handling
- Maintenance challenges

## Phase 1: Audit Current Keyword Usage

### Identified Problem Patterns

1. **Autocomplete Context (lines 1880-1885)**
```rust
if trimmed.to_uppercase().ends_with(" AND")
    || trimmed.to_uppercase().ends_with(" OR")
    || trimmed.to_uppercase().ends_with(" AND ")
    || trimmed.to_uppercase().ends_with(" OR ")
```

2. **Direct String Comparisons**
```rust
if token_str.to_uppercase() == "SELECT"
if word.to_uppercase() == "FROM"
```

3. **Multiple Case Variations**
```rust
matches!(s.to_uppercase().as_str(), "ASC" | "DESC")
```

## Phase 2: Solution Design

### Option A: Enhance Token System (Recommended)
Leverage the existing Token enum more effectively:

```rust
// Current Token enum already has many keywords
pub enum Token {
    Select, From, Where, And, Or, // etc.
}

// Add helper methods
impl Token {
    pub fn from_keyword(s: &str) -> Option<Token> {
        match s.to_uppercase().as_str() {
            "SELECT" => Some(Token::Select),
            "FROM" => Some(Token::From),
            // ... etc
        }
    }

    pub fn is_logical_operator(&self) -> bool {
        matches!(self, Token::And | Token::Or)
    }
}
```

### Option B: Keyword Module with Constants
Create a dedicated keywords module:

```rust
pub mod keywords {
    pub const SELECT: &str = "SELECT";
    pub const FROM: &str = "FROM";

    pub fn is_keyword(s: &str) -> bool {
        KEYWORDS.contains(&s.to_uppercase().as_str())
    }

    lazy_static! {
        static ref KEYWORDS: HashSet<&'static str> = {
            // All SQL keywords
        };
    }
}
```

## Phase 3: Implementation Steps

### Step 1: Create Keyword Infrastructure
- [ ] Enhance Token enum with all SQL keywords
- [ ] Add Token::from_keyword() method
- [ ] Add Token classification methods (is_join_type, is_aggregate, etc.)
- [ ] Create case-insensitive comparison utilities

### Step 2: Replace String Comparisons
- [ ] Replace autocomplete context checks (lines 1880-1885)
- [ ] Replace parse_select_list keyword checks
- [ ] Replace parse_from_clause keyword checks
- [ ] Replace parse_where_clause keyword checks
- [ ] Replace parse_order_by keyword checks
- [ ] Replace parse_group_by keyword checks

### Step 3: Optimize Hot Paths
- [ ] Cache uppercase conversions where needed
- [ ] Use token lookahead instead of string peeking
- [ ] Pre-compute keyword lengths for position tracking

### Step 4: Testing
- [ ] Ensure all existing tests pass
- [ ] Add performance benchmarks for parsing
- [ ] Test case-insensitive parsing thoroughly

## Phase 4: Web CTE Module Extraction

After keyword cleanup, extract WEB CTE:

### Module Structure
```
src/web_cte/
├── mod.rs           # Public API
├── spec.rs          # WebCTESpec types
├── parser.rs        # Parse WEB keyword and spec
├── executor.rs      # HTTP execution
├── auth/
│   ├── mod.rs      # Auth trait
│   ├── bearer.rs   # Bearer token auth
│   └── basic.rs    # Basic auth
├── cache.rs         # Response caching
└── error.rs         # Error types
```

### Future NTLM Support (if needed)
```rust
// In auth/ntlm.rs (future)
pub struct NtlmAuthProvider {
    // Could delegate to Flask proxy
    proxy_url: Option<String>,
}
```

## Success Criteria

1. **Performance**: Fewer string allocations, no redundant uppercase conversions
2. **Maintainability**: Clear keyword handling, easy to add new keywords
3. **Correctness**: All tests pass, case-insensitive parsing works
4. **Modularity**: WEB CTE in its own module with clear interfaces

## Order of Operations

1. Start with Token enum enhancements
2. Replace easiest string comparisons first (single keywords)
3. Handle complex cases (autocomplete, multi-word keywords)
4. Extract WEB CTE module
5. Add auth provider abstraction

---

*Note on NTLM: The Flask proxy solution is actually quite good. Native NTLM in Rust is complex and platform-specific. We could document the proxy pattern as an official solution for legacy auth systems.*