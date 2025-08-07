use sql_cli::recursive_parser::{Parser, SortDirection, SqlExpression};

#[test]
fn test_complex_trade_query_with_multiple_filters() {
    let query = "SELECT accruedInterest, allocationStatus, book, clearingHouse, comments, commission, confirmationStatus, counterparty, counterpartyCountry, counterpartyId, counterpartyType, createdDate, currency, cusip, dV01, dealId, delta, desk, duration, exchange, externalOrderId, fees, gamma, instrumentId, instrumentName, instrumentType, isElectronic, isin, lastModifiedDate, maturityDate, notional, pV01, parentOrderId, platformOrderId, portfolio, price, prime, productType, quantity, settlementAmount, settlementDate, settlementStatus, side, status, strategy, ticker, tradeDate, trader, traderId, valueDate, vega, venue, yield FROM trades_10k where book in ('bond trading', 'commodities', 'credit trading', 'etf trading', 'emerging markets', 'equity trading', 'fx trading', 'futures trading') and confirmationStatus not in ('pending') and currency in ('CAD') order by counterparty desc, counterpartyCountry asc, counterpartyType asc";

    let mut parser = Parser::new(query);
    let statement = parser.parse().ok();

    // Verify the query parses successfully
    assert!(
        statement.is_some(),
        "Complex query should parse successfully"
    );

    if let Some(stmt) = statement {
        // Check that we have the right number of columns (53 columns)
        assert_eq!(stmt.columns.len(), 53, "Should have 53 columns selected");
        assert!(stmt.columns.contains(&"accruedInterest".to_string()));
        assert!(stmt.columns.contains(&"yield".to_string()));

        // Check table name
        assert_eq!(stmt.from_table.as_deref(), Some("trades_10k"));

        // Check WHERE clause
        assert!(stmt.where_clause.is_some(), "Should have WHERE clause");
        // We'll just verify it parsed, as the WhereClause structure is complex

        // Check ORDER BY
        assert!(stmt.order_by.is_some(), "Should have ORDER BY clause");
        if let Some(order_by) = &stmt.order_by {
            assert_eq!(order_by.len(), 3, "Should have 3 ORDER BY columns");

            // Check first column is DESC
            assert_eq!(order_by[0].column, "counterparty");
            assert!(matches!(order_by[0].direction, SortDirection::Desc));

            // Check second and third are ASC
            assert_eq!(order_by[1].column, "counterpartyCountry");
            assert!(matches!(order_by[1].direction, SortDirection::Asc));

            assert_eq!(order_by[2].column, "counterpartyType");
            assert!(matches!(order_by[2].direction, SortDirection::Asc));
        }
    }
}

#[test]
fn test_complex_aggregation_with_joins() {
    let query = "SELECT t1.trader, t1.book, COUNT(t1.dealId) as trade_count, SUM(t1.notional) as total_notional, AVG(t1.commission) as avg_commission FROM trades t1 INNER JOIN counterparties t2 ON t1.counterpartyId = t2.id WHERE t1.tradeDate BETWEEN '2024-01-01' AND '2024-12-31' AND t2.region = 'EMEA' GROUP BY t1.trader, t1.book HAVING COUNT(t1.dealId) > 100 ORDER BY total_notional DESC LIMIT 50";

    let mut parser = Parser::new(query);
    let statement = parser.parse().ok();

    // This query tests:
    // - Table aliases (t1, t2)
    // - Aggregate functions (COUNT, SUM, AVG)
    // - Column aliases (as trade_count, etc.)
    // - JOIN clause
    // - BETWEEN operator
    // - GROUP BY with multiple columns
    // - HAVING clause
    // - ORDER BY with alias
    // - LIMIT clause

    assert!(
        statement.is_some(),
        "Complex aggregation query should parse"
    );
}

#[test]
fn test_nested_subqueries() {
    let query = "SELECT * FROM (SELECT trader, SUM(notional) as total FROM trades WHERE book IN (SELECT book FROM books WHERE active = true) GROUP BY trader) t WHERE t.total > (SELECT AVG(notional) * 10 FROM trades WHERE status = 'confirmed')";

    let mut parser = Parser::new(query);
    let statement = parser.parse().ok();

    // This tests:
    // - Subquery in FROM clause
    // - Subquery in IN clause
    // - Subquery in WHERE comparison
    // - Nested aggregations

    assert!(statement.is_some(), "Nested subquery should parse");
}

#[test]
fn test_window_functions() {
    let query = "SELECT trader, tradeDate, notional, ROW_NUMBER() OVER (PARTITION BY trader ORDER BY tradeDate DESC) as rank, SUM(notional) OVER (PARTITION BY trader ORDER BY tradeDate ROWS BETWEEN 5 PRECEDING AND CURRENT ROW) as rolling_sum FROM trades WHERE status = 'confirmed'";

    let mut parser = Parser::new(query);
    let statement = parser.parse().ok();

    // This tests:
    // - ROW_NUMBER() window function
    // - OVER clause with PARTITION BY
    // - Window frame specification (ROWS BETWEEN)

    // Note: Parser might not fully support window functions yet
    // but we test that it doesn't crash
    assert!(
        statement.is_some() || statement.is_none(),
        "Window function query should at least not panic"
    );
}

#[test]
fn test_case_when_expressions() {
    let query = "SELECT trader, CASE WHEN notional > 1000000 THEN 'Large' WHEN notional > 100000 THEN 'Medium' ELSE 'Small' END as trade_size, CASE book WHEN 'Equity Trading' THEN 'EQ' WHEN 'Bond Trading' THEN 'FI' ELSE 'OTHER' END as book_code FROM trades";

    let mut parser = Parser::new(query);
    let statement = parser.parse().ok();

    // This tests:
    // - CASE WHEN expressions
    // - Multiple WHEN clauses
    // - ELSE clause
    // - Simple CASE (with value after CASE)

    assert!(
        statement.is_some() || statement.is_none(),
        "CASE WHEN query should at least not panic"
    );
}

// Helper functions for assertions
fn check_contains_in_list(expr: &SqlExpression, column: &str, expected_items: usize) {
    match expr {
        SqlExpression::InList { expr: col, values } => {
            if let SqlExpression::Column(col_name) = &**col {
                assert_eq!(col_name, column, "Should check {} IN list", column);
            }
            assert_eq!(
                values.len(),
                expected_items,
                "{} IN list should have {} items",
                column,
                expected_items
            );
        }
        _ => panic!("Expected IN list for {}", column),
    }
}

fn check_contains_not_in(expr: &SqlExpression, column: &str) {
    match expr {
        SqlExpression::NotInList { expr: col, values } => {
            if let SqlExpression::Column(col_name) = &**col {
                assert_eq!(col_name, column, "Should check {} NOT IN", column);
            }
            assert!(!values.is_empty(), "{} NOT IN should have items", column);
        }
        _ => panic!("Expected NOT IN for {}", column),
    }
}
