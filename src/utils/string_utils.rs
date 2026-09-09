use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

/// Truncate a string so that its display width is at most `max_width` columns,
/// appending an ASCII ellipsis (`...`) when anything was cut.
///
/// Like [`display_width`], this skips ANSI SGR escape sequences when measuring
/// (they are copied through, so colours survive truncation) and uses Unicode
/// display width, so it never splits a multi-byte character or a wide glyph.
/// The returned string is guaranteed to satisfy
/// `display_width(&result) <= max_width`, which is what table renderers rely on
/// to keep every row the same width as the border.
///
/// # Examples
///
/// ```
/// use sql_cli::utils::string_utils::{display_width, truncate_to_width};
///
/// assert_eq!(truncate_to_width("hello", 10), "hello");
/// assert_eq!(truncate_to_width("hello world", 8), "hello...");
/// assert_eq!(display_width(&truncate_to_width("a very long value", 6)), 6);
///
/// // Never splits a wide character
/// assert_eq!(truncate_to_width("⚡⚡⚡", 3), "⚡");
/// ```
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    if display_width(s) <= max_width {
        return s.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    // Reserve room for the ellipsis, unless the budget is too small to bother.
    const ELLIPSIS: &str = "...";
    let (budget, ellipsis) = if max_width > ELLIPSIS.len() {
        (max_width - ELLIPSIS.len(), ELLIPSIS)
    } else {
        (max_width, "")
    };

    let mut out = String::new();
    let mut used = 0usize;
    let mut had_ansi = false;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            // Copy the escape sequence through without charging it to the budget
            had_ansi = true;
            out.push(ch);
            out.push('[');
            chars.next();
            for next_ch in chars.by_ref() {
                out.push(next_ch);
                if next_ch.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        let ch_width = ch.width().unwrap_or(0);
        if used + ch_width > budget {
            break;
        }
        out.push(ch);
        used += ch_width;
    }

    if had_ansi {
        out.push_str("\x1b[0m");
    }
    out.push_str(ellipsis);
    out
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

    #[test]
    fn test_truncate_shorter_than_max_is_unchanged() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("hello", 5), "hello");
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn test_truncate_appends_ellipsis() {
        assert_eq!(truncate_to_width("hello world", 8), "hello...");
        assert_eq!(
            truncate_to_width("xTrader2 / Trading Services / Pricing", 20),
            "xTrader2 / Tradin..."
        );
    }

    #[test]
    fn test_truncate_never_exceeds_max_width() {
        let long = "xTrader2 / Trading Services / Pricing / Analytics / master";
        for max in 0..=60 {
            let truncated = truncate_to_width(long, max);
            assert!(
                display_width(&truncated) <= max,
                "width {} exceeded max {} for {:?}",
                display_width(&truncated),
                max,
                truncated
            );
        }
    }

    #[test]
    fn test_truncate_tiny_budgets_drop_the_ellipsis() {
        assert_eq!(truncate_to_width("hello", 0), "");
        assert_eq!(truncate_to_width("hello", 1), "h");
        assert_eq!(truncate_to_width("hello", 3), "hel");
        assert_eq!(truncate_to_width("hello", 4), "h...");
    }

    #[test]
    fn test_truncate_respects_unicode_boundaries() {
        // Wide characters are never split in half
        assert_eq!(truncate_to_width("\u{26a1}\u{26a1}\u{26a1}", 3), "\u{26a1}");
        assert_eq!(display_width(&truncate_to_width("caf\u{e9} au lait", 6)), 6);
    }

    #[test]
    fn test_truncate_preserves_ansi_and_ignores_it_for_width() {
        let colored = "\x1b[31mhello world\x1b[0m";
        let truncated = truncate_to_width(colored, 8);
        assert_eq!(display_width(&truncated), 8);
        assert!(truncated.starts_with("\x1b[31m"));
        assert!(truncated.ends_with("..."));
    }
}
