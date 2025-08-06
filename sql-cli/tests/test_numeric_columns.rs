use sql_cli::where_ast::{WhereExpr, WhereValue};
use sql_cli::where_parser::WhereParser;

#[test]
fn test_numeric_column_names() {
    // Test columns that are entirely numeric like "202204" for April 2022
    let columns = vec![
        "Borough".to_string(),
        "202202".to_string(),
        "202203".to_string(),
        "202204".to_string(),
        "202205".to_string(),
    ];

    // Test parsing with numeric column in WHERE clause
    let expr = WhereParser::parse_with_columns("202204 > 2.0", columns.clone())
        .expect("Should parse numeric column");

    // Verify it's parsed as a column comparison
    match expr {
        WhereExpr::GreaterThan(col, val) => {
            assert_eq!(col, "202204");
            assert_eq!(val, WhereValue::Number(2.0));
        }
        _ => panic!("Expected GreaterThan expression"),
    }

    // Test with AND expression
    let expr2 =
        WhereParser::parse_with_columns("Borough = \"London\" AND 202204 > 1.0", columns.clone())
            .expect("Should parse complex expression with numeric column");

    match expr2 {
        WhereExpr::And(left, right) => {
            // Check left side
            match &*left {
                WhereExpr::Equal(col, val) => {
                    assert_eq!(col, "Borough");
                    assert_eq!(val, &WhereValue::String("London".to_string()));
                }
                _ => panic!("Expected Equal expression on left"),
            }

            // Check right side
            match &*right {
                WhereExpr::GreaterThan(col, val) => {
                    assert_eq!(col, "202204");
                    assert_eq!(val, &WhereValue::Number(1.0));
                }
                _ => panic!("Expected GreaterThan expression on right"),
            }
        }
        _ => panic!("Expected And expression"),
    }

    // Test with numeric column in BETWEEN
    let expr3 = WhereParser::parse_with_columns("202204 BETWEEN 1.0 AND 5.0", columns.clone())
        .expect("Should parse BETWEEN with numeric column");

    match expr3 {
        WhereExpr::Between(col, min_val, max_val) => {
            assert_eq!(col, "202204");
            assert_eq!(min_val, WhereValue::Number(1.0));
            assert_eq!(max_val, WhereValue::Number(5.0));
        }
        _ => panic!("Expected Between expression"),
    }
}

#[test]
fn test_numeric_not_column() {
    // Test that numbers not in column list are still treated as numbers
    let columns = vec!["price".to_string(), "quantity".to_string()];

    let expr = WhereParser::parse_with_columns("price > 100", columns)
        .expect("Should parse regular numeric literal");

    match expr {
        WhereExpr::GreaterThan(col, val) => {
            assert_eq!(col, "price");
            assert_eq!(val, WhereValue::Number(100.0));
        }
        _ => panic!("Expected GreaterThan expression"),
    }
}
