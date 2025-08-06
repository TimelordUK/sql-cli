use sql_cli::recursive_parser::{OrderByColumn, Parser, SortDirection};

#[test]
fn test_order_by_single_column_asc() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY price ASC");
    let stmt = parser.parse().expect("Should parse ORDER BY with ASC");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 1);
    assert_eq!(order_by[0].column, "price");
    assert!(matches!(order_by[0].direction, SortDirection::Asc));
}

#[test]
fn test_order_by_single_column_desc() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY price DESC");
    let stmt = parser.parse().expect("Should parse ORDER BY with DESC");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 1);
    assert_eq!(order_by[0].column, "price");
    assert!(matches!(order_by[0].direction, SortDirection::Desc));
}

#[test]
fn test_order_by_default_asc() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY price");
    let stmt = parser
        .parse()
        .expect("Should parse ORDER BY without direction");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 1);
    assert_eq!(order_by[0].column, "price");
    assert!(matches!(order_by[0].direction, SortDirection::Asc)); // Default is ASC
}

#[test]
fn test_order_by_multiple_columns() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY category DESC, price ASC, name");
    let stmt = parser
        .parse()
        .expect("Should parse multiple ORDER BY columns");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 3);

    assert_eq!(order_by[0].column, "category");
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    assert_eq!(order_by[1].column, "price");
    assert!(matches!(order_by[1].direction, SortDirection::Asc));

    assert_eq!(order_by[2].column, "name");
    assert!(matches!(order_by[2].direction, SortDirection::Asc)); // Default
}

#[test]
fn test_order_by_with_quoted_columns() {
    let mut parser =
        Parser::new(r#"SELECT * FROM customers ORDER BY "Customer Name" DESC, "Order Date" ASC"#);
    let stmt = parser
        .parse()
        .expect("Should parse ORDER BY with quoted columns");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 2);

    assert_eq!(order_by[0].column, "Customer Name");
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    assert_eq!(order_by[1].column, "Order Date");
    assert!(matches!(order_by[1].direction, SortDirection::Asc));
}

#[test]
fn test_order_by_with_where_clause() {
    let mut parser =
        Parser::new("SELECT * FROM customers WHERE price > 100 ORDER BY category DESC, price ASC");
    let stmt = parser
        .parse()
        .expect("Should parse WHERE and ORDER BY together");

    assert!(stmt.where_clause.is_some());
    assert!(stmt.order_by.is_some());

    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 2);

    assert_eq!(order_by[0].column, "category");
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    assert_eq!(order_by[1].column, "price");
    assert!(matches!(order_by[1].direction, SortDirection::Asc));
}

#[test]
fn test_order_by_numeric_columns() {
    // Test with numeric column names like in crime statistics CSV
    let mut parser = Parser::new("SELECT * FROM crime_stats ORDER BY 202204 DESC, 202205 ASC");
    let columns = vec![
        "Borough".to_string(),
        "202204".to_string(),
        "202205".to_string(),
    ];
    let mut parser = parser.with_columns(columns);

    let stmt = parser
        .parse()
        .expect("Should parse ORDER BY with numeric columns");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 2);

    assert_eq!(order_by[0].column, "202204");
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    assert_eq!(order_by[1].column, "202205");
    assert!(matches!(order_by[1].direction, SortDirection::Asc));
}
