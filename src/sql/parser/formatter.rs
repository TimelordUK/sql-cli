// SQL Formatter Module
// Handles pretty-printing of SQL queries and AST structures

use super::ast::{LogicalOp, SelectStatement, SortDirection, SqlExpression, WhereClause};
use super::lexer::{Lexer, Token};
use crate::sql::recursive_parser::Parser;

#[must_use]
pub fn format_sql_pretty(query: &str) -> Vec<String> {
    format_sql_pretty_compact(query, 5) // Default to 5 columns per line
}

// Pretty print AST for debug visualization
#[must_use]
pub fn format_ast_tree(query: &str) -> String {
    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(stmt) => format_select_statement(&stmt, 0),
        Err(e) => format!("❌ PARSE ERROR ❌\n{e}\n\n⚠️  The query could not be parsed correctly.\n💡 Check parentheses, operators, and syntax."),
    }
}

fn format_select_statement(stmt: &SelectStatement, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    result.push_str(&format!("{indent_str}SelectStatement {{\n"));

    // Format columns
    result.push_str(&format!("{indent_str}  columns: ["));
    if stmt.columns.is_empty() {
        result.push_str("],\n");
    } else {
        result.push('\n');
        for col in &stmt.columns {
            result.push_str(&format!("{indent_str}    \"{col}\",\n"));
        }
        result.push_str(&format!("{indent_str}  ],\n"));
    }

    // Format from table
    if let Some(table) = &stmt.from_table {
        result.push_str(&format!("{indent_str}  from_table: \"{table}\",\n"));
    }

    // Format where clause
    if let Some(where_clause) = &stmt.where_clause {
        result.push_str(&format!("{indent_str}  where_clause: {{\n"));
        result.push_str(&format_where_clause(where_clause, indent + 2));
        result.push_str(&format!("{indent_str}  }},\n"));
    }

    // Format order by
    if let Some(order_by) = &stmt.order_by {
        result.push_str(&format!("{indent_str}  order_by: ["));
        if order_by.is_empty() {
            result.push_str("],\n");
        } else {
            result.push('\n');
            for col in order_by {
                let dir = match col.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                result.push_str(&format!(
                    "{indent_str}    {{ column: \"{}\", direction: {dir} }},\n",
                    col.column
                ));
            }
            result.push_str(&format!("{indent_str}  ],\n"));
        }
    }

    // Format group by
    if let Some(group_by) = &stmt.group_by {
        result.push_str(&format!("{indent_str}  group_by: ["));
        if group_by.is_empty() {
            result.push_str("],\n");
        } else {
            result.push('\n');
            for expr in group_by {
                result.push_str(&format!("{indent_str}    \"{:?}\",\n", expr));
            }
            result.push_str(&format!("{indent_str}  ],\n"));
        }
    }

    // Format limit
    if let Some(limit) = stmt.limit {
        result.push_str(&format!("{indent_str}  limit: {limit},\n"));
    }

    // Format distinct
    if stmt.distinct {
        result.push_str(&format!("{indent_str}  distinct: true,\n"));
    }

    result.push_str(&format!("{indent_str}}}\n"));
    result
}

fn format_where_clause(clause: &WhereClause, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    result.push_str(&format!("{indent_str}conditions: [\n"));
    for (i, condition) in clause.conditions.iter().enumerate() {
        result.push_str(&format!("{indent_str}  {{\n"));
        result.push_str(&format!(
            "{indent_str}    expr: {},\n",
            format_expression_ast(&condition.expr)
        ));

        if let Some(connector) = &condition.connector {
            let conn_str = match connector {
                LogicalOp::And => "AND",
                LogicalOp::Or => "OR",
            };
            result.push_str(&format!("{indent_str}    connector: {conn_str},\n"));
        }

        result.push_str(&format!("{indent_str}  }}"));
        if i < clause.conditions.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&format!("{indent_str}]\n"));

    result
}

