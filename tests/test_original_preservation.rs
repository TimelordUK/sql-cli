use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{Buffer, BufferAPI};
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::services::{QueryExecutionService, QueryOrchestrator};
use sql_cli::ui::search::vim_search_adapter::VimSearchAdapter;
use std::cell::RefCell;
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
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(3),
            DataValue::Integer(4),
            DataValue::Integer(5),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(6),
            DataValue::Integer(7),
            DataValue::Integer(8),
            DataValue::Integer(9),
            DataValue::Integer(10),
        ]))
        .unwrap();

    table
}

#[test]
fn test_original_preserved_after_computed_query() {
    // Create a buffer with test data
    let table = create_test_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(Arc::new(table)));

    // Verify original source is preserved
    let original = buffer.get_original_source().unwrap();
    assert_eq!(original.column_count(), 5, "Original should have 5 columns");
    assert_eq!(original.column_names(), vec!["a", "b", "c", "d", "e"]);

    // Create query execution service
    let service = QueryExecutionService::new(false, false);

    // Execute a query with computed columns
    let dataview = buffer.get_dataview();
    let original_for_query = buffer.get_original_source();

    let result = service
        .execute(
            "SELECT a, b * 2 as double_b FROM test_data",
            dataview,
            original_for_query,
        )
        .unwrap();

    // The result should have only 2 columns
    assert_eq!(result.dataview.column_count(), 2);
    assert_eq!(result.dataview.column_names(), vec!["a", "double_b"]);

    // But the original source in the buffer should still have 5 columns
    let original_after = buffer.get_original_source().unwrap();
    assert_eq!(
        original_after.column_count(),
        5,
        "Original should still have 5 columns after query"
    );
    assert_eq!(original_after.column_names(), vec!["a", "b", "c", "d", "e"]);

    // Now execute a SELECT * query - it should use the original source
    let result2 = service
        .execute(
            "SELECT * FROM test_data",
            Some(&result.dataview),
            original_for_query,
        )
        .unwrap();

    // Should get all 5 original columns back
    assert_eq!(
        result2.dataview.column_count(),
        5,
        "SELECT * should return all 5 original columns"
    );
    assert_eq!(
        result2.dataview.column_names(),
        vec!["a", "b", "c", "d", "e"]
    );
}

#[test]
#[ignore] // Requires history file infrastructure
fn test_orchestrator_preserves_original() {
    use sql_cli::buffer::BufferManager;

    // Create BufferManager
    let mut buffers = BufferManager::new();

    // Create a buffer with test data
    let table = create_test_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(Arc::new(table.clone())));

    // Add buffer to manager
    buffers.add_buffer(buffer);

    let mut state_container = AppStateContainer::new(buffers).unwrap();
    let vim_adapter = RefCell::new(VimSearchAdapter::new());

    // Create orchestrator
    let mut orchestrator = QueryOrchestrator::new(false, false);

    // Execute a computed query
    let ctx = orchestrator
        .execute_query(
            "SELECT a, a + b as sum_ab FROM test_data",
            &mut state_container,
            &vim_adapter,
        )
        .unwrap();

    // Result should have computed columns
    assert_eq!(ctx.result.dataview.column_count(), 2);
    assert_eq!(ctx.result.dataview.column_names(), vec!["a", "sum_ab"]);

    // But original source should still be intact
    let original = state_container.get_original_source().unwrap();
    assert_eq!(
        original.column_count(),
        5,
        "Original should still have 5 columns"
    );

    // Execute SELECT * - should get original columns
    let ctx2 = orchestrator
        .execute_query(
            "SELECT * FROM test_data",
            &mut state_container,
            &vim_adapter,
        )
        .unwrap();

    assert_eq!(
        ctx2.result.dataview.column_count(),
        5,
        "SELECT * should return all original columns"
    );
    assert_eq!(
        ctx2.result.dataview.column_names(),
        vec!["a", "b", "c", "d", "e"]
    );
}
