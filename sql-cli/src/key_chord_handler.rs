use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents a chord sequence (e.g., "yy", "gg", "dd")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChordSequence {
    keys: Vec<KeyEvent>,
}

impl ChordSequence {
    pub fn new(keys: Vec<KeyEvent>) -> Self {
        Self { keys }
    }

    /// Create a chord from string notation like "yy" or "gg"
    pub fn from_notation(notation: &str) -> Option<Self> {
        let chars: Vec<char> = notation.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let keys: Vec<KeyEvent> = chars
            .iter()
            .map(|&c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
            .collect();

        Some(Self { keys })
    }

    /// Convert to human-readable string
    pub fn to_string(&self) -> String {
        self.keys
            .iter()
            .map(|k| format_key(k))
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Result of processing a key
#[derive(Debug, Clone)]
pub enum ChordResult {
    /// No chord matched, single key press
    SingleKey(KeyEvent),
    /// Partial chord match, waiting for more keys
    PartialChord(String), // Description of what we're waiting for
    /// Complete chord matched
    CompleteChord(String), // Action name
    /// Chord cancelled (timeout or escape)
    Cancelled,
}

/// Manages key chord sequences and history
pub struct KeyChordHandler {
    /// Map of chord sequences to action names
    chord_map: HashMap<ChordSequence, String>,
    /// Current chord being built
    current_chord: Vec<KeyEvent>,
    /// Time when current chord started
    chord_start: Option<Instant>,
    /// Timeout for chord sequences (milliseconds)
    chord_timeout: Duration,
    /// History of key presses for debugging
    key_history: Vec<String>,
    /// Maximum number of key presses to keep in history
    max_history: usize,
    /// Whether chord mode is active
    chord_mode_active: bool,
    /// Description of current chord mode (e.g., "Yank mode")
    chord_mode_description: Option<String>,
}

impl KeyChordHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            chord_map: HashMap::new(),
            current_chord: Vec::new(),
            chord_start: None,
            chord_timeout: Duration::from_millis(1000), // 1 second default
            key_history: Vec::new(),
            max_history: 50,
            chord_mode_active: false,
            chord_mode_description: None,
        };
        handler.setup_default_chords();
        handler
    }

    /// Set up default chord mappings
    fn setup_default_chords(&mut self) {
        // Yank chords - these are the only actual chords in use
        self.register_chord("yy", "yank_row");
        self.register_chord("yr", "yank_row"); // Alternative for yank row
        self.register_chord("yc", "yank_column");
        self.register_chord("ya", "yank_all");
        self.register_chord("yv", "yank_cell"); // Yank cell value in row mode
        self.register_chord("yq", "yank_query"); // Yank current query text

        // Future chord possibilities (not currently implemented):
        // self.register_chord("gg", "go_to_top");  // Currently single 'g'
        // self.register_chord("dd", "delete_line"); // No line deletion in results
        // self.register_chord("dw", "delete_word"); // Only in command mode with Alt+D
    }

    /// Register a chord sequence
    pub fn register_chord(&mut self, notation: &str, action: &str) {
        if let Some(chord) = ChordSequence::from_notation(notation) {
            self.chord_map.insert(chord, action.to_string());
        }
    }

    /// Process a key event
    pub fn process_key(&mut self, key: KeyEvent) -> ChordResult {
        // Log the key press
        self.log_key_press(&key);

        // Check for timeout
        if let Some(start) = self.chord_start {
            if start.elapsed() > self.chord_timeout {
                self.cancel_chord();
                // Process this key as a new sequence
                return self.process_key_internal(key);
            }
        }

        // Handle escape - always cancels chord
        if key.code == KeyCode::Esc && !self.current_chord.is_empty() {
            self.cancel_chord();
            return ChordResult::Cancelled;
        }

        self.process_key_internal(key)
    }