pub fn format_expression_ast(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(name) => format!("Column(\"{name}\")"),
        SqlExpression::StringLiteral(value) => format!("StringLiteral(\"{value}\")"),
        SqlExpression::NumberLiteral(value) => format!("NumberLiteral({value})"),
        SqlExpression::BinaryOp { left, op, right } => {
            format!(
                "BinaryOp {{ left: {}, op: \"{op}\", right: {} }}",
                format_expression_ast(left),
                format_expression_ast(right)
            )
        }
        SqlExpression::FunctionCall {
            name,
            args,
            distinct,
        } => {
            let args_str = args
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            if *distinct {
                format!("FunctionCall {{ name: \"{name}\", args: [{args_str}], distinct: true }}")
            } else {
                format!("FunctionCall {{ name: \"{name}\", args: [{args_str}] }}")
            }
        }
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => {
            let args_str = args
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "MethodCall {{ object: \"{object}\", method: \"{method}\", args: [{args_str}] }}"
            )
        }
        SqlExpression::InList { expr, values } => {
            let values_str = values
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "InList {{ expr: {}, values: [{values_str}] }}",
                format_expression_ast(expr)
            )
        }
        SqlExpression::NotInList { expr, values } => {
            let values_str = values
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "NotInList {{ expr: {}, values: [{values_str}] }}",
                format_expression_ast(expr)
            )
        }
        SqlExpression::Between { expr, lower, upper } => {
            format!(
                "Between {{ expr: {}, lower: {}, upper: {} }}",
                format_expression_ast(expr),
                format_expression_ast(lower),
                format_expression_ast(upper)
            )
        }
        SqlExpression::Null => "Null".to_string(),
        SqlExpression::BooleanLiteral(b) => format!("BooleanLiteral({b})"),
        SqlExpression::DateTimeConstructor {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => {
            let time_part = match (hour, minute, second) {
                (Some(h), Some(m), Some(s)) => format!(" {h:02}:{m:02}:{s:02}"),
                (Some(h), Some(m), None) => format!(" {h:02}:{m:02}"),
                _ => String::new(),
            };
            format!("DateTimeConstructor({year}-{month:02}-{day:02}{time_part})")
        }
        SqlExpression::DateTimeToday {
            hour,
            minute,
            second,
        } => {
            let time_part = match (hour, minute, second) {
                (Some(h), Some(m), Some(s)) => format!(" {h:02}:{m:02}:{s:02}"),
                (Some(h), Some(m), None) => format!(" {h:02}:{m:02}"),
                _ => String::new(),
            };
            format!("DateTimeToday({time_part})")
        }
        SqlExpression::WindowFunction {
            name,
            args,
            window_spec: _,
        } => {
            let args_str = args
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            format!("WindowFunction {{ name: \"{name}\", args: [{args_str}], window_spec: ... }}")
        }
        SqlExpression::ChainedMethodCall { base, method, args } => {
            let args_str = args
                .iter()
                .map(format_expression_ast)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "ChainedMethodCall {{ base: {}, method: \"{method}\", args: [{args_str}] }}",
                format_expression_ast(base)
            )
        }
        SqlExpression::Not { expr } => {
            format!("Not {{ expr: {} }}", format_expression_ast(expr))
        }
        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => {
            let mut result = String::from("CaseExpression { when_branches: [");
            for branch in when_branches {
                result.push_str(&format!(
                    " {{ condition: {}, result: {} }},",
                    format_expression_ast(&branch.condition),
                    format_expression_ast(&branch.result)
                ));
            }
            result.push_str("], else_branch: ");
            if let Some(else_expr) = else_branch {
                result.push_str(&format_expression_ast(else_expr));
            } else {
                result.push_str("None");
            }
            result.push_str(" }");
            result
        }
        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => {
            let mut result = format!(
                "SimpleCaseExpression {{ expr: {}, when_branches: [",
                format_expression_ast(expr)
            );
            for branch in when_branches {
                result.push_str(&format!(
                    " {{ value: {}, result: {} }},",
                    format_expression_ast(&branch.value),
                    format_expression_ast(&branch.result)
                ));
            }
            result.push_str("], else_branch: ");
            if let Some(else_expr) = else_branch {
                result.push_str(&format_expression_ast(else_expr));
            } else {
                result.push_str("None");
            }
            result.push_str(" }");
            result
        }
        SqlExpression::ScalarSubquery { query: _ } => {
            format!("ScalarSubquery {{ query: <SelectStatement> }}")
        }
        SqlExpression::InSubquery { expr, subquery: _ } => {
            format!(
                "InSubquery {{ expr: {}, subquery: <SelectStatement> }}",
                format_expression_ast(expr)
            )
        }
        SqlExpression::NotInSubquery { expr, subquery: _ } => {
            format!(
                "NotInSubquery {{ expr: {}, subquery: <SelectStatement> }}",
                format_expression_ast(expr)
            )
        }
    }
}

