# Query Preprocessor: Current State & Enhancement Plan

## Executive Summary

**Current State:** We have a **solid foundation** in `src/query_plan/` with several working transformers, but they're not fully utilized. The recent JOIN improvements (Phase 1+2 with complex expressions) have dramatically improved our scoping capabilities.

**Key Insight:** We don't need to "hack" things in - we need to **build on what exists** with a **structural, incremental approach** that understands AST semantics.

**Strategy:** Enhance the existing preprocessor infrastructure incrementally, starting with simple high-value wins (HAVING auto-aliasing, parallel CTE detection) and progressing to complex rewrites (correlated subqueries).

---

## Current Infrastructure Audit

### ✅ What Exists (Good Foundation)

Located in `src/query_plan/`:

#### 1. **CTEHoister** (`cte_hoister.rs`)
**Purpose:** Hoist nested CTEs to top level

**Example:**
```sql
-- Input:
SELECT * FROM (
    WITH inner AS (SELECT * FROM data)
    SELECT * FROM inner
)

-- Output:
WITH inner AS (SELECT * FROM data)
SELECT * FROM inner
```

**Status:** ✅ **Working**
**Invoked:** Yes, in `non_interactive.rs:430`
**Quality:** Good structural understanding of AST nesting

#### 2. **ExpressionLifter** (`expression_lifter.rs`)
**Purpose:** Lift window functions from WHERE to CTEs

**Example:**
```sql
-- Input:
SELECT * FROM data
WHERE ROW_NUMBER() OVER (ORDER BY id) = 1

-- Output:
WITH __lifted_1 AS (
    SELECT *, ROW_NUMBER() OVER (ORDER BY id) as lifted_value
    FROM data
)
SELECT * FROM __lifted_1 WHERE lifted_value = 1
```

**Status:** ✅ **Working**
**Invoked:** Yes, in `non_interactive.rs:419-424`
**Quality:** Good - understands which expressions need lifting

#### 3. **InOperatorLifter** (`in_operator_lifter.rs`)
**Purpose:** Optimize IN operator with large value lists

**Status:** ✅ **Working**
**Invoked:** Yes, in `non_interactive.rs:412`
**Quality:** Specialized optimizer

#### 4. **IntoClauseRemover** (`into_clause_remover.rs`)
**Purpose:** Remove INTO clause (SQL Server syntax)

**Status:** ✅ **Working**
**Quality:** Simple syntax normalizer

#### 5. **DependencyAnalyzer** (`dependency_analyzer.rs`)
**Purpose:** Analyze dependencies between CTEs in scripts

**Status:** ✅ **Working**
**Quality:** Good - can detect CTE dependency chains

#### 6. **QueryPlan** (`query_plan.rs`)
**Purpose:** Represent execution plan as WorkUnits with dependencies

**Key Features:**
- Dependency graph
- Parallelizable flag
- Cost estimates
- WorkUnitType enum (TableScan, CTE, Filter, etc.)

**Status:** ⚠️ **Partially Used**
**Issue:** Infrastructure exists but not fully integrated with executor

---

### ❌ What Doesn't Exist Yet

1. **Correlated Subquery Rewriter** - Main gap
2. **Aggregate Auto-Aliasing in HAVING** - Easy win
3. **Parallel CTE Detector** - Easy win
4. **Complex Expression Scope Analyzer** - Needed for smart rewrites
5. **Query Optimizer** - Cost-based decisions
6. **Debugging/Visualization** - `--show-rewritten` flag

---

## Recent Improvements That Help Preprocessing

### JOIN Phase 1+2: Complex Expression Support

**Before:** Only simple column references in JOINs
**Now:** Full expressions on both sides!

```sql
-- This now works:
JOIN table ON LOWER(TRIM(t1.email)) = LOWER(TRIM(t2.email))
```

**Impact on Preprocessing:**
- ✅ Scoping is much better - can resolve `t1.col`, `t2.col` correctly
- ✅ Expression evaluation works in join context
- ✅ Can use this for subquery hoisting with complex join conditions

**Key Files:**
- `src/data/hash_join.rs` - Evaluates expressions in join context
- `src/data/arithmetic_evaluator.rs` - Expression evaluator with table context
- `src/sql/parser/ast.rs` - `SingleJoinCondition.left_expr/right_expr`

---

## Scoping Analysis

### What We Can Now Scope Properly

