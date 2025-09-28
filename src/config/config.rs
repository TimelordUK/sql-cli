use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub display: DisplayConfig,
    pub keybindings: KeybindingConfig,
    pub behavior: BehaviorConfig,
    pub theme: ThemeConfig,
    pub redis_cache: RedisCacheConfig,
    pub web: WebConfig,
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
    /// Format: "action" -> "`key_sequence`"
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

    /// Start mode when loading files: "command" or "results"
    pub start_mode: String,

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

    /// Default date notation: "us" (MM/DD/YYYY) or "european" (DD/MM/YYYY)
    /// This determines how ambiguous dates like 04/09/2025 are interpreted
    pub default_date_notation: String,
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

    /// Background color if `use_background` is true
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisCacheConfig {
    /// Whether cache is enabled (can be overridden by SQL_CLI_CACHE env var)
    pub enabled: bool,

    /// Redis URL (can be overridden by SQL_CLI_REDIS_URL env var)
    pub redis_url: String,

    /// Default cache duration in seconds when CACHE is specified without duration
    pub default_duration: u64,

    /// Cache durations by URL pattern (glob patterns supported)
    pub duration_rules: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    /// Default timeout for web requests in seconds
    pub timeout: u64,

    /// Maximum response size in MB
    pub max_response_size: usize,
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
    #[must_use]
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
            start_mode: "results".to_string(), // Default to results mode for immediate data view
            max_display_rows: 10000,
            cache_dir: None,
            enable_history: true,
            max_history_entries: 1000,
            hide_empty_columns: false, // Default to false to avoid hiding data unexpectedly
            default_date_notation: "us".to_string(), // Default to US format (MM/DD/YYYY)
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

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_duration: 600, // 10 minutes default
            duration_rules: HashMap::new(),
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            max_response_size: 100,
        }
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Very simple glob matching for * wildcard
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            // First part must match at beginning
            if i == 0 && !text.starts_with(part) {
                return false;
            }
            // Last part must match at end
            else if i == parts.len() - 1 && !text.ends_with(part) {
                return false;
            }
            // Middle parts must be found
            else if let Some(idx) = text[pos..].find(part) {
                pos += idx + part.len();
            } else {
                return false;
            }
        }
        true
    } else {
        text.contains(pattern)
    }
}

impl Config {
    /// Generate debug info string for display
    #[must_use]
    pub fn debug_info(&self) -> String {
        let mut info = String::new();
        info.push_str("\n========== CONFIGURATION ==========\n");

        // Display configuration
        info.push_str("[display]\n");
        info.push_str(&format!("  use_glyphs = {}\n", self.display.use_glyphs));
        info.push_str(&format!(
            "  show_row_numbers = {}\n",
            self.display.show_row_numbers
        ));
        info.push_str(&format!("  compact_mode = {}\n", self.display.compact_mode));
        info.push_str(&format!(
            "  show_key_indicator = {}\n",
            self.display.show_key_indicator
        ));

        // Behavior configuration
        info.push_str("\n[behavior]\n");
        info.push_str(&format!(
            "  auto_execute_on_load = {}\n",
            self.behavior.auto_execute_on_load
        ));
        info.push_str(&format!(
            "  case_insensitive_default = {}\n",
            self.behavior.case_insensitive_default
        ));
        info.push_str(&format!(
            "  start_mode = \"{}\"\n",
            self.behavior.start_mode
        ));
        info.push_str(&format!(
            "  max_display_rows = {}\n",
            self.behavior.max_display_rows
        ));
        info.push_str(&format!(
            "  enable_history = {}\n",
            self.behavior.enable_history
        ));
        info.push_str(&format!(
            "  max_history_entries = {}\n",
            self.behavior.max_history_entries
        ));
        info.push_str(&format!(
            "  hide_empty_columns = {}\n",
            self.behavior.hide_empty_columns
        ));
        info.push_str(&format!(
            "  default_date_notation = \"{}\"\n",
            self.behavior.default_date_notation
        ));

        // Keybindings configuration
        info.push_str("\n[keybindings]\n");
        info.push_str(&format!("  vim_mode = {}\n", self.keybindings.vim_mode));

        // Theme configuration
        info.push_str("\n[theme]\n");
        info.push_str(&format!("  color_scheme = {}\n", self.theme.color_scheme));
        info.push_str(&format!(
            "  rainbow_parentheses = {}\n",
            self.theme.rainbow_parentheses
        ));
        info.push_str(&format!(
            "  syntax_highlighting = {}\n",
            self.theme.syntax_highlighting
        ));
        info.push_str(&format!(
            "  cell_selection_style = {}\n",
            self.theme.cell_selection_style.mode
        ));

        // Redis cache configuration
        info.push_str("\n[redis_cache]\n");
        info.push_str(&format!("  enabled = {}\n", self.redis_cache.enabled));
        info.push_str(&format!("  redis_url = {}\n", self.redis_cache.redis_url));
        info.push_str(&format!(
            "  default_duration = {}s\n",
            self.redis_cache.default_duration
        ));
        info.push_str(&format!(
            "  duration_rules = {} patterns\n",
            self.redis_cache.duration_rules.len()
        ));

        // Web configuration
        info.push_str("\n[web]\n");
        info.push_str(&format!("  timeout = {}s\n", self.web.timeout));
        info.push_str(&format!(
            "  max_response_size = {} MB\n",
            self.web.max_response_size
        ));

        info.push_str("==========================================\n");
        info
    }

