// Quick unit test for IndexOf behavior
use sql_cli::datatable::{DataTable, DataValue, DataRow, DataColumn};
use sql_cli::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use sql_cli::recursive_parser::{Parser, WhereClause};

fn main() {
    // Create a simple test table
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("book"));
    
    // Add test data
    table.add_row(DataRow::new(vec![
        DataValue::Integer(1),
        DataValue::String("derivatives".to_string()),
    ])).unwrap();
    
    table.add_row(DataRow::new(vec![
        DataValue::Integer(2),
        DataValue::String("equity trading".to_string()),
    ])).unwrap();
    
    table.add_row(DataRow::new(vec![
        DataValue::Integer(3),
        DataValue::String(" leading".to_string()),
    ])).unwrap();
    
    table.add_row(DataRow::new(vec![
        DataValue::Integer(4),
        DataValue::String("trailing ".to_string()),
    ])).unwrap();
    
    // Test IndexOf(' ') = 0
    println!("Testing: book.IndexOf(' ') = 0");
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') = 0");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");
    let evaluator = RecursiveWhereEvaluator::new(&table);
    
    println!("Row 0 'derivatives': {}", evaluator.evaluate(&where_clause, 0).unwrap());
    println!("Row 1 'equity trading': {}", evaluator.evaluate(&where_clause, 1).unwrap());
    println!("Row 2 ' leading': {}", evaluator.evaluate(&where_clause, 2).unwrap());
    println!("Row 3 'trailing ': {}", evaluator.evaluate(&where_clause, 3).unwrap());
    
    // Test IndexOf(' ') = -1
    println!("\nTesting: book.IndexOf(' ') = -1");
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') = -1");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");
    
    println!("Row 0 'derivatives': {}", evaluator.evaluate(&where_clause, 0).unwrap());
    println!("Row 1 'equity trading': {}", evaluator.evaluate(&where_clause, 1).unwrap());
    println!("Row 2 ' leading': {}", evaluator.evaluate(&where_clause, 2).unwrap());
    println!("Row 3 'trailing ': {}", evaluator.evaluate(&where_clause, 3).unwrap());
    
    // Test IndexOf(' ') > 0
    println!("\nTesting: book.IndexOf(' ') > 0");
    let mut parser = Parser::new("SELECT * FROM test WHERE book.IndexOf(' ') > 0");
    let statement = parser.parse().expect("Failed to parse");
    let where_clause = statement.where_clause.expect("Expected WHERE clause");
    
    println!("Row 0 'derivatives': {}", evaluator.evaluate(&where_clause, 0).unwrap());
    println!("Row 1 'equity trading': {}", evaluator.evaluate(&where_clause, 1).unwrap());
    println!("Row 2 ' leading': {}", evaluator.evaluate(&where_clause, 2).unwrap());
    println!("Row 3 'trailing ': {}", evaluator.evaluate(&where_clause, 3).unwrap());
}