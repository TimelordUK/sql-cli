use sql_cli::sql::cursor_aware_parser::CursorAwareParser;

#[test]
fn test_datetime_completion_after_comparison() {
    let parser = CursorAwareParser::new();

    // Test completion after datetime column comparison
    let result = parser.get_completions("SELECT * FROM trade_deal WHERE createdDate > ", 45);

    assert!(result.context.contains("AfterComparison"));
    assert!(result.suggestions.contains(&"DateTime(".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Today".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Now".to_string()));
}

#[test]
fn test_datetime_completion_with_partial() {
    let parser = CursorAwareParser::new();

    // Test completion with partial "Date"
    let result = parser.get_completions("SELECT * FROM trade_deal WHERE createdDate > Date", 49);

    assert!(result.context.contains("AfterComparison"));
    // Should filter suggestions starting with "Date"
    assert!(result.suggestions.contains(&"DateTime(".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Today".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Now".to_string()));
}

#[test]
fn test_datetime_parsing() {
    use sql_cli::sql::recursive_parser::Parser;

    let mut parser =
        Parser::new("SELECT * FROM trade_deal WHERE createdDate > DateTime(2025, 10, 20)");
    let stmt = parser.parse().unwrap();

    assert!(stmt.where_clause.is_some());
    let where_clause = stmt.where_clause.unwrap();
    assert_eq!(where_clause.conditions.len(), 1);

    // DateTime(...) lowers to an ordinary call on the registry's DATETIME
    // function, so its arguments are expressions rather than parse-time
    // constants. That is what lets DateTime(Year, Month, Day) work at all.
    use sql_cli::sql::recursive_parser::SqlExpression;
    if let SqlExpression::BinaryOp { left, op, right } = &where_clause.conditions[0].expr {
        assert_eq!(op, ">");
        assert!(matches!(left.as_ref(), SqlExpression::Column(col) if col.name == "createdDate"));
        match right.as_ref() {
            SqlExpression::FunctionCall { name, args, .. } => {
                assert_eq!(name, "DATETIME");
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0], SqlExpression::NumberLiteral(n) if n == "2025"));
                assert!(matches!(&args[1], SqlExpression::NumberLiteral(n) if n == "10"));
                assert!(matches!(&args[2], SqlExpression::NumberLiteral(n) if n == "20"));
            }
            other => panic!("Expected DATETIME function call, got {other:?}"),
        }
    } else {
        panic!("Expected BinaryOp with DateTime constructor");
    }
}

#[test]
fn test_datetime_accepts_column_arguments() {
    use sql_cli::sql::recursive_parser::{Parser, SqlExpression};

    // The whole point of the change: components that are columns, not literals.
    let mut parser = Parser::new("SELECT DateTime(Year, Month, Day) AS d FROM birthdays");
    let stmt = parser.parse().unwrap();

    let SqlExpression::FunctionCall { name, args, .. } = extract_first_expression(&stmt) else {
        panic!("Expected DATETIME function call");
    };
    assert_eq!(name, "DATETIME");
    assert_eq!(args.len(), 3);
    for (arg, expected) in args.iter().zip(["Year", "Month", "Day"]) {
        assert!(
            matches!(arg, SqlExpression::Column(col) if col.name == expected),
            "expected column {expected}, got {arg:?}"
        );
    }
}

#[test]
fn test_datetime_no_args_is_still_today() {
    use sql_cli::sql::recursive_parser::{Parser, SqlExpression};

    // The registry signature needs 3-7 args, so the bare form keeps its own node.
    let mut parser = Parser::new("SELECT DateTime() AS d FROM t");
    let stmt = parser.parse().unwrap();

    assert!(matches!(
        extract_first_expression(&stmt),
        SqlExpression::DateTimeToday { .. }
    ));
}

fn extract_first_expression(
    stmt: &sql_cli::sql::recursive_parser::SelectStatement,
) -> &sql_cli::sql::recursive_parser::SqlExpression {
    use sql_cli::sql::recursive_parser::SelectItem;
    match &stmt.select_items[0] {
        SelectItem::Expression { expr, .. } => expr,
        other => panic!("Expected an expression select item, got {other:?}"),
    }
}
