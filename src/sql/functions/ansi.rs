use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use anyhow::{anyhow, Result};

/// ANSI color codes
const ANSI_RESET: &str = "\x1b[0m";

/// Named color mappings
fn get_color_code(color_name: &str) -> Option<&'static str> {
    match color_name.to_lowercase().as_str() {
        // Foreground colors (30-37)
        "black" => Some("\x1b[30m"),
        "red" => Some("\x1b[31m"),
        "green" => Some("\x1b[32m"),
        "yellow" => Some("\x1b[33m"),
        "blue" => Some("\x1b[34m"),
        "magenta" | "purple" => Some("\x1b[35m"),
        "cyan" => Some("\x1b[36m"),
        "white" => Some("\x1b[37m"),
        // Bright foreground colors (90-97)
        "bright_black" | "gray" | "grey" => Some("\x1b[90m"),
        "bright_red" => Some("\x1b[91m"),
        "bright_green" => Some("\x1b[92m"),
        "bright_yellow" => Some("\x1b[93m"),
        "bright_blue" => Some("\x1b[94m"),
        "bright_magenta" | "bright_purple" => Some("\x1b[95m"),
        "bright_cyan" => Some("\x1b[96m"),
        "bright_white" => Some("\x1b[97m"),
        _ => None,
    }
}

/// Background color mappings
fn get_bg_color_code(color_name: &str) -> Option<&'static str> {
    match color_name.to_lowercase().as_str() {
        // Background colors (40-47)
        "black" => Some("\x1b[40m"),
        "red" => Some("\x1b[41m"),
        "green" => Some("\x1b[42m"),
        "yellow" => Some("\x1b[43m"),
        "blue" => Some("\x1b[44m"),
        "magenta" | "purple" => Some("\x1b[45m"),
        "cyan" => Some("\x1b[46m"),
        "white" => Some("\x1b[47m"),
        // Bright background colors (100-107)
        "bright_black" | "gray" | "grey" => Some("\x1b[100m"),
        "bright_red" => Some("\x1b[101m"),
        "bright_green" => Some("\x1b[102m"),
        "bright_yellow" => Some("\x1b[103m"),
        "bright_blue" => Some("\x1b[104m"),
        "bright_magenta" | "bright_purple" => Some("\x1b[105m"),
        "bright_cyan" => Some("\x1b[106m"),
        "bright_white" => Some("\x1b[107m"),
        _ => None,
    }
}

/// ANSI_COLOR(color_name, text) - Apply foreground color to text
pub struct AnsiColorFunction;

impl SqlFunction for AnsiColorFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_COLOR",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(2),
            description: "Apply ANSI foreground color to text",
            returns: "Colored text with ANSI escape codes",
            examples: vec![
                "SELECT ANSI_COLOR('red', 'ERROR')",
                "SELECT ANSI_COLOR('green', 'SUCCESS')",
                "SELECT ANSI_COLOR('yellow', amount) FROM sales",
                "SELECT ANSI_COLOR('bright_blue', order_id) FROM orders",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let color_name = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(args[1].clone()),
            _ => return Err(anyhow!("ANSI_COLOR: first argument must be a color name")),
        };

        let text = match &args[1] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[1].to_string(),
        };

        let color_code = get_color_code(color_name).ok_or_else(|| {
            anyhow!(
                "ANSI_COLOR: unknown color '{}'. Available: black, red, green, yellow, blue, magenta, cyan, white, bright_* variants",
                color_name
            )
        })?;

        Ok(DataValue::String(format!(
            "{}{}{}",
            color_code, text, ANSI_RESET
        )))
    }
}

/// ANSI_BG(color_name, text) - Apply background color to text
pub struct AnsiBgFunction;

impl SqlFunction for AnsiBgFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_BG",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(2),
            description: "Apply ANSI background color to text",
            returns: "Text with colored background using ANSI escape codes",
            examples: vec![
                "SELECT ANSI_BG('red', 'CRITICAL')",
                "SELECT ANSI_BG('green', 'ACTIVE')",
                "SELECT ANSI_BG('yellow', 'WARNING')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let color_name = match &args[0] {
            DataValue::String(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(args[1].clone()),
            _ => return Err(anyhow!("ANSI_BG: first argument must be a color name")),
        };

        let text = match &args[1] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[1].to_string(),
        };

        let color_code = get_bg_color_code(color_name).ok_or_else(|| {
            anyhow!(
                "ANSI_BG: unknown color '{}'. Available: black, red, green, yellow, blue, magenta, cyan, white, bright_* variants",
                color_name
            )
        })?;

        Ok(DataValue::String(format!(
            "{}{}{}",
            color_code, text, ANSI_RESET
        )))
    }
}

/// ANSI_RGB(r, g, b, text) - Apply RGB foreground color to text
pub struct AnsiRgbFunction;

