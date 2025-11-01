use sql_cli::sql::parser::ast::SqlExpression;
use sql_cli::sql::recursive_parser::{Parser, SortDirection};

#[test]
fn test_order_by_single_column_asc() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY price ASC");
    let stmt = parser.parse().expect("Should parse ORDER BY with ASC");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 1);
    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "price");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[0].direction, SortDirection::Asc));
}

#[test]
fn test_order_by_single_column_desc() {
    let mut parser = Parser::new("SELECT * FROM customers ORDER BY price DESC");
    let stmt = parser.parse().expect("Should parse ORDER BY with DESC");

    assert!(stmt.order_by.is_some());
    let order_by = stmt.order_by.unwrap();
    assert_eq!(order_by.len(), 1);
    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "price");
    } else {
        panic!("Expected Column expression");
    }
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
    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "price");
    } else {
        panic!("Expected Column expression");
    }
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

    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "category");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    if let SqlExpression::Column(col_ref) = &order_by[1].expr {
        assert_eq!(col_ref.name, "price");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[1].direction, SortDirection::Asc));

    if let SqlExpression::Column(col_ref) = &order_by[2].expr {
        assert_eq!(col_ref.name, "name");
    } else {
        panic!("Expected Column expression");
    }
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

    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "Customer Name");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    if let SqlExpression::Column(col_ref) = &order_by[1].expr {
        assert_eq!(col_ref.name, "Order Date");
    } else {
        panic!("Expected Column expression");
    }
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

    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "category");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    if let SqlExpression::Column(col_ref) = &order_by[1].expr {
        assert_eq!(col_ref.name, "price");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[1].direction, SortDirection::Asc));
}

#[test]
fn test_order_by_numeric_columns() {
    // Test with numeric column names like in crime statistics CSV
    let parser = Parser::new("SELECT * FROM crime_stats ORDER BY 202204 DESC, 202205 ASC");
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

    if let SqlExpression::Column(col_ref) = &order_by[0].expr {
        assert_eq!(col_ref.name, "202204");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[0].direction, SortDirection::Desc));

    if let SqlExpression::Column(col_ref) = &order_by[1].expr {
        assert_eq!(col_ref.name, "202205");
    } else {
        panic!("Expected Column expression");
    }
    assert!(matches!(order_by[1].direction, SortDirection::Asc));
}