// Helper function to extract text between positions
fn extract_text_between_positions(text: &str, start: usize, end: usize) -> String {
    if start >= text.len() || end > text.len() || start >= end {
        return String::new();
    }
    text[start..end].to_string()
}

// Helper to find the position of a specific token in the query
fn find_token_position(query: &str, target: Token, skip_count: usize) -> Option<usize> {
    let mut lexer = Lexer::new(query);
    let mut found_count = 0;

    loop {
        let pos = lexer.get_position();
        let token = lexer.next_token();
        if token == Token::Eof {
            break;
        }
        if token == target {
            if found_count == skip_count {
                return Some(pos);
            }
            found_count += 1;
        }
    }
    None
}

pub fn format_sql_with_preserved_parens(query: &str, cols_per_line: usize) -> Vec<String> {
    let mut parser = Parser::new(query);
    let stmt = match parser.parse() {
        Ok(s) => s,
        Err(_) => return vec![query.to_string()],
    };

    let mut lines = Vec::new();
    let mut lexer = Lexer::new(query);
    let mut tokens_with_pos = Vec::new();

    // Collect all tokens with their positions
    loop {
        let pos = lexer.get_position();
        let token = lexer.next_token();
        if token == Token::Eof {
            break;
        }
        tokens_with_pos.push((token, pos));
    }

    // Process SELECT clause
    let mut i = 0;
    while i < tokens_with_pos.len() {
        match &tokens_with_pos[i].0 {
            Token::Select => {
                let _select_start = tokens_with_pos[i].1;
                i += 1;

                // Check for DISTINCT
                let has_distinct = if i < tokens_with_pos.len() {
                    matches!(tokens_with_pos[i].0, Token::Distinct)
                } else {
                    false
                };

                if has_distinct {
                    i += 1;
                }

                // Find the end of SELECT clause (before FROM)
                let _select_end = query.len();
                let _col_count = 0;
                let _current_line_cols: Vec<String> = Vec::new();
                let mut all_select_lines = Vec::new();

                // Determine if we should use pretty formatting
                let use_pretty_format = stmt.columns.len() > cols_per_line;

                if use_pretty_format {
                    // Multi-line formatting
                    let select_text = if has_distinct {
                        "SELECT DISTINCT".to_string()
                    } else {
                        "SELECT".to_string()
                    };
                    all_select_lines.push(select_text);

                    // Process columns with proper indentation
                    for (idx, col) in stmt.columns.iter().enumerate() {
                        let is_last = idx == stmt.columns.len() - 1;
                        // Check if column needs quotes
                        let formatted_col = if needs_quotes(col) {
                            format!("\"{}\"", col)
                        } else {
                            col.clone()
                        };
                        let col_text = if is_last {
                            format!("    {}", formatted_col)
                        } else {
                            format!("    {},", formatted_col)
                        };
                        all_select_lines.push(col_text);
                    }
                } else {
                    // Single-line formatting for few columns
                    let mut select_line = if has_distinct {
                        "SELECT DISTINCT ".to_string()
                    } else {
                        "SELECT ".to_string()
                    };

                    for (idx, col) in stmt.columns.iter().enumerate() {
                        if idx > 0 {
                            select_line.push_str(", ");
                        }
                        // Check if column needs quotes
                        if needs_quotes(col) {
                            select_line.push_str(&format!("\"{}\"", col));
                        } else {
                            select_line.push_str(col);
                        }
                    }
                    all_select_lines.push(select_line);
                }

                lines.extend(all_select_lines);

                // Skip tokens until we reach FROM
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].0 {
                        Token::From => break,
                        _ => i += 1,
                    }
                }
            }
            Token::From => {
                let from_start = tokens_with_pos[i].1;
                i += 1;

                // Find the end of FROM clause
                let mut from_end = query.len();
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].0 {
                        Token::Where
                        | Token::GroupBy
                        | Token::OrderBy
                        | Token::Limit
                        | Token::Having
                        | Token::Eof => {
                            from_end = tokens_with_pos[i].1;
                            break;
                        }
                        _ => i += 1,
                    }
                }

                let from_text = extract_text_between_positions(query, from_start, from_end);
                lines.push(from_text.trim().to_string());
            }
            Token::Where => {
                let where_start = tokens_with_pos[i].1;
                i += 1;

                // Find the end of WHERE clause
                let mut where_end = query.len();
                let mut paren_depth = 0;
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].0 {
                        Token::LeftParen => {
                            paren_depth += 1;
                            i += 1;
                        }
                        Token::RightParen => {
                            paren_depth -= 1;
                            i += 1;
                        }
                        Token::GroupBy
                        | Token::OrderBy
                        | Token::Limit
                        | Token::Having
                        | Token::Eof
                            if paren_depth == 0 =>
                        {
                            where_end = tokens_with_pos[i].1;
                            break;
                        }
                        _ => i += 1,
                    }
                }

                let where_text = extract_text_between_positions(query, where_start, where_end);
                let formatted_where = format_where_clause_with_parens(where_text.trim());
                lines.extend(formatted_where);
            }
            Token::GroupBy => {
                let group_start = tokens_with_pos[i].1;
                i += 1;

                // Skip BY token
                if i < tokens_with_pos.len() && matches!(tokens_with_pos[i].0, Token::By) {
                    i += 1;
                }

                // Find the end of GROUP BY clause
                while i < tokens_with_pos.len() {
                    match &tokens_with_pos[i].0 {
                        Token::OrderBy | Token::Limit | Token::Having | Token::Eof => break,
                        _ => i += 1,
                    }
                }

                if i > 0 {
                    let group_text = extract_text_between_positions(
                        query,
                        group_start,
                        tokens_with_pos[i - 1].1,
                    );
                    lines.push(format!("GROUP BY {}", group_text.trim()));
                }
            }
            _ => i += 1,
        }
    }

    lines
}

