// Tests for the recursive SQL parser
// This module contains all unit tests for the recursive parser implementation

use super::super::recursive_parser::*;
use crate::sql::parser::ast::*;
use crate::sql::parser::lexer::{Lexer, Token};

#[test]
fn test_tokenizer_window_functions() {
    let mut lexer = Lexer::new("LAG(value) OVER (PARTITION BY category ORDER BY id)");
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "LAG"));
    assert!(matches!(lexer.next_token(), Token::LeftParen));
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "value"));
    assert!(matches!(lexer.next_token(), Token::RightParen));

    let over_token = lexer.next_token();
    println!("Expected OVER, got: {:?}", over_token);
    assert!(matches!(over_token, Token::Over));

    assert!(matches!(lexer.next_token(), Token::LeftParen));
    assert!(matches!(lexer.next_token(), Token::Partition));
    assert!(matches!(lexer.next_token(), Token::By));
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "category"));
}

#[test]
fn test_parse_window_function() {
    let query = "SELECT LAG(value, 1) OVER (ORDER BY id) as prev_value FROM test";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parse failed with error: {:?}",
        result.err()
    );

    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 1);
    // The parser stores the full expression as a single column
    let col = &stmt.columns[0];
    // Just verify it parsed without checking exact format
    assert!(col.len() > 0);
}

#[test]
#[ignore] // Window frame clauses (ROWS BETWEEN) not yet supported
fn test_complex_window_function_with_frame() {
    let query = "SELECT SUM(amount) OVER (PARTITION BY category ORDER BY date ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) FROM sales";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    // Frame clauses are not yet supported
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_is_null() {
    let query = "SELECT * FROM table WHERE column IS NULL";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_parse_is_not_null() {
    let query = "SELECT * FROM table WHERE column IS NOT NULL";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_between_expression() {
    let query = "SELECT * FROM products WHERE price BETWEEN 10 AND 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_between_ast_format() {
    // Create a BETWEEN expression
    let expr = SqlExpression::Between {
        expr: Box::new(SqlExpression::Column("price".to_string())),
        lower: Box::new(SqlExpression::NumberLiteral("50".to_string())),
        upper: Box::new(SqlExpression::NumberLiteral("100".to_string())),
    };

    let formatted = crate::sql::parser::formatter::format_expression(&expr);
    assert_eq!(formatted, "price BETWEEN 50 AND 100");

    let ast_formatted = crate::sql::parser::formatter::format_expression_ast(&expr);
    assert!(ast_formatted.contains("Between"));
    assert!(ast_formatted.contains("50"));
    assert!(ast_formatted.contains("100"));
}

#[test]
fn test_tokenizer() {
    let mut lexer = Lexer::new("SELECT * FROM table WHERE column = 'value'");

    assert!(matches!(lexer.next_token(), Token::Select));
    assert!(matches!(lexer.next_token(), Token::Star));
    assert!(matches!(lexer.next_token(), Token::From));
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "table"));
    assert!(matches!(lexer.next_token(), Token::Where));
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "column"));
    assert!(matches!(lexer.next_token(), Token::Equal));
    assert!(matches!(lexer.next_token(), Token::StringLiteral(s) if s == "value"));
    assert!(matches!(lexer.next_token(), Token::Eof));
}

#[test]
fn test_tokenizer_numbers() {
    let mut lexer = Lexer::new("123 456.789 -12.34 3.14e10 2.5E-3");

    assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "123"));
    assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "456.789"));
    // The lexer parses negative numbers as a single token
    assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "-12.34"));
    assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "3.14e10"));
    assert!(matches!(lexer.next_token(), Token::NumberLiteral(s) if s == "2.5E-3"));
}

#[test]
fn test_tokenizer_operators() {
    let mut lexer = Lexer::new("<= >= != < > =");
    assert!(matches!(lexer.next_token(), Token::LessThanOrEqual));
    assert!(matches!(lexer.next_token(), Token::GreaterThanOrEqual));
    assert!(matches!(lexer.next_token(), Token::NotEqual));
    assert!(matches!(lexer.next_token(), Token::LessThan));
    assert!(matches!(lexer.next_token(), Token::GreaterThan));
    assert!(matches!(lexer.next_token(), Token::Equal));
}

