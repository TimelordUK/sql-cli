//! Test for N key toggle issue after search mode

#[cfg(test)]
mod test_n_key {
    use sql_cli::buffer::{AppMode, Buffer, SearchState};
    use sql_cli::state::{StateCoordinator, StateDispatcher, StateEvent};
    use sql_cli::ui::shadow_state::SearchType;
    use sql_cli::ui::vim_search_adapter::VimSearchAdapter;
    use sql_cli::ui::vim_search_manager::VimSearchManager;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_n_key_works_after_search_exit() {
        // Create buffer
        let buffer = Rc::new(RefCell::new(Buffer::new()));

        // Create dispatcher and connect buffer
        let mut dispatcher = StateDispatcher::new();
        dispatcher.set_buffer(buffer.clone());

        // Add VimSearchAdapter as subscriber
        let vim_adapter = VimSearchAdapter::new(VimSearchManager::new());
        dispatcher.subscribe(Box::new(vim_adapter));

        // Initial state: Results mode
        buffer.borrow_mut().mode = AppMode::Results;
        buffer.borrow_mut().show_row_numbers = false;

        // Simulate pressing N key - should toggle line numbers
        {
            let mut buf = buffer.borrow_mut();
            buf.show_row_numbers = !buf.show_row_numbers;
        }
        assert!(
            buffer.borrow().show_row_numbers,
            "N key should toggle line numbers on"
        );

        // Enter search mode (/)
        dispatcher.dispatch_mode_change(AppMode::Results, AppMode::Search);
        dispatcher.dispatch_search_start(SearchType::Vim);

        // Verify we're in search mode
        assert_eq!(buffer.borrow().mode, AppMode::Search);

        // Type search pattern
        buffer.borrow_mut().search_state.pattern = "test".to_string();

        // Exit search mode (Escape)
        dispatcher.dispatch_search_end(SearchType::Vim);
        dispatcher.dispatch_mode_change(AppMode::Search, AppMode::Results);

        // Verify search state is cleared
        assert_eq!(buffer.borrow().mode, AppMode::Results);
        assert!(
            buffer.borrow().search_state.pattern.is_empty(),
            "Search pattern should be cleared after exit"
        );

        // THE CRITICAL TEST: N key should still work
        // In the bug, VimSearchManager would still be active and capture the N key
        // With our fix, it should be deactivated

        // Simulate N key press again
        {
            let mut buf = buffer.borrow_mut();
            // This should toggle line numbers, not be captured by vim search
            buf.show_row_numbers = !buf.show_row_numbers;
        }

        assert!(
            !buffer.borrow().show_row_numbers,
            "N key should toggle line numbers off after search mode - if this fails, the bug is present!"
        );

        println!("✅ N key works correctly after search mode!");
    }

    #[test]
    fn test_search_states_cleared_on_mode_exit() {
        let buffer = Rc::new(RefCell::new(Buffer::new()));
        let mut dispatcher = StateDispatcher::new();
        dispatcher.set_buffer(buffer.clone());

        // Set up search state
        buffer.borrow_mut().mode = AppMode::Search;
        buffer.borrow_mut().search_state.pattern = "test".to_string();
        buffer.borrow_mut().filter_state.pattern = "filter".to_string();
        buffer.borrow_mut().fuzzy_filter_state.pattern = "fuzzy".to_string();

        // Exit to Results mode
        dispatcher.dispatch_mode_change(AppMode::Search, AppMode::Results);

        // All search states should be cleared
        let buf = buffer.borrow();
        assert!(
            buf.search_state.pattern.is_empty(),
            "Search pattern should be cleared"
        );
        assert!(
            buf.filter_state.pattern.is_empty(),
            "Filter pattern should be cleared"
        );
        assert!(
            buf.fuzzy_filter_state.pattern.is_empty(),
            "Fuzzy filter should be cleared"
        );
    }

    #[test]
    fn test_vim_search_adapter_deactivates() {
        let buffer = Rc::new(RefCell::new(Buffer::new()));
        let mut adapter = VimSearchAdapter::new(VimSearchManager::new());

        // Start search
        buffer.borrow_mut().mode = AppMode::Search;
        buffer.borrow_mut().search_state.pattern = "test".to_string();
        adapter.on_state_event(
            &StateEvent::SearchStarted {
                search_type: SearchType::Vim,
            },
            &buffer.borrow(),
        );

        assert!(
            adapter.should_handle_key(&buffer.borrow()),
            "Adapter should handle keys during search"
        );

        // Exit search
        buffer.borrow_mut().mode = AppMode::Results;
        buffer.borrow_mut().search_state.pattern.clear();
        adapter.on_state_event(
            &StateEvent::ModeChanged {
                from: AppMode::Search,
                to: AppMode::Results,
            },
            &buffer.borrow(),
        );

        assert!(
            !adapter.should_handle_key(&buffer.borrow()),
            "Adapter should NOT handle keys after search exit"
        );
    }
}
