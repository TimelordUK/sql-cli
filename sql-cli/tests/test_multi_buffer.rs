use anyhow::Result;
use sql_cli::buffer::{Buffer, BufferAPI, BufferManager};

#[test]
fn test_buffer_manager_creation() -> Result<()> {
    let mut manager = BufferManager::new();

    // Add first buffer
    let buffer1 = Buffer::new(0);
    manager.add_buffer(buffer1);

    assert_eq!(manager.current_index(), 0);
    assert_eq!(manager.all_buffers().len(), 1);
    assert!(!manager.has_multiple());

    // Add second buffer
    let buffer2 = Buffer::new(0);
    manager.add_buffer(buffer2);

    assert_eq!(manager.current_index(), 1); // Should switch to new buffer
    assert_eq!(manager.all_buffers().len(), 2);
    assert!(manager.has_multiple());

    Ok(())
}

#[test]
fn test_buffer_navigation() -> Result<()> {
    let mut manager = BufferManager::new();

    // Create 3 buffers
    for i in 0..3 {
        let mut buffer = Buffer::new(0);
        buffer.set_input_text(format!("Buffer {}", i));
        manager.add_buffer(buffer);
    }

    assert_eq!(manager.current_index(), 2); // Should be at last buffer

    // Test next_buffer
    manager.next_buffer();
    assert_eq!(manager.current_index(), 0); // Should wrap to first

    manager.next_buffer();
    assert_eq!(manager.current_index(), 1);

    // Test prev_buffer
    manager.prev_buffer();
    assert_eq!(manager.current_index(), 0);

    manager.prev_buffer();
    assert_eq!(manager.current_index(), 2); // Should wrap to last

    // Test switch_to
    manager.switch_to(1);
    assert_eq!(manager.current_index(), 1);

    // Test invalid switch_to
    manager.switch_to(10); // Invalid index
    assert_eq!(manager.current_index(), 1); // Should stay at current

    Ok(())
}

#[test]
fn test_buffer_close() -> Result<()> {
    let mut manager = BufferManager::new();

    // Create 3 buffers with different content
    for i in 0..3 {
        let mut buffer = Buffer::new(0);
        buffer.set_input_text(format!("Buffer {}", i));
        manager.add_buffer(buffer);
    }

    // We're at buffer 2 (index 2)
    assert_eq!(manager.current_index(), 2);

    // Close current buffer
    assert!(manager.close_current());
    assert_eq!(manager.all_buffers().len(), 2);
    assert_eq!(manager.current_index(), 1); // Should adjust index

    // Content should be Buffer 1 now
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "Buffer 1");
    }

    // Close another buffer
    assert!(manager.close_current());
    assert_eq!(manager.all_buffers().len(), 1);
    assert_eq!(manager.current_index(), 0);

    // Should not be able to close last buffer
    assert!(!manager.close_current());
    assert_eq!(manager.all_buffers().len(), 1);

    Ok(())
}

#[test]
fn test_buffer_independence() -> Result<()> {
    let mut manager = BufferManager::new();

    // Create two buffers
    let mut buffer1 = Buffer::new(0);
    buffer1.set_input_text("SELECT * FROM users".to_string());
    buffer1.set_input_cursor_position(6);
    manager.add_buffer(buffer1);

    let mut buffer2 = Buffer::new(0);
    buffer2.set_input_text("INSERT INTO products".to_string());
    buffer2.set_input_cursor_position(11);
    manager.add_buffer(buffer2);

    // We're at buffer 2 now
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "INSERT INTO products");
        assert_eq!(buffer.get_input_cursor_position(), 11);
    }

    // Switch to buffer 1
    manager.switch_to(0);
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "SELECT * FROM users");
        assert_eq!(buffer.get_input_cursor_position(), 6);
    }

    // Modify buffer 1
    if let Some(buffer) = manager.current_mut() {
        buffer.set_input_text("SELECT * FROM customers".to_string());
    }

    // Switch to buffer 2 and verify it's unchanged
    manager.switch_to(1);
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "INSERT INTO products");
    }

    // Switch back to buffer 1 and verify changes
    manager.switch_to(0);
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "SELECT * FROM customers");
    }

    Ok(())
}

#[test]
fn test_buffer_undo_independence() -> Result<()> {
    let mut manager = BufferManager::new();

    // Create two buffers with undo history
    let mut buffer1 = Buffer::new(0);
    buffer1.set_input_text("A".to_string());
    buffer1.save_state_for_undo();
    buffer1.set_input_text("AB".to_string());
    buffer1.save_state_for_undo();
    buffer1.set_input_text("ABC".to_string());
    manager.add_buffer(buffer1);

    let mut buffer2 = Buffer::new(0);
    buffer2.set_input_text("X".to_string());
    buffer2.save_state_for_undo();
    buffer2.set_input_text("XY".to_string());
    buffer2.save_state_for_undo();
    buffer2.set_input_text("XYZ".to_string());
    manager.add_buffer(buffer2);

    // Undo in buffer 2
    if let Some(buffer) = manager.current_mut() {
        buffer.perform_undo();
        assert_eq!(buffer.get_input_text(), "XY");
    }

    // Switch to buffer 1 - should have its own undo history
    manager.switch_to(0);
    if let Some(buffer) = manager.current_mut() {
        assert_eq!(buffer.get_input_text(), "ABC");
        buffer.perform_undo();
        assert_eq!(buffer.get_input_text(), "AB");
        buffer.perform_undo();
        assert_eq!(buffer.get_input_text(), "A");
    }

    // Switch back to buffer 2 - should maintain its state
    manager.switch_to(1);
    if let Some(buffer) = manager.current() {
        assert_eq!(buffer.get_input_text(), "XY");
    }

    Ok(())
}
