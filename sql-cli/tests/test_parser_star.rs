use sql_cli::sql::recursive_parser::{Parser, SelectItem};

#[test]
fn test_parse_star_comma_expression() {
    let query = "SELECT *, quantity * price as total FROM test_table";
    let mut parser = Parser::new(query);

    match parser.parse() {
        Ok(stmt) => {
            println!("Select items: {:?}", stmt.select_items);
            println!("Columns (legacy): {:?}", stmt.columns);

            // Should have 2 select items: Star and Expression
            assert_eq!(stmt.select_items.len(), 2, "Expected 2 select items");

            // First should be Star
            assert!(matches!(stmt.select_items[0], SelectItem::Star));

            // Second should be Expression with alias "total"
            match &stmt.select_items[1] {
                SelectItem::Expression { alias, .. } => {
                    assert_eq!(alias, "total");
                }
                _ => panic!("Expected Expression, got {:?}", stmt.select_items[1]),
            }
        }
        Err(e) => {
            panic!("Parser failed: {}", e);
        }
    }
}

#[test]
fn test_parse_expression_comma_star() {
    let query = "SELECT quantity * price as total, * FROM test_table";
    let mut parser = Parser::new(query);

    match parser.parse() {
        Ok(stmt) => {
            println!("Select items: {:?}", stmt.select_items);

            // Should have 2 select items: Expression and Star
            assert_eq!(stmt.select_items.len(), 2, "Expected 2 select items");

            // First should be Expression
            match &stmt.select_items[0] {
                SelectItem::Expression { alias, .. } => {
                    assert_eq!(alias, "total");
                }
                _ => panic!("Expected Expression, got {:?}", stmt.select_items[0]),
            }

            // Second should be Star
            assert!(matches!(stmt.select_items[1], SelectItem::Star));
        }
        Err(e) => {
            panic!("Parser failed: {}", e);
        }
    }
}
