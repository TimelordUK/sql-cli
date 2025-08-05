use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct SqlHighlighter {
    // Since syntect types don't implement Clone, we'll create them on-demand
}

impl Clone for SqlHighlighter {
    fn clone(&self) -> Self {
        SqlHighlighter {}
    }
}

impl SqlHighlighter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn highlight_sql_line(&self, text: &str) -> Line<'static> {
        // Create syntect objects on-demand
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        // Find SQL syntax
        let syntax = syntax_set
            .find_syntax_by_extension("sql")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

        // Use a dark theme suitable for terminals
        let theme = &theme_set.themes["base16-ocean.dark"];

        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut spans = Vec::new();

        // Handle single line highlighting
        if let Ok(ranges) = highlighter.highlight_line(text, &syntax_set) {
            for (style, text_piece) in ranges {
                let ratatui_style = self.syntect_to_ratatui_style(style);
                spans.push(Span::styled(text_piece.to_string(), ratatui_style));
            }
        } else {
            // Fallback to plain text if highlighting fails
            spans.push(Span::raw(text.to_string()));
        }

        Line::from(spans)
    }

    pub fn highlight_sql_multiline(&self, text: &str) -> Vec<Line<'static>> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let syntax = syntax_set
            .find_syntax_by_extension("sql")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

        let theme = &theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();

        for line in LinesWithEndings::from(text) {
            let mut spans = Vec::new();

            if let Ok(ranges) = highlighter.highlight_line(line, &syntax_set) {
                for (style, text_piece) in ranges {
                    let ratatui_style = self.syntect_to_ratatui_style(style);
                    spans.push(Span::styled(text_piece.to_string(), ratatui_style));
                }
            } else {
                spans.push(Span::raw(line.to_string()));
            }

            lines.push(Line::from(spans));
        }

        lines
    }

    fn syntect_to_ratatui_style(&self, syntect_style: SyntectStyle) -> Style {
        let mut style = Style::default();

        // Convert syntect color to ratatui color
        let fg_color = syntect_style.foreground;
        let ratatui_color = Color::Rgb(fg_color.r, fg_color.g, fg_color.b);
        style = style.fg(ratatui_color);

        // Handle background if needed
        // let bg_color = syntect_style.background;
        // style = style.bg(Color::Rgb(bg_color.r, bg_color.g, bg_color.b));

        style
    }

    /// Simple keyword-based highlighting as fallback
    pub fn simple_sql_highlight(&self, text: &str) -> Line<'static> {
        let keywords = [
            "SELECT", "FROM", "WHERE", "AND", "OR", "IN", "ORDER", "BY", "ASC", "DESC", "INSERT",
            "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "GROUP", "HAVING",
            "LIMIT", "OFFSET", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "NULL", "NOT", "IS",
            "LIKE", "BETWEEN", "EXISTS", "DISTINCT", "AS", "ON",
        ];

        let linq_methods = [
            "Contains",
            "StartsWith",
            "EndsWith",
            "Length",
            "ToUpper",
            "ToLower",
            "Trim",
            "Substring",
            "IndexOf",
            "Replace",
        ];

        let operators = ["=", "!=", "<>", "<", ">", "<=", ">=", "+", "-", "*", "/"];
        let string_delimiters = ["'", "\""];

        let mut spans = Vec::new();
        let mut current_word = String::new();
        let mut in_string = false;
        let mut string_delimiter = '\0';

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if in_string {
                current_word.push(ch);
                if ch == string_delimiter {
                    // End of string
                    spans.push(Span::styled(
                        current_word.clone(),
                        Style::default().fg(Color::Green),
                    ));
                    current_word.clear();
                    in_string = false;
                }
                i += 1;
                continue;
            }

            if string_delimiters.contains(&ch.to_string().as_str()) {
                // Start of string
                if !current_word.is_empty() {
                    self.push_word_span(
                        &mut spans,
                        &current_word,
                        &keywords,
                        &operators,
                        &linq_methods,
                    );
                    current_word.clear();
                }
                current_word.push(ch);
                in_string = true;
                string_delimiter = ch;
                i += 1;
                continue;
            }

            if ch.is_alphabetic() || ch == '_' || (ch.is_numeric() && !current_word.is_empty()) {
                current_word.push(ch);
            } else if ch == '.' && !current_word.is_empty() {
                // Check if this is a method call pattern (identifier.method)
                let mut j = i + 1;
                let mut method_name = String::new();

                // Look ahead to see if a method name follows
                while j < chars.len() && (chars[j].is_alphabetic() || chars[j] == '_') {
                    method_name.push(chars[j]);
                    j += 1;
                }

                if linq_methods.contains(&method_name.as_str()) {
                    // This is a LINQ method call
                    spans.push(Span::raw(current_word.clone())); // Column name in default color
                    spans.push(Span::styled(".", Style::default().fg(Color::Magenta))); // Dot in magenta
                    spans.push(Span::styled(
                        method_name.clone(),
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    )); // Method in bold magenta
                    current_word.clear();
                    i = j - 1; // Skip the method name we just processed
                } else {
                    // Regular dot
                    self.push_word_span(
                        &mut spans,
                        &current_word,
                        &keywords,
                        &operators,
                        &linq_methods,
                    );
                    current_word.clear();
                    spans.push(Span::raw("."));
                }
            } else {
                // End of word
                if !current_word.is_empty() {
                    self.push_word_span(
                        &mut spans,
                        &current_word,
                        &keywords,
                        &operators,
                        &linq_methods,
                    );
                    current_word.clear();
                }

                // Handle operators and punctuation
                if operators.contains(&ch.to_string().as_str()) {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::Cyan),
                    ));
                } else if ch == '(' || ch == ')' {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                } else {
                    spans.push(Span::raw(ch.to_string()));
                }
            }

            i += 1;
        }

        // Handle remaining word
        if !current_word.is_empty() {
            if in_string {
                spans.push(Span::styled(
                    current_word,
                    Style::default().fg(Color::Green),
                ));
            } else {
                self.push_word_span(
                    &mut spans,
                    &current_word,
                    &keywords,
                    &operators,
                    &linq_methods,
                );
            }
        }

        Line::from(spans)
    }

    fn push_word_span(
        &self,
        spans: &mut Vec<Span<'static>>,
        word: &str,
        keywords: &[&str],
        operators: &[&str],
        linq_methods: &[&str],
    ) {
        let upper_word = word.to_uppercase();

        if keywords.contains(&upper_word.as_str()) {
            // SQL Keyword
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(Color::Blue),
            ));
        } else if linq_methods.contains(&word) {
            // LINQ Method - use bright magenta/purple
            spans.push(Span::styled(
                word.to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if operators.contains(&word) {
            // Operator
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(Color::Cyan),
            ));
        } else if word.chars().all(|c| c.is_numeric() || c == '.') {
            // Number
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(Color::Magenta),
            ));
        } else {
            // Regular identifier/text
            spans.push(Span::raw(word.to_string()));
        }
    }
}
