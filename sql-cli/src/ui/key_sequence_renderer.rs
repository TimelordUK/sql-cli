use std::collections::VecDeque;
use std::time::Instant;

/// Represents a key sequence that might be repeated
#[derive(Debug, Clone)]
struct KeySequence {
    key: String,
    count: usize,
    first_press: Instant,
    last_press: Instant,
}

/// Smart key sequence renderer that:
/// - Collapses repeated keys (jjj -> 3j)
/// - Limits display to last N sequences
/// - Shows chord completions
/// - Handles timeout/fading
pub struct KeySequenceRenderer {
    /// Recent key sequences
    sequences: VecDeque<KeySequence>,
    /// Maximum number of sequences to display
    max_display: usize,
    /// Time window for collapsing repeated keys (ms)
    collapse_window_ms: u64,
    /// Whether we're in chord mode with available completions
    chord_mode: Option<String>,
    /// Total fade time (ms)
    fade_duration_ms: u64,
    /// Enabled state
    enabled: bool,
}

impl KeySequenceRenderer {
    pub fn new() -> Self {
        Self {
            sequences: VecDeque::with_capacity(10),
            max_display: 5,
            collapse_window_ms: 500, // Keys pressed within 500ms are considered "rapid"
            chord_mode: None,
            fade_duration_ms: 2000,
            enabled: true, // Enable by default for better debugging
        }
    }

    /// Enable or disable the renderer
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.sequences.clear();
            self.chord_mode = None;
        }
    }

    /// Record a key press
    pub fn record_key(&mut self, key: String) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();

        // Check if this is a repeat of the last key
        if let Some(last) = self.sequences.back_mut() {
            if last.key == key
                && last.last_press.elapsed().as_millis() < self.collapse_window_ms as u128
            {
                // It's a rapid repeat - increment count
                last.count += 1;
                last.last_press = now;
                return;
            }
        }

        // Not a repeat, add new sequence
        self.sequences.push_back(KeySequence {
            key,
            count: 1,
            first_press: now,
            last_press: now,
        });

        // Clean up old sequences
        self.cleanup_sequences();
    }

    /// Set chord mode with available completions
    pub fn set_chord_mode(&mut self, description: &str) {
        self.chord_mode = Some(description.to_string());
    }

    /// Clear chord mode
    pub fn clear_chord_mode(&mut self) {
        self.chord_mode = None;
    }

    /// Get the display string for the status line
    pub fn get_display(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        // If in chord mode, show that with priority
        if let Some(ref chord_desc) = self.chord_mode {
            return self.format_chord_display(chord_desc);
        }

        // Otherwise show recent key sequences
        self.format_sequence_display()
    }

    /// Format chord mode display (e.g., "y(a,c,q,v)")
    fn format_chord_display(&self, description: &str) -> String {
        // Parse special yank mode format
        if description.starts_with("Yank mode:") {
            // Extract just the key options
            if let Some(options) = description.strip_prefix("Yank mode: ") {
                // Convert "y=row, c=column, a=all, ESC=cancel" to "y(y,c,a,v)"
                let keys: Vec<&str> = options
                    .split(", ")
                    .filter_map(|part| {
                        let key = part.split('=').next()?;
                        if key == "ESC" {
                            None // Skip ESC in display
                        } else {
                            Some(key)
                        }
                    })
                    .collect();

                if !keys.is_empty() {
                    return format!("y({})", keys.join(","));
                }
            }
        }

        // For other chord modes, show simplified format
        if description.contains("Waiting for:") {
            // Extract the waiting keys
            if let Some(waiting) = description.strip_prefix("Waiting for: ") {
                let parts: Vec<&str> = waiting
                    .split(", ")
                    .map(|p| p.split(" → ").next().unwrap_or(p))
                    .collect();
                if !parts.is_empty() && self.sequences.back().is_some() {
                    if let Some(last) = self.sequences.back() {
                        return format!("{}({})", last.key, parts.join(","));
                    }
                }
            }
        }

        // Default: show the description as-is but shortened
        if description.len() > 20 {
            format!("{}...", &description[..17])
        } else {
            description.to_string()
        }
    }

    /// Format the sequence display
    fn format_sequence_display(&self) -> String {
        let now = Instant::now();
        let mut display_sequences = Vec::new();

        // Collect sequences that aren't too old
        for seq in self.sequences.iter().rev().take(self.max_display) {
            let age_ms = now.duration_since(seq.last_press).as_millis() as u64;

            // Skip if completely faded
            if age_ms > self.fade_duration_ms {
                continue;
            }

            // Format the sequence
            let formatted = if seq.count > 1 {
                // Show count for repeated keys (vim style)
                format!("{}{}", seq.count, seq.key)
            } else {
                seq.key.clone()
            };

            display_sequences.push(formatted);
        }

        // Reverse to show oldest to newest (left to right)
        display_sequences.reverse();

        // Join with spaces (more compact than arrows)
        display_sequences.join(" ")
    }

    /// Clean up old sequences
    fn cleanup_sequences(&mut self) {
        let now = Instant::now();

        // Remove sequences older than fade duration
        self.sequences.retain(|seq| {
            now.duration_since(seq.last_press).as_millis() < self.fade_duration_ms as u128
        });

        // Keep only last N sequences for memory efficiency
        while self.sequences.len() > self.max_display * 2 {
            self.sequences.pop_front();
        }
    }

    /// Check if there's anything to display
    pub fn has_content(&self) -> bool {
        self.enabled && (!self.sequences.is_empty() || self.chord_mode.is_some())
    }

    /// Clear all sequences
    pub fn clear(&mut self) {
        self.sequences.clear();
        self.chord_mode = None;
    }

    /// Configure display parameters
    pub fn configure(
        &mut self,
        max_display: Option<usize>,
        collapse_window_ms: Option<u64>,
        fade_duration_ms: Option<u64>,
    ) {
        if let Some(max) = max_display {
            self.max_display = max;
        }
        if let Some(window) = collapse_window_ms {
            self.collapse_window_ms = window;
        }
        if let Some(fade) = fade_duration_ms {
            self.fade_duration_ms = fade;
        }
    }

    // Debug getters for accessing internal state
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_chord_mode(&self) -> &Option<String> {
        &self.chord_mode
    }

    pub fn sequence_count(&self) -> usize {
        self.sequences.len()
    }

    pub fn get_sequences(&self) -> Vec<(String, usize)> {
        self.sequences
            .iter()
            .map(|seq| (seq.key.clone(), seq.count))
            .collect()
    }
}