fn format_where_clause_with_parens(where_text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::from("WHERE ");
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = where_text.chars().peekable();

    // Skip "WHERE" if it's at the beginning
    if where_text.trim_start().starts_with("WHERE") || where_text.trim_start().starts_with("where")
    {
        let skip_len = if where_text.trim_start().starts_with("WHERE") {
            5
        } else {
            5
        };
        for _ in 0..skip_len {
            chars.next();
        }
        // Skip whitespace after WHERE
        while chars.peek() == Some(&' ') {
            chars.next();
        }
    }

    while let Some(ch) = chars.next() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                current.push(ch);
                escape_next = true;
            }
            '\'' => {
                current.push(ch);
                in_string = !in_string;
            }
            '(' if !in_string => {
                current.push(ch);
                paren_depth += 1;
            }
            ')' if !in_string => {
                current.push(ch);
                paren_depth -= 1;
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Clean up the result
    let cleaned = current.trim().to_string();
    if !cleaned.is_empty() {
        lines.push(cleaned);
    }

    lines
}

#[must_use]
pub fn format_sql_pretty_compact(query: &str, cols_per_line: usize) -> Vec<String> {
    // Use preserved parentheses formatting
    let formatted = format_sql_with_preserved_parens(query, cols_per_line);

    // Post-process to ensure clean output
    formatted
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // Ensure proper spacing after keywords
            let mut result = line;
            for keyword in &[
                "SELECT", "FROM", "WHERE", "GROUP BY", "ORDER BY", "HAVING", "LIMIT",
            ] {
                let pattern = format!("{keyword}");
                if result.starts_with(&pattern) && !result.starts_with(&format!("{keyword} ")) {
                    result = format!("{keyword} {}", &result[keyword.len()..].trim_start());
                }
            }
            result
        })
        .collect()
}

