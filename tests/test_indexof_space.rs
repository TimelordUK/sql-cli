use sql_cli::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use sql_cli::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::recursive_parser::Parser;

fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("book"));

    // Add test data
    // Row 0: "derivatives" - no space
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("derivatives".to_string()),
        ]))
        .unwrap();

    // Row 1: "equity trading" - space at position 6
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("equity trading".to_string()),
        ]))
        .unwrap();

    // Row 2: " leading" - space at position 0
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String(" leading".to_string()),
        ]))
        .unwrap();

    // Row 3: "trailing " - space at position 8
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::String("trailing ".to_string()),
        ]))
        .unwrap();

    // Row 4: "FX" - no space
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::String("FX".to_string()),
        ]))
        .unwrap();

    table
}

#[test]
fn test_indexof_space_at_zero() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf(' ') = 0 - should only match strings with space at position 0
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') = 0");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "derivatives" has no space
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // "equity trading" has space at position 6
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // " leading" has space at position 0
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "trailing " has space at position 8
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "FX" has no space
}

#[test]
fn test_indexof_space_not_found() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf(' ') = -1 - should only match strings without spaces
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') = -1");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // "derivatives" has no space
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // "equity trading" has space
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // " leading" has space
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "trailing " has space
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), true); // "FX" has no space
}

#[test]
fn test_indexof_space_greater_than_zero() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf(' ') > 0 - should match strings with space not at beginning
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') > 0");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "derivatives" has no space (-1 > 0 = false)
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // "equity trading" has space at position 6 (6 > 0 = true)
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // " leading" has space at position 0 (0 > 0 = false)
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // "trailing " has space at position 8 (8 > 0 = true)
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "FX" has no space (-1 > 0 = false)
}

#[test]
fn test_indexof_greater_or_equal_zero() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf(' ') >= 0 - should match all strings WITH spaces
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') >= 0");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "derivatives" has no space (-1 >= 0 = false)
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // "equity trading" has space at position 6 (6 >= 0 = true)
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // " leading" has space at position 0 (0 >= 0 = true)
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // "trailing " has space at position 8 (8 >= 0 = true)
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "FX" has no space (-1 >= 0 = false)
}

#[test]
fn test_indexof_specific_positions() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test specific positions
    // "equity trading" has space at position 6
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') = 6");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "derivatives"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // "equity trading" - space at 6
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // " leading" - space at 0
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "trailing " - space at 8
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "FX"
}
