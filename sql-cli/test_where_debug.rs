use std::sync::Arc;
use sql_cli::data::datatable::{DataTable, DataColumn, DataRow, DataValue};
use sql_cli::data::query_engine::QueryEngine;

fn main() {

    // Create a simple test table
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("confirmationStatus"));
    table.add_column(DataColumn::new("amount"));

    // Add test data
    table.add_row(DataRow::new(vec![
        DataValue::Integer(1),
        DataValue::String("Pending".to_string()),
        DataValue::Float(100.50),
    ])).unwrap();
    
    table.add_row(DataRow::new(vec![
        DataValue::Integer(2),
        DataValue::String("Confirmed".to_string()),
        DataValue::Float(200.25),
    ])).unwrap();

    table.add_row(DataRow::new(vec![
        DataValue::Integer(3),
        DataValue::String("Pending".to_string()),
        DataValue::Float(150.00),
    ])).unwrap();

    let table = Arc::new(table);
    
    println!("Created table with {} rows", table.row_count());
    println!("Columns: {:?}", table.column_names());
    
    // Show some raw data
    for i in 0..table.row_count() {
        let status = table.get_value(i, 1);
        println!("Row {}: confirmationStatus = {:?}", i, status);
    }

    // Test the query
    let engine = QueryEngine::new();
    let sql = "SELECT * FROM data where confirmationStatus.Contains('pend')";
    println!("\nExecuting query: {}", sql);
    
    match engine.execute(table.clone(), sql) {
        Ok(view) => {
            println!("Query succeeded! Found {} rows", view.row_count());
            
            // Show the filtered data
            for i in 0..view.row_count() {
                println!("Filtered row {}: {:?}", i, view.get_row(i));
            }
        }
        Err(e) => {
            println!("Query failed: {}", e);
        }
    }
}