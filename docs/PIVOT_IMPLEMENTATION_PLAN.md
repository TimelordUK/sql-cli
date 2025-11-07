# PIVOT/UNPIVOT Implementation Plan

**Feature**: PIVOT/UNPIVOT SQL syntax support
**Priority**: Medium
**Difficulty**: Medium
**Estimated Effort**: 3-5 days
**Approach**: Transformer-based (rewrite to CASE expressions + GROUP BY)

## Overview

PIVOT transforms rows into columns by aggregating values. It's a common data reshaping operation, especially useful for reporting and data analysis.

## Example Use Case (FoodEaten)

### Input Data
```sql
SELECT * FROM FoodEaten;
```

| Id | Date       | FoodName | AmountEaten |
|----|------------|----------|-------------|
| 1  | 2019-08-01 | Sammich  | 2           |
| 2  | 2019-08-01 | Pickle   | 3           |
| 3  | 2019-08-01 | Apple    | 1           |
| 4  | 2019-08-02 | Sammich  | 1           |
| 5  | 2019-08-02 | Pickle   | 1           |
| 6  | 2019-08-02 | Apple    | 4           |
| 7  | 2019-08-03 | Cake     | 2           |
| 8  | 2019-08-04 | Sammich  | 1           |
| 9  | 2019-08-04 | Pickle   | 2           |
| 10 | 2019-08-04 | Apple    | 3           |

### PIVOT Query
```sql
SELECT [Date] AS 'Day',
    [Sammich], [Pickle], [Apple], [Cake]
FROM (
    SELECT [Date], FoodName, AmountEaten FROM FoodEaten
) AS SourceTable
PIVOT (
    MAX(AmountEaten)
    FOR FoodName IN ([Sammich], [Pickle], [Apple], [Cake])
) AS PivotTable
```

### Expected Output

| Day        | Sammich | Pickle | Apple | Cake |
|------------|---------|--------|-------|------|
| 2019-08-01 | 2       | 3      | 1     | NULL |
| 2019-08-02 | 1       | 1      | 4     | NULL |
| 2019-08-03 | NULL    | NULL   | NULL  | 2    |
| 2019-08-04 | 1       | 2      | 3     | NULL |

### With NULL Handling (using COALESCE)
```sql
SELECT [Date] AS 'Day',
    COALESCE([Sammich], 0) AS Sammich,
    COALESCE([Pickle],  0) AS Pickle,
    COALESCE([Apple],   0) AS Apple,
    COALESCE([Cake],    0) AS Cake
FROM (
    SELECT [Date], FoodName, AmountEaten FROM FoodEaten
) AS SourceTable
PIVOT (
    MAX(AmountEaten)
    FOR FoodName IN ([Sammich], [Pickle], [Apple], [Cake])
) AS PivotTable
```

| Day        | Sammich | Pickle | Apple | Cake |
|------------|---------|--------|-------|------|
| 2019-08-01 | 2       | 3      | 1     | 0    |
| 2019-08-02 | 1       | 1      | 4     | 0    |
| 2019-08-03 | 0       | 0      | 0     | 2    |
| 2019-08-04 | 1       | 2      | 3     | 0    |

## Transformation Strategy

### Input SQL (PIVOT syntax)
```sql
SELECT Date, Sammich, Pickle, Apple, Cake
FROM (
    SELECT Date, FoodName, AmountEaten FROM FoodEaten
) AS SourceTable
PIVOT (
    MAX(AmountEaten)
    FOR FoodName IN (Sammich, Pickle, Apple, Cake)
) AS PivotTable
```

### Transformed SQL (Standard SQL)
```sql
SELECT Date,
    MAX(CASE WHEN FoodName = 'Sammich' THEN AmountEaten ELSE NULL END) AS Sammich,
    MAX(CASE WHEN FoodName = 'Pickle' THEN AmountEaten ELSE NULL END) AS Pickle,
    MAX(CASE WHEN FoodName = 'Apple' THEN AmountEaten ELSE NULL END) AS Apple,
    MAX(CASE WHEN FoodName = 'Cake' THEN AmountEaten ELSE NULL END) AS Cake
FROM (
    SELECT Date, FoodName, AmountEaten FROM FoodEaten
) AS SourceTable
GROUP BY Date
```

