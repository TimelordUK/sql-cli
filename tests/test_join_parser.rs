// Test file for JOIN parser functionality
use sql_cli::sql::recursive_parser::{JoinOperator, JoinType, Parser, SqlExpression, TableSource};

// Helper function to extract column name from SqlExpression
fn get_column_name(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(col_ref) => {
            if let Some(table_prefix) = &col_ref.table_prefix {
                format!("{}.{}", table_prefix, col_ref.name)
            } else {
                col_ref.name.clone()
            }
        }
        _ => panic!("Expected column reference, got: {:?}", expr),
    }
}

#[test]
fn test_simple_inner_join() {
    let query = "SELECT * FROM users JOIN orders ON users.id = orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.from_table, Some("users".to_string()));
    assert_eq!(stmt.joins.len(), 1);

    let join = &stmt.joins[0];
    assert_eq!(join.join_type, JoinType::Inner);
    assert!(matches!(&join.table, TableSource::Table(name) if name == "orders"));
    assert_eq!(join.condition.conditions.len(), 1);
    assert_eq!(
        get_column_name(&join.condition.conditions[0].left_expr),
        "users.id"
    );
    assert!(matches!(
        join.condition.conditions[0].operator,
        JoinOperator::Equal
    ));
    assert_eq!(
        get_column_name(&join.condition.conditions[0].right_expr),
        "orders.user_id"
    );
}

#[test]
fn test_left_join() {
    let query = "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    let join = &stmt.joins[0];
    assert_eq!(join.join_type, JoinType::Left);
}

#[test]
fn test_right_join() {
    let query = "SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    let join = &stmt.joins[0];
    assert_eq!(join.join_type, JoinType::Right);
}

#[test]
fn test_full_outer_join() {
    let query = "SELECT * FROM users FULL OUTER JOIN orders ON users.id = orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    let join = &stmt.joins[0];
    assert_eq!(join.join_type, JoinType::Full);
}

#[test]
fn test_cross_join() {
    let query = "SELECT * FROM users CROSS JOIN products";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    let join = &stmt.joins[0];
    assert_eq!(join.join_type, JoinType::Cross);
    // CROSS JOIN doesn't have ON condition
}

#[test]
fn test_multiple_joins() {
    let query = "SELECT * FROM users 
                 JOIN orders ON users.id = orders.user_id 
                 JOIN products ON orders.product_id = products.id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 2);

    // First join
    assert_eq!(stmt.joins[0].join_type, JoinType::Inner);
    assert!(matches!(&stmt.joins[0].table, TableSource::Table(name) if name == "orders"));

    // Second join
    assert_eq!(stmt.joins[1].join_type, JoinType::Inner);
    assert!(matches!(&stmt.joins[1].table, TableSource::Table(name) if name == "products"));
}

#[test]
fn test_join_with_table_alias() {
    let query = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.from_alias, Some("u".to_string()));
    assert_eq!(stmt.joins[0].alias, Some("o".to_string()));
    assert_eq!(stmt.joins[0].condition.conditions.len(), 1);
    assert_eq!(
        get_column_name(&stmt.joins[0].condition.conditions[0].left_expr),
        "u.id"
    );
    assert_eq!(
        get_column_name(&stmt.joins[0].condition.conditions[0].right_expr),
        "o.user_id"
    );
}

#[test]
fn test_join_with_cte() {
    let query = "WITH active_users AS (SELECT * FROM users WHERE active = 1)
                 SELECT * FROM active_users JOIN orders ON active_users.id = orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.ctes.len(), 1);
    assert_eq!(stmt.ctes[0].name, "active_users");
    assert_eq!(stmt.from_table, Some("active_users".to_string()));
    assert_eq!(stmt.joins.len(), 1);
}

// TODO: This test is currently disabled due to a parser limitation
// The parser incorrectly interprets "orders.total" in the WHERE clause as a method call
// rather than a table-qualified column reference. This requires implementing proper
// context tracking to distinguish between table.column and object.method() syntax.
#[test]
#[ignore] // Remove when table context tracking is implemented
fn test_join_with_where_clause() {
    let query =
        "SELECT * FROM users JOIN orders ON users.id = orders.user_id WHERE orders.total > 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    if let Err(ref e) = result {
        eprintln!("Parse error in test_join_with_where_clause: {}", e);
    }
    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    assert!(stmt.where_clause.is_some());
}

#[test]
fn test_join_with_subquery() {
    let query = "SELECT * FROM users JOIN (SELECT * FROM orders WHERE total > 100) o ON users.id = o.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();

    assert_eq!(stmt.joins.len(), 1);
    let join = &stmt.joins[0];

    // Check that the joined table is a subquery
    match &join.table {
        TableSource::DerivedTable { query, alias } => {
            assert_eq!(alias, "o");
            assert!(query.where_clause.is_some());
        }
        _ => panic!("Expected DerivedTable for subquery join"),
    }
}

#[test]
fn test_join_with_different_operators() {
    // Test greater than
    let query = "SELECT * FROM users JOIN orders ON users.created > orders.created";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(matches!(
        stmt.joins[0].condition.conditions[0].operator,
        JoinOperator::GreaterThan
    ));

    // Test less than
    let query = "SELECT * FROM users JOIN orders ON users.id < orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(matches!(
        stmt.joins[0].condition.conditions[0].operator,
        JoinOperator::LessThan
    ));

    // Test not equal
    let query = "SELECT * FROM users JOIN orders ON users.id != orders.user_id";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok());
    let stmt = result.unwrap();
    assert!(matches!(
        stmt.joins[0].condition.conditions[0].operator,
        JoinOperator::NotEqual
    ));
}