impl SqlFunction for AnsiRgbFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_RGB",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(4),
            description: "Apply RGB foreground color to text (0-255 for each component)",
            returns: "Text colored with RGB value using ANSI escape codes",
            examples: vec![
                "SELECT ANSI_RGB(255, 0, 0, 'Bright Red')",
                "SELECT ANSI_RGB(0, 255, 0, 'Bright Green')",
                "SELECT ANSI_RGB(255, 165, 0, 'Orange')",
                "SELECT ANSI_RGB(138, 43, 226, 'Blue Violet')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let r = match &args[0] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB: red value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB: red value must be an integer")),
        };

        let g = match &args[1] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB: green value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB: green value must be an integer")),
        };

        let b = match &args[2] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB: blue value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB: blue value must be an integer")),
        };

        let text = match &args[3] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[3].to_string(),
        };

        // ANSI 24-bit true color: ESC[38;2;r;g;bm
        Ok(DataValue::String(format!(
            "\x1b[38;2;{};{};{}m{}{}",
            r, g, b, text, ANSI_RESET
        )))
    }
}

/// ANSI_RGB_BG(r, g, b, text) - Apply RGB background color to text
pub struct AnsiRgbBgFunction;

impl SqlFunction for AnsiRgbBgFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_RGB_BG",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(4),
            description: "Apply RGB background color to text (0-255 for each component)",
            returns: "Text with RGB background using ANSI escape codes",
            examples: vec![
                "SELECT ANSI_RGB_BG(255, 0, 0, 'Red Background')",
                "SELECT ANSI_RGB_BG(0, 128, 0, 'Dark Green BG')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let r = match &args[0] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB_BG: red value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB_BG: red value must be an integer")),
        };

        let g = match &args[1] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB_BG: green value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB_BG: green value must be an integer")),
        };

        let b = match &args[2] {
            DataValue::Integer(n) => {
                if *n < 0 || *n > 255 {
                    return Err(anyhow!("ANSI_RGB_BG: blue value must be 0-255, got {}", n));
                }
                *n as u8
            }
            _ => return Err(anyhow!("ANSI_RGB_BG: blue value must be an integer")),
        };

        let text = match &args[3] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[3].to_string(),
        };

        // ANSI 24-bit true color background: ESC[48;2;r;g;bm
        Ok(DataValue::String(format!(
            "\x1b[48;2;{};{};{}m{}{}",
            r, g, b, text, ANSI_RESET
        )))
    }
}

/// ANSI_BOLD(text) - Make text bold
pub struct AnsiBoldFunction;

impl SqlFunction for AnsiBoldFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_BOLD",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Make text bold using ANSI escape codes",
            returns: "Bold text",
            examples: vec![
                "SELECT ANSI_BOLD('Important')",
                "SELECT ANSI_BOLD(total) FROM sales",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[1m{}{}", text, ANSI_RESET)))
    }
}

/// ANSI_ITALIC(text) - Make text italic
pub struct AnsiItalicFunction;

impl SqlFunction for AnsiItalicFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_ITALIC",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Make text italic using ANSI escape codes",
            returns: "Italic text",
            examples: vec!["SELECT ANSI_ITALIC('Emphasized')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[3m{}{}", text, ANSI_RESET)))
    }
}

/// ANSI_UNDERLINE(text) - Underline text
pub struct AnsiUnderlineFunction;

impl SqlFunction for AnsiUnderlineFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_UNDERLINE",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Underline text using ANSI escape codes",
            returns: "Underlined text",
            examples: vec!["SELECT ANSI_UNDERLINE('Important')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[4m{}{}", text, ANSI_RESET)))
    }
}

/// ANSI_BLINK(text) - Make text blink (slow)
pub struct AnsiBlinkFunction;

impl SqlFunction for AnsiBlinkFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_BLINK",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Make text blink using ANSI escape codes",
            returns: "Blinking text",
            examples: vec!["SELECT ANSI_BLINK('ALERT')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[5m{}{}", text, ANSI_RESET)))
    }
}

/// ANSI_REVERSE(text) - Reverse video (swap fg/bg)
pub struct AnsiReverseFunction;

impl SqlFunction for AnsiReverseFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_REVERSE",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Reverse video (swap foreground and background colors)",
            returns: "Text with reversed colors",
            examples: vec!["SELECT ANSI_REVERSE('Highlighted')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[7m{}{}", text, ANSI_RESET)))
    }
}

/// ANSI_STRIKETHROUGH(text) - Strikethrough text
pub struct AnsiStrikethroughFunction;

impl SqlFunction for AnsiStrikethroughFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ANSI_STRIKETHROUGH",
            category: FunctionCategory::Terminal,
            arg_count: ArgCount::Fixed(1),
            description: "Add strikethrough to text using ANSI escape codes",
            returns: "Strikethrough text",
            examples: vec!["SELECT ANSI_STRIKETHROUGH('Deprecated')"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let text = match &args[0] {
            DataValue::String(s) => s.clone(),
            DataValue::InternedString(s) => s.to_string(),
            DataValue::Integer(n) => n.to_string(),
            DataValue::Float(f) => f.to_string(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => args[0].to_string(),
        };

        Ok(DataValue::String(format!("\x1b[9m{}{}", text, ANSI_RESET)))
    }
}
