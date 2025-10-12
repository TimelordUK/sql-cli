use anyhow::{Context, Result};
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Configuration for styling table output with colors and formatting
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StyleConfig {
    #[serde(default)]
    pub version: u32,

    /// Rules for styling specific columns by value
    #[serde(default)]
    pub columns: HashMap<String, Vec<ColumnRule>>,

    /// Rules for styling numeric values based on ranges
    #[serde(default)]
    pub numeric_ranges: HashMap<String, Vec<NumericRule>>,

    /// Pattern-based rules using regex
    #[serde(default)]
    pub patterns: Vec<PatternRule>,

    /// Default styles for table elements
    #[serde(default)]
    pub defaults: DefaultStyles,
}

/// Rule for styling a column based on exact value match
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ColumnRule {
    pub value: String,
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    #[serde(default)]
    pub bold: bool,
}

/// Rule for styling numeric values based on conditions
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NumericRule {
    pub condition: String, // "< 100", ">= 100 AND < 300"
    pub fg_color: Option<String>,
    #[serde(default)]
    pub bold: bool,
}

/// Pattern-based rule using regex matching
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatternRule {
    pub regex: String,
    pub fg_color: Option<String>,
    #[serde(default)]
    pub bold: bool,
}

/// Default styles for table elements
#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultStyles {
    pub header_color: Option<String>,
    #[serde(default = "default_true")]
    pub header_bold: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DefaultStyles {
    fn default() -> Self {
        Self {
            header_color: Some("white".to_string()),
            header_bold: true,
        }
    }
}

impl StyleConfig {
    /// Load style configuration from a YAML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read style file: {}", path.display()))?;

        let config: StyleConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse style YAML: {}", path.display()))?;

        Ok(config)
    }

    /// Get the default style file path (~/.config/sql-cli/styles.yaml)
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("sql-cli");
            p.push("styles.yaml");
            p
        })
    }

    /// Load the default style configuration if it exists
    pub fn load_default() -> Option<Self> {
        Self::default_path().and_then(|path| {
            if path.exists() {
                Self::from_file(&path).ok()
            } else {
                None
            }
        })
    }
}

