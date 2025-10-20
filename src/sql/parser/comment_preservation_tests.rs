//! Tests for comment preservation functionality (Phase 3)

use crate::sql::parser::ast_formatter::format_select_statement;
use crate::sql::recursive_parser::{Parser, ParserMode};

#[test]
fn test_standard_mode_skips_comments() {
    let sql = r#"-- This is a comment
SELECT id, name FROM users"#;

    let mut parser = Parser::new(sql);
    let stmt = parser.parse().expect("Should parse successfully");

    // Standard mode should have empty comment fields
    assert_eq!(
        stmt.leading_comments.len(),
        0,
        "Standard mode should skip comments"
    );
    assert!(
        stmt.trailing_comment.is_none(),
        "Standard mode should skip trailing comments"
    );
}

#[test]
fn test_preserve_mode_captures_leading_comments() {
    let sql = r#"-- This is an important query
-- It demonstrates comment preservation
SELECT id, name FROM users"#;

    let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser.parse().expect("Should parse successfully");

    // PreserveComments mode should capture leading comments
    assert_eq!(
        stmt.leading_comments.len(),
        2,
        "Should capture 2 leading comments"
    );
    assert_eq!(
        stmt.leading_comments[0].text.trim(),
        "This is an important query"
    );
    assert_eq!(
        stmt.leading_comments[1].text.trim(),
        "It demonstrates comment preservation"
    );
    assert!(stmt.leading_comments[0].is_line_comment);
    assert!(stmt.leading_comments[1].is_line_comment);
}

#[test]
fn test_formatter_emits_comments() {
    let sql = r#"-- Important query
SELECT id, name FROM users"#;

    let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser.parse().expect("Should parse successfully");

    let formatted = format_select_statement(&stmt);

    // Formatted output should contain the comment
    assert!(
        formatted.contains("-- Important query"),
        "Formatted SQL should contain comment"
    );
    assert!(
        formatted.contains("SELECT"),
        "Formatted SQL should contain SELECT"
    );
}

#[test]
fn test_full_round_trip_with_comments() {
    let sql = r#"-- User data query
-- Fetch active users only
SELECT
    id,
    name,
    email
FROM users
WHERE active = true"#;

    // Parse with comment preservation
    let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser.parse().expect("Should parse successfully");

    // Format back to SQL
    let formatted = format_select_statement(&stmt);

    // Verify comments are present in output
    assert!(
        formatted.contains("-- User data query"),
        "Should preserve first comment"
    );
    assert!(
        formatted.contains("-- Fetch active users only"),
        "Should preserve second comment"
    );

    // Verify SQL structure is correct
    assert!(formatted.contains("SELECT"), "Should have SELECT");
    assert!(formatted.contains("FROM users"), "Should have FROM clause");
    assert!(formatted.contains("WHERE"), "Should have WHERE clause");
}

#[test]
fn test_backward_compatibility() {
    // Test that existing code using Parser::new() continues to work
    let sql = "-- Comment\nSELECT * FROM test";

    let mut parser = Parser::new(sql);
    let stmt = parser.parse().expect("Should parse successfully");

    // Should work exactly as before - no comments
    assert_eq!(stmt.leading_comments.len(), 0);

    let formatted = format_select_statement(&stmt);
    // Should not contain comment in standard mode
    assert!(!formatted.contains("-- Comment"));
}

#[test]
fn test_block_comment_preservation() {
    let sql = r#"/* This is a block comment */
SELECT id FROM users"#;

    let mut parser = Parser::with_mode(sql, ParserMode::PreserveComments);
    let stmt = parser.parse().expect("Should parse successfully");

    assert_eq!(stmt.leading_comments.len(), 1);
    assert!(
        !stmt.leading_comments[0].is_line_comment,
        "Should be a block comment"
    );
    assert_eq!(
        stmt.leading_comments[0].text.trim(),
        "This is a block comment"
    );

    let formatted = format_select_statement(&stmt);
    assert!(
        formatted.contains("/*"),
        "Should contain block comment start"
    );
    assert!(formatted.contains("*/"), "Should contain block comment end");
}
