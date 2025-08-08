use serde_json::{json, Value};
use sql_cli::where_ast::{evaluate_where_expr, evaluate_where_expr_with_options, WhereExpr};
use sql_cli::where_parser::WhereParser;

#[test]
fn test_case_insensitive_contains() {
    let data = json!({
        "name": "John Doe",
        "email": "JOHN@EXAMPLE.COM"
    });

    // Case-sensitive (default)
    let expr = WhereParser::parse_with_options(
        "name.Contains('john')",
        vec!["name".to_string(), "email".to_string()],
        false,
    )
    .unwrap();
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), false);

    // Case-insensitive
    let expr = WhereParser::parse_with_options(
        "name.Contains('john')",
        vec!["name".to_string(), "email".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );
}

#[test]
fn test_case_insensitive_starts_with() {
    let data = json!({
        "company": "Microsoft Corporation"
    });

    // Case-sensitive
    let expr = WhereParser::parse_with_options(
        "company.StartsWith('micro')",
        vec!["company".to_string()],
        false,
    )
    .unwrap();
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), false);

    // Case-insensitive
    let expr = WhereParser::parse_with_options(
        "company.StartsWith('micro')",
        vec!["company".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );
}

#[test]
fn test_case_insensitive_ends_with() {
    let data = json!({
        "filename": "Document.PDF"
    });

    // Case-sensitive
    let expr = WhereParser::parse_with_options(
        "filename.EndsWith('.pdf')",
        vec!["filename".to_string()],
        false,
    )
    .unwrap();
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), false);

    // Case-insensitive
    let expr = WhereParser::parse_with_options(
        "filename.EndsWith('.pdf')",
        vec!["filename".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );
}

#[test]
fn test_case_insensitive_equality() {
    // Test case for the exact issue reported with confirmationStatus = 'pending'
    let data = json!({
        "confirmationStatus": "Pending",  // Note: capital P
        "status": "active"
    });

    // Case-sensitive equality (should fail with 'pending')
    let expr = WhereParser::parse_with_options(
        "confirmationStatus = 'pending'",
        vec!["confirmationStatus".to_string(), "status".to_string()],
        false,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, false).unwrap(),
        false
    );

    // Case-insensitive equality (should succeed with 'pending')
    let expr = WhereParser::parse_with_options(
        "confirmationStatus = 'pending'",
        vec!["confirmationStatus".to_string(), "status".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );

    // Case-insensitive equality with exact case (should also succeed)
    let expr = WhereParser::parse_with_options(
        "confirmationStatus = 'Pending'",
        vec!["confirmationStatus".to_string(), "status".to_string()],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );
}

#[test]
fn test_case_insensitive_not_equal() {
    let data = json!({
        "status": "Active"
    });

    // Case-sensitive not equal
    let expr =
        WhereParser::parse_with_options("status != 'active'", vec!["status".to_string()], false)
            .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, false).unwrap(),
        true
    );

    // Case-insensitive not equal (should be false since 'Active' equals 'active' ignoring case)
    let expr =
        WhereParser::parse_with_options("status != 'active'", vec!["status".to_string()], true)
            .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        false
    );
}

#[test]
fn test_case_insensitive_complex_query() {
    let data = json!({
        "first_name": "JARED",
        "last_name": "Smith",
        "email": "jared@company.COM"
    });

    // Complex query with multiple conditions
    let expr = WhereParser::parse_with_options(
        "first_name.Contains('jared') OR email.EndsWith('.com')",
        vec![
            "first_name".to_string(),
            "last_name".to_string(),
            "email".to_string(),
        ],
        true,
    )
    .unwrap();
    assert_eq!(
        evaluate_where_expr_with_options(&expr, &data, true).unwrap(),
        true
    );
}
