// Primary expression parsing
// Handles literals, identifiers, function calls, and parenthesized expressions

use crate::sql::parser::ast::{ColumnRef, SqlExpression, WindowSpec};
use crate::sql::parser::lexer::Token;
use tracing::{debug, trace};

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit, ExpressionParser};

/// Parser context for primary expressions
pub struct PrimaryExpressionContext<'a> {
    pub columns: &'a [String],
    pub in_method_args: bool,
}

impl<'a> Default for PrimaryExpressionContext<'a> {
    fn default() -> Self {
        Self {
            columns: &[],
            in_method_args: false,
        }
    }
}

/// Parse a primary expression (literals, identifiers, functions, parentheses)
/// This is the bottom of the expression hierarchy
pub fn parse_primary<P>(
    parser: &mut P,
    ctx: &PrimaryExpressionContext,
) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    trace_parse_entry("parse_primary", ExpressionParser::current_token(parser));

    // Special case: check if a number literal could actually be a column name
    // This handles cases where columns are named with pure numbers like "202204"
    if let Token::NumberLiteral(num_str) = ExpressionParser::current_token(parser) {
        if ctx.columns.iter().any(|col| col == num_str) {
            log_parse_decision(
                "parse_primary",
                ExpressionParser::current_token(parser),
                "Number literal matches column name, treating as column",
            );
            let expr = SqlExpression::Column(ColumnRef::unquoted(num_str.clone()));
            ExpressionParser::advance(parser);
            let result = Ok(expr);
            trace_parse_exit("parse_primary", &result);
            return result;
        }
    }

    let result = match ExpressionParser::current_token(parser) {
        Token::Case => {
            debug!("Parsing CASE expression");
            parser.parse_case_expression()
        }

        Token::DateTime => {
            debug!("Parsing DateTime constructor");
            parse_datetime_constructor(parser)
        }

        Token::Unnest => {
            debug!("Parsing UNNEST expression");
            parse_unnest(parser)
        }

        Token::Identifier(id) => {
            let id_upper = id.to_uppercase();
            let id_clone = id.clone();

            // Check for boolean literals first
            if id_upper == "TRUE" {
                log_parse_decision(
                    "parse_primary",
                    ExpressionParser::current_token(parser),
                    "Boolean literal TRUE",
                );
                ExpressionParser::advance(parser);
                Ok(SqlExpression::BooleanLiteral(true))
            } else if id_upper == "FALSE" {
                log_parse_decision(
                    "parse_primary",
                    ExpressionParser::current_token(parser),
                    "Boolean literal FALSE",
                );
                ExpressionParser::advance(parser);
                Ok(SqlExpression::BooleanLiteral(false))
            } else {
                ExpressionParser::advance(parser);

                // Check for table.column notation or method calls
                if matches!(ExpressionParser::current_token(parser), Token::Dot) {
                    ExpressionParser::advance(parser); // consume dot

                    if let Token::Identifier(next_id) = ExpressionParser::current_token(parser) {
                        let next_id = next_id.clone();
                        ExpressionParser::advance(parser);

                        // Check if this is a method call (followed by parentheses)
                        if matches!(ExpressionParser::current_token(parser), Token::LeftParen) {
                            debug!(object = %id_clone, method = %next_id, "Parsing method call");
                            ExpressionParser::advance(parser); // consume (

                            // Handle empty argument list
                            let args = if matches!(
                                ExpressionParser::current_token(parser),
                                Token::RightParen
                            ) {
                                Vec::new()
                            } else {
                                parser.parse_expression_list()?
                            };
                            ExpressionParser::consume(parser, Token::RightParen)?;

                            log_parse_decision(
                                "parse_primary",
                                &Token::Identifier(next_id.clone()),
                                "Method call",
                            );
                            Ok(SqlExpression::MethodCall {
                                object: id_clone,
                                method: next_id,
                                args,
                            })
                        } else {
                            // It's a qualified column reference
                            let col_ref = ColumnRef::qualified(id_clone, next_id.clone());
                            log_parse_decision(
                                "parse_primary",
                                &Token::Identifier(next_id),
                                "Qualified column reference",
                            );
                            Ok(SqlExpression::Column(col_ref))
                        }
                    } else {
                        Err("Expected identifier after '.'".to_string())
                    }
                // CAST(expr AS type) / TRY_CAST(expr AS type) — the `AS type`
                // form is not a normal argument list, so intercept it here and
                // lower it into a two-arg function call CAST(expr, 'TYPE').
                } else if (id_upper == "CAST" || id_upper == "TRY_CAST")
                    && matches!(ExpressionParser::current_token(parser), Token::LeftParen)
                {
                    parse_cast_expression(parser, &id_upper)
                // Check if this is a function call
                } else if matches!(ExpressionParser::current_token(parser), Token::LeftParen) {
                    debug!(function = %id_upper, "Parsing function call");
                    ExpressionParser::advance(parser); // consume (
                    let (args, has_distinct) = parser.parse_function_args()?;
                    ExpressionParser::consume(parser, Token::RightParen)?;

                    // Check for OVER clause for window functions
                    if matches!(ExpressionParser::current_token(parser), Token::Over) {
                        debug!(function = %id_upper, "Window function detected");
                        ExpressionParser::advance(parser); // consume OVER
                        ExpressionParser::consume(parser, Token::LeftParen)?;
                        let window_spec = parser.parse_window_spec()?;
                        ExpressionParser::consume(parser, Token::RightParen)?;
                        Ok(SqlExpression::WindowFunction {
                            name: id_upper,
                            args,
                            window_spec,
                        })
                    } else {
                        Ok(SqlExpression::FunctionCall {
                            name: id_upper,
                            args,
                            distinct: has_distinct,
                        })
                    }
                } else {
                    // Otherwise treat as simple column
                    log_parse_decision(
                        "parse_primary",
                        &Token::Identifier(id_clone.clone()),
                        "Column reference",
                    );
                    Ok(SqlExpression::Column(ColumnRef::unquoted(id_clone)))
                }
            }
        }

        Token::QuotedIdentifier(id) => {
            let expr = if ctx.in_method_args {
                // In method arguments, treat quoted identifiers as string literals
                log_parse_decision(
                    "parse_primary",
                    ExpressionParser::current_token(parser),
                    "Quoted identifier in method args - treating as string",
                );
                SqlExpression::StringLiteral(id.clone())
            } else {
                // Otherwise it's a column name like "Customer Id"
                log_parse_decision(
                    "parse_primary",
                    ExpressionParser::current_token(parser),
                    "Quoted identifier as column name",
                );
                SqlExpression::Column(ColumnRef::quoted(id.clone()))
            };
            ExpressionParser::advance(parser);
            Ok(expr)
        }

        Token::StringLiteral(s) => {
            trace!("String literal: {}", s);
            let expr = SqlExpression::StringLiteral(s.clone());
            ExpressionParser::advance(parser);
            Ok(expr)
        }

        Token::NumberLiteral(n) => {
            trace!("Number literal: {}", n);
            let expr = SqlExpression::NumberLiteral(n.clone());
            ExpressionParser::advance(parser);
            Ok(expr)
        }

        Token::Null => {
            trace!("NULL literal");
            ExpressionParser::advance(parser);
            Ok(SqlExpression::Null)
        }

        // Handle LEFT and RIGHT as function names when followed by parentheses
        Token::Left | Token::Right => {
            let func_name = match ExpressionParser::current_token(parser) {
                Token::Left => "LEFT".to_string(),
                Token::Right => "RIGHT".to_string(),
                _ => unreachable!(),
            };

            ExpressionParser::advance(parser);

            // Check if this is a function call
            if matches!(ExpressionParser::current_token(parser), Token::LeftParen) {
                debug!(function = %func_name, "Parsing LEFT/RIGHT function call");
                ExpressionParser::advance(parser); // consume (
                let (args, _has_distinct) = parser.parse_function_args()?;
                ExpressionParser::consume(parser, Token::RightParen)?;

                Ok(SqlExpression::FunctionCall {
                    name: func_name,
                    args,
                    distinct: false,
                })
            } else {
                // If not followed by parenthesis, this is likely a JOIN keyword - error
                Err(format!(
                    "{} keyword unexpected in expression context",
                    func_name
                ))
            }
        }

        Token::LeftParen => {
            debug!("Parsing parenthesized expression or subquery");
            ExpressionParser::advance(parser); // consume (

            // Check if this is a subquery. It starts with SELECT, or with WITH
            // for a CTE in expression position (P12) — parse_subquery() dispatches
            // a leading WITH to the CTE parser, so both forms flow through here.
            if matches!(
                ExpressionParser::current_token(parser),
                Token::Select | Token::With
            ) {
                debug!("Detected subquery - parsing SELECT/WITH statement");
                let subquery = parser.parse_subquery()?;
                ExpressionParser::consume(parser, Token::RightParen)?;
                Ok(SqlExpression::ScalarSubquery {
                    query: Box::new(subquery),
                })
            } else {
                // Parenthesized expression, possibly a tuple for tuple IN:
                // (a, b) IN (SELECT x, y FROM ...)
                let first = parser.parse_logical_or()?;

                if matches!(ExpressionParser::current_token(parser), Token::Comma) {
                    // Collect the remaining tuple elements
                    let mut exprs = vec![first];
                    while matches!(ExpressionParser::current_token(parser), Token::Comma) {
                        ExpressionParser::advance(parser); // consume ,
                        exprs.push(parser.parse_logical_or()?);
                    }
                    ExpressionParser::consume(parser, Token::RightParen)?;

                    // Expect IN or NOT IN immediately after
                    match ExpressionParser::current_token(parser) {
                        Token::In => {
                            ExpressionParser::advance(parser); // consume IN
                            ExpressionParser::consume(parser, Token::LeftParen)?;
                            if !matches!(
                                ExpressionParser::current_token(parser),
                                Token::Select | Token::With
                            ) {
                                return Err("Tuple IN requires a subquery on the right".to_string());
                            }
                            let subquery = parser.parse_subquery()?;
                            ExpressionParser::consume(parser, Token::RightParen)?;
                            Ok(SqlExpression::InSubqueryTuple {
                                exprs,
                                subquery: Box::new(subquery),
                            })
                        }
                        Token::Not => {
                            ExpressionParser::advance(parser); // consume NOT
                            if !matches!(ExpressionParser::current_token(parser), Token::In) {
                                return Err("Expected IN after NOT for tuple".to_string());
                            }
                            ExpressionParser::advance(parser); // consume IN
                            ExpressionParser::consume(parser, Token::LeftParen)?;
                            if !matches!(
                                ExpressionParser::current_token(parser),
                                Token::Select | Token::With
                            ) {
                                return Err(
                                    "Tuple NOT IN requires a subquery on the right".to_string()
                                );
                            }
                            let subquery = parser.parse_subquery()?;
                            ExpressionParser::consume(parser, Token::RightParen)?;
                            Ok(SqlExpression::NotInSubqueryTuple {
                                exprs,
                                subquery: Box::new(subquery),
                            })
                        }
                        _ => Err(
                            "A tuple (expr, expr, ...) may only appear as the left side of IN / NOT IN"
                                .to_string(),
                        ),
                    }
                } else {
                    // Regular parenthesized expression
                    debug!("Regular parenthesized expression");
                    ExpressionParser::consume(parser, Token::RightParen)?;
                    Ok(first)
                }
            }
        }

        Token::Not => {
            debug!("Parsing NOT expression");
            parse_not_expression(parser)
        }

        Token::Star => {
            // Handle * as a literal (like in COUNT(*))
            trace!("Star token as literal");
            ExpressionParser::advance(parser);
            Ok(SqlExpression::StringLiteral("*".to_string()))
        }

        // Handle window-related keywords that can also be column names
        Token::Row => {
            trace!("ROW token treated as identifier in expression context");
            ExpressionParser::advance(parser);
            Ok(SqlExpression::Column(ColumnRef::unquoted(
                "row".to_string(),
            )))
        }

        Token::Rows => {
            trace!("ROWS token treated as identifier in expression context");
            ExpressionParser::advance(parser);
            Ok(SqlExpression::Column(ColumnRef::unquoted(
                "rows".to_string(),
            )))
        }

        Token::Range => {
            trace!("RANGE token treated as identifier in expression context");
            ExpressionParser::advance(parser);
            Ok(SqlExpression::Column(ColumnRef::unquoted(
                "range".to_string(),
            )))
        }

        Token::Minus => {
            // Unary minus: -expr is parsed as 0 - expr
            debug!("Parsing unary minus expression");
            ExpressionParser::advance(parser);
            let operand = parse_primary(parser, ctx)?;
            Ok(SqlExpression::BinaryOp {
                left: Box::new(SqlExpression::NumberLiteral("0".to_string())),
                op: "-".to_string(),
                right: Box::new(operand),
            })
        }

        _ => {
            let err = format!(
                "Unexpected token in primary expression: {:?}",
                ExpressionParser::current_token(parser)
            );
            debug!(error = %err);
            Err(err)
        }
    };

    trace_parse_exit("parse_primary", &result);
    result
}

