use sql_cli::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use sql_cli::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::recursive_parser::{Parser, WhereClause};

fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("book"));
    table.add_column(DataColumn::new("description"));

    // Add test data with various whitespace patterns
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("  derivatives  ".to_string()), // spaces both sides
            DataValue::String("with spaces".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("  equity trading".to_string()), // leading spaces only
            DataValue::String("no trim needed".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("FX  ".to_string()), // trailing spaces only
            DataValue::String("clean".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::String("bonds".to_string()), // no spaces
            DataValue::String("already clean".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::String("   ".to_string()), // only spaces
            DataValue::String("empty after trim".to_string()),
        ]))
        .unwrap();

    table
}

#[test]
fn test_trim_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test Trim() = 'derivatives' - should match row 0 after trimming
    let mut parser = Parser::new("SELECT * FROM test WHERE book.Trim() = 'derivatives'");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), true); // "  derivatives  " -> "derivatives"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // "  equity trading" -> "equity trading"
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // "FX  " -> "FX"
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "bonds" -> "bonds"
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "   " -> ""
}

#[test]
fn test_trimstart_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test TrimStart() = 'equity trading' - should match row 1 after trimming start
    let mut parser = Parser::new("SELECT * FROM test WHERE book.TrimStart() = 'equity trading'");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "  derivatives  " -> "derivatives  "
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true); // "  equity trading" -> "equity trading"
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // "FX  " -> "FX  "
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "bonds" -> "bonds"
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "   " -> ""
}

#[test]
fn test_trimend_method() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test TrimEnd() = 'FX' - should match row 2 after trimming end
    let mut parser = Parser::new("SELECT * FROM test WHERE book.TrimEnd() = 'FX'");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "  derivatives  " -> "  derivatives"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // "  equity trading" -> "  equity trading"
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), true); // "FX  " -> "FX"
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "bonds" -> "bonds"
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), false); // "   " -> "   " (TrimEnd leaves leading spaces)
}

#[test]
fn test_trim_empty_string() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test Trim() = '' - should match row 4 which has only spaces
    let mut parser = Parser::new("SELECT * FROM test WHERE book.Trim() = ''");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    assert_eq!(evaluator.evaluate(&where_clause, 0).unwrap(), false); // "  derivatives  " -> "derivatives"
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), false); // "  equity trading" -> "equity trading"
    assert_eq!(evaluator.evaluate(&where_clause, 2).unwrap(), false); // "FX  " -> "FX"
    assert_eq!(evaluator.evaluate(&where_clause, 3).unwrap(), false); // "bonds" -> "bonds"
    assert_eq!(evaluator.evaluate(&where_clause, 4).unwrap(), true); // "   " -> ""
}

#[test]
fn test_trim_with_startswith() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Test combining Trim() with StartsWith
    let mut parser = Parser::new("SELECT * FROM test WHERE book.Trim().StartsWith('equity')");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    // This should fail for now as we don't support chained methods yet
    // But it demonstrates the intention for future enhancement

    // For now, we'd need to use: book.Trim() = 'equity trading' AND book.StartsWith('equity')
}

#[test]
fn test_trim_preserves_internal_spaces() {
    let table = create_test_table();
    let evaluator = RecursiveWhereEvaluator::new(&table);

    // Verify that Trim only removes leading/trailing spaces, not internal ones
    let mut parser = Parser::new("SELECT * FROM test WHERE book.TrimStart() = 'equity trading'");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");

    // "  equity trading" should become "equity trading" (with space preserved between words)
    assert_eq!(evaluator.evaluate(&where_clause, 1).unwrap(), true);
}
