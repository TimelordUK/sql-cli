use sql_cli::sql::cursor_aware_parser::CursorAwareParser;
use sql_cli::sql::parser::{ColumnInfo, ColumnType, TableInfo};

/// Datetime completion is now driven by the schema rather than by a hardcoded
/// list of trade-desk column names (T2). These tests therefore have to say
/// what the column *is*; before T2 `createdDate` was datetime because it was
/// spelled that way, and every column on any other dataset was a string.
fn parser_with(columns: Vec<ColumnInfo>) -> CursorAwareParser {
    let mut parser = CursorAwareParser::new();
    parser.update_single_table_info(TableInfo::new("trade_deal", columns));
    parser
}

#[test]
fn test_datetime_completion_after_comparison() {
    let parser = parser_with(vec![
        ColumnInfo::new("createdDate").with_type(ColumnType::DateTime)
    ]);

    // Test completion after datetime column comparison
    let result = parser.get_completions("SELECT * FROM trade_deal WHERE createdDate > ", 45);

    assert!(result.context.contains("AfterComparison"));
    assert!(result.suggestions.contains(&"DateTime(".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Today".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Now".to_string()));
}

#[test]
fn test_datetime_completion_with_partial() {
    let parser = parser_with(vec![
        ColumnInfo::new("createdDate").with_type(ColumnType::DateTime)
    ]);

    // Test completion with partial "Date"
    let result = parser.get_completions("SELECT * FROM trade_deal WHERE createdDate > Date", 49);

    assert!(result.context.contains("AfterComparison"));
    // Should filter suggestions starting with "Date"
    assert!(result.suggestions.contains(&"DateTime(".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Today".to_string()));
    assert!(result.suggestions.contains(&"DateTime.Now".to_string()));
}

/// The other half of T2: a column that merely *looks* like a date is not one.
/// The old name list matched on spelling, so a CSV column called `tradeDate`
/// holding free text was offered `DateTime(`.
#[test]
fn test_datetime_suggestions_follow_the_schema_not_the_name() {
    let parser = parser_with(vec![
        ColumnInfo::new("tradeDate").with_type(ColumnType::String)
    ]);

    let query = "SELECT * FROM trade_deal WHERE tradeDate > ";
    let result = parser.get_completions(query, query.len());

    assert!(result.context.contains("AfterComparison"));
    assert!(
        !result.suggestions.contains(&"DateTime(".to_string()),
        "a string column must not be offered a DateTime constructor: {:?}",
        result.suggestions
    );
    assert!(result.suggestions.contains(&"''".to_string()));
}

/// Numeric columns used to fall through the name list to `string`, so they
/// were offered `Contains('')` ahead of anything numeric.
#[test]
fn test_numeric_column_gets_numeric_methods() {
    let parser = parser_with(vec![
        ColumnInfo::new("population").with_type(ColumnType::Numeric)
    ]);

    let query = "SELECT * FROM trade_deal WHERE population.";
    let result = parser.get_completions(query, query.len());

    assert!(
        result.suggestions.contains(&"ToString()".to_string()),
        "numeric columns should offer ToString(): {:?}",
        result.suggestions
    );
    assert!(
        !result.suggestions.contains(&"Trim()".to_string()),
        "numeric columns should not offer string-only methods: {:?}",
        result.suggestions
    );
}

/// Boolean columns had no representation at all before T2 - `independent` on
/// `data/countries.csv` is the motivating case.
#[test]
fn test_boolean_column_suggests_literals() {
    let parser = parser_with(vec![
        ColumnInfo::new("independent").with_type(ColumnType::Boolean)
    ]);

    let query = "SELECT * FROM trade_deal WHERE independent = ";
    let result = parser.get_completions(query, query.len());

    assert!(result.context.contains("AfterComparison"));
    assert_eq!(
        result.suggestions,
        vec!["true".to_string(), "false".to_string()]
    );
}

/// An unknown name - no file loaded, or text that is not a column - still
/// falls back to string methods rather than offering nothing.
#[test]
fn test_unknown_column_falls_back_to_string_methods() {
    let parser = CursorAwareParser::new();

    let query = "SELECT * FROM whatever WHERE mystery.";
    let result = parser.get_completions(query, query.len());

    assert!(
        result.suggestions.contains(&"Contains('')".to_string()),
        "unknown columns keep the safe string default: {:?}",
        result.suggestions
    );
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
