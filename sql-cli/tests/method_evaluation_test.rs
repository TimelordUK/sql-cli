use sql_cli::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use sql_cli::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::recursive_parser::{Parser, WhereClause};

fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("description"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("category"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Widget".to_string()),
            DataValue::String("A useful widget for various tasks".to_string()),
            DataValue::Float(19.99),
            DataValue::String("Tools".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Gadget".to_string()),
            DataValue::String("An innovative gadget device".to_string()),
            DataValue::Float(29.99),
            DataValue::String("Electronics".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Gizmo".to_string()),
            DataValue::String("A clever gizmo for entertainment".to_string()),
            DataValue::Float(9.99),
            DataValue::String("Toys".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::String("Device".to_string()),
            DataValue::String("Professional device for work".to_string()),
            DataValue::Float(99.99),
            DataValue::String("Electronics".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::String("Tool".to_string()),
            DataValue::String("Essential tool for projects".to_string()),
            DataValue::Float(49.99),
            DataValue::String("Tools".to_string()),
        ]))
        .unwrap();

    table
}

fn extract_where_clause(query: &str) -> WhereClause {
    let mut parser = Parser::new(query);
    let statement = parser.parse().expect("Failed to parse query");

    statement.where_clause.expect("Expected WHERE clause")
}

#[test]
fn test_contains_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test case-insensitive contains
    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.Contains('get')");

    // Should match "Widget" (true - contains 'get'), "Gadget" (true - contains 'get'), "Gizmo" (false), "Device" (false), "Tool" (false)
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // Widget - contains 'get' at the end
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget - contains 'get' in the middle
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo - does not contain 'get'
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // Device - does not contain 'get'
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool - does not contain 'get'
}

#[test]
fn test_startswith_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.StartsWith('G')");

    // Should match "Widget" (false), "Gadget" (true), "Gizmo" (true), "Device" (false), "Tool" (false)
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // Widget
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // Gizmo
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // Device
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool
}

#[test]
fn test_endswith_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.EndsWith('et')");

    // Should match "Widget" (true), "Gadget" (true), "Gizmo" (false), "Device" (false), "Tool" (false)
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // Widget
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // Device
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool
}

#[test]
fn test_length_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.Length() > 5");

    // Should match "Widget" (6, true), "Gadget" (6, true), "Gizmo" (5, false), "Device" (6, true), "Tool" (4, false)
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // Widget (6 chars)
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget (6 chars)
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo (5 chars)
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // Device (6 chars)
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool (4 chars)
}

#[test]
fn test_indexof_method_found() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf when substring is found
    let where_clause =
        extract_where_clause("SELECT * FROM test WHERE description.IndexOf('device') > 0");

    // "A useful widget for various tasks" - no "device" (-1)
    // "An innovative gadget device" - "device" at position 21
    // "A clever gizmo for entertainment" - no "device" (-1)
    // "Professional device for work" - "device" at position 13
    // "Essential tool for projects" - no "device" (-1)

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // Widget: -1 > 0 = false
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget: 21 > 0 = true
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo: -1 > 0 = false
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // Device: 13 > 0 = true
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool: -1 > 0 = false
}

#[test]
fn test_indexof_method_not_found() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf when substring is not found (returns -1)
    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.IndexOf('xyz') = -1");

    // None of the names contain "xyz", so all should return -1
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // Widget: -1 = -1
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget: -1 = -1
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // Gizmo: -1 = -1
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // Device: -1 = -1
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), true); // Tool: -1 = -1
}

#[test]
fn test_indexof_at_beginning() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test IndexOf when substring is at the beginning (returns 0)
    let where_clause = extract_where_clause("SELECT * FROM test WHERE name.IndexOf('Wid') = 0");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // Widget starts with "Wid"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // Gadget doesn't
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo doesn't
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // Device doesn't
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool doesn't
}

#[test]
fn test_numeric_column_with_string_methods() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test string methods on numeric columns (should convert to string)
    let where_clause = extract_where_clause("SELECT * FROM test WHERE price.Contains('9.99')");

    // Prices: 19.99, 29.99, 9.99, 99.99, 49.99
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // 19.99 contains "9.99"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // 29.99 contains "9.99"
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // 9.99 contains "9.99"
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // 99.99 contains "9.99"
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), true); // 49.99 contains "9.99"
}

#[test]
fn test_complex_expressions_with_methods() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test complex expression with multiple conditions
    let where_clause = extract_where_clause(
        "SELECT * FROM test WHERE name.Length() > 4 AND category.Contains('tron')",
    );

    // name.Length() > 4: Widget(6), Gadget(6), Gizmo(5), Device(6), Tool(4)
    // category.Contains('tron'): Tools(false), Electronics(true), Toys(false), Electronics(true), Tools(false)
    // Combined: false, true, false, true, false
    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // Widget & Tools
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // Gadget & Electronics
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // Gizmo & Toys
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), true); // Device & Electronics
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // Tool & Tools
}

#[test]
fn test_indexof_with_greater_than() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test the specific case from the user's debug output
    let where_clause =
        extract_where_clause("SELECT * FROM test WHERE description.IndexOf('ful') > 2");

    // "A useful widget for various tasks" - "ful" at position 4 (case-insensitive)
    // "An innovative gadget device" - no "ful" (-1)
    // "A clever gizmo for entertainment" - no "ful" (-1)
    // "Professional device for work" - no "ful" (-1)
    // "Essential tool for projects" - no "ful" (-1)

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // 4 > 2 = true
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // -1 > 2 = false
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // -1 > 2 = false
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // -1 > 2 = false
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // -1 > 2 = false
}

#[test]
fn test_case_sensitivity() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test that methods are case-insensitive for search strings
    let where_clause1 = extract_where_clause("SELECT * FROM test WHERE name.Contains('WIDGET')");
    let where_clause2 = extract_where_clause("SELECT * FROM test WHERE name.Contains('widget')");
    let where_clause3 = extract_where_clause("SELECT * FROM test WHERE name.Contains('WiDgEt')");

    // All should match the same row (Widget)
    assert_eq!(evaluator.evaluate(&where_clause1, 0).unwrap(), true);
    assert_eq!(evaluator.evaluate(&where_clause2, 0).unwrap(), true);
    assert_eq!(evaluator.evaluate(&where_clause3, 0).unwrap(), true);
}
