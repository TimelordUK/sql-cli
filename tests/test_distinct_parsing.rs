/// Test to verify DISTINCT keyword parsing in aggregate functions
use sql_cli::sql::recursive_parser::Parser;

#[test]
fn test_count_distinct_parsing() {
    let query = "SELECT COUNT(DISTINCT counterparty) FROM trades";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "COUNT(DISTINCT column) should parse successfully, got: {result:?}"
    );

    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 1);
    assert_eq!(stmt.columns[0], "expr_1"); // Default alias for computed expressions
}

#[test]
fn test_multiple_distinct_aggregates() {
    let query = r"
        SELECT 
            COUNT(DISTINCT counterparty) as unique_counterparties,
            SUM(DISTINCT amount) as total_unique_amounts,
            AVG(DISTINCT price) as avg_unique_price
        FROM trades
    ";

    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Multiple DISTINCT aggregates should parse successfully, got: {result:?}"
    );

    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 3);
    assert_eq!(stmt.columns[0], "unique_counterparties");
    assert_eq!(stmt.columns[1], "total_unique_amounts");
    assert_eq!(stmt.columns[2], "avg_unique_price");
}

#[test]
fn test_original_failing_query() {
    let query = r"
        SELECT 
            COUNT(*) as total_trades,
            AVG(commission) as avg_commission,
            MIN(commission) as min_commission,
            MAX(commission) as max_commission,
            COUNT(DISTINCT counterparty) as unique_counterparties
        FROM trades 
        WHERE NOT confirmationStatus.Contains('reject') 
        AND NOT confirmationStatus.Contains('cancel')
    ";

    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Original failing query should now parse successfully, got: {result:?}"
    );

    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 5);
    assert_eq!(stmt.columns[4], "unique_counterparties"); // The DISTINCT aggregate should be parsed

    // Ensure WHERE clause is also parsed correctly
    assert!(stmt.where_clause.is_some());
    let where_clause = stmt.where_clause.unwrap();
    assert!(!where_clause.conditions.is_empty());
}
