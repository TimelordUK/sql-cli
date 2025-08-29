/// Test that ViewportManager is correctly synced with the current buffer
/// when loading multiple files
use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{Buffer, BufferAPI, BufferManager};
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::DataTable;
use std::sync::Arc;

#[test]
fn test_viewport_sync_after_multi_file_load() {
    // Create mock DataTables with distinct data
    let mut datatable1 = DataTable::new();
    datatable1.set_headers(vec!["id".to_string(), "name".to_string()]);
    datatable1.add_row(vec!["1".to_string(), "first_buffer".to_string()]);
    datatable1.add_row(vec!["2".to_string(), "first_buffer".to_string()]);
    let dataview1 = DataView::new(Arc::new(datatable1));

    let mut datatable2 = DataTable::new();
    datatable2.set_headers(vec!["code".to_string(), "desc".to_string()]);
    datatable2.add_row(vec!["A".to_string(), "second_buffer".to_string()]);
    datatable2.add_row(vec!["B".to_string(), "second_buffer".to_string()]);
    let dataview2 = DataView::new(Arc::new(datatable2));

    // Create a BufferManager and add buffers
    let mut buffer_manager = BufferManager::new();

    // Create first buffer with dataview1
    let mut buffer1 = Buffer::new(1);
    buffer1.set_dataview(Some(dataview1.clone()));
    buffer1.set_name("file1.csv".to_string());
    buffer_manager.add_buffer(buffer1);

    // Create second buffer with dataview2
    let mut buffer2 = Buffer::new(2);
    buffer2.set_dataview(Some(dataview2.clone()));
    buffer2.set_name("file2.csv".to_string());
    buffer_manager.add_buffer(buffer2);

    // Switch to buffer 2 (simulating what happens when adding files)
    buffer_manager.switch_to(1);
    assert_eq!(buffer_manager.current_index(), 1, "Should be on buffer 2");

    // Switch back to buffer 1 (simulating what run_enhanced_tui_multi does)
    buffer_manager.switch_to(0);
    assert_eq!(
        buffer_manager.current_index(),
        0,
        "Should be on buffer 1 after switch"
    );

    // Verify the current buffer has the correct dataview
    let current_buffer = buffer_manager.current().unwrap();
    let buffer_dataview = current_buffer.get_dataview().unwrap();

    assert_eq!(
        buffer_dataview.headers(),
        vec!["id", "name"],
        "Buffer 1 should have first dataview headers"
    );

    assert_eq!(
        buffer_dataview.row_count(),
        2,
        "Buffer 1 should have 2 rows"
    );
}

#[test]
fn test_buffer_switching_preserves_dataview() {
    // Create two DataTables with different data
    let mut datatable1 = DataTable::new();
    datatable1.set_headers(vec!["col1".to_string()]);
    datatable1.add_row(vec!["buffer1_data".to_string()]);
    let dataview1 = DataView::new(Arc::new(datatable1));

    let mut datatable2 = DataTable::new();
    datatable2.set_headers(vec!["col2".to_string()]);
    datatable2.add_row(vec!["buffer2_data".to_string()]);
    let dataview2 = DataView::new(Arc::new(datatable2));

    // Create BufferManager
    let mut buffer_manager = BufferManager::new();

    // Add both buffers
    let mut buffer1 = Buffer::new(1);
    buffer1.set_dataview(Some(dataview1.clone()));
    buffer1.set_name("test1.csv".to_string());
    buffer_manager.add_buffer(buffer1);

    let mut buffer2 = Buffer::new(2);
    buffer2.set_dataview(Some(dataview2.clone()));
    buffer2.set_name("test2.csv".to_string());
    buffer_manager.add_buffer(buffer2);

    // Switch between buffers multiple times and verify data is preserved
    for _ in 0..3 {
        // Switch to buffer 1
        buffer_manager.switch_to(0);
        let current = buffer_manager.current().unwrap();
        let dataview = current.get_dataview().unwrap();
        assert_eq!(
            dataview.headers(),
            vec!["col1"],
            "Buffer 1 should have col1 header"
        );

        // Switch to buffer 2
        buffer_manager.switch_to(1);
        let current = buffer_manager.current().unwrap();
        let dataview = current.get_dataview().unwrap();
        assert_eq!(
            dataview.headers(),
            vec!["col2"],
            "Buffer 2 should have col2 header"
        );
    }
}