impl Default for KeySequenceRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_collapse_repeated_keys() {
        let mut renderer = KeySequenceRenderer::new();
        renderer.set_enabled(true);

        // Simulate rapid j presses
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(50));
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(50));
        renderer.record_key("j".to_string());

        let display = renderer.get_display();
        assert_eq!(display, "3j");
    }

    #[test]
    fn test_separate_sequences() {
        let mut renderer = KeySequenceRenderer::new();
        renderer.set_enabled(true);

        // Keys with delays between them
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(600)); // Longer than collapse window
        renderer.record_key("k".to_string());
        sleep(Duration::from_millis(600));
        renderer.record_key("h".to_string());

        let display = renderer.get_display();
        assert_eq!(display, "j k h");
    }

    #[test]
    fn test_chord_mode_display() {
        let mut renderer = KeySequenceRenderer::new();
        renderer.set_enabled(true);

        renderer.record_key("y".to_string());
        renderer.set_chord_mode(Some(
            "Yank mode: y=row, c=column, a=all, ESC=cancel".to_string(),
        ));

        let display = renderer.get_display();
        assert_eq!(display, "y(y,c,a)");
    }

    #[test]
    fn test_max_display_limit() {
        let mut renderer = KeySequenceRenderer::new();
        renderer.set_enabled(true);
        renderer.configure(Some(3), None, None); // Limit to 3

        // Add more than limit
        for i in 1..=10 {
            renderer.record_key(format!("{}", i));
            sleep(Duration::from_millis(600));
        }

        let display = renderer.get_display();
        let parts: Vec<&str> = display.split(' ').collect();
        assert!(parts.len() <= 3);
    }

    #[test]
    fn test_mixed_repeated_and_single() {
        let mut renderer = KeySequenceRenderer::new();
        renderer.set_enabled(true);

        // Mix of repeated and single keys
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(50));
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(50));
        renderer.record_key("j".to_string());
        sleep(Duration::from_millis(600)); // Gap
        renderer.record_key("g".to_string());
        sleep(Duration::from_millis(600));
        renderer.record_key("k".to_string());
        sleep(Duration::from_millis(50));
        renderer.record_key("k".to_string());

        let display = renderer.get_display();
        assert_eq!(display, "3j g 2k");
    }
}
