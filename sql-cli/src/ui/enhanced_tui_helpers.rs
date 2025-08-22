// Helper functions extracted from enhanced_tui.rs
// These are pure functions with no dependencies on self

use anyhow::Result;

/// Sanitize table name by removing special characters and limiting length
pub fn sanitize_table_name(name: &str) -> String {
    // Replace spaces and other problematic characters with underscores
    // to create SQL-friendly table names
    // Examples: "Business Crime Borough Level" -> "Business_Crime_Borough_Level"
    let sanitized: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // If the sanitized name is too complex (too long or has too many underscores),
    // fall back to a simple default name
    const MAX_LENGTH: usize = 30;
    const MAX_UNDERSCORES: usize = 5;

    let underscore_count = sanitized.chars().filter(|&c| c == '_').count();

    if sanitized.len() > MAX_LENGTH || underscore_count > MAX_UNDERSCORES {
        // Use a simple fallback name
        "data".to_string()
    } else if sanitized.is_empty() || sanitized.chars().all(|c| c == '_') {
        // If the name is empty or all underscores after sanitization
        "data".to_string()
    } else {
        sanitized
    }
}
