use serde_json::{json, Value};
use sql_cli::where_ast::{evaluate_where_expr, WhereExpr};
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
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), true);
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
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), true);
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
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), true);
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
    assert_eq!(evaluate_where_expr(&expr, &data).unwrap(), true);
}
