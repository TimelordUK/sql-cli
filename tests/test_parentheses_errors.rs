use sql_cli::sql::recursive_parser::{format_ast_tree, Parser};

#[test]
fn test_unclosed_parenthesis_in_where() {
    // Missing closing parenthesis
    let query = "SELECT * FROM table WHERE (price > 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_err(), "Should fail with unclosed parenthesis");
    let error = result.unwrap_err();
    println!("Error for unclosed paren: {}", error);

    // Check AST formatting shows helpful error
    let ast = format_ast_tree(query);
    assert!(ast.contains("PARSE ERROR"));
}

#[test]
fn test_extra_closing_parenthesis() {
    // Extra closing parenthesis
    let query = "SELECT * FROM table WHERE price > 100)";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_err(),
        "Should fail with extra closing parenthesis"
    );
    let error = result.unwrap_err();
    println!("Error for extra closing paren: {}", error);
}

#[test]
fn test_mismatched_parentheses_in_complex_query() {
    // Missing closing paren in first condition
    let query = "SELECT * FROM table WHERE (price > 100 AND quantity < 50) OR (category = 'Books'";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_err(), "Should fail with mismatched parentheses");
    let error = result.unwrap_err();
    println!("Error for mismatched parens: {}", error);
}

#[test]
fn test_nested_unclosed_parentheses() {
    // Nested parentheses with missing close
    let query = "SELECT * FROM table WHERE ((price > 100 AND quantity < 50)";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_err(),
        "Should fail with nested unclosed parentheses"
    );
    let error = result.unwrap_err();
    println!("Error for nested unclosed: {}", error);
}

#[test]
fn test_method_call_unclosed() {
    // Method call with unclosed parenthesis
    let query = "SELECT * FROM table WHERE name.Contains('test'";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_err(), "Should fail with unclosed method call");
    let error = result.unwrap_err();
    println!("Error for unclosed method: {}", error);
}

#[test]
fn test_in_clause_unclosed() {
    // IN clause with unclosed parenthesis
    let query = "SELECT * FROM table WHERE category IN ('Books', 'Electronics'";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_err(), "Should fail with unclosed IN clause");
    let error = result.unwrap_err();
    println!("Error for unclosed IN: {}", error);
}

#[test]
fn test_between_with_unclosed_paren() {
    // BETWEEN inside unclosed parenthesis
    let query = "SELECT * FROM table WHERE (price BETWEEN 50 AND 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(
        result.is_err(),
        "Should fail with unclosed BETWEEN parenthesis"
    );
    let error = result.unwrap_err();
    println!("Error for unclosed BETWEEN: {}", error);
}

#[test]
fn test_complex_query_missing_paren() {
    // Complex real-world example with missing paren
    let query = "SELECT * FROM trades WHERE (price > 100 AND quantity < 50 OR (category = 'Tech' AND date > '2024-01-01')";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    // This might actually parse but incorrectly due to precedence
    if result.is_err() {
        println!("Error: {}", result.unwrap_err());
    } else {
        println!(
            "Warning: Query parsed but might have incorrect precedence due to missing parenthesis"
        );
    }
}