pub fn format_expression(expr: &SqlExpression) -> String {
    match expr {
        SqlExpression::Column(name) => {
            // Check if column name needs quotes (contains special characters)
            if needs_quotes(name) {
                format!("\"{}\"", name)
            } else {
                name.clone()
            }
        }
        SqlExpression::StringLiteral(value) => format!("'{value}'"),
        SqlExpression::NumberLiteral(value) => value.clone(),
        SqlExpression::BinaryOp { left, op, right } => {
            format!(
                "{} {} {}",
                format_expression(left),
                op,
                format_expression(right)
            )
        }
        SqlExpression::FunctionCall {
            name,
            args,
            distinct,
        } => {
            let args_str = args
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");
            if *distinct {
                format!("{name}(DISTINCT {args_str})")
            } else {
                format!("{name}({args_str})")
            }
        }
        SqlExpression::MethodCall {
            object,
            method,
            args,
        } => {
            let args_str = args
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");
            if args.is_empty() {
                format!("{object}.{method}()")
            } else {
                format!("{object}.{method}({args_str})")
            }
        }
        SqlExpression::InList { expr, values } => {
            let values_str = values
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} IN ({})", format_expression(expr), values_str)
        }
        SqlExpression::NotInList { expr, values } => {
            let values_str = values
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} NOT IN ({})", format_expression(expr), values_str)
        }
        SqlExpression::Between { expr, lower, upper } => {
            format!(
                "{} BETWEEN {} AND {}",
                format_expression(expr),
                format_expression(lower),
                format_expression(upper)
            )
        }
        SqlExpression::Null => "NULL".to_string(),
        SqlExpression::BooleanLiteral(b) => b.to_string().to_uppercase(),
        SqlExpression::DateTimeConstructor {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => {
            let time_part = match (hour, minute, second) {
                (Some(h), Some(m), Some(s)) => format!(" {h:02}:{m:02}:{s:02}"),
                (Some(h), Some(m), None) => format!(" {h:02}:{m:02}"),
                _ => String::new(),
            };
            format!("DATETIME({year}, {month}, {day}{time_part})")
        }
        SqlExpression::DateTimeToday {
            hour,
            minute,
            second,
        } => {
            let time_part = match (hour, minute, second) {
                (Some(h), Some(m), Some(s)) => format!(", {h}, {m}, {s}"),
                (Some(h), Some(m), None) => format!(", {h}, {m}"),
                (Some(h), None, None) => format!(", {h}"),
                _ => String::new(),
            };
            format!("TODAY({time_part})")
        }
        SqlExpression::WindowFunction {
            name,
            args,
            window_spec,
        } => {
            let args_str = args
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");

            let mut result = format!("{name}({args_str}) OVER (");

            // Format partition by
            if !window_spec.partition_by.is_empty() {
                result.push_str("PARTITION BY ");
                result.push_str(&window_spec.partition_by.join(", "));
            }

            // Format order by
            if !window_spec.order_by.is_empty() {
                if !window_spec.partition_by.is_empty() {
                    result.push(' ');
                }
                result.push_str("ORDER BY ");
                let order_strs: Vec<String> = window_spec
                    .order_by
                    .iter()
                    .map(|col| {
                        let dir = match col.direction {
                            SortDirection::Asc => " ASC",
                            SortDirection::Desc => " DESC",
                        };
                        format!("{}{}", col.column, dir)
                    })
                    .collect();
                result.push_str(&order_strs.join(", "));
            }

            result.push(')');
            result
        }
        SqlExpression::ChainedMethodCall { base, method, args } => {
            let base_str = format_expression(base);
            let args_str = args
                .iter()
                .map(format_expression)
                .collect::<Vec<_>>()
                .join(", ");
            if args.is_empty() {
                format!("{base_str}.{method}()")
            } else {
                format!("{base_str}.{method}({args_str})")
            }
        }
        SqlExpression::Not { expr } => {
            format!("NOT {}", format_expression(expr))
        }
        SqlExpression::CaseExpression {
            when_branches,
            else_branch,
        } => format_case_expression(when_branches, else_branch.as_ref().map(|v| &**v)),
        SqlExpression::SimpleCaseExpression {
            expr,
            when_branches,
            else_branch,
        } => format_simple_case_expression(expr, when_branches, else_branch.as_ref().map(|v| &**v)),
        SqlExpression::ScalarSubquery { query: _ } => {
            // For now, just format as a placeholder - proper SQL formatting would need the full query
            "(SELECT ...)".to_string()
        }
        SqlExpression::InSubquery { expr, subquery: _ } => {
            format!("{} IN (SELECT ...)", format_expression(expr))
        }
        SqlExpression::NotInSubquery { expr, subquery: _ } => {
            format!("{} NOT IN (SELECT ...)", format_expression(expr))
        }
    }
}