/// Parse `DATETIME(...)`.
///
/// `DATETIME` is lexed as a keyword rather than an identifier, because
/// `CAST(x AS DATETIME)` needs that type spelling reserved. The cost is that it
/// never reaches the generic function-call arm of `parse_primary`, so it used to
/// be assembled here out of `NumberLiteral` tokens straight into a
/// `DateTimeConstructor` node. That made the components *parse-time constants*:
/// `DATETIME(2024, 1, 15)` worked, but `DATETIME(Year, Month, Day)` failed with
/// "Expected year in DateTime constructor" before evaluation ever began, and no
/// amount of casting helped.
///
/// The registry already carries a `DATETIME` function taking runtime values
/// (`functions::date_time::DateTimeConstructor`, 3-7 args, NULL-propagating), so
/// the fix is simply to stop intercepting: parse an ordinary argument list and
/// let the registry evaluate it. Both paths format as `%Y-%m-%d %H:%M:%S%.3f`,
/// so literal calls are unaffected.
///
/// The no-argument `DATETIME()` (today at midnight) keeps its own node — the
/// registry signature requires at least three arguments, so there is nothing to
/// delegate to.
fn parse_datetime_constructor<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    ExpressionParser::advance(parser); // consume DateTime
    ExpressionParser::consume(parser, Token::LeftParen)?;

    // DATETIME() with no arguments is today's date
    if matches!(ExpressionParser::current_token(parser), Token::RightParen) {
        ExpressionParser::advance(parser); // consume )
        debug!("DateTime() - today's date");
        return Ok(SqlExpression::DateTimeToday {
            hour: None,
            minute: None,
            second: None,
        });
    }

    let (args, _distinct) = parser.parse_function_args()?;
    ExpressionParser::consume(parser, Token::RightParen)?;

    debug!(
        arg_count = args.len(),
        "DATETIME parsed as registry function call"
    );

    Ok(SqlExpression::FunctionCall {
        name: "DATETIME".to_string(),
        args,
        distinct: false,
    })
}

