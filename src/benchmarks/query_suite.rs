use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCategory {
    BasicOperations,
    Aggregations,
    SortingAndLimits,
    WindowFunctions,
    ComplexQueries,
}

impl QueryCategory {
    pub fn as_str(&self) -> &str {
        match self {
            QueryCategory::BasicOperations => "basic",
            QueryCategory::Aggregations => "aggregation",
            QueryCategory::SortingAndLimits => "sorting",
            QueryCategory::WindowFunctions => "window",
            QueryCategory::ComplexQueries => "complex",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkQuery {
    pub name: String,
    pub category: QueryCategory,
    pub sql: String,
    pub description: String,
    pub table_type: String,
}

impl BenchmarkQuery {
    pub fn new(
        name: impl Into<String>,
        category: QueryCategory,
        sql: impl Into<String>,
        description: impl Into<String>,
        table_type: impl Into<String>,
    ) -> Self {
        BenchmarkQuery {
            name: name.into(),
            category,
            sql: sql.into(),
            description: description.into(),
            table_type: table_type.into(),
        }
    }
}

pub struct QuerySuite;

impl QuerySuite {
    pub fn get_basic_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new(
                "full_scan",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench",
                "Full table scan",
                "all",
            ),
            BenchmarkQuery::new(
                "simple_filter",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE id > 50000",
                "Simple filter on indexed column",
                "all",
            ),
            BenchmarkQuery::new(
                "multi_condition",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE category = 'Electronics' AND price > 500",
                "Multiple conditions",
                "mixed",
            ),
            BenchmarkQuery::new(
                "string_equality",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE status = 'Active'",
                "String equality filter",
                "mixed",
            ),
            BenchmarkQuery::new(
                "like_pattern",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE status LIKE 'Act%'",
                "LIKE pattern matching",
                "mixed",
            ),
            BenchmarkQuery::new(
                "in_operator",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE category IN ('Electronics', 'Books', 'Clothing')",
                "IN operator with multiple values",
                "mixed",
            ),
            BenchmarkQuery::new(
                "between_range",
                QueryCategory::BasicOperations,
                "SELECT * FROM bench WHERE price BETWEEN 100 AND 500",
                "BETWEEN range filter",
                "mixed",
            ),
            BenchmarkQuery::new(
                "projection",
                QueryCategory::BasicOperations,
                "SELECT id, price, quantity FROM bench",
                "Column projection",
                "mixed",
            ),
            BenchmarkQuery::new(
                "calculated_column",
                QueryCategory::BasicOperations,
                "SELECT id, price * quantity AS total FROM bench",
                "Calculated column",
                "mixed",
            ),
        ]
    }

    pub fn get_aggregation_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new(
                "simple_aggregates",
                QueryCategory::Aggregations,
                "SELECT COUNT(*), SUM(value1), AVG(value2), MIN(value3), MAX(value3) FROM bench",
                "Simple aggregate functions",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "group_by_single",
                QueryCategory::Aggregations,
                "SELECT group_id, COUNT(*), SUM(value1) FROM bench GROUP BY group_id",
                "GROUP BY single column",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "group_by_multiple",
                QueryCategory::Aggregations,
                "SELECT group_id, sub_group, SUM(value1), AVG(value2) FROM bench GROUP BY group_id, sub_group",
                "GROUP BY multiple columns",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "having_clause",
                QueryCategory::Aggregations,
                "SELECT group_id, AVG(value1) AS avg_val FROM bench GROUP BY group_id HAVING AVG(value1) > 500",
                "GROUP BY with HAVING",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "count_distinct",
                QueryCategory::Aggregations,
                "SELECT COUNT(DISTINCT group_id), COUNT(DISTINCT sub_group) FROM bench",
                "COUNT DISTINCT",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "filtered_aggregation",
                QueryCategory::Aggregations,
                "SELECT COUNT(*), SUM(value1) FROM bench WHERE group_id > 10",
                "Aggregation with WHERE filter",
                "aggregation",
            ),
        ]
    }

    pub fn get_sorting_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new(
                "order_by_single",
                QueryCategory::SortingAndLimits,
                "SELECT * FROM bench ORDER BY price DESC",
                "ORDER BY single column",
                "mixed",
            ),
            BenchmarkQuery::new(
                "order_by_multiple",
                QueryCategory::SortingAndLimits,
                "SELECT * FROM bench ORDER BY category, price DESC",
                "ORDER BY multiple columns",
                "mixed",
            ),
            BenchmarkQuery::new(
                "top_n",
                QueryCategory::SortingAndLimits,
                "SELECT * FROM bench ORDER BY price DESC LIMIT 100",
                "TOP-N query",
                "mixed",
            ),
            BenchmarkQuery::new(
                "offset_pagination",
                QueryCategory::SortingAndLimits,
                "SELECT * FROM bench ORDER BY id LIMIT 100 OFFSET 1000",
                "OFFSET pagination",
                "mixed",
            ),
            BenchmarkQuery::new(
                "sorted_aggregation",
                QueryCategory::SortingAndLimits,
                "SELECT category, SUM(price) AS total FROM bench GROUP BY category ORDER BY total DESC",
                "Sorted aggregation results",
                "mixed",
            ),
        ]
    }

    pub fn get_window_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new(
                "row_number",
                QueryCategory::WindowFunctions,
                "SELECT *, ROW_NUMBER() OVER (ORDER BY sales DESC) AS rank FROM bench",
                "ROW_NUMBER window function",
                "window",
            ),
            BenchmarkQuery::new(
                "running_total",
                QueryCategory::WindowFunctions,
                "SELECT *, SUM(sales) OVER (ORDER BY timestamp) AS running_total FROM bench",
                "Running total with SUM OVER",
                "window",
            ),
            BenchmarkQuery::new(
                "lag_lead",
                QueryCategory::WindowFunctions,
                "SELECT *, LAG(sales, 1) OVER (ORDER BY timestamp) AS prev_sales FROM bench",
                "LAG window function",
                "window",
            ),
            BenchmarkQuery::new(
                "partitioned_rank",
                QueryCategory::WindowFunctions,
                "SELECT *, RANK() OVER (PARTITION BY department ORDER BY sales DESC) AS dept_rank FROM bench",
                "Partitioned RANK",
                "window",
            ),
            BenchmarkQuery::new(
                "moving_average",
                QueryCategory::WindowFunctions,
                "SELECT *, AVG(sales) OVER (ORDER BY timestamp ROWS BETWEEN 10 PRECEDING AND CURRENT ROW) AS moving_avg FROM bench",
                "Moving average window",
                "window",
            ),
            BenchmarkQuery::new(
                "percent_rank",
                QueryCategory::WindowFunctions,
                "SELECT *, PERCENT_RANK() OVER (ORDER BY sales) AS pct_rank FROM bench",
                "PERCENT_RANK window function",
                "window",
            ),
        ]
    }

    pub fn get_complex_queries() -> Vec<BenchmarkQuery> {
        vec![
            BenchmarkQuery::new(
                "simple_cte",
                QueryCategory::ComplexQueries,
                "WITH summary AS (SELECT group_id, AVG(value1) as avg_val FROM bench GROUP BY group_id) SELECT * FROM summary WHERE avg_val > 500",
                "Simple CTE",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "nested_cte",
                QueryCategory::ComplexQueries,
                "WITH level1 AS (SELECT * FROM bench WHERE group_id IN (1,2,3)), level2 AS (SELECT * FROM level1 WHERE value1 > 100) SELECT COUNT(*) FROM level2",
                "Nested CTEs",
                "aggregation",
            ),
            BenchmarkQuery::new(
                "complex_expression",
                QueryCategory::ComplexQueries,
                "SELECT id, price * quantity * 1.1 AS total_with_tax, CASE WHEN price > 1000 THEN 'Premium' WHEN price > 500 THEN 'Standard' ELSE 'Budget' END AS tier FROM bench",
                "Complex expressions with CASE",
                "mixed",
            ),
            BenchmarkQuery::new(
                "multiple_aggregates",
                QueryCategory::ComplexQueries,
                "SELECT category, COUNT(*) as cnt, SUM(price) as total_price, AVG(quantity) as avg_qty, MIN(price) as min_price, MAX(price) as max_price FROM bench GROUP BY category ORDER BY total_price DESC",
                "Multiple aggregates with sorting",
                "mixed",
            ),
            BenchmarkQuery::new(
                "complex_filter",
                QueryCategory::ComplexQueries,
                "SELECT * FROM bench WHERE (category = 'Electronics' AND price > 500) OR (category = 'Books' AND quantity > 10) OR (status = 'Active' AND price BETWEEN 100 AND 300)",
                "Complex OR/AND conditions",
                "mixed",
            ),
        ]
    }

    pub fn get_all_queries() -> Vec<BenchmarkQuery> {
        let mut queries = Vec::new();
        queries.extend(Self::get_basic_queries());
        queries.extend(Self::get_aggregation_queries());
        queries.extend(Self::get_sorting_queries());
        queries.extend(Self::get_window_queries());
        queries.extend(Self::get_complex_queries());
        queries
    }

    pub fn get_queries_for_table_type(table_type: &str) -> Vec<BenchmarkQuery> {
        Self::get_all_queries()
            .into_iter()
            .filter(|q| q.table_type == "all" || q.table_type == table_type)
            .collect()
    }

    pub fn get_progressive_queries() -> HashMap<usize, Vec<String>> {
        let mut progressive = HashMap::new();

        progressive.insert(
            100,
            vec![
                "SELECT * FROM bench".to_string(),
                "SELECT * FROM bench WHERE id > 50".to_string(),
                "SELECT COUNT(*) FROM bench".to_string(),
            ],
        );

        progressive.insert(
            1000,
            vec![
                "SELECT * FROM bench".to_string(),
                "SELECT * FROM bench WHERE id > 500".to_string(),
                "SELECT COUNT(*), SUM(price) FROM bench".to_string(),
                "SELECT category, COUNT(*) FROM bench GROUP BY category".to_string(),
            ],
        );

        progressive.insert(
            10000,
            vec![
                "SELECT * FROM bench LIMIT 100".to_string(),
                "SELECT * FROM bench WHERE id > 5000".to_string(),
                "SELECT category, COUNT(*), AVG(price) FROM bench GROUP BY category".to_string(),
                "SELECT * FROM bench ORDER BY price DESC LIMIT 100".to_string(),
                "SELECT *, ROW_NUMBER() OVER (ORDER BY price) AS rn FROM bench LIMIT 100"
                    .to_string(),
            ],
        );

        for size in &[
            20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000,
        ] {
            progressive.insert(
                *size,
                vec![
                    format!("SELECT * FROM bench WHERE id > {}", size / 2),
                    format!("SELECT COUNT(*) FROM bench WHERE id <= {}", size / 2),
                    "SELECT category, COUNT(*), SUM(price * quantity) FROM bench GROUP BY category".to_string(),
                    "SELECT * FROM bench ORDER BY price DESC LIMIT 1000".to_string(),
                    "SELECT *, RANK() OVER (PARTITION BY category ORDER BY price DESC) FROM bench LIMIT 1000".to_string(),
                ],
            );
        }

        progressive
    }
}