    /// Load config from the default location
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Create default config if it doesn't exist
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config.with_env_overrides());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;

        // Apply simple mode if glyphs are disabled
        let mut config = config;
        if !config.display.use_glyphs {
            config.display.icons = IconConfig::simple();
        }

        Ok(config.with_env_overrides())
    }

    /// Apply environment variable overrides
    fn with_env_overrides(mut self) -> Self {
        // Override cache enabled from env
        if let Ok(val) = std::env::var("SQL_CLI_CACHE") {
            self.redis_cache.enabled =
                val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("yes") || val == "1";
        }

        // Override Redis URL from env
        if let Ok(url) = std::env::var("SQL_CLI_REDIS_URL") {
            self.redis_cache.redis_url = url;
        }

        // Override default cache duration from env
        if let Ok(duration) = std::env::var("SQL_CLI_CACHE_DEFAULT_DURATION") {
            if let Ok(seconds) = duration.parse::<u64>() {
                self.redis_cache.default_duration = seconds;
            }
        }

        self
    }

    /// Get cache duration for a URL, checking rules first then falling back to default
    pub fn get_cache_duration(&self, url: &str) -> u64 {
        // Check if any rule matches the URL
        for (pattern, duration) in &self.redis_cache.duration_rules {
            if url.contains(pattern) || glob_match(pattern, url) {
                return *duration;
            }
        }

        // Fall back to default
        self.redis_cache.default_duration
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
    #[must_use]
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

# Start mode when loading files: "command" or "results"
# - "command": Start in command mode (focus on SQL input)
# - "results": Start in results mode (focus on data, press 'i' to edit query)
start_mode = "results"

# Maximum rows to display without warning
max_display_rows = 10000

# Cache directory (leave commented to use default)
# cache_dir = "/path/to/cache"

# Enable query history
enable_history = true

# Maximum number of history entries to keep
max_history_entries = 1000

# Automatically hide empty/null columns when data is loaded (can be toggled with 'E' key)
hide_empty_columns = false

# Default date notation for parsing ambiguous dates
# "us" = MM/DD/YYYY format (e.g., 04/09/2025 = April 9, 2025)
# "european" = DD/MM/YYYY format (e.g., 04/09/2025 = September 4, 2025)
default_date_notation = "us"

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

[redis_cache]
# Enable Redis cache (can be overridden by SQL_CLI_CACHE env var)
enabled = false

# Redis connection URL (can be overridden by SQL_CLI_REDIS_URL env var)
redis_url = "redis://127.0.0.1:6379"

# Default cache duration in seconds when CACHE is specified without a value
# or when no CACHE directive is present in the query
default_duration = 600  # 10 minutes

# Cache duration rules based on URL patterns
# Pattern matching uses simple glob syntax (* for wildcards)
# These override the default_duration for matching URLs
[redis_cache.duration_rules]
# Production APIs - cache for 1 hour
"*.bloomberg.com/*" = 3600
"*prod*" = 3600
"*production*" = 3600

# Staging/UAT - cache for 5 minutes
"*staging*" = 300
"*uat*" = 300

# Historical data endpoints - cache for 24 hours
"*/historical/*" = 86400
"*/archive/*" = 86400
"*/trades/20*" = 43200  # Yesterday's trades - 12 hours

# Real-time/volatile data - short cache
"*/realtime/*" = 60
"*/live/*" = 30
"*/prices/*" = 120

# Specific endpoints
"api.barclays.com/trades" = 7200  # 2 hours for Barclays trades
"api.jpmorgan.com/fx" = 1800      # 30 minutes for JPM FX

[web]
# Default timeout for web requests in seconds
timeout = 30

# Maximum response size in MB
max_response_size = 100
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