/// Parse NOT expression
fn parse_not_expression<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    ExpressionParser::advance(parser); // consume NOT

    // Check if this is a NOT IN expression
    if let Ok(inner_expr) = parser.parse_comparison() {
        // After parsing the inner expression, check if we're followed by IN
        if matches!(ExpressionParser::current_token(parser), Token::In) {
            debug!("NOT IN expression detected");
            ExpressionParser::advance(parser); // consume IN
            ExpressionParser::consume(parser, Token::LeftParen)?;
            let values = parser.parse_expression_list()?;
            ExpressionParser::consume(parser, Token::RightParen)?;

            Ok(SqlExpression::NotInList {
                expr: Box::new(inner_expr),
                values,
            })
        } else {
            // Regular NOT expression
            debug!("Regular NOT expression");
            Ok(SqlExpression::Not {
                expr: Box::new(inner_expr),
            })
        }
    } else {
        Err("Expected expression after NOT".to_string())
    }
}

/// Parse UNNEST expression
/// Syntax: UNNEST(column_expr, 'delimiter')
fn parse_unnest<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    debug!("parse_unnest: starting");
    ExpressionParser::advance(parser); // consume UNNEST
    ExpressionParser::consume(parser, Token::LeftParen)?;

    // Parse the column expression (first argument)
    let column = parser.parse_logical_or()?;
    debug!("parse_unnest: parsed column expression");

    // Expect comma
    ExpressionParser::consume(parser, Token::Comma)?;

    // Parse the delimiter (second argument - must be a string literal)
    let delimiter = match ExpressionParser::current_token(parser) {
        Token::StringLiteral(s) => {
            let delim = s.clone();
            ExpressionParser::advance(parser);
            delim
        }
        _ => {
            return Err("UNNEST delimiter must be a string literal".to_string());
        }
    };

    debug!(delimiter = %delimiter, "parse_unnest: parsed delimiter");

    ExpressionParser::consume(parser, Token::RightParen)?;

    debug!("parse_unnest: complete");
    Ok(SqlExpression::Unnest {
        column: Box::new(column),
        delimiter,
    })
}

