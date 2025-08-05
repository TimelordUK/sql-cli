use sql_cli::cursor_aware_parser::CursorAwareParser;

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
    use sql_cli::recursive_parser::Parser;
    
    let mut parser = Parser::new("SELECT * FROM trade_deal WHERE createdDate > DateTime(2025, 10, 20)");
    let stmt = parser.parse().unwrap();
    
    assert!(stmt.where_clause.is_some());
    let where_clause = stmt.where_clause.unwrap();
    assert_eq!(where_clause.conditions.len(), 1);
    
    // Verify the DateTime constructor was parsed correctly
    use sql_cli::recursive_parser::SqlExpression;
    if let SqlExpression::BinaryOp { left, op, right } = &where_clause.conditions[0].expr {
        assert_eq!(op, ">");
        assert!(matches!(left.as_ref(), SqlExpression::Column(col) if col == "createdDate"));
        assert!(matches!(right.as_ref(), SqlExpression::DateTimeConstructor { year: 2025, month: 10, day: 20, .. }));
    } else {
        panic!("Expected BinaryOp with DateTime constructor");
    }
}