## Implementation Phases

### Phase 1: Parser Support (1 day)

**Goal**: Parse PIVOT syntax into AST

**Changes needed**:
1. Extend `TableSource` enum in `src/sql/parser/ast.rs`:
```rust
pub enum TableSource {
    Table { name: String, alias: Option<String> },
    Subquery { query: Box<SelectStatement>, alias: String },

    // NEW
    Pivot {
        source: Box<TableSource>,      // The input table/subquery
        aggregate: PivotAggregate,      // MAX(AmountEaten)
        pivot_column: String,           // FoodName
        pivot_values: Vec<String>,      // [Sammich, Pickle, Apple, Cake]
        alias: Option<String>,          // PivotTable
    },
}

pub struct PivotAggregate {
    pub function: String,      // "MAX", "SUM", "MIN", etc.
    pub column: String,         // "AmountEaten"
}
```

2. Add PIVOT keyword to lexer (`src/sql/lexer.rs`):
```rust
"PIVOT" => Token::Keyword(Keyword::Pivot),
```

3. Extend parser in `src/sql/recursive_parser.rs`:
```rust
fn parse_table_source(&mut self) -> Result<TableSource, String> {
    // ... existing code for Table, Subquery ...

    // After parsing subquery or table, check for PIVOT
    if self.peek_keyword("PIVOT") {
        return self.parse_pivot(base_source);
    }

    Ok(base_source)
}

fn parse_pivot(&mut self, source: TableSource) -> Result<TableSource, String> {
    self.consume_keyword("PIVOT")?;
    self.consume(Token::LeftParen)?;

    // Parse aggregate: MAX(AmountEaten)
    let aggregate = self.parse_pivot_aggregate()?;

    self.consume_keyword("FOR")?;

    // Parse pivot column: FoodName
    let pivot_column = self.parse_identifier()?;

    self.consume_keyword("IN")?;
    self.consume(Token::LeftParen)?;

    // Parse pivot values: Sammich, Pickle, Apple, Cake
    let pivot_values = self.parse_identifier_list()?;

    self.consume(Token::RightParen)?;
    self.consume(Token::RightParen)?;

    // Optional alias
    let alias = if self.peek_keyword("AS") {
        self.consume_keyword("AS")?;
        Some(self.parse_identifier()?)
    } else {
        None
    };

    Ok(TableSource::Pivot {
        source: Box::new(source),
        aggregate,
        pivot_column,
        pivot_values,
        alias,
    })
}
```

**Testing**:
- Parse PIVOT syntax without errors
- AST structure captures all components
- Test with various aggregate functions (MAX, SUM, MIN, AVG, COUNT)

---

### Phase 2: PivotExpander Transformer (2 days)

**Goal**: Transform PIVOT AST into standard SQL with CASE expressions

**Location**: `src/query_plan/pivot_expander.rs` (new file)

**Algorithm**:
1. Extract PIVOT specification from TableSource
2. Generate CASE expression for each pivot value
3. Wrap each CASE in the aggregate function
4. Determine GROUP BY columns (all non-pivot, non-aggregate columns)
5. Build new SelectStatement with GROUP BY