fn format_token(token: &Token) -> String {
    match token {
        Token::Identifier(s) => s.clone(),
        Token::QuotedIdentifier(s) => format!("\"{s}\""),
        Token::StringLiteral(s) => format!("'{s}'"),
        Token::NumberLiteral(n) => n.clone(),
        Token::DateTime => "DateTime".to_string(),
        Token::Case => "CASE".to_string(),
        Token::When => "WHEN".to_string(),
        Token::Then => "THEN".to_string(),
        Token::Else => "ELSE".to_string(),
        Token::End => "END".to_string(),
        Token::Distinct => "DISTINCT".to_string(),
        Token::Over => "OVER".to_string(),
        Token::Partition => "PARTITION".to_string(),
        Token::By => "BY".to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::Comma => ",".to_string(),
        Token::Dot => ".".to_string(),
        Token::Equal => "=".to_string(),
        Token::NotEqual => "!=".to_string(),
        Token::LessThan => "<".to_string(),
        Token::GreaterThan => ">".to_string(),
        Token::LessThanOrEqual => "<=".to_string(),
        Token::GreaterThanOrEqual => ">=".to_string(),
        Token::In => "IN".to_string(),
        _ => format!("{token:?}").to_uppercase(),
    }
}

// Check if a column name needs quotes (contains special characters or is a reserved word)
fn needs_quotes(name: &str) -> bool {
    // Check for special characters that require quoting
    if name.contains('-') || name.contains(' ') || name.contains('.') || name.contains('/') {
        return true;
    }

    // Check if it starts with a number
    if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return true;
    }

    // Check if it's a SQL reserved word (common ones)
    let reserved_words = [
        "SELECT", "FROM", "WHERE", "ORDER", "GROUP", "BY", "HAVING", "INSERT", "UPDATE", "DELETE",
        "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW", "AND", "OR", "NOT", "IN", "EXISTS",
        "BETWEEN", "LIKE", "CASE", "WHEN", "THEN", "ELSE", "END", "JOIN", "LEFT", "RIGHT", "INNER",
        "OUTER", "ON", "AS", "DISTINCT", "ALL", "TOP", "LIMIT", "OFFSET", "ASC", "DESC",
    ];

    let upper_name = name.to_uppercase();
    if reserved_words.contains(&upper_name.as_str()) {
        return true;
    }

    // Check if all characters are valid for unquoted identifiers
    // Valid: letters, numbers, underscore (but not starting with number)
    !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// Format CASE expressions with proper indentation
