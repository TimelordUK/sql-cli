/// Scroll and cursor position utilities extracted from enhanced_tui
/// Contains scroll offset calculations and cursor position management

/// Calculate horizontal scroll offset for input field
pub fn calculate_horizontal_scroll(cursor_pos: usize, terminal_width: u16) -> u16 {
    let inner_width = terminal_width.saturating_sub(3) as usize; // Account for borders + padding

    if cursor_pos >= inner_width {
        // Cursor is beyond visible area, scroll to show it
        (cursor_pos + 1).saturating_sub(inner_width) as u16
    } else {
        0
    }
}

/// Update scroll offset based on cursor position
pub fn update_scroll_offset(cursor_pos: usize, current_offset: u16, visible_width: usize) -> u16 {
    // If cursor is before the scroll window, scroll left
    if cursor_pos < current_offset as usize {
        cursor_pos as u16
    }
    // If cursor is after the scroll window, scroll right
    else if cursor_pos >= current_offset as usize + visible_width {
        (cursor_pos + 1).saturating_sub(visible_width) as u16
    }
    // Otherwise keep current offset
    else {
        current_offset
    }
}

/// Calculate visible range for viewport
pub fn calculate_visible_range(
    total_items: usize,
    viewport_start: usize,
    viewport_size: usize,
) -> (usize, usize) {
    let start = viewport_start.min(total_items);
    let end = (viewport_start + viewport_size).min(total_items);
    (start, end)
}

/// Calculate page jump offset
pub fn calculate_page_jump(
    current_pos: usize,
    page_size: usize,
    total_items: usize,
    direction_up: bool,
) -> usize {
    if direction_up {
        current_pos.saturating_sub(page_size)
    } else {
        (current_pos + page_size).min(total_items.saturating_sub(1))
    }
}

/// Calculate centered viewport position
/// Centers the given position in the viewport if possible
pub fn calculate_centered_viewport(
    target_pos: usize,
    viewport_size: usize,
    total_items: usize,
) -> usize {
    if total_items <= viewport_size {
        // Everything fits, start at 0
        0
    } else if target_pos < viewport_size / 2 {
        // Target is near the beginning
        0
    } else if target_pos > total_items.saturating_sub(viewport_size / 2) {
        // Target is near the end
        total_items.saturating_sub(viewport_size)
    } else {
        // Center the target
        target_pos.saturating_sub(viewport_size / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_scroll() {
        // No scroll needed
        assert_eq!(calculate_horizontal_scroll(5, 80), 0);

        // Scroll needed
        assert_eq!(calculate_horizontal_scroll(100, 80), 24); // 100 + 1 - 77
    }

    #[test]
    fn test_update_scroll_offset() {
        // Cursor before window - scroll left
        assert_eq!(update_scroll_offset(5, 10, 20), 5);

        // Cursor in window - no change
        assert_eq!(update_scroll_offset(15, 10, 20), 10);

        // Cursor after window - scroll right
        assert_eq!(update_scroll_offset(35, 10, 20), 16); // 35 + 1 - 20
    }

    #[test]
    fn test_visible_range() {
        assert_eq!(calculate_visible_range(100, 10, 20), (10, 30));
        assert_eq!(calculate_visible_range(100, 90, 20), (90, 100));
        assert_eq!(calculate_visible_range(5, 0, 10), (0, 5));
    }

    #[test]
    fn test_page_jump() {
        // Jump up
        assert_eq!(calculate_page_jump(50, 10, 100, true), 40);
        assert_eq!(calculate_page_jump(5, 10, 100, true), 0);

        // Jump down
        assert_eq!(calculate_page_jump(50, 10, 100, false), 60);
        assert_eq!(calculate_page_jump(95, 10, 100, false), 99);
    }

    #[test]
    fn test_centered_viewport() {
        // Everything fits
        assert_eq!(calculate_centered_viewport(5, 20, 10), 0);

        // Near beginning
        assert_eq!(calculate_centered_viewport(5, 20, 100), 0);

        // Near end
        assert_eq!(calculate_centered_viewport(95, 20, 100), 80);

        // Middle - centered
        assert_eq!(calculate_centered_viewport(50, 20, 100), 40);
    }
}
