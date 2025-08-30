use sql_cli::buffer::{Buffer, BufferAPI};
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::services::{QueryExecutionService, QueryOrchestrator};
use sql_cli::app_state_container::AppStateContainer;
use std::sync::Arc;

fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test_data");
    
    // Add 5 columns
    table.add_column(DataColumn::new("a"));
    table.add_column(DataColumn::new("b"));
    table.add_column(DataColumn::new("c"));
    table.add_column(DataColumn::new("d"));
    table.add_column(DataColumn::new("e"));
    
    // Add test rows
    table.add_row(DataRow::new(vec![
        DataValue::Integer(1),
        DataValue::Integer(2),
        DataValue::Integer(3),
        DataValue::Integer(4),
        DataValue::Integer(5),
    ])).unwrap();
    
    table.add_row(DataRow::new(vec![
        DataValue::Integer(6),
        DataValue::Integer(7),
        DataValue::Integer(8),
        DataValue::Integer(9),
        DataValue::Integer(10),
    ])).unwrap();
    
    table
}

fn main() {
    // Create a buffer with test data
    let table = create_test_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(table));
    
    println!("Initial setup:");
    println!("  DataTable columns: {:?}", buffer.get_datatable().unwrap().column_names());
    println!("  Original source columns: {:?}", buffer.get_original_source().unwrap().column_names());
    
    // Create query execution service
    let service = QueryExecutionService::new(false, false);
    
    // Execute a query with computed columns
    let dataview = buffer.get_dataview();
    let original_for_query = buffer.get_original_source();
    
    println!("\nExecuting: SELECT a, b * 2 as double_b FROM test_data");
    let result = service.execute(
        "SELECT a, b * 2 as double_b FROM test_data",
        dataview,
        original_for_query
    ).unwrap();
    
    println!("Query result:");
    println!("  Result columns: {:?}", result.dataview.column_names());
    println!("  Result source columns: {:?}", result.dataview.source().column_names());
    
    // Simulate what happens when we set this result back
    buffer.set_dataview(Some(result.dataview));
    
    println!("\nAfter setting query result:");
    println!("  DataTable columns: {:?}", buffer.get_datatable().unwrap().column_names());
    println!("  DataView columns: {:?}", buffer.get_dataview().unwrap().column_names());
    println!("  DataView source columns: {:?}", buffer.get_dataview().unwrap().source().column_names());
    println!("  Original source columns: {:?}", buffer.get_original_source().unwrap().column_names());
    
    // Now execute a SELECT * query - it should use the original source
    println!("\nExecuting: SELECT * FROM test_data");
    let result2 = service.execute(
        "SELECT * FROM test_data",
        buffer.get_dataview(),
        buffer.get_original_source()
    ).unwrap();
    
    println!("Query result:");
    println!("  Result columns: {:?}", result2.dataview.column_names());
    println!("  Result source columns: {:?}", result2.dataview.source().column_names());
}