1. **Table-qualified columns:** `users.id`, `orders.user_id`
2. **Alias-qualified columns:** `u.id`, `o.user_id` (after JOIN alias resolution)
3. **Expressions in JOIN:** `TRIM(u.name) = c.name`
4. **Window functions:** `PARTITION BY u.region`
5. **Aggregates:** `GROUP BY u.country`

### Where Scoping Still Has Limits

1. **Correlated subqueries:** No "outer" context passing yet
2. **Nested scopes:** Multiple levels of subquery nesting
3. **Lateral correlation:** Cannot reference left side of CROSS APPLY

**Conclusion:** Scoping is now **good enough** for 90% of preprocessing use cases!

---

## Quick Wins We Can Implement Now

### Quick Win 1: HAVING Aggregate Auto-Aliasing

**Problem:**
```sql
-- This fails:
HAVING COUNT(*) > 5

-- User must write:
SELECT COUNT(*) as cnt
HAVING cnt > 5
```

**Solution:** Preprocessor auto-adds alias

**Implementation:**
```rust
// In new module: src/query_plan/having_normalizer.rs

pub struct HavingNormalizer;

impl HavingNormalizer {
    pub fn normalize(mut stmt: SelectStatement) -> SelectStatement {
        if let Some(having) = &stmt.having_clause {
            // Scan HAVING for aggregate functions
            let aggregates = find_aggregates_in_having(having);

            for agg in aggregates {
                // Check if this aggregate exists in SELECT
                if !select_contains_aggregate(&stmt.select_items, &agg) {
                    // Add it with auto-generated alias
                    let alias = format!("__having_agg_{}", hash(&agg));
                    stmt.select_items.push(SelectItem::Expression {
                        expr: agg.clone(),
                        alias: alias.clone(),
                    });

                    // Replace aggregate in HAVING with alias reference
                    replace_aggregate_with_alias(&mut stmt.having_clause, &agg, &alias);
                }
            }
        }
        stmt
    }
}
```

