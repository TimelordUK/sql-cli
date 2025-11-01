//! AST-based SQL Formatter
//!
//! This module provides proper SQL formatting by traversing the parsed AST,
//! which is more reliable than regex-based formatting and handles complex
//! features like CTEs, subqueries, and expressions correctly.

use crate::sql::parser::ast::*;
use std::fmt::Write;

/// Configuration for SQL formatting
pub struct FormatConfig {
    /// Indentation string (e.g., "  " for 2 spaces, "\t" for tab)
    pub indent: String,
    /// Maximum number of items per line for lists (SELECT columns, etc.)
    pub items_per_line: usize,
    /// Whether to uppercase keywords
    pub uppercase_keywords: bool,
    /// Whether to add newlines between major clauses
    pub compact: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            items_per_line: 5,
            uppercase_keywords: true,
            compact: false,
        }
    }
}

/// Format a SELECT statement into pretty SQL
pub fn format_select_statement(stmt: &SelectStatement) -> String {
    format_select_with_config(stmt, &FormatConfig::default())
}

/// Format a SELECT statement with custom configuration
pub fn format_select_with_config(stmt: &SelectStatement, config: &FormatConfig) -> String {
    let formatter = AstFormatter::new(config);
    formatter.format_select(stmt, 0)
}

/// Format a single SQL expression into a string
/// This is useful for extracting and displaying parts of a query
pub fn format_expression(expr: &SqlExpression) -> String {
    let config = FormatConfig::default();
    let formatter = AstFormatter::new(&config);
    formatter.format_expression(expr)
}

struct AstFormatter<'a> {
    config: &'a FormatConfig,
}

impl<'a> AstFormatter<'a> {
    fn new(config: &'a FormatConfig) -> Self {
        Self { config }
    }

    fn keyword(&self, word: &str) -> String {
        if self.config.uppercase_keywords {
            word.to_uppercase()
        } else {
            word.to_lowercase()
        }
    }

    fn indent(&self, level: usize) -> String {
        self.config.indent.repeat(level)
    }