fn format_case_expression(
    when_branches: &[crate::sql::recursive_parser::WhenBranch],
    else_branch: Option<&SqlExpression>,
) -> String {
    // Check if the CASE expression is simple enough for single line
    let is_simple = when_branches.len() <= 1
        && when_branches
            .iter()
            .all(|b| expr_is_simple(&b.condition) && expr_is_simple(&b.result))
        && else_branch.map_or(true, expr_is_simple);

    if is_simple {
        // Single line format for simple cases
        let mut result = String::from("CASE");
        for branch in when_branches {
            result.push_str(&format!(
                " WHEN {} THEN {}",
                format_expression(&branch.condition),
                format_expression(&branch.result)
            ));
        }
        if let Some(else_expr) = else_branch {
            result.push_str(&format!(" ELSE {}", format_expression(else_expr)));
        }
        result.push_str(" END");
        result
    } else {
        // Multi-line format for complex cases
        let mut result = String::from("CASE");
        for branch in when_branches {
            result.push_str(&format!(
                "\n        WHEN {} THEN {}",
                format_expression(&branch.condition),
                format_expression(&branch.result)
            ));
        }
        if let Some(else_expr) = else_branch {
            result.push_str(&format!("\n        ELSE {}", format_expression(else_expr)));
        }
        result.push_str("\n    END");
        result
    }
}

// Format simple CASE expressions (CASE expr WHEN val1 THEN result1 ...)
fn format_simple_case_expression(
    expr: &SqlExpression,
    when_branches: &[crate::sql::parser::ast::SimpleWhenBranch],
    else_branch: Option<&SqlExpression>,
) -> String {
    // Check if the CASE expression is simple enough for single line
    let is_simple = when_branches.len() <= 2
        && expr_is_simple(expr)
        && when_branches
            .iter()
            .all(|b| expr_is_simple(&b.value) && expr_is_simple(&b.result))
        && else_branch.map_or(true, expr_is_simple);

    if is_simple {
        // Single line format for simple cases
        let mut result = format!("CASE {}", format_expression(expr));
        for branch in when_branches {
            result.push_str(&format!(
                " WHEN {} THEN {}",
                format_expression(&branch.value),
                format_expression(&branch.result)
            ));
        }
        if let Some(else_expr) = else_branch {
            result.push_str(&format!(" ELSE {}", format_expression(else_expr)));
        }
        result.push_str(" END");
        result
    } else {
        // Multi-line format for complex cases
        let mut result = format!("CASE {}", format_expression(expr));
        for branch in when_branches {
            result.push_str(&format!(
                "\n        WHEN {} THEN {}",
                format_expression(&branch.value),
                format_expression(&branch.result)
            ));
        }
        if let Some(else_expr) = else_branch {
            result.push_str(&format!("\n        ELSE {}", format_expression(else_expr)));
        }
        result.push_str("\n    END");
        result
    }
}

// Check if an expression is simple enough for single-line formatting
fn expr_is_simple(expr: &SqlExpression) -> bool {
    match expr {
        SqlExpression::Column(_)
        | SqlExpression::StringLiteral(_)
        | SqlExpression::NumberLiteral(_)
        | SqlExpression::BooleanLiteral(_)
        | SqlExpression::Null => true,
        SqlExpression::BinaryOp { left, right, .. } => {
            expr_is_simple(left) && expr_is_simple(right)
        }
        SqlExpression::FunctionCall { args, .. } => {
            args.len() <= 2 && args.iter().all(expr_is_simple)
        }
        _ => false,
    }
}