**Complexity:** **Easy** (1-2 days)
**Value:** **High** (user-facing improvement)
**Risk:** **Low** (doesn't affect execution, just adds columns)

---

### Quick Win 2: Parallel CTE Detection

**Problem:** Two independent Web CTEs block each other

```sql
WITH
    WEB users AS (URL 'http://api.com/users' FORMAT JSON),
    WEB orders AS (URL 'http://api.com/orders' FORMAT JSON)  -- Waits for users!
SELECT * FROM users JOIN orders ON users.id = orders.user_id;
```

**Solution:** Mark as parallelizable

**Implementation:**
```rust
// In new module: src/query_plan/parallel_cte_detector.rs

pub struct ParallelCTEDetector;

impl ParallelCTEDetector {
    pub fn analyze(stmt: &SelectStatement) -> Vec<ParallelGroup> {
        let dep_graph = build_cte_dependency_graph(&stmt.ctes);

        // Find CTEs with no dependencies between them
        let mut groups = Vec::new();
        for cte_set in find_independent_sets(&dep_graph) {
            groups.push(ParallelGroup {
                ctes: cte_set,
                can_execute_parallel: true,
            });
        }
        groups
    }
}

// In executor:
if let Some(parallel_groups) = plan.parallel_cte_groups {
    for group in parallel_groups {
        // Use tokio::spawn or rayon to execute in parallel
        let futures = group.ctes.iter()
            .map(|cte| async { execute_cte(cte) });
        let results = join_all(futures).await;
    }
}
```

**Complexity:** **Medium** (3-4 days with async integration)
**Value:** **High** (performance improvement for Web CTEs)
**Risk:** **Medium** (requires async execution changes)

---

### Quick Win 3: Simple Correlated Scalar Subquery

**Example:**
```sql
-- Input (doesn't work):
SELECT
    c.id,
    (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.id) as order_count
FROM customers c;

-- Output (works):
WITH __corr_1 AS (
    SELECT customer_id, COUNT(*) as __result
    FROM orders
    GROUP BY customer_id
)
SELECT
    c.id,
    COALESCE(__corr_1.__result, 0) as order_count
FROM customers c
LEFT JOIN __corr_1 ON c.id = __corr_1.customer_id;
```

**Implementation:**
```rust
// In new module: src/query_plan/correlated_subquery_rewriter.rs

pub struct CorrelatedSubqueryRewriter {
    cte_counter: usize,
}

impl CorrelatedSubqueryRewriter {
    pub fn rewrite(mut stmt: SelectStatement) -> SelectStatement {
        let mut rewriter = Self { cte_counter: 0 };

        // Scan SELECT items for scalar subqueries
        for item in &mut stmt.select_items {
            if let SelectItem::Expression { expr, .. } = item {
                if let Some(new_cte) = rewriter.try_lift_scalar_subquery(expr) {
                    stmt.ctes.push(new_cte);
                    // Replace expression with CTE reference
                }
            }
        }

        stmt
    }

    fn try_lift_scalar_subquery(&mut self, expr: &SqlExpression) -> Option<CTE> {
        match expr {
            SqlExpression::ScalarSubquery { subquery } => {
                // Analyze if it's correlated
                let correlation = find_outer_references(subquery)?;

                if correlation.is_simple_equality() {
                    // Can rewrite!
                    Some(self.create_aggregate_cte(subquery, &correlation))
                } else {
                    None // Too complex
                }
            }
            _ => None,
        }
    }
}
```

**Complexity:** **Hard** (2-3 weeks)
**Value:** **Very High** (major SQL compatibility win)
**Risk:** **Medium-High** (complex transformation, many edge cases)

---

## Incremental Enhancement Strategy

### Phase 0: Foundation (Week 1)
**Goal:** Clean up existing infrastructure

1. **Audit current preprocessor invocation**
   - Understand execution order
   - Document which transformers run when
   - Create integration test suite

2. **Add debugging infrastructure**
   - `--show-preprocessing` flag
   - Log each transformation applied
   - Show before/after AST

3. **Create preprocessor registry**
   ```rust
   pub struct PreprocessorPipeline {
       transformers: Vec<Box<dyn ASTTransformer>>,
   }

   pub trait ASTTransformer {
       fn name(&self) -> &str;
       fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement>;
       fn enabled(&self) -> bool;
   }
   ```

**Deliverable:** Clean, testable pipeline infrastructure

---

### Phase 1: Quick Wins (Weeks 2-3)

**1.1: HAVING Auto-Aliasing**
- Implement `HavingNormalizer`
- Add tests
- Enable by default

**1.2: SELECT * Expansion in CTEs**
- Expand `SELECT *` early in pipeline
- Helps with column dependency analysis

**1.3: Implicit COALESCE for Outer Joins**
- Auto-wrap nullable columns with COALESCE(col, default)
- Optional (flag-controlled)

**Deliverable:** 3 new transformers, all user-visible improvements

---

### Phase 2: Parallel Detection (Week 4-5)

**2.1: CTE Dependency Graph Enhancement**
- Enhance existing `DependencyAnalyzer`
- Detect truly independent CTEs
- Mark with `parallelizable: true`

**2.2: Executor Integration**
- Modify executor to respect parallel hints
- Use rayon or tokio for parallel execution
- Benchmark improvements

**Deliverable:** Parallel CTE execution for Web CTEs

---

### Phase 3: Simple Correlated Subquery (Weeks 6-9)

**3.1: Outer Reference Detector**
- Analyze subquery AST for outer table references
- Build correlation map: `{outer_table.col: inner_condition}`

**3.2: Correlation Classifier**
- Simple: Equality in WHERE (`WHERE o.cust_id = c.id`)
- Medium: Equality with AND (`WHERE o.cust_id = c.id AND o.status = 'ACTIVE'`)
- Complex: Expression (`WHERE o.amount > c.threshold * 1.5`)

**3.3: Scalar Aggregate Rewriter**
- For simple correlations only
- Create CTE with GROUP BY on correlated column
- Replace subquery with LEFT JOIN to CTE
- Wrap with COALESCE for NULL handling

**3.4: EXISTS Rewriter**
- Convert EXISTS to SEMI JOIN pattern
- Create CTE with DISTINCT on correlated column
- Use INNER JOIN

**Deliverable:** Working correlated subquery support for 60-70% of patterns

---

### Phase 4: Advanced Rewrites (Weeks 10-12)

**4.1: Top-N per Group**
- Detect pattern: `WHERE (SELECT COUNT(*) ... ) < N`
- Rewrite to window function

**4.2: Nested Correlated Subqueries**
- Handle subquery within subquery
- Recursive rewriting

**4.3: IN Correlated Subquery**
- Convert to SEMI JOIN pattern

**Deliverable:** 90%+ correlated subquery pattern coverage

---

### Phase 5: Optimization & Polish (Weeks 13-14)

**5.1: Cost-Based Decisions**
- When to rewrite vs execute natively
- Heuristics for performance

**5.2: Error Messages**
- Clear messages for unrewritable patterns
- Suggest manual CTE refactoring

**5.3: Documentation**
- User guide for preprocessing
- Before/after examples
- Performance benchmarks

**Deliverable:** Production-ready preprocessor

---

## Structural AST Understanding

### Key Principle: **Semantic Analysis, Not Pattern Matching**

Instead of:
```rust
// Bad: Brittle pattern matching
if let Some(select) = find_select_in_where(where_clause) {
    // Hack something in
}
```

Do:
```rust
// Good: Semantic understanding
struct SubqueryAnalyzer {
    outer_scope: ScopeContext,
    inner_scope: ScopeContext,
}

impl SubqueryAnalyzer {
    fn analyze(&self, subquery: &SelectStatement) -> CorrelationAnalysis {
        let outer_refs = find_outer_references(subquery, &self.outer_scope);
        let correlation_type = classify_correlation(&outer_refs);

        CorrelationAnalysis {
            is_correlated: !outer_refs.is_empty(),
            correlation_type,
            can_rewrite: correlation_type.is_rewritable(),
            rewrite_strategy: determine_strategy(&correlation_type),
        }
    }
}
```

### Scope Context Design

```rust
pub struct ScopeContext {
    /// Tables available in this scope
    available_tables: HashMap<String, TableInfo>,

    /// Aliases in scope (u -> users, o -> orders)
    table_aliases: HashMap<String, String>,

    /// Column visibility (which columns are accessible)
    visible_columns: HashSet<QualifiedColumn>,

    /// Parent scope (for nested subqueries)
    parent: Option<Box<ScopeContext>>,
}

impl ScopeContext {
    /// Check if a column reference is from outer scope
    fn is_outer_reference(&self, col_ref: &ColumnRef) -> bool {
        // Check if column is not in current scope
        !self.visible_columns.contains(col_ref) &&
        self.parent.as_ref().map_or(false, |p| p.can_resolve(col_ref))
    }

    /// Resolve which table a column belongs to
    fn resolve_column(&self, col: &str) -> Option<QualifiedColumn> {
        // With our improved JOIN scoping, this is now reliable!
        for (alias, table) in &self.table_aliases {
            if let Some(table_info) = self.available_tables.get(table) {
                if table_info.has_column(col) {
                    return Some(QualifiedColumn {
                        table: alias.clone(),
                        column: col.to_string(),
                    });
                }
            }
        }
        None
    }
}
```

### Rewrite Decision Tree

```rust
pub enum RewriteStrategy {
    /// No rewrite needed
    None,

    /// Simple: Aggregate + GROUP BY + LEFT JOIN
    AggregateHoist {
        group_by_columns: Vec<String>,
        aggregate_expr: SqlExpression,
        join_condition: JoinCondition,
    },

    /// Semi Join: EXISTS → INNER JOIN
    SemiJoin {
        correlation_columns: Vec<(String, String)>,
        filter: Option<SqlExpression>,
    },

    /// Window Function: Top-N pattern
    WindowFunction {
        partition_by: Vec<String>,
        order_by: Vec<OrderByColumn>,
        filter: WindowFilter,
    },

    /// Cannot rewrite: Too complex
    Unrewritable {
        reason: String,
        manual_workaround: Option<String>,
    },
}

impl CorrelationAnalysis {
    fn determine_strategy(&self) -> RewriteStrategy {
        match self.correlation_type {
            CorrelationType::SimpleEquality { left, right } => {
                if self.has_aggregate {
                    RewriteStrategy::AggregateHoist { ... }
                } else {
                    RewriteStrategy::SemiJoin { ... }
                }
            }

            CorrelationType::ComplexExpression => {
                RewriteStrategy::Unrewritable {
                    reason: "Outer column used in expression".to_string(),
                    manual_workaround: Some(generate_manual_cte_example()),
                }
            }

            _ => RewriteStrategy::None,
        }
    }
}
```

---

## Testing Strategy

### Level 1: Unit Tests
Each transformer has isolated tests:

```rust
#[test]
fn test_having_normalizer_count_star() {
    let input = parse_sql("SELECT * FROM t GROUP BY a HAVING COUNT(*) > 5");
    let output = HavingNormalizer::normalize(input);

    assert!(output.select_items.iter().any(|item| {
        matches!(item, SelectItem::Expression {
            expr: SqlExpression::FunctionCall { name, .. },
            alias,
        } if name == "COUNT" && alias.starts_with("__having_agg_"))
    }));
}
```

### Level 2: Integration Tests
End-to-end query transformation:

```rust
#[test]
fn test_correlated_scalar_rewrite() {
    let input = r#"
        SELECT c.id, (SELECT COUNT(*) FROM orders WHERE customer_id = c.id)
        FROM customers c
    "#;

    let stmt = parse_sql(input);
    let rewritten = CorrelatedSubqueryRewriter::rewrite(stmt);

    // Should have generated a CTE
    assert!(!rewritten.ctes.is_empty());

    // CTE should have GROUP BY
    if let CTEType::Standard(cte_stmt) = &rewritten.ctes[0].cte_type {
        assert!(cte_stmt.group_by.is_some());
    }
}
```

### Level 3: SQL Equivalence Tests
Ensure rewrite produces same results:

```sql
-- tests/preprocessing/correlated_scalar_equivalence.sql

-- Original query (will be rewritten)
SELECT c.customer_id, c.customer_name,
       (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.customer_id) as order_count
FROM customers c
ORDER BY c.customer_id;

-- Expected equivalent (manual CTE)
WITH order_counts AS (
    SELECT customer_id, COUNT(*) as cnt FROM orders GROUP BY customer_id
)
SELECT c.customer_id, c.customer_name, COALESCE(oc.cnt, 0) as order_count
FROM customers c
LEFT JOIN order_counts oc ON c.customer_id = oc.customer_id
ORDER BY c.customer_id;
```

Test framework runs both and compares results.

---

## Success Metrics

### Coverage Metrics
- **HAVING patterns:** 100% auto-aliased
- **Parallel CTEs:** 100% detected
- **Correlated subqueries:** 90%+ rewritable patterns supported

### Performance Metrics
- **Parallel CTEs:** 2-4x speedup for independent Web CTEs
- **Correlated rewrites:** 100-1000x speedup vs hypothetical row-by-row execution

### Usability Metrics
- **Error rate:** <5% of queries fail preprocessing
- **Error clarity:** 100% of failures have helpful messages
- **Debugging:** `--show-preprocessing` shows all transformations

---

## Risk Mitigation

### Risk 1: Breaking Existing Queries
**Mitigation:**
- Comprehensive test suite (500+ queries)
- Each transformer has `enabled` flag
- `--no-preprocessing` escape hatch

### Risk 2: Performance Regression
**Mitigation:**
- Benchmark every transformation
- Cost estimates guide rewrite decisions
- User can disable specific transformers

### Risk 3: Complex Bug Debugging
**Mitigation:**
- `--show-preprocessing` shows each step
- Error messages reference original query
- Extensive logging at DEBUG level

---

## Next Steps

1. **Review this plan** - Ensure alignment on approach
2. **Phase 0 implementation** - Clean up infrastructure (this week)
3. **Quick Win: HAVING normalizer** - Prove the concept (next week)
4. **Iterate** - Build incrementally, test extensively

---

## Appendix: File Structure

```
src/
├── query_plan/
│   ├── mod.rs                          # Existing - main module
│   ├── query_plan.rs                   # Existing - WorkUnit types
│   ├── cte_hoister.rs                  # Existing - works well
│   ├── expression_lifter.rs            # Existing - works well
│   ├── in_operator_lifter.rs           # Existing - specialized
│   ├── into_clause_remover.rs          # Existing - simple
│   ├── dependency_analyzer.rs          # Existing - good foundation
│   │
│   ├── pipeline.rs                     # NEW - orchestration
│   ├── scope_analyzer.rs               # NEW - semantic understanding
│   ├── having_normalizer.rs            # NEW - Phase 1
│   ├── parallel_cte_detector.rs        # NEW - Phase 2
│   ├── correlated_subquery_rewriter.rs # NEW - Phase 3
│   ├── correlation_analyzer.rs         # NEW - Phase 3 support
│   └── rewrite_strategies.rs           # NEW - Phase 3 support
│
└── preprocessing/                      # NEW directory
    ├── mod.rs
    ├── transformers.rs                 # Trait definitions
    └── testing.rs                      # Test utilities
```

**Principle:** Build on what exists, add incrementally, maintain structure.

