use sql_cli::recursive_parser::Parser;

#[test]
fn test_between_simple() {
    let mut parser = Parser::new("SELECT * FROM table WHERE price BETWEEN 50 AND 100");
    let stmt = parser.parse().expect("Should parse BETWEEN");

    assert!(stmt.where_clause.is_some());
    // The query should parse successfully
}

#[test]
fn test_between_in_parentheses() {
    let mut parser = Parser::new("SELECT * FROM table WHERE (price BETWEEN 50 AND 100)");
    let stmt = parser.parse().expect("Should parse BETWEEN in parentheses");

    assert!(stmt.where_clause.is_some());
    // The query should parse successfully
}

#[test]
fn test_between_with_or() {
    let query = "SELECT * FROM test_sorting WHERE (Price BETWEEN 50 AND 100) OR (Product.Length() > 5) ORDER BY Category ASC, price DESC";
    let mut parser = Parser::new(query);
    let stmt = parser
        .parse()
        .expect("Should parse complex query with BETWEEN and OR");

    assert!(stmt.where_clause.is_some());
    assert!(stmt.order_by.is_some());

    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 2);
    assert_eq!(order_by[0].column, "Category");
    assert_eq!(order_by[1].column, "price");
}

#[test]
fn test_between_with_and() {
    let query = "SELECT * FROM table WHERE category = 'Books' AND price BETWEEN 10 AND 50";
    let mut parser = Parser::new(query);
    let stmt = parser.parse().expect("Should parse BETWEEN with AND");

    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_multiple_between() {
    let query =
        "SELECT * FROM table WHERE (price BETWEEN 10 AND 50) AND (quantity BETWEEN 5 AND 20)";
    let mut parser = Parser::new(query);
    let stmt = parser
        .parse()
        .expect("Should parse multiple BETWEEN clauses");

    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_between_with_expressions() {
    // Test BETWEEN with column names and numeric literals
    let query = "SELECT * FROM table WHERE total BETWEEN min_value AND max_value";
    let mut parser = Parser::new(query);
    let stmt = parser
        .parse()
        .expect("Should parse BETWEEN with column references");

    assert!(stmt.where_clause.is_some());
}