/// Parse a CAST / TRY_CAST expression.
/// Syntax: `CAST(expr AS type)`.
/// The current token on entry is the opening `(`. The result is lowered into a
/// `FunctionCall` so it flows through the existing evaluator and AST machinery:
/// `CAST(expr, 'TYPE')` where the type name is carried as a string literal.
fn parse_cast_expression<P>(parser: &mut P, func_name: &str) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    ExpressionParser::advance(parser); // consume (

    let inner = parser.parse_logical_or()?;

    ExpressionParser::consume(parser, Token::As)?;

    let type_name = parse_cast_type_name(parser)?;

    ExpressionParser::consume(parser, Token::RightParen)?;

    let name = if func_name.eq_ignore_ascii_case("TRY_CAST") {
        "TRY_CAST"
    } else {
        "CAST"
    };

    debug!(target = %type_name, "Parsed CAST expression");
    Ok(SqlExpression::FunctionCall {
        name: name.to_string(),
        args: vec![inner, SqlExpression::StringLiteral(type_name)],
        distinct: false,
    })
}

/// Read a SQL type name for CAST, e.g. `INTEGER`, `VARCHAR`, `DOUBLE`,
/// `TIMESTAMP`. An optional precision/scale specifier such as `DECIMAL(10, 2)`
/// or `VARCHAR(50)` is consumed and discarded — we coerce within our own type
/// confines and do not honour width or scale.
fn parse_cast_type_name<P>(parser: &mut P) -> Result<String, String>
where
    P: ParsePrimary + ExpressionParser + ?Sized,
{
    let type_name = match ExpressionParser::current_token(parser) {
        Token::Identifier(id) => id.clone(),
        // DATETIME is the one type spelling the lexer reserves as a keyword.
        Token::DateTime => "DATETIME".to_string(),
        other => {
            return Err(format!(
                "Expected a type name after AS in CAST, got {other:?}"
            ))
        }
    };
    ExpressionParser::advance(parser);

    // Skip an optional (precision) or (precision, scale) specifier.
    if matches!(ExpressionParser::current_token(parser), Token::LeftParen) {
        ExpressionParser::advance(parser); // consume (
        while !matches!(ExpressionParser::current_token(parser), Token::RightParen) {
            if matches!(ExpressionParser::current_token(parser), Token::Eof) {
                return Err("Unterminated type specifier in CAST".to_string());
            }
            ExpressionParser::advance(parser);
        }
        ExpressionParser::consume(parser, Token::RightParen)?;
    }

    Ok(type_name)
}

/// Trait that parsers must implement to use primary expression parsing
pub trait ParsePrimary {
    fn current_token(&self) -> &Token;
    fn advance(&mut self);
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    // These methods are called from parse_primary
    fn parse_case_expression(&mut self) -> Result<SqlExpression, String>;
    fn parse_function_args(&mut self) -> Result<(Vec<SqlExpression>, bool), String>;
    fn parse_window_spec(&mut self) -> Result<WindowSpec, String>;
    fn parse_logical_or(&mut self) -> Result<SqlExpression, String>;
    fn parse_comparison(&mut self) -> Result<SqlExpression, String>;
    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String>;

    // For subquery parsing (without parenthesis balance validation)
    fn parse_subquery(&mut self) -> Result<crate::sql::parser::ast::SelectStatement, String>;
}