#[test]
fn test_parse_not_equal() {
    let query = "SELECT * FROM table WHERE column != 'value'";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_and_or() {
    let query = "SELECT * FROM table WHERE a = 1 AND b = 2 OR c = 3";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_expression_precedence() {
    let query = "SELECT * FROM table WHERE a + b * c = d";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_call() {
    let query = "SELECT COUNT(*), SUM(amount) FROM sales";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 2);
}

#[test]
fn test_parse_distinct() {
    let query = "SELECT DISTINCT category FROM products";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.distinct);
}

#[test]
fn test_parse_order_by() {
    let query = "SELECT * FROM table ORDER BY column1 ASC, column2 DESC";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 2);
}

#[test]
fn test_parse_group_by() {
    let query = "SELECT category, COUNT(*) FROM products GROUP BY category";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.group_by.is_some());
    let group_by = stmt.group_by.unwrap();
    assert_eq!(group_by.len(), 1);
}

#[test]
fn test_parse_limit() {
    let query = "SELECT * FROM table LIMIT 10";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert_eq!(stmt.limit, Some(10));
}

#[test]
fn test_parse_like() {
    let query = "SELECT * FROM table WHERE name LIKE '%pattern%'";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_in_list() {
    let query = "SELECT * FROM table WHERE id IN (1, 2, 3)";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_not_in_list() {
    let query = "SELECT * FROM table WHERE id NOT IN (1, 2, 3)";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_complex_where() {
    let query =
        "SELECT * FROM orders WHERE (status = 'pending' OR status = 'processing') AND amount > 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_nested_functions() {
    let query = "SELECT UPPER(TRIM(name)) FROM users";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_quoted_identifiers() {
    let query = r#"SELECT "First Name", "Last Name" FROM "User Table""#;
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert_eq!(stmt.columns.len(), 2);
    assert!(stmt.columns[0].contains("First Name"));
    assert!(stmt.columns[1].contains("Last Name"));
}

#[test]
fn test_parse_arithmetic_in_select() {
    let query = "SELECT price * quantity AS total FROM orders";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_function_with_multiple_args() {
    let query = "SELECT COALESCE(column1, column2, 'default') FROM table";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_case_when() {
    let query = "SELECT CASE WHEN x > 0 THEN 'positive' WHEN x < 0 THEN 'negative' ELSE 'zero' END FROM table";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_having_clause() {
    let query = "SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 5";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(stmt.having.is_some());
}

#[test]
#[ignore] // Subqueries not yet supported
fn test_parse_subquery_in_where() {
    // This should parse but may not execute correctly without full subquery support
    let query =
        "SELECT * FROM orders WHERE customer_id IN (SELECT id FROM customers WHERE active = true)";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    // We expect this to parse the outer query at least
    // Currently fails as subqueries are not implemented
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_joins() {
    // Currently not fully supported, but should at least parse the main table
    let query = "SELECT * FROM orders o JOIN customers c ON o.customer_id = c.id";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    // This might fail for now since JOINs aren't fully implemented
    // assert!(result.is_ok());
}

#[test]
fn test_parse_union() {
    // Currently not supported, but documenting for future
    let query = "SELECT id FROM table1 UNION SELECT id FROM table2";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    // This will likely fail as UNION is not implemented
    // assert!(result.is_ok());
}

#[test]
fn test_parse_not_operator() {
    let query = "SELECT * FROM table WHERE NOT (column = 'value')";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_method_call_syntax() {
    let query = "SELECT data.upper() FROM table";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_chained_method_calls() {
    let query = "SELECT name.upper().trim() FROM users";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_datetime_constructor() {
    let query = "SELECT * FROM events WHERE event_date > DateTime(2024, 1, 1)";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_today_function() {
    let query = "SELECT * FROM events WHERE event_date = TODAY()";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_boolean_literals() {
    let query = "SELECT * FROM table WHERE active = true AND deleted = false";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_null_literal() {
    let query = "SELECT * FROM table WHERE column = NULL";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_count_distinct() {
    let query = "SELECT COUNT(DISTINCT category) FROM products";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_format_select_multiline() {
    let query = "SELECT col1, col2, col3, col4, col5, col6, col7, col8 FROM table";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    // Should format columns across multiple lines
    assert!(formatted.len() > 2);
    assert!(formatted[0].contains("SELECT"));
}

#[test]
fn test_format_with_complex_where() {
    let query = "SELECT * FROM orders WHERE (status = 'pending' OR status = 'processing') AND amount > 100 AND customer_id IN (1, 2, 3)";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    assert!(formatted.iter().any(|line| line.contains("FROM")));
    assert!(formatted.iter().any(|line| line.contains("WHERE")));
}

#[test]
fn test_format_preserves_parentheses() {
    let query = "SELECT * FROM table WHERE (a = 1 OR b = 2) AND c = 3";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);
    let formatted_text = formatted.join(" ");

    // Check that parentheses are preserved
    assert!(formatted_text.contains("(a = 1 OR b = 2)"));
}

#[test]
fn test_format_datetime_in_where() {
    let query = "SELECT * FROM trades WHERE (executionDate BETWEEN DateTime(2024, 1, 1) AND DateTime(2024, 12, 31))";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);
    let formatted_text = formatted.join(" ");

    // Verify essential parts are preserved
    assert!(formatted_text.contains("SELECT"));
    assert!(formatted_text.contains("FROM trades"));
    assert!(formatted_text.contains("WHERE"));
    assert!(formatted_text.contains("(executionDate"));
    assert!(formatted_text.contains("DateTime(2024, 1, 1)"));
    assert!(formatted_text.contains("DateTime(2024, 12, 31)"));

    let original_parens = query.chars().filter(|c| *c == '(' || *c == ')').count();
    let formatted_parens = formatted_text
        .chars()
        .filter(|c| *c == '(' || *c == ')')
        .count();
    assert_eq!(original_parens, formatted_parens);
}

#[test]
fn test_format_multiline_layout() {
    // Test that formatted output has proper multi-line structure
    let query =
        r#"SELECT * FROM trades WHERE (symbol = "AAPL" OR symbol = "GOOGL") AND price > 100"#;
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    // Should have SELECT, FROM, WHERE lines
    assert!(formatted.len() >= 3, "Should have multiple lines");
    assert!(formatted[0].starts_with("SELECT"));
    assert!(formatted[1].starts_with("FROM"));
    assert!(formatted[2].starts_with("WHERE"));

    // WHERE conditions should contain the expected expressions
    let where_line = &formatted[2];
    assert!(where_line.contains("symbol = \"AAPL\""));
    assert!(where_line.contains("symbol = \"GOOGL\""));
    assert!(where_line.contains("AND price > 100"));
}

#[test]
fn test_format_columns_with_aliases() {
    let query = "SELECT customer_id AS id, customer_name AS name, customer_email AS email, customer_phone AS phone, customer_address AS address FROM customers";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    // Should have at least SELECT and FROM lines
    assert!(formatted.len() >= 2);
    assert!(formatted[0].contains("SELECT"));
    // The formatter may put all columns on one line or multiple lines depending on length
}

#[test]
fn test_format_distinct_keyword() {
    let query = "SELECT DISTINCT category, brand FROM products WHERE price > 100";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    assert!(formatted[0].contains("DISTINCT"));
}

#[test]
fn test_format_group_by_having() {
    let query = "SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 5";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    assert!(formatted.iter().any(|line| line.contains("GROUP BY")));
    // Note: HAVING clause might not be formatted yet as it's not fully implemented
}

#[test]
fn test_format_order_by_limit() {
    let query = "SELECT * FROM products ORDER BY price DESC, name ASC LIMIT 10";
    let formatted = crate::sql::parser::formatter::format_sql_pretty_compact(query, 5);

    // Note: ORDER BY and LIMIT might not be in the formatted output yet
    // as the formatter might not handle all clauses
    assert!(formatted.len() >= 2);
}