**Implementation**:
```rust
pub struct PivotExpander;

impl PivotExpander {
    pub fn transform(stmt: SelectStatement) -> Result<SelectStatement> {
        // Check if FROM contains a PIVOT
        if let Some(TableSource::Pivot { source, aggregate, pivot_column, pivot_values, .. }) = &stmt.from {
            return Self::expand_pivot(stmt, source, aggregate, pivot_column, pivot_values);
        }
        Ok(stmt)
    }

    fn expand_pivot(
        stmt: SelectStatement,
        source: &TableSource,
        aggregate: &PivotAggregate,
        pivot_column: &str,
        pivot_values: &[String],
    ) -> Result<SelectStatement> {
        // 1. Build CASE expressions for each pivot value
        let mut new_select_items = Vec::new();

        // Keep non-pivoted columns (for GROUP BY)
        let group_by_columns = Self::extract_grouping_columns(source, pivot_column, &aggregate.column);

        for col in &group_by_columns {
            new_select_items.push(SelectItem::Column {
                column: ColumnRef::unquoted(col.clone()),
                leading_comments: vec![],
                trailing_comment: None,
            });
        }

        // Generate pivoted columns
        for value in pivot_values {
            let case_expr = Self::build_case_expression(
                pivot_column,
                value,
                &aggregate.column,
            );

            let aggregate_expr = SqlExpression::FunctionCall {
                name: aggregate.function.clone(),
                args: vec![case_expr],
                distinct: false,
            };

            new_select_items.push(SelectItem::Expression {
                expr: aggregate_expr,
                alias: value.clone(),
                leading_comments: vec![],
                trailing_comment: None,
            });
        }

        // 2. Build new SELECT with GROUP BY
        Ok(SelectStatement {
            select_items: new_select_items,
            from: Some(*source.clone()),
            where_clause: stmt.where_clause,
            group_by: Some(group_by_columns.iter().map(|c|
                SqlExpression::Column(ColumnRef::unquoted(c.clone()))
            ).collect()),
            having: stmt.having,
            order_by: stmt.order_by,
            limit: stmt.limit,
            ..stmt
        })
    }

    fn build_case_expression(
        pivot_column: &str,
        pivot_value: &str,
        aggregate_column: &str,
    ) -> SqlExpression {
        // CASE WHEN FoodName = 'Sammich' THEN AmountEaten ELSE NULL END
        SqlExpression::CaseExpression {
            when_branches: vec![
                WhenBranch {
                    condition: Box::new(SqlExpression::BinaryOp {
                        left: Box::new(SqlExpression::Column(
                            ColumnRef::unquoted(pivot_column.to_string())
                        )),
                        op: "=".to_string(),
                        right: Box::new(SqlExpression::StringLiteral(pivot_value.to_string())),
                    }),
                    result: Box::new(SqlExpression::Column(
                        ColumnRef::unquoted(aggregate_column.to_string())
                    )),
                }
            ],
            else_branch: Some(Box::new(SqlExpression::Null)),
        }
    }

    fn extract_grouping_columns(
        source: &TableSource,
        pivot_column: &str,
        aggregate_column: &str,
    ) -> Vec<String> {
        // For now, we'll need to infer from the source
        // This requires schema information
        // Simplified: assume all columns except pivot_column and aggregate_column
        vec!["Date".to_string()] // TODO: Make dynamic
    }
}
```

**Challenges**:
1. **Schema inference**: Need to know which columns to GROUP BY
   - Solution: Analyze subquery SELECT list
   - Exclude pivot_column and aggregate_column
   - Use remaining columns for GROUP BY

2. **Column selection in outer query**:
   - User may select specific columns: `SELECT Date, Sammich FROM ... PIVOT ...`
   - Need to filter generated columns to match SELECT list

**Testing**:
- Transform FoodEaten example correctly
- Verify CASE expressions are generated properly
- Check GROUP BY columns are correct
- Test with different aggregates (SUM, MIN, AVG)

---

### Phase 3: Integration & Testing (1 day)

**Changes needed**:
1. Register transformer in `src/execution/statement_executor.rs`:
```rust
transformers.push(Box::new(PivotExpander));
```

2. Add to transformer pipeline (after subquery processing, before execution)

**Test Cases**:

1. **Basic PIVOT** (FoodEaten example)
2. **PIVOT with SUM** instead of MAX
3. **PIVOT with COUNT**
4. **PIVOT with NULL handling** (COALESCE in outer SELECT)
5. **PIVOT with WHERE clause** on source
6. **PIVOT with ORDER BY** on result
7. **Multiple grouping columns**

**Create test files**:
- `data/food_eaten.csv` - Test dataset
- `examples/pivot_demo.sql` - Comprehensive examples
- `tests/python_tests/test_pivot.py` - Formal tests