    fn process_key_internal(&mut self, key: KeyEvent) -> ChordResult {
        // Add key to current chord
        self.current_chord.push(key.clone());

        // Start timer if this is the first key
        if self.current_chord.len() == 1 {
            self.chord_start = Some(Instant::now());
        }

        // Check for exact match
        let current = ChordSequence::new(self.current_chord.clone());
        if let Some(action) = self.chord_map.get(&current) {
            let result = ChordResult::CompleteChord(action.clone());
            self.reset_chord();
            return result;
        }

        // Check for partial matches
        let has_partial = self.chord_map.keys().any(|chord| {
            chord.keys.len() > self.current_chord.len()
                && chord.keys[..self.current_chord.len()] == self.current_chord[..]
        });

        if has_partial {
            // Build description of possible completions
            let possible: Vec<String> = self
                .chord_map
                .iter()
                .filter_map(|(chord, action)| {
                    if chord.keys.len() > self.current_chord.len()
                        && chord.keys[..self.current_chord.len()] == self.current_chord[..]
                    {
                        Some(format!(
                            "{} → {}",
                            format_key(&chord.keys[self.current_chord.len()]),
                            action
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            let description = if self.current_chord.len() == 1
                && self.current_chord[0].code == KeyCode::Char('y')
            {
                "Yank mode: y=row, c=column, a=all, ESC=cancel".to_string()
            } else {
                format!("Waiting for: {}", possible.join(", "))
            };

            self.chord_mode_active = true;
            self.chord_mode_description = Some(description.clone());
            ChordResult::PartialChord(description)
        } else {
            // No match, treat as single key
            let result = if self.current_chord.len() == 1 {
                ChordResult::SingleKey(key)
            } else {
                // Multiple keys but no match - return the first as single, reset
                ChordResult::SingleKey(self.current_chord[0].clone())
            };
            self.reset_chord();
            result
        }
    }

    /// Cancel current chord
    pub fn cancel_chord(&mut self) {
        self.reset_chord();
    }

    /// Reset chord state
    fn reset_chord(&mut self) {
        self.current_chord.clear();
        self.chord_start = None;
        self.chord_mode_active = false;
        self.chord_mode_description = None;
    }

    /// Log a key press to history
    pub fn log_key_press(&mut self, key: &KeyEvent) {
        if self.key_history.len() >= self.max_history {
            self.key_history.remove(0);
        }

        let timestamp = Local::now().format("%H:%M:%S.%3f");
        let key_str = format_key(key);
        let modifiers = format_modifiers(key.modifiers);

        let entry = if modifiers.is_empty() {
            format!("[{}] {}", timestamp, key_str)
        } else {
            format!("[{}] {} ({})", timestamp, key_str, modifiers)
        };

        self.key_history.push(entry);
    }

    /// Get the key press history
    pub fn get_history(&self) -> &[String] {
        &self.key_history
    }

    /// Clear the key press history
    pub fn clear_history(&mut self) {
        self.key_history.clear();
    }

    /// Get current chord mode status
    pub fn is_chord_mode_active(&self) -> bool {
        self.chord_mode_active
    }

    /// Get chord mode description
    pub fn get_chord_mode_description(&self) -> Option<&str> {
        self.chord_mode_description.as_deref()
    }

    /// Set chord timeout
    pub fn set_timeout(&mut self, millis: u64) {
        self.chord_timeout = Duration::from_millis(millis);
    }

    /// Pretty print for debug view
    pub fn format_debug_info(&self) -> String {
        let mut output = String::new();

        // Current chord state
        output.push_str("========== CHORD STATE ==========\n");
        if !self.current_chord.is_empty() {
            output.push_str(&format!(
                "Current chord: {}\n",
                self.current_chord
                    .iter()
                    .map(|k| format_key(k))
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
            if let Some(desc) = &self.chord_mode_description {
                output.push_str(&format!("Mode: {}\n", desc));
            }
            if let Some(start) = self.chord_start {
                let elapsed = start.elapsed().as_millis();
                let remaining = self.chord_timeout.as_millis().saturating_sub(elapsed);
                output.push_str(&format!("Timeout in: {}ms\n", remaining));
            }
        } else {
            output.push_str("No active chord\n");
        }

        // Registered chords
        output.push_str("\n========== REGISTERED CHORDS ==========\n");
        let mut chords: Vec<_> = self.chord_map.iter().collect();
        chords.sort_by_key(|(chord, _)| chord.to_string());
        for (chord, action) in chords {
            output.push_str(&format!("{} → {}\n", chord.to_string(), action));
        }

        // Key history
        output.push_str("\n========== KEY PRESS HISTORY ==========\n");
        output.push_str("(Most recent at bottom, last 50 keys)\n");
        for entry in &self.key_history {
            output.push_str(entry);
            output.push('\n');
        }

        output
    }

    /// Load custom bindings from config (for future)
    pub fn load_from_config(&mut self, config: &HashMap<String, String>) {
        for (notation, action) in config {
            self.register_chord(notation, action);
        }
    }
}

/// Format a key event for display
fn format_key(key: &KeyEvent) -> String {
    let mut result = String::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    // Add key code
    match key.code {
        KeyCode::Char(c) => result.push(c),
        KeyCode::Enter => result.push_str("Enter"),
        KeyCode::Esc => result.push_str("Esc"),
        KeyCode::Backspace => result.push_str("Backspace"),
        KeyCode::Tab => result.push_str("Tab"),
        KeyCode::Delete => result.push_str("Del"),
        KeyCode::Insert => result.push_str("Ins"),
        KeyCode::F(n) => result.push_str(&format!("F{}", n)),
        KeyCode::Left => result.push_str("←"),
        KeyCode::Right => result.push_str("→"),
        KeyCode::Up => result.push_str("↑"),
        KeyCode::Down => result.push_str("↓"),
        KeyCode::Home => result.push_str("Home"),
        KeyCode::End => result.push_str("End"),
        KeyCode::PageUp => result.push_str("PgUp"),
        KeyCode::PageDown => result.push_str("PgDn"),
        _ => result.push_str("?"),
    }

    result
}

/// Format modifiers for display
fn format_modifiers(mods: KeyModifiers) -> String {
    let mut parts = Vec::new();
    if mods.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if mods.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if mods.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_sequence() {
        let chord = ChordSequence::from_notation("yy").unwrap();
        assert_eq!(chord.keys.len(), 2);
        assert_eq!(chord.to_string(), "yy");
    }

    #[test]
    fn test_single_key() {
        let mut handler = KeyChordHandler::new();
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        match handler.process_key(key) {
            ChordResult::SingleKey(_) => {}
            _ => panic!("Expected single key"),
        }
    }

    #[test]
    fn test_chord_completion() {
        let mut handler = KeyChordHandler::new();

        // First 'y' should be partial
        let key1 = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        match handler.process_key(key1) {
            ChordResult::PartialChord(_) => {}
            _ => panic!("Expected partial chord"),
        }

        // Second 'y' should complete
        let key2 = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        match handler.process_key(key2) {
            ChordResult::CompleteChord(action) => {
                assert_eq!(action, "yank_row");
            }
            _ => panic!("Expected complete chord"),
        }
    }
}