    fn format_select(&self, stmt: &SelectStatement, indent_level: usize) -> String {
        let mut result = String::new();
        let indent = self.indent(indent_level);

        // Emit leading comments if present
        for comment in &stmt.leading_comments {
            self.format_comment(&mut result, comment, &indent);
        }

        // CTEs (WITH clause)
        if !stmt.ctes.is_empty() {
            writeln!(&mut result, "{}{}", indent, self.keyword("WITH")).unwrap();
            for (i, cte) in stmt.ctes.iter().enumerate() {
                let is_last = i == stmt.ctes.len() - 1;
                self.format_cte(&mut result, cte, indent_level + 1, is_last);
            }
        }

        // SELECT clause (with newline if we had leading comments OR CTEs)
        if !stmt.leading_comments.is_empty() || !stmt.ctes.is_empty() {
            writeln!(&mut result, "{}{}", indent, self.keyword("SELECT")).unwrap();
        } else {
            write!(&mut result, "{}{}", indent, self.keyword("SELECT")).unwrap();
        }
        if stmt.distinct {
            write!(&mut result, " {}", self.keyword("DISTINCT")).unwrap();
        }

        // Format select items
        if stmt.select_items.is_empty() && !stmt.columns.is_empty() {
            // Legacy columns field
            self.format_column_list(&mut result, &stmt.columns, indent_level);
        } else {
            self.format_select_items(&mut result, &stmt.select_items, indent_level);
        }

        // INTO clause (for SELECT INTO #temp)
        if let Some(ref into_table) = stmt.into_table {
            writeln!(&mut result).unwrap();
            write!(
                &mut result,
                "{}{} {}",
                indent,
                self.keyword("INTO"),
                into_table.name
            )
            .unwrap();
        }

        // FROM clause
        if let Some(ref table) = stmt.from_table {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} {}", indent, self.keyword("FROM"), table).unwrap();
        } else if let Some(ref subquery) = stmt.from_subquery {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} (", indent, self.keyword("FROM")).unwrap();
            writeln!(&mut result).unwrap();
            let subquery_sql = self.format_select(subquery, indent_level + 1);
            write!(&mut result, "{}", subquery_sql).unwrap();
            write!(&mut result, "\n{}", indent).unwrap();
            write!(&mut result, ")").unwrap();
            if let Some(ref alias) = stmt.from_alias {
                write!(&mut result, " {} {}", self.keyword("AS"), alias).unwrap();
            }
        } else if let Some(ref func) = stmt.from_function {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} ", indent, self.keyword("FROM")).unwrap();
            self.format_table_function(&mut result, func);
            if let Some(ref alias) = stmt.from_alias {
                write!(&mut result, " {} {}", self.keyword("AS"), alias).unwrap();
            }
        }

        // JOIN clauses
        for join in &stmt.joins {
            writeln!(&mut result).unwrap();
            self.format_join(&mut result, join, indent_level);
        }

        // WHERE clause
        if let Some(ref where_clause) = stmt.where_clause {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{}", indent, self.keyword("WHERE")).unwrap();
            self.format_where_clause(&mut result, where_clause, indent_level);
        }

        // GROUP BY clause
        if let Some(ref group_by) = stmt.group_by {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} ", indent, self.keyword("GROUP BY")).unwrap();
            for (i, expr) in group_by.iter().enumerate() {
                if i > 0 {
                    write!(&mut result, ", ").unwrap();
                }
                write!(&mut result, "{}", self.format_expression(expr)).unwrap();
            }
        }

        // HAVING clause
        if let Some(ref having) = stmt.having {
            writeln!(&mut result).unwrap();
            write!(
                &mut result,
                "{}{} {}",
                indent,
                self.keyword("HAVING"),
                self.format_expression(having)
            )
            .unwrap();
        }

        // ORDER BY clause
        if let Some(ref order_by) = stmt.order_by {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} ", indent, self.keyword("ORDER BY")).unwrap();
            for (i, col) in order_by.iter().enumerate() {
                if i > 0 {
                    write!(&mut result, ", ").unwrap();
                }
                write!(&mut result, "{}", self.format_expression(&col.expr)).unwrap();
                match col.direction {
                    SortDirection::Asc => write!(&mut result, " {}", self.keyword("ASC")).unwrap(),
                    SortDirection::Desc => {
                        write!(&mut result, " {}", self.keyword("DESC")).unwrap()
                    }
                }
            }
        }

        // LIMIT clause
        if let Some(limit) = stmt.limit {
            writeln!(&mut result).unwrap();
            write!(&mut result, "{}{} {}", indent, self.keyword("LIMIT"), limit).unwrap();
        }

        // OFFSET clause
        if let Some(offset) = stmt.offset {
            writeln!(&mut result).unwrap();
            write!(
                &mut result,
                "{}{} {}",
                indent,
                self.keyword("OFFSET"),
                offset
            )
            .unwrap();
        }

        // Emit trailing comment if present
        if let Some(ref comment) = stmt.trailing_comment {
            write!(&mut result, "  ").unwrap();
            self.format_inline_comment(&mut result, comment);
        }

        result
    }

    /// Format a comment on its own line with proper indentation
    fn format_comment(&self, result: &mut String, comment: &Comment, indent: &str) {
        if comment.is_line_comment {
            writeln!(result, "{}-- {}", indent, comment.text.trim()).unwrap();
        } else {
            // Block comments
            writeln!(result, "{}/* {} */", indent, comment.text.trim()).unwrap();
        }
    }

    /// Format an inline trailing comment (on the same line as SQL)
    fn format_inline_comment(&self, result: &mut String, comment: &Comment) {
        if comment.is_line_comment {
            write!(result, "-- {}", comment.text.trim()).unwrap();
        } else {
            write!(result, "/* {} */", comment.text.trim()).unwrap();
        }
    }

    fn format_cte(&self, result: &mut String, cte: &CTE, indent_level: usize, is_last: bool) {
        let indent = self.indent(indent_level);

        // Add WEB keyword for Web CTEs
        let is_web = matches!(&cte.cte_type, crate::sql::parser::ast::CTEType::Web(_));
        if is_web {
            write!(result, "{}{} {}", indent, self.keyword("WEB"), cte.name).unwrap();
        } else {
            write!(result, "{}{}", indent, cte.name).unwrap();
        }

        if let Some(ref columns) = cte.column_list {
            write!(result, "(").unwrap();
            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    write!(result, ", ").unwrap();
                }
                write!(result, "{}", col).unwrap();
            }
            write!(result, ")").unwrap();
        }

        writeln!(result, " {} (", self.keyword("AS")).unwrap();
        let cte_sql = match &cte.cte_type {
            crate::sql::parser::ast::CTEType::Standard(query) => {
                self.format_select(query, indent_level + 1)
            }
            crate::sql::parser::ast::CTEType::Web(web_spec) => {
                // Format WEB CTE
                let mut web_str = format!(
                    "{}{} '{}'",
                    "    ".repeat(indent_level + 1),
                    self.keyword("URL"),
                    web_spec.url
                );

                // Add METHOD if specified
                if let Some(method) = &web_spec.method {
                    web_str.push_str(&format!(
                        " {} {}",
                        self.keyword("METHOD"),
                        match method {
                            crate::sql::parser::ast::HttpMethod::GET => "GET",
                            crate::sql::parser::ast::HttpMethod::POST => "POST",
                            crate::sql::parser::ast::HttpMethod::PUT => "PUT",
                            crate::sql::parser::ast::HttpMethod::DELETE => "DELETE",
                            crate::sql::parser::ast::HttpMethod::PATCH => "PATCH",
                        }
                    ));
                }

                // Add BODY if specified
                if let Some(body) = &web_spec.body {
                    // Check if the body looks like JSON (starts with { or [)
                    let trimmed_body = body.trim();
                    if (trimmed_body.starts_with('{') && trimmed_body.ends_with('}'))
                        || (trimmed_body.starts_with('[') && trimmed_body.ends_with(']'))
                    {
                        // Try to prettify JSON
                        match serde_json::from_str::<serde_json::Value>(trimmed_body) {
                            Ok(json_val) => {
                                // Pretty print JSON with 2-space indentation
                                match serde_json::to_string_pretty(&json_val) {
                                    Ok(pretty_json) => {
                                        // Check if JSON is complex (multiline or has special chars)
                                        let is_complex = pretty_json.lines().count() > 1
                                            || pretty_json.contains('"')
                                            || pretty_json.contains('\\');

                                        if is_complex {
                                            // Use $JSON$ delimiters for complex JSON
                                            let base_indent = "    ".repeat(indent_level + 1);
                                            let json_lines: Vec<String> = pretty_json
                                                .lines()
                                                .enumerate()
                                                .map(|(i, line)| {
                                                    if i == 0 {
                                                        line.to_string()
                                                    } else {
                                                        format!("{}{}", base_indent, line)
                                                    }
                                                })
                                                .collect();
                                            let formatted_json = json_lines.join("\n");

                                            web_str.push_str(&format!(
                                                " {} $JSON${}\n{}$JSON$\n{}",
                                                self.keyword("BODY"),
                                                formatted_json,
                                                base_indent,
                                                base_indent
                                            ));
                                        } else {
                                            // Simple JSON, use regular single quotes
                                            web_str.push_str(&format!(
                                                " {} '{}'",
                                                self.keyword("BODY"),
                                                pretty_json
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        // Fall back to original if pretty print fails
                                        web_str.push_str(&format!(
                                            " {} '{}'",
                                            self.keyword("BODY"),
                                            body
                                        ));
                                    }
                                }
                            }
                            Err(_) => {
                                // Not valid JSON, use as-is
                                web_str.push_str(&format!(" {} '{}'", self.keyword("BODY"), body));
                            }
                        }
                    } else {
                        // Not JSON, use as-is
                        web_str.push_str(&format!(" {} '{}'", self.keyword("BODY"), body));
                    }
                }

                // Add FORMAT if specified
                if let Some(format) = &web_spec.format {
                    web_str.push_str(&format!(
                        " {} {}",
                        self.keyword("FORMAT"),
                        match format {
                            crate::sql::parser::ast::DataFormat::CSV => "CSV",
                            crate::sql::parser::ast::DataFormat::JSON => "JSON",
                            crate::sql::parser::ast::DataFormat::Auto => "AUTO",
                        }
                    ));
                }

                // Add JSON_PATH if specified
                if let Some(json_path) = &web_spec.json_path {
                    web_str.push_str(&format!(" {} '{}'", self.keyword("JSON_PATH"), json_path));
                }

                // Add CACHE if specified
                if let Some(cache) = web_spec.cache_seconds {
                    web_str.push_str(&format!(" {} {}", self.keyword("CACHE"), cache));
                }

                // Add FORM_FILE entries if specified
                for (field_name, file_path) in &web_spec.form_files {
                    web_str.push_str(&format!(
                        "\n{}{} '{}' '{}'",
                        "    ".repeat(indent_level + 1),
                        self.keyword("FORM_FILE"),
                        field_name,
                        file_path
                    ));
                }

                // Add FORM_FIELD entries if specified
                for (field_name, value) in &web_spec.form_fields {
                    // Check if the value looks like JSON (starts with { or [)
                    let trimmed_value = value.trim();
                    if (trimmed_value.starts_with('{') && trimmed_value.ends_with('}'))
                        || (trimmed_value.starts_with('[') && trimmed_value.ends_with(']'))
                    {
                        // Try to prettify JSON
                        match serde_json::from_str::<serde_json::Value>(trimmed_value) {
                            Ok(json_val) => {
                                // Pretty print JSON with 2-space indentation
                                match serde_json::to_string_pretty(&json_val) {
                                    Ok(pretty_json) => {
                                        // Check if JSON is complex (multiline or has special chars)
                                        let is_complex = pretty_json.lines().count() > 1
                                            || pretty_json.contains('"')
                                            || pretty_json.contains('\\');

                                        if is_complex {
                                            // Use $JSON$ delimiters for complex JSON
                                            let base_indent = "    ".repeat(indent_level + 1);
                                            let json_lines: Vec<String> = pretty_json
                                                .lines()
                                                .enumerate()
                                                .map(|(i, line)| {
                                                    if i == 0 {
                                                        line.to_string()
                                                    } else {
                                                        format!("{}{}", base_indent, line)
                                                    }
                                                })
                                                .collect();
                                            let formatted_json = json_lines.join("\n");

                                            web_str.push_str(&format!(
                                                "\n{}{} '{}' $JSON${}\n{}$JSON$",
                                                base_indent,
                                                self.keyword("FORM_FIELD"),
                                                field_name,
                                                formatted_json,
                                                base_indent
                                            ));
                                        } else {
                                            // Simple JSON, use regular single quotes
                                            web_str.push_str(&format!(
                                                "\n{}{} '{}' '{}'",
                                                "    ".repeat(indent_level + 1),
                                                self.keyword("FORM_FIELD"),
                                                field_name,
                                                pretty_json
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        // Fall back to original if pretty print fails
                                        web_str.push_str(&format!(
                                            "\n{}{} '{}' '{}'",
                                            "    ".repeat(indent_level + 1),
                                            self.keyword("FORM_FIELD"),
                                            field_name,
                                            value
                                        ));
                                    }
                                }
                            }
                            Err(_) => {
                                // Not valid JSON, use as-is
                                web_str.push_str(&format!(
                                    "\n{}{} '{}' '{}'",
                                    "    ".repeat(indent_level + 1),
                                    self.keyword("FORM_FIELD"),
                                    field_name,
                                    value
                                ));
                            }
                        }
                    } else {
                        // Not JSON, use as-is
                        web_str.push_str(&format!(
                            "\n{}{} '{}' '{}'",
                            "    ".repeat(indent_level + 1),
                            self.keyword("FORM_FIELD"),
                            field_name,
                            value
                        ));
                    }
                }

                // Add HEADERS if specified
                if !web_spec.headers.is_empty() {
                    web_str.push_str(&format!(" {} (", self.keyword("HEADERS")));
                    for (i, (key, value)) in web_spec.headers.iter().enumerate() {
                        if i > 0 {
                            web_str.push_str(", ");
                        }
                        web_str.push_str(&format!("'{}': '{}'", key, value));
                    }
                    web_str.push(')');
                }

                web_str
            }
        };
        write!(result, "{}", cte_sql).unwrap();
        writeln!(result).unwrap();
        write!(result, "{}", indent).unwrap();
        if is_last {
            writeln!(result, ")").unwrap();
        } else {
            writeln!(result, "),").unwrap();
        }
    }

    fn format_column_list(&self, result: &mut String, columns: &[String], indent_level: usize) {
        if columns.len() <= self.config.items_per_line {
            // Single line
            write!(result, " ").unwrap();
            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    write!(result, ", ").unwrap();
                }
                write!(result, "{}", col).unwrap();
            }
        } else {
            // Multi-line
            writeln!(result).unwrap();
            let indent = self.indent(indent_level + 1);
            for (i, col) in columns.iter().enumerate() {
                write!(result, "{}{}", indent, col).unwrap();
                if i < columns.len() - 1 {
                    writeln!(result, ",").unwrap();
                }
            }
        }
    }

    fn format_select_items(&self, result: &mut String, items: &[SelectItem], indent_level: usize) {
        if items.is_empty() {
            write!(result, " *").unwrap();
            return;
        }

        // Count non-star items for formatting decision
        let _non_star_count = items
            .iter()
            .filter(|i| !matches!(i, SelectItem::Star { .. }))
            .count();

        // Check if any item is complex (function calls, CASE expressions, etc.)
        let has_complex_items = items.iter().any(|item| match item {
            SelectItem::Expression { expr, .. } => self.is_complex_expression(expr),
            _ => false,
        });

        // Calculate total approximate length if on single line
        let single_line_length: usize = items
            .iter()
            .map(|item| {
                match item {
                    SelectItem::Star { .. } => 1,
                    SelectItem::Column { column: col, .. } => col.name.len(),
                    SelectItem::Expression { expr, alias, .. } => {
                        self.format_expression(expr).len() + 4 + alias.len() // " AS " = 4
                    }
                }
            })
            .sum::<usize>()
            + (items.len() - 1) * 2; // ", " between items

        // Use multi-line formatting by default unless:
        // - It's a single simple column or star
        // - It's 2-3 simple columns with total length < 40 chars
        let use_single_line = match items.len() {
            1 => !has_complex_items, // Single item: only if simple
            2..=3 => !has_complex_items && single_line_length < 40, // 2-3 items: only if very short
            _ => false,              // 4+ items: always multi-line
        };

        if !use_single_line {
            // Multi-line
            writeln!(result).unwrap();
            let indent = self.indent(indent_level + 1);
            for (i, item) in items.iter().enumerate() {
                write!(result, "{}", indent).unwrap();
                self.format_select_item(result, item);
                if i < items.len() - 1 {
                    writeln!(result, ",").unwrap();
                }
            }
        } else {
            // Single line
            write!(result, " ").unwrap();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    write!(result, ", ").unwrap();
                }
                self.format_select_item(result, item);
            }
        }
    }

    fn is_complex_expression(&self, expr: &SqlExpression) -> bool {
        match expr {
            SqlExpression::CaseExpression { .. } => true,
            SqlExpression::FunctionCall { .. } => true,
            SqlExpression::WindowFunction { .. } => true,
            SqlExpression::ScalarSubquery { .. } => true,
            SqlExpression::InSubquery { .. } => true,
            SqlExpression::NotInSubquery { .. } => true,
            SqlExpression::BinaryOp { left, right, .. } => {
                self.is_complex_expression(left) || self.is_complex_expression(right)
            }
            _ => false,
        }
    }

    fn format_select_item(&self, result: &mut String, item: &SelectItem) {
        match item {
            SelectItem::Star { .. } => write!(result, "*").unwrap(),
            SelectItem::Column { column: col, .. } => write!(result, "{}", col.to_sql()).unwrap(),
            SelectItem::Expression { expr, alias, .. } => {
                write!(
                    result,
                    "{} {} {}",
                    self.format_expression(expr),
                    self.keyword("AS"),
                    alias
                )
                .unwrap();
            }
        }
    }

    fn format_expression(&self, expr: &SqlExpression) -> String {
        match expr {
            SqlExpression::Column(column_ref) => column_ref.to_sql(),
            SqlExpression::StringLiteral(s) => format!("'{}'", s),
            SqlExpression::NumberLiteral(n) => n.clone(),
            SqlExpression::BooleanLiteral(b) => b.to_string().to_uppercase(),
            SqlExpression::Null => self.keyword("NULL"),
            SqlExpression::BinaryOp { left, op, right } => {
                // Special handling for IS NULL / IS NOT NULL operators
                if op == "IS NULL" || op == "IS NOT NULL" {
                    format!("{} {}", self.format_expression(left), op)
                } else {
                    format!(
                        "{} {} {}",
                        self.format_expression(left),
                        op,
                        self.format_expression(right)
                    )
                }
            }
            SqlExpression::FunctionCall {
                name,
                args,
                distinct,
            } => {
                let mut result = name.clone();
                result.push('(');
                if *distinct {
                    result.push_str(&self.keyword("DISTINCT"));
                    result.push(' ');
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg));
                }
                result.push(')');
                result
            }
            SqlExpression::CaseExpression {
                when_branches,
                else_branch,
            } => {
                // Format CASE expressions on multiple lines for readability
                let mut result = String::new();
                result.push_str(&self.keyword("CASE"));
                result.push('\n');

                // Format each WHEN branch on its own line with indentation
                for branch in when_branches {
                    result.push_str("        "); // 8 spaces for WHEN indent
                    result.push_str(&format!(
                        "{} {} {} {}",
                        self.keyword("WHEN"),
                        self.format_expression(&branch.condition),
                        self.keyword("THEN"),
                        self.format_expression(&branch.result)
                    ));
                    result.push('\n');
                }

                // Format ELSE clause if present
                if let Some(else_expr) = else_branch {
                    result.push_str("        "); // 8 spaces for ELSE indent
                    result.push_str(&format!(
                        "{} {}",
                        self.keyword("ELSE"),
                        self.format_expression(else_expr)
                    ));
                    result.push('\n');
                }

                result.push_str("    "); // 4 spaces for END
                result.push_str(&self.keyword("END"));
                result
            }
            SqlExpression::SimpleCaseExpression {
                expr,
                when_branches,
                else_branch,
            } => {
                // Format simple CASE expressions on multiple lines for readability
                let mut result = String::new();
                result.push_str(&format!(
                    "{} {}",
                    self.keyword("CASE"),
                    self.format_expression(expr)
                ));
                result.push('\n');

                // Format each WHEN branch on its own line with indentation
                for branch in when_branches {
                    result.push_str("        "); // 8 spaces for WHEN indent
                    result.push_str(&format!(
                        "{} {} {} {}",
                        self.keyword("WHEN"),
                        self.format_expression(&branch.value),
                        self.keyword("THEN"),
                        self.format_expression(&branch.result)
                    ));
                    result.push('\n');
                }

                // Format ELSE clause if present
                if let Some(else_expr) = else_branch {
                    result.push_str("        "); // 8 spaces for ELSE indent
                    result.push_str(&format!(
                        "{} {}",
                        self.keyword("ELSE"),
                        self.format_expression(else_expr)
                    ));
                    result.push('\n');
                }

                result.push_str("    "); // 4 spaces for END
                result.push_str(&self.keyword("END"));
                result
            }
            SqlExpression::Between { expr, lower, upper } => {
                format!(
                    "{} {} {} {} {}",
                    self.format_expression(expr),
                    self.keyword("BETWEEN"),
                    self.format_expression(lower),
                    self.keyword("AND"),
                    self.format_expression(upper)
                )
            }
            SqlExpression::InList { expr, values } => {
                let mut result =
                    format!("{} {} (", self.format_expression(expr), self.keyword("IN"));
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(val));
                }
                result.push(')');
                result
            }
            SqlExpression::NotInList { expr, values } => {
                let mut result = format!(
                    "{} {} {} (",
                    self.format_expression(expr),
                    self.keyword("NOT"),
                    self.keyword("IN")
                );
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(val));
                }
                result.push(')');
                result
            }
            SqlExpression::Not { expr } => {
                format!("{} {}", self.keyword("NOT"), self.format_expression(expr))
            }
            SqlExpression::ScalarSubquery { query } => {
                // Check if subquery is complex enough to warrant multi-line formatting
                let subquery_str = self.format_select(query, 0);
                if subquery_str.contains('\n') || subquery_str.len() > 60 {
                    // Multi-line formatting
                    format!("(\n{}\n)", self.format_select(query, 1))
                } else {
                    // Inline formatting
                    format!("({})", subquery_str)
                }
            }
            SqlExpression::InSubquery { expr, subquery } => {
                let subquery_str = self.format_select(subquery, 0);
                if subquery_str.contains('\n') || subquery_str.len() > 60 {
                    // Multi-line formatting
                    format!(
                        "{} {} (\n{}\n)",
                        self.format_expression(expr),
                        self.keyword("IN"),
                        self.format_select(subquery, 1)
                    )
                } else {
                    // Inline formatting
                    format!(
                        "{} {} ({})",
                        self.format_expression(expr),
                        self.keyword("IN"),
                        subquery_str
                    )
                }
            }
            SqlExpression::NotInSubquery { expr, subquery } => {
                let subquery_str = self.format_select(subquery, 0);
                if subquery_str.contains('\n') || subquery_str.len() > 60 {
                    // Multi-line formatting
                    format!(
                        "{} {} {} (\n{}\n)",
                        self.format_expression(expr),
                        self.keyword("NOT"),
                        self.keyword("IN"),
                        self.format_select(subquery, 1)
                    )
                } else {
                    // Inline formatting
                    format!(
                        "{} {} {} ({})",
                        self.format_expression(expr),
                        self.keyword("NOT"),
                        self.keyword("IN"),
                        subquery_str
                    )
                }
            }
            SqlExpression::MethodCall {
                object,
                method,
                args,
            } => {
                let mut result = format!("{}.{}", object, method);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg));
                }
                result.push(')');
                result
            }
            SqlExpression::ChainedMethodCall { base, method, args } => {
                let mut result = format!("{}.{}", self.format_expression(base), method);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg));
                }
                result.push(')');
                result
            }
            SqlExpression::WindowFunction {
                name,
                args,
                window_spec,
            } => {
                let mut result = format!("{}(", name);

                // Add function arguments
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg));
                }
                result.push_str(") ");
                result.push_str(&self.keyword("OVER"));
                result.push_str(" (");

                // Add PARTITION BY clause if present
                if !window_spec.partition_by.is_empty() {
                    result.push_str(&self.keyword("PARTITION BY"));
                    result.push(' ');
                    for (i, col) in window_spec.partition_by.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(col);
                    }
                }

                // Add ORDER BY clause if present
                if !window_spec.order_by.is_empty() {
                    if !window_spec.partition_by.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&self.keyword("ORDER BY"));
                    result.push(' ');
                    for (i, col) in window_spec.order_by.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&self.format_expression(&col.expr));
                        match col.direction {
                            SortDirection::Asc => {
                                result.push(' ');
                                result.push_str(&self.keyword("ASC"));
                            }
                            SortDirection::Desc => {
                                result.push(' ');
                                result.push_str(&self.keyword("DESC"));
                            }
                        }
                    }
                }

                // Add window frame specification if present
                if let Some(frame) = &window_spec.frame {
                    // Add space before frame specification
                    if !window_spec.partition_by.is_empty() || !window_spec.order_by.is_empty() {
                        result.push(' ');
                    }

                    // Format frame unit (ROWS or RANGE)
                    match frame.unit {
                        FrameUnit::Rows => result.push_str(&self.keyword("ROWS")),
                        FrameUnit::Range => result.push_str(&self.keyword("RANGE")),
                    }

                    result.push(' ');

                    // Format frame bounds
                    if let Some(end) = &frame.end {
                        // BETWEEN start AND end
                        result.push_str(&self.keyword("BETWEEN"));
                        result.push(' ');
                        result.push_str(&self.format_frame_bound(&frame.start));
                        result.push(' ');
                        result.push_str(&self.keyword("AND"));
                        result.push(' ');
                        result.push_str(&self.format_frame_bound(end));
                    } else {
                        // Just a single bound (uncommon but valid)
                        result.push_str(&self.format_frame_bound(&frame.start));
                    }
                }

                result.push(')');
                result
            }
            SqlExpression::DateTimeConstructor {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                if let (Some(h), Some(m), Some(s)) = (hour, minute, second) {
                    format!(
                        "DateTime({}, {}, {}, {}, {}, {})",
                        year, month, day, h, m, s
                    )
                } else {
                    format!("DateTime({}, {}, {})", year, month, day)
                }
            }
            SqlExpression::DateTimeToday {
                hour,
                minute,
                second,
            } => {
                if let (Some(h), Some(m), Some(s)) = (hour, minute, second) {
                    format!("Today({}, {}, {})", h, m, s)
                } else {
                    "Today()".to_string()
                }
            }
            _ => format!("{:?}", expr), // Fallback for unhandled expression types
        }
    }

    fn format_where_clause(
        &self,
        result: &mut String,
        where_clause: &WhereClause,
        indent_level: usize,
    ) {
        let needs_multiline = where_clause.conditions.len() > 1;

        if needs_multiline {
            writeln!(result).unwrap();
            let indent = self.indent(indent_level + 1);
            for (i, condition) in where_clause.conditions.iter().enumerate() {
                if i > 0 {
                    if let Some(ref connector) = where_clause.conditions[i - 1].connector {
                        let connector_str = match connector {
                            LogicalOp::And => self.keyword("AND"),
                            LogicalOp::Or => self.keyword("OR"),
                        };
                        writeln!(result).unwrap();
                        write!(result, "{}{} ", indent, connector_str).unwrap();
                    }
                } else {
                    write!(result, "{}", indent).unwrap();
                }
                write!(result, "{}", self.format_expression(&condition.expr)).unwrap();
            }
        } else if let Some(condition) = where_clause.conditions.first() {
            write!(result, " {}", self.format_expression(&condition.expr)).unwrap();
        }
    }

    fn format_frame_bound(&self, bound: &FrameBound) -> String {
        match bound {
            FrameBound::UnboundedPreceding => self.keyword("UNBOUNDED PRECEDING"),
            FrameBound::CurrentRow => self.keyword("CURRENT ROW"),
            FrameBound::UnboundedFollowing => self.keyword("UNBOUNDED FOLLOWING"),
            FrameBound::Preceding(n) => format!("{} {}", n, self.keyword("PRECEDING")),
            FrameBound::Following(n) => format!("{} {}", n, self.keyword("FOLLOWING")),
        }
    }

    fn format_join(&self, result: &mut String, join: &JoinClause, indent_level: usize) {
        let indent = self.indent(indent_level);
        let join_type = match join.join_type {
            JoinType::Inner => self.keyword("INNER JOIN"),
            JoinType::Left => self.keyword("LEFT JOIN"),
            JoinType::Right => self.keyword("RIGHT JOIN"),
            JoinType::Full => self.keyword("FULL JOIN"),
            JoinType::Cross => self.keyword("CROSS JOIN"),
        };

        write!(result, "{}{} ", indent, join_type).unwrap();

        match &join.table {
            TableSource::Table(name) => write!(result, "{}", name).unwrap(),
            TableSource::DerivedTable { query, alias } => {
                writeln!(result, "(").unwrap();
                let subquery_sql = self.format_select(query, indent_level + 1);
                write!(result, "{}", subquery_sql).unwrap();
                writeln!(result).unwrap();
                write!(result, "{}) {} {}", indent, self.keyword("AS"), alias).unwrap();
            }
        }

        if let Some(ref alias) = join.alias {
            write!(result, " {} {}", self.keyword("AS"), alias).unwrap();
        }

        if !join.condition.conditions.is_empty() {
            write!(result, " {}", self.keyword("ON")).unwrap();
            for (i, condition) in join.condition.conditions.iter().enumerate() {
                if i > 0 {
                    write!(result, " {}", self.keyword("AND")).unwrap();
                }
                write!(
                    result,
                    " {} {} {}",
                    self.format_expression(&condition.left_expr),
                    self.format_join_operator(&condition.operator),
                    self.format_expression(&condition.right_expr)
                )
                .unwrap();
            }
        }
    }

    fn format_join_operator(&self, op: &JoinOperator) -> String {
        match op {
            JoinOperator::Equal => "=",
            JoinOperator::NotEqual => "!=",
            JoinOperator::LessThan => "<",
            JoinOperator::GreaterThan => ">",
            JoinOperator::LessThanOrEqual => "<=",
            JoinOperator::GreaterThanOrEqual => ">=",
        }
        .to_string()
    }

    fn format_table_function(&self, result: &mut String, func: &TableFunction) {
        match func {
            TableFunction::Generator { name, args } => {
                write!(result, "{}(", self.keyword(&name.to_uppercase())).unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(result, ", ").unwrap();
                    }
                    write!(result, "{}", self.format_expression(arg)).unwrap();
                }
                write!(result, ")").unwrap();
            }
        }
    }
}

/// Parse and format SQL query using the AST
pub fn format_sql_ast(query: &str) -> Result<String, String> {
    use crate::sql::recursive_parser::Parser;

    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => Ok(format_select_statement(&stmt)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Parse and format SQL with custom configuration
pub fn format_sql_ast_with_config(query: &str, config: &FormatConfig) -> Result<String, String> {
    use crate::sql::recursive_parser::Parser;

    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => Ok(format_select_with_config(&stmt, &config)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