/// Apply style configuration to a table by building styled headers and rows
/// Returns (styled_headers, styled_rows) that can be added to the table
pub fn build_styled_table(
    column_names: &[String],
    rows: &[Vec<String>],
    config: &StyleConfig,
) -> (Vec<Cell>, Vec<Vec<Cell>>) {
    // Build styled headers
    let styled_headers: Vec<Cell> = column_names
        .iter()
        .map(|name| {
            let mut header_cell = Cell::new(name);

            if config.defaults.header_bold {
                header_cell = header_cell.add_attribute(Attribute::Bold);
            }

            if let Some(ref color_str) = config.defaults.header_color {
                if let Some(color) = parse_color(color_str) {
                    header_cell = header_cell.fg(color);
                }
            }

            header_cell
        })
        .collect();

    // Build styled data rows
    let styled_rows: Vec<Vec<Cell>> = rows
        .iter()
        .map(|row_data| {
            row_data
                .iter()
                .enumerate()
                .map(|(col_idx, cell_value)| {
                    if col_idx >= column_names.len() {
                        return Cell::new(cell_value);
                    }

                    let col_name = &column_names[col_idx];
                    let mut cell = Cell::new(cell_value);
                    let mut style_applied = false;

                    // 1. Apply column-specific rules (exact match)
                    if let Some(rules) = config.columns.get(col_name) {
                        for rule in rules {
                            if rule.value == *cell_value {
                                if let Some(ref color_str) = rule.fg_color {
                                    if let Some(color) = parse_color(color_str) {
                                        cell = cell.fg(color);
                                    }
                                }
                                if rule.bold {
                                    cell = cell.add_attribute(Attribute::Bold);
                                }
                                style_applied = true;
                                break;
                            }
                        }
                    }

                    // 2. Apply numeric range rules
                    if !style_applied {
                        if let Some(rules) = config.numeric_ranges.get(col_name) {
                            if let Ok(num) = cell_value.parse::<f64>() {
                                for rule in rules {
                                    if evaluate_condition(num, &rule.condition) {
                                        if let Some(ref color_str) = rule.fg_color {
                                            if let Some(color) = parse_color(color_str) {
                                                cell = cell.fg(color);
                                            }
                                        }
                                        if rule.bold {
                                            cell = cell.add_attribute(Attribute::Bold);
                                        }
                                        style_applied = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // 3. Apply pattern rules (regex)
                    if !style_applied {
                        for pattern in &config.patterns {
                            if let Ok(re) = regex::Regex::new(&pattern.regex) {
                                if re.is_match(cell_value) {
                                    if let Some(ref color_str) = pattern.fg_color {
                                        if let Some(color) = parse_color(color_str) {
                                            cell = cell.fg(color);
                                        }
                                    }
                                    if pattern.bold {
                                        cell = cell.add_attribute(Attribute::Bold);
                                    }
                                    break;
                                }
                            }
                        }
                    }

                    cell
                })
                .collect()
        })
        .collect();

    (styled_headers, styled_rows)
}

/// Apply style configuration to a table
pub fn apply_styles_to_table(
    table: &mut Table,
    column_names: &[String],
    rows: &[Vec<String>],
    config: &StyleConfig,
) -> Result<()> {
    let (styled_headers, styled_rows) = build_styled_table(column_names, rows, config);

    table.set_header(styled_headers);
    for styled_row in styled_rows {
        table.add_row(styled_row);
    }

    Ok(())
}

/// Parse color string to comfy_table::Color
fn parse_color(color_str: &str) -> Option<Color> {
    match color_str.to_lowercase().as_str() {
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "white" => Some(Color::White),
        "black" => Some(Color::Black),
        "dark_grey" | "dark_gray" => Some(Color::DarkGrey),
        "dark_red" => Some(Color::DarkRed),
        "dark_green" => Some(Color::DarkGreen),
        "dark_blue" => Some(Color::DarkBlue),
        "dark_yellow" => Some(Color::DarkYellow),
        "dark_cyan" => Some(Color::DarkCyan),
        "dark_magenta" => Some(Color::DarkMagenta),
        "grey" | "gray" => Some(Color::Grey),
        _ => None,
    }
}

/// Evaluate a numeric condition
fn evaluate_condition(value: f64, condition: &str) -> bool {
    // Handle compound conditions with AND
    if condition.contains("AND") {
        let parts: Vec<&str> = condition.split("AND").map(|s| s.trim()).collect();
        return parts
            .iter()
            .all(|part| evaluate_simple_condition(value, part));
    }

    evaluate_simple_condition(value, condition)
}

/// Evaluate a simple numeric condition (e.g., "< 100", ">= 300")
fn evaluate_simple_condition(value: f64, condition: &str) -> bool {
    let condition = condition.trim();

    if let Some(num_str) = condition.strip_prefix("<=") {
        if let Ok(threshold) = num_str.trim().parse::<f64>() {
            return value <= threshold;
        }
    } else if let Some(num_str) = condition.strip_prefix("<") {
        if let Ok(threshold) = num_str.trim().parse::<f64>() {
            return value < threshold;
        }
    } else if let Some(num_str) = condition.strip_prefix(">=") {
        if let Ok(threshold) = num_str.trim().parse::<f64>() {
            return value >= threshold;
        }
    } else if let Some(num_str) = condition.strip_prefix(">") {
        if let Ok(threshold) = num_str.trim().parse::<f64>() {
            return value > threshold;
        }
    } else if let Some(num_str) = condition.strip_prefix("==") {
        if let Ok(threshold) = num_str.trim().parse::<f64>() {
            return (value - threshold).abs() < f64::EPSILON;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple_condition() {
        assert!(evaluate_simple_condition(50.0, "< 100"));
        assert!(!evaluate_simple_condition(150.0, "< 100"));

        assert!(evaluate_simple_condition(100.0, ">= 100"));
        assert!(evaluate_simple_condition(150.0, ">= 100"));
        assert!(!evaluate_simple_condition(99.0, ">= 100"));

        assert!(evaluate_simple_condition(50.0, "<= 50"));
        assert!(!evaluate_simple_condition(51.0, "<= 50"));

        assert!(evaluate_simple_condition(200.0, "> 100"));
        assert!(!evaluate_simple_condition(100.0, "> 100"));
    }

    #[test]
    fn test_evaluate_compound_condition() {
        assert!(evaluate_condition(150.0, ">= 100 AND < 200"));
        assert!(!evaluate_condition(99.0, ">= 100 AND < 200"));
        assert!(!evaluate_condition(200.0, ">= 100 AND < 200"));
    }

    #[test]
    fn test_parse_color() {
        assert!(matches!(parse_color("red"), Some(Color::Red)));
        assert!(matches!(parse_color("Red"), Some(Color::Red)));
        assert!(matches!(parse_color("RED"), Some(Color::Red)));
        assert!(matches!(parse_color("green"), Some(Color::Green)));
        assert!(matches!(parse_color("blue"), Some(Color::Blue)));
        assert!(matches!(parse_color("dark_grey"), Some(Color::DarkGrey)));
        assert!(matches!(parse_color("dark_gray"), Some(Color::DarkGrey)));
        assert!(parse_color("invalid_color").is_none());
    }
}
