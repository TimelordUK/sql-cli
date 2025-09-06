/// Proof of concept test for parsing SQL functions
/// This demonstrates that our parser architecture can easily support
/// function calls like ROUND(quantity * price, 2)
use sql_cli::sql::recursive_parser::{Parser, SelectItem};

#[test]
#[ignore] // Not yet implemented - this is a design test
fn test_parse_round_function() {
    // This is what we want to support:
    let query = "SELECT ROUND(quantity * price, 2) as rounded_total FROM products";

    // Expected AST structure:
    // SelectItem::Expression {
    //     expr: SqlExpression::FunctionCall {
    //         name: "ROUND",
    //         args: vec![
    //             SqlExpression::BinaryOp {
    //                 left: Column("quantity"),
    //                 op: "*",
    //                 right: Column("price")
    //             },
    //             SqlExpression::NumberLiteral("2")
    //         ]
    //     },
    //     alias: "rounded_total"
    // }

    // The parser changes needed are minimal:
    // 1. Recognize function names in parse_primary()
    // 2. Parse comma-separated argument list
    // 3. Arguments can be any expression (we already support this!)
}

#[test]
#[ignore] // Design test for GROUP BY
fn test_parse_group_by_with_aggregates() {
    // This is what we want to support:
    let query = "SELECT product_type, SUM(quantity * price) as revenue, COUNT(*) as transactions FROM sales GROUP BY product_type";

    // The parser already has group_by field in SelectStatement!
    // We just need to:
    // 1. Recognize aggregate function names (SUM, COUNT, etc.)
    // 2. Handle COUNT(*) special case
    // 3. The rest is query engine work
}

#[test]
#[ignore] // Design test for nested functions
fn test_parse_nested_functions() {
    // Even complex nesting would work with our recursive parser:
    let query = "SELECT ROUND(ABS(profit - loss) * 1.1, 2) as adjusted_net FROM trades";

    // Would parse as:
    // FunctionCall("ROUND", [
    //     BinaryOp(
    //         FunctionCall("ABS", [
    //             BinaryOp(Column("profit"), "-", Column("loss"))
    //         ]),
    //         "*",
    //         NumberLiteral("1.1")
    //     ),
    //     NumberLiteral("2")
    // ])

    // Our recursive descent parser handles this naturally!
}

#[test]
fn test_current_arithmetic_works() {
    // What we already support today:
    let query = "SELECT quantity * price + tax - discount as total FROM products";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok());

    let stmt = result.unwrap();
    assert_eq!(stmt.select_items.len(), 1);

    // Verify it parsed as an expression with the right alias
    match &stmt.select_items[0] {
        SelectItem::Expression { alias, .. } => {
            assert_eq!(alias, "total");
        }
        _ => panic!("Expected Expression SelectItem"),
    }
}
