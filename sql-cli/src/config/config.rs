use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub display: DisplayConfig,
    pub keybindings: KeybindingConfig,
    pub behavior: BehaviorConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Use Unicode/Nerd Font glyphs for icons
    pub use_glyphs: bool,

    /// Show row numbers by default
    pub show_row_numbers: bool,

    /// Compact mode by default
    pub compact_mode: bool,

    /// Icons for different states (can be overridden)
    pub icons: IconConfig,

    /// Show key press indicator by default
    pub show_key_indicator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IconConfig {
    pub pin: String,
    pub lock: String,
    pub cache: String,
    pub file: String,
    pub database: String,
    pub api: String,
    pub case_insensitive: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub success: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingConfig {
    /// Whether to use vim-style keybindings
    pub vim_mode: bool,

    /// Custom key mappings (future expansion)
    /// Format: "action" -> "key_sequence"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_mappings: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Auto-execute SELECT * when loading CSV/JSON
    pub auto_execute_on_load: bool,

    /// Case-insensitive by default
    pub case_insensitive_default: bool,

    /// Maximum rows to display without pagination warning
    pub max_display_rows: usize,

    /// Default cache directory
    pub cache_dir: Option<PathBuf>,

    /// Enable query history
    pub enable_history: bool,

    /// Maximum history entries
    pub max_history_entries: usize,

    /// Automatically hide empty/null columns on data load
    pub hide_empty_columns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Color scheme: "default", "dark", "light", "solarized"
    pub color_scheme: String,

    /// Rainbow parentheses
    pub rainbow_parentheses: bool,

    /// Syntax highlighting
    pub syntax_highlighting: bool,

    /// Cell selection style
    pub cell_selection_style: CellSelectionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CellSelectionStyle {
    /// Style mode: "underline", "block", "border", "corners", "subtle"
    pub mode: String,

    /// Foreground color for selected cell (e.g., "yellow", "orange", "cyan")
    pub foreground: String,

    /// Whether to use background color
    pub use_background: bool,

    /// Background color if use_background is true
    pub background: String,

    /// Whether to bold the text
    pub bold: bool,

    /// Whether to underline the text (legacy, use mode instead)
    pub underline: bool,

    /// Border style for "border" mode: "single", "double", "rounded", "thick"
    pub border_style: String,

    /// Whether to show cell corners in "corners" mode
    pub corner_chars: String, // e.g., "┌┐└┘" or "╭╮╰╯" for rounded
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            keybindings: KeybindingConfig::default(),
            behavior: BehaviorConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            use_glyphs: true, // Default to glyphs, can be disabled
            show_row_numbers: false,
            compact_mode: false,
            icons: IconConfig::default(),
            show_key_indicator: true, // Default on for better debugging
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            // Default to Unicode/Nerd Font icons
            pin: "📌".to_string(),
            lock: "🔒".to_string(),
            cache: "📦".to_string(),
            file: "📁".to_string(),
            database: "🗄️".to_string(),
            api: "🌐".to_string(),
            case_insensitive: "Ⓘ".to_string(),
            warning: "⚠️".to_string(),
            error: "❌".to_string(),
            info: "ℹ️".to_string(),
            success: "✅".to_string(),
        }
    }
}

impl IconConfig {
    /// Get simple ASCII alternatives for terminals without glyph support
    pub fn simple() -> Self {
        Self {
            pin: "[P]".to_string(),
            lock: "[L]".to_string(),
            cache: "[C]".to_string(),
            file: "[F]".to_string(),
            database: "[DB]".to_string(),
            api: "[API]".to_string(),
            case_insensitive: "[i]".to_string(),
            warning: "[!]".to_string(),
            error: "[X]".to_string(),
            info: "[i]".to_string(),
            success: "[OK]".to_string(),
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            vim_mode: true,
            custom_mappings: None,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_execute_on_load: true,
            case_insensitive_default: true, // Default to case-insensitive for practical use
            max_display_rows: 10000,
            cache_dir: None,
            enable_history: true,
            max_history_entries: 1000,
            hide_empty_columns: true, // Default to true for cleaner display with large datasets
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            color_scheme: "default".to_string(),
            rainbow_parentheses: true,
            syntax_highlighting: true,
            cell_selection_style: CellSelectionStyle::default(),
        }
    }
}

