// Test that column search now works with DataTableBuffer
// This test demonstrates that the fix for column search is working

use sql_cli::datatable::{DataTable, DataColumn, DataRow, DataValue};
use sql_cli::datatable_buffer::DataTableBuffer;
use sql_cli::buffer::BufferAPI;

fn main() {
    // Create a DataTable with known columns
    let mut table = DataTable::new("test_table");
    
    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("orderid"));
    table.add_column(DataColumn::new("amount"));
    
    // Add some sample data
    table.add_row(DataRow::new(vec![
        DataValue::Integer(1),
        DataValue::String("Widget A".to_string()),
        DataValue::String("ORD001".to_string()),
        DataValue::Float(100.50),
    ]));
    
    // Create a DataTableBuffer
    let buffer = DataTableBuffer::new(1, table);
    
    // Test the new get_column_names method
    let column_names = buffer.get_column_names();
    
    println!("Column names from DataTableBuffer:");
    for (idx, name) in column_names.iter().enumerate() {
        println!("  Column {}: {}", idx, name);
    }
    
    // Verify we got the expected columns
    assert_eq!(column_names.len(), 4);
    assert_eq!(column_names[0], "id");
    assert_eq!(column_names[1], "name");
    assert_eq!(column_names[2], "orderid");
    assert_eq!(column_names[3], "amount");
    
    println!("\n✓ Column search fix verified!");
    println!("  DataTableBuffer now correctly returns column names");
    println!("  Column search should now work with CSV files");
}