---

### Phase 4: UNPIVOT Support (Optional - 1 day)

**UNPIVOT** does the reverse: columns → rows

**Example**:
```sql
SELECT Date, FoodName, AmountEaten
FROM PivotedData
UNPIVOT (
    AmountEaten FOR FoodName IN (Sammich, Pickle, Apple, Cake)
) AS UnpivotTable
```

**Transformation**:
```sql
SELECT Date, 'Sammich' AS FoodName, Sammich AS AmountEaten FROM PivotedData
UNION ALL
SELECT Date, 'Pickle' AS FoodName, Pickle AS AmountEaten FROM PivotedData
UNION ALL
SELECT Date, 'Apple' AS FoodName, Apple AS AmountEaten FROM PivotedData
UNION ALL
SELECT Date, 'Cake' AS FoodName, Cake AS AmountEaten FROM PivotedData
```

**Effort**: ~1 day if PIVOT is working

---

## SQL Syntax Variants to Support

### Microsoft SQL Server Style (Primary)
```sql
SELECT * FROM source
PIVOT (
    MAX(value_column)
    FOR pivot_column IN (val1, val2, val3)
) AS alias
```

### Optional Extensions
```sql
-- Multiple aggregates (harder)
PIVOT (
    MAX(amount), SUM(quantity)
    FOR month IN ('Jan', 'Feb', 'Mar')
)
```

---

## NULL Handling

PIVOT naturally produces NULLs when no data exists for a combination.

**User can handle with COALESCE**:
```sql
SELECT Date,
    COALESCE(Sammich, 0) AS Sammich,
    COALESCE(Pickle, 0) AS Pickle
FROM ... PIVOT ...
```

**Note**: We already have COALESCE function, so this works!

---

## Edge Cases to Handle

1. **No data for a pivot value** → NULL (expected)
2. **Pivot column contains NULL** → Skip (or include as "NULL" value?)
3. **Empty result set** → Empty output
4. **Duplicate pivot values** → Should be caught by aggregate (MAX/MIN/SUM handle this)
5. **Pivot values not in data** → All NULL column
6. **Very large number of pivot values** → Performance concern (100s of CASE expressions)

---

## Performance Considerations

PIVOT becomes:
- N CASE expressions (one per pivot value)
- GROUP BY on remaining columns
- One table scan

**Performance**: Should be comparable to manual CASE + GROUP BY (which users would write anyway)

**Optimization opportunity**: If pivot values are known to be mutually exclusive, we could use simpler logic, but CASE/GROUP BY is safe and standard.

---

## Documentation Updates

1. **README.md** - Add PIVOT to feature list
2. **CHANGELOG.md** - Document new feature
3. **examples/pivot_demo.sql** - Comprehensive examples
4. **docs/SQL_FEATURE_GAPS_AND_ROADMAP.md** - Mark as completed

---

## Success Criteria

- ✅ FoodEaten example works correctly
- ✅ Output matches expected pivoted table
- ✅ COALESCE for NULL handling works
- ✅ Multiple aggregates supported (MAX, SUM, MIN, AVG, COUNT)
- ✅ Works in all execution modes (-q, -f, --execute-statement)
- ✅ --show-transformations shows PIVOT → CASE rewrite
- ✅ Formal Python test coverage
- ✅ Example file demonstrates use cases

---

## Timeline

**Day 1**: Parser support + basic AST
**Day 2**: PivotExpander transformer (core logic)
**Day 3**: Schema inference + GROUP BY column detection
**Day 4**: Testing + edge cases
**Day 5**: UNPIVOT (optional), documentation, examples

**Total**: 4-5 days

---

## Next Steps

1. Create FoodEaten test dataset
2. Implement parser support for PIVOT keyword
3. Build PivotExpander transformer
4. Test with FoodEaten example
5. Add comprehensive test coverage
6. Consider UNPIVOT if PIVOT goes smoothly

This feature would be a **significant addition** to SQL-CLI's data reshaping capabilities!