impl Default for CellSelectionStyle {
    fn default() -> Self {
        Self {
            mode: "underline".to_string(), // Default to current behavior
            foreground: "yellow".to_string(),
            use_background: false,
            background: "cyan".to_string(),
            bold: true,
            underline: true, // Keep for backward compatibility
            border_style: "single".to_string(),
            corner_chars: "┌┐└┘".to_string(),
        }
    }
}

impl Config {
    /// Load config from the default location
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Create default config if it doesn't exist
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;

        // Apply simple mode if glyphs are disabled
        let mut config = config;
        if !config.display.use_glyphs {
            config.display.icons = IconConfig::simple();
        }

        Ok(config)
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    /// Get the default config file path
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        Ok(config_dir.join("sql-cli").join("config.toml"))
    }

    /// Create a default config file with comments
    pub fn create_default_with_comments() -> String {
        r#"# SQL CLI Configuration File
# Location: ~/.config/sql-cli/config.toml (Linux/macOS)
#           %APPDATA%\sql-cli\config.toml (Windows)

[display]
# Use Unicode/Nerd Font glyphs for icons
# Set to false for ASCII-only mode (better compatibility)
use_glyphs = true

# Show row numbers by default in results view
show_row_numbers = false

# Use compact mode by default (less padding, more data visible)
compact_mode = false

# Show key press indicator on status line (useful for debugging)
show_key_indicator = true

# Icon configuration
# These are automatically set to ASCII when use_glyphs = false
[display.icons]
pin = "📌"
lock = "🔒"
cache = "📦"
file = "📁"
database = "🗄️"
api = "🌐"
case_insensitive = "Ⓘ"
warning = "⚠️"
error = "❌"
info = "ℹ️"
success = "✅"

[keybindings]
# Use vim-style keybindings (j/k navigation, yy to yank, etc.)
vim_mode = true

# Custom key mappings (future feature)
# [keybindings.custom_mappings]
# "copy_row" = "ctrl+c"
# "paste" = "ctrl+v"

[behavior]
# Automatically execute SELECT * when loading CSV/JSON files
auto_execute_on_load = true

# Use case-insensitive string comparisons by default (recommended for practical use)
case_insensitive_default = true

# Maximum rows to display without warning
max_display_rows = 10000

# Cache directory (leave commented to use default)
# cache_dir = "/path/to/cache"

# Enable query history
enable_history = true

# Maximum number of history entries to keep
max_history_entries = 1000

# Automatically hide empty/null columns when data is loaded (can be toggled with 'E' key)
hide_empty_columns = true

[theme]
# Color scheme: "default", "dark", "light", "solarized"
color_scheme = "default"

# Enable rainbow parentheses in SQL queries
rainbow_parentheses = true

# Enable syntax highlighting
syntax_highlighting = true

# Cell selection highlighting style (for cell mode)
[theme.cell_selection_style]
# Foreground color: "yellow", "red", "green", "blue", "magenta", "cyan", "white"
foreground = "yellow"

# Whether to change background color (can be hard to read with some color schemes)
use_background = false

# Background color if use_background is true
background = "cyan"

# Text styling
bold = true
underline = true
"#
        .to_string()
    }

    /// Initialize config with a setup wizard
    pub fn init_wizard() -> Result<Self> {
        println!("SQL CLI Configuration Setup");
        println!("============================");

        // Ask about glyph support
        print!("Does your terminal support Unicode/Nerd Font icons? (y/n) [y]: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let use_glyphs = !input.trim().eq_ignore_ascii_case("n");

        let mut config = Config::default();
        config.display.use_glyphs = use_glyphs;
        if !use_glyphs {
            config.display.icons = IconConfig::simple();
        }

        // Ask about vim mode
        print!("Enable vim-style keybindings? (y/n) [y]: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        config.keybindings.vim_mode = !input.trim().eq_ignore_ascii_case("n");

        config.save()?;

        println!("\nConfiguration saved to: {:?}", Config::get_config_path()?);
        println!("You can edit this file directly to customize further.");

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.display.use_glyphs);
        assert!(config.keybindings.vim_mode);
    }

    #[test]
    fn test_simple_icons() {
        let icons = IconConfig::simple();
        assert_eq!(icons.pin, "[P]");
        assert_eq!(icons.lock, "[L]");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.display.use_glyphs, parsed.display.use_glyphs);
    }
}
