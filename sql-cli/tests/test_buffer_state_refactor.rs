/// Test for Phase 1 of Buffer State Refactor
/// Verifies that ViewState is preserved when switching between buffers
/// using the new proxy-based architecture
use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{Buffer, BufferAPI, BufferManager, SelectionMode};

#[test]
fn test_buffer_state_preserved_on_switch() {
    // Create a BufferManager with two buffers
    let mut buffer_manager = BufferManager::new();

    // Add first buffer
    let mut buffer1 = Buffer::new(1);
    buffer1.name = "Buffer 1".to_string();
    buffer_manager.add_buffer(buffer1);

    // Add second buffer
    let mut buffer2 = Buffer::new(2);
    buffer2.name = "Buffer 2".to_string();
    buffer_manager.add_buffer(buffer2);

    // Create AppStateContainer
    let mut state = AppStateContainer::new(buffer_manager).unwrap();

    // Switch to first buffer
    state.buffers_mut().switch_to(0);

    // Set some state in buffer 1 using proxies
    {
        let mut nav_proxy = state.navigation_proxy_mut();
        nav_proxy.set_selected_row(10);
        nav_proxy.set_selected_column(5);
        nav_proxy.set_scroll_offset((20, 3));
        nav_proxy.set_viewport_lock(true);
    }

    {
        let mut sel_proxy = state.selection_proxy_mut();
        sel_proxy.set_mode(SelectionMode::Cell);
        sel_proxy.add_selected_cell((10, 5));
        sel_proxy.set_selection_anchor(Some((10, 5)));
    }

    // Switch to buffer 2
    state.buffers_mut().switch_to(1);

    // Set different state in buffer 2
    {
        let mut nav_proxy = state.navigation_proxy_mut();
        nav_proxy.set_selected_row(25);
        nav_proxy.set_selected_column(8);
        nav_proxy.set_scroll_offset((50, 10));
        nav_proxy.set_viewport_lock(false);
    }

    {
        let mut sel_proxy = state.selection_proxy_mut();
        sel_proxy.set_mode(SelectionMode::Row);
        sel_proxy.clear_selections();
    }

    // Switch back to buffer 1
    state.buffers_mut().switch_to(0);

    // Verify buffer 1's state was preserved
    {
        let nav_proxy = state.navigation_proxy();
        assert_eq!(
            nav_proxy.selected_row(),
            10,
            "Buffer 1 row should be preserved"
        );
        assert_eq!(
            nav_proxy.selected_column(),
            5,
            "Buffer 1 column should be preserved"
        );
        assert_eq!(
            nav_proxy.scroll_offset(),
            (20, 3),
            "Buffer 1 scroll offset should be preserved"
        );
        assert_eq!(
            nav_proxy.viewport_lock(),
            true,
            "Buffer 1 viewport lock should be preserved"
        );
    }

    {
        let sel_proxy = state.selection_proxy();
        assert_eq!(
            sel_proxy.mode(),
            SelectionMode::Cell,
            "Buffer 1 selection mode should be preserved"
        );
        assert_eq!(
            sel_proxy.selected_cells(),
            vec![(10, 5)],
            "Buffer 1 selected cells should be preserved"
        );
        assert_eq!(
            sel_proxy.selection_anchor(),
            Some((10, 5)),
            "Buffer 1 selection anchor should be preserved"
        );
    }

    // Switch to buffer 2 again
    state.buffers_mut().switch_to(1);

    // Verify buffer 2's state was preserved
    {
        let nav_proxy = state.navigation_proxy();
        assert_eq!(
            nav_proxy.selected_row(),
            25,
            "Buffer 2 row should be preserved"
        );
        assert_eq!(
            nav_proxy.selected_column(),
            8,
            "Buffer 2 column should be preserved"
        );
        assert_eq!(
            nav_proxy.scroll_offset(),
            (50, 10),
            "Buffer 2 scroll offset should be preserved"
        );
        assert_eq!(
            nav_proxy.viewport_lock(),
            false,
            "Buffer 2 viewport lock should be preserved"
        );
    }

    {
        let sel_proxy = state.selection_proxy();
        assert_eq!(
            sel_proxy.mode(),
            SelectionMode::Row,
            "Buffer 2 selection mode should be preserved"
        );
        assert_eq!(
            sel_proxy.selected_cells().len(),
            0,
            "Buffer 2 should have no selected cells"
        );
        assert_eq!(
            sel_proxy.selection_anchor(),
            None,
            "Buffer 2 should have no selection anchor"
        );
    }
}

#[test]
fn test_proxy_with_no_buffer() {
    // Create AppStateContainer with empty BufferManager using Default impl
    // This avoids file system access issues in tests
    let state = AppStateContainer::default();

    // Test that proxies return defaults when no buffer exists
    let nav_proxy = state.navigation_proxy();
    assert_eq!(nav_proxy.selected_row(), 0);
    assert_eq!(nav_proxy.selected_column(), 0);
    assert_eq!(nav_proxy.scroll_offset(), (0, 0));
    assert_eq!(nav_proxy.viewport_lock(), false);

    let sel_proxy = state.selection_proxy();
    assert_eq!(sel_proxy.mode(), SelectionMode::Row);
    assert_eq!(sel_proxy.selected_cells().len(), 0);
    assert_eq!(sel_proxy.selection_anchor(), None);
}

#[test]
fn test_direct_buffer_viewstate_access() {
    // Test that we can also access ViewState directly from Buffer
    let mut buffer = Buffer::new(1);

    // Modify ViewState directly
    buffer.view_state.crosshair_row = 15;
    buffer.view_state.crosshair_col = 7;
    buffer.view_state.selection_mode = SelectionMode::Column;
    buffer.view_state.viewport_lock = true;

    // Verify the changes
    assert_eq!(buffer.view_state.crosshair_row, 15);
    assert_eq!(buffer.view_state.crosshair_col, 7);
    assert_eq!(buffer.view_state.selection_mode, SelectionMode::Column);
    assert_eq!(buffer.view_state.viewport_lock, true);

    // Use the proper API method to set selected row (which syncs both ViewState and table_state)
    buffer.set_selected_row(Some(15));

    // Now verify through BufferAPI methods
    assert_eq!(buffer.get_selected_row(), Some(15));
    assert_eq!(buffer.get_current_column(), 7);
    assert_eq!(buffer.is_viewport_lock(), true);
}
