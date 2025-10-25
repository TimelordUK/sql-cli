use unicode_width::UnicodeWidthStr;

/// Strip ANSI escape codes from a string and return the display width
/// This handles ANSI SGR (Select Graphic Rendition) codes like colors and styles
/// and correctly calculates Unicode character widths (e.g., emoji take 2 columns)
///
/// # Examples
///
/// ```
/// use sql_cli::utils::string_utils::display_width;
///
/// // Simple ASCII text
/// assert_eq!(display_width("hello"), 5);
///
/// // ANSI colored text (escape codes don't count toward width)
/// assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
///
/// // Unicode emoji (takes 2 columns)
/// assert_eq!(display_width("→"), 1);  // Right arrow
/// assert_eq!(display_width("⚡"), 2);  // Lightning bolt emoji
/// ```
pub fn display_width(s: &str) -> usize {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Check for ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                              // Skip until we find a letter (the command character)
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    // Use unicode-width to get the actual display width
    // This correctly handles:
    // - ASCII characters (width 1)
    // - Emoji and other wide characters (width 2)
    // - Zero-width characters (width 0)
    // - Combining characters
    result.width()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("test"), 4);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_display_width_ansi_codes() {
        // Red colored text
        assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
        // Bold text
        assert_eq!(display_width("\x1b[1mbold\x1b[0m"), 4);
        // RGB colored text
        assert_eq!(display_width("\x1b[38;2;255;0;0mRGB red\x1b[0m"), 7);
    }

    #[test]
    fn test_display_width_unicode() {
        // Single-width Unicode
        assert_eq!(display_width("→"), 1);
        assert_eq!(display_width("café"), 4);

        // Double-width characters (emoji)
        assert_eq!(display_width("⚡"), 2);
        assert_eq!(display_width("😀"), 2);
    }

    #[test]
    fn test_display_width_mixed() {
        // ANSI + Unicode
        assert_eq!(
            display_width("\x1b[32m→ CONSEC\x1b[0m"),
            8 // 1 (arrow) + 1 (space) + 6 (CONSEC)
        );
        assert_eq!(
            display_width("\x1b[31m⚡ REPEAT\x1b[0m"),
            9 // 2 (bolt) + 1 (space) + 6 (REPEAT)
        );
    }
}
