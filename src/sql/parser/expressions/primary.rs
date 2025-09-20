// Primary expression parsing
// Handles literals, identifiers, function calls, and parenthesized expressions

use crate::sql::parser::ast::{ColumnRef, SqlExpression, WindowSpec};
use crate::sql::parser::lexer::Token;
use tracing::{debug, trace};

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

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
    P: ParsePrimary + ?Sized,
{
    trace_parse_entry("parse_primary", parser.current_token());

    // Special case: check if a number literal could actually be a column name
    // This handles cases where columns are named with pure numbers like "202204"
    if let Token::NumberLiteral(num_str) = parser.current_token() {
        if ctx.columns.iter().any(|col| col == num_str) {
            log_parse_decision(
                "parse_primary",
                parser.current_token(),
                "Number literal matches column name, treating as column",
            );
            let expr = SqlExpression::Column(ColumnRef::unquoted(num_str.clone()));
            parser.advance();
            let result = Ok(expr);
            trace_parse_exit("parse_primary", &result);
            return result;
        }
    }

    let result = match parser.current_token() {
        Token::Case => {
            debug!("Parsing CASE expression");
            parser.parse_case_expression()
        }

        Token::DateTime => {
            debug!("Parsing DateTime constructor");
            parse_datetime_constructor(parser)
        }

        Token::Identifier(id) => {
            let id_upper = id.to_uppercase();
            let id_clone = id.clone();

            // Check for boolean literals first
            if id_upper == "TRUE" {
                log_parse_decision(
                    "parse_primary",
                    parser.current_token(),
                    "Boolean literal TRUE",
                );
                parser.advance();
                Ok(SqlExpression::BooleanLiteral(true))
            } else if id_upper == "FALSE" {
                log_parse_decision(
                    "parse_primary",
                    parser.current_token(),
                    "Boolean literal FALSE",
                );
                parser.advance();
                Ok(SqlExpression::BooleanLiteral(false))
            } else {
                parser.advance();

                // Check if this is a function call
                if matches!(parser.current_token(), Token::LeftParen) {
                    debug!(function = %id_upper, "Parsing function call");
                    parser.advance(); // consume (
                    let (args, has_distinct) = parser.parse_function_args()?;
                    parser.consume(Token::RightParen)?;

                    // Check for OVER clause for window functions
                    if matches!(parser.current_token(), Token::Over) {
                        debug!(function = %id_upper, "Window function detected");
                        parser.advance(); // consume OVER
                        parser.consume(Token::LeftParen)?;
                        let window_spec = parser.parse_window_spec()?;
                        parser.consume(Token::RightParen)?;
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
                    // Otherwise treat as column
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
                    parser.current_token(),
                    "Quoted identifier in method args - treating as string",
                );
                SqlExpression::StringLiteral(id.clone())
            } else {
                // Otherwise it's a column name like "Customer Id"
                log_parse_decision(
                    "parse_primary",
                    parser.current_token(),
                    "Quoted identifier as column name",
                );
                SqlExpression::Column(ColumnRef::quoted(id.clone()))
            };
            parser.advance();
            Ok(expr)
        }

        Token::StringLiteral(s) => {
            trace!("String literal: {}", s);
            let expr = SqlExpression::StringLiteral(s.clone());
            parser.advance();
            Ok(expr)
        }

        Token::NumberLiteral(n) => {
            trace!("Number literal: {}", n);
            let expr = SqlExpression::NumberLiteral(n.clone());
            parser.advance();
            Ok(expr)
        }

        Token::Null => {
            trace!("NULL literal");
            parser.advance();
            Ok(SqlExpression::Null)
        }

        // Handle LEFT and RIGHT as function names when followed by parentheses
        Token::Left | Token::Right => {
            let func_name = match parser.current_token() {
                Token::Left => "LEFT".to_string(),
                Token::Right => "RIGHT".to_string(),
                _ => unreachable!(),
            };

            parser.advance();

            // Check if this is a function call
            if matches!(parser.current_token(), Token::LeftParen) {
                debug!(function = %func_name, "Parsing LEFT/RIGHT function call");
                parser.advance(); // consume (
                let (args, _has_distinct) = parser.parse_function_args()?;
                parser.consume(Token::RightParen)?;

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
            parser.advance(); // consume (

            // Check if this is a subquery (starts with SELECT)
            if matches!(parser.current_token(), Token::Select) {
                debug!("Detected subquery - parsing SELECT statement");
                let subquery = parser.parse_subquery()?;
                parser.consume(Token::RightParen)?;
                Ok(SqlExpression::ScalarSubquery {
                    query: Box::new(subquery),
                })
            } else {
                // Regular parenthesized expression
                debug!("Regular parenthesized expression");
                let expr = parser.parse_logical_or()?;
                parser.consume(Token::RightParen)?;
                Ok(expr)
            }
        }

        Token::Not => {
            debug!("Parsing NOT expression");
            parse_not_expression(parser)
        }

        Token::Star => {
            // Handle * as a literal (like in COUNT(*))
            trace!("Star token as literal");
            parser.advance();
            Ok(SqlExpression::StringLiteral("*".to_string()))
        }

        _ => {
            let err = format!(
                "Unexpected token in primary expression: {:?}",
                parser.current_token()
            );
            debug!(error = %err);
            Err(err)
        }
    };

    trace_parse_exit("parse_primary", &result);
    result
}

/// Parse DateTime constructor
fn parse_datetime_constructor<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ?Sized,
{
    parser.advance(); // consume DateTime
    parser.consume(Token::LeftParen)?;

    // Check if empty parentheses for DateTime() - today's date
    if matches!(parser.current_token(), Token::RightParen) {
        parser.advance(); // consume )
        debug!("DateTime() - today's date");
        return Ok(SqlExpression::DateTimeToday {
            hour: None,
            minute: None,
            second: None,
        });
    }

    // Parse year
    let year = if let Token::NumberLiteral(n) = parser.current_token() {
        n.parse::<i32>().map_err(|_| "Invalid year")?
    } else {
        return Err("Expected year in DateTime constructor".to_string());
    };
    parser.advance();
    parser.consume(Token::Comma)?;

    // Parse month
    let month = if let Token::NumberLiteral(n) = parser.current_token() {
        n.parse::<u32>().map_err(|_| "Invalid month")?
    } else {
        return Err("Expected month in DateTime constructor".to_string());
    };
    parser.advance();
    parser.consume(Token::Comma)?;

    // Parse day
    let day = if let Token::NumberLiteral(n) = parser.current_token() {
        n.parse::<u32>().map_err(|_| "Invalid day")?
    } else {
        return Err("Expected day in DateTime constructor".to_string());
    };
    parser.advance();

    // Check for optional time components
    let mut hour = None;
    let mut minute = None;
    let mut second = None;

    if matches!(parser.current_token(), Token::Comma) {
        parser.advance(); // consume comma

        // Parse hour
        if let Token::NumberLiteral(n) = parser.current_token() {
            hour = Some(n.parse::<u32>().map_err(|_| "Invalid hour")?);
            parser.advance();

            // Check for minute
            if matches!(parser.current_token(), Token::Comma) {
                parser.advance(); // consume comma

                if let Token::NumberLiteral(n) = parser.current_token() {
                    minute = Some(n.parse::<u32>().map_err(|_| "Invalid minute")?);
                    parser.advance();

                    // Check for second
                    if matches!(parser.current_token(), Token::Comma) {
                        parser.advance(); // consume comma

                        if let Token::NumberLiteral(n) = parser.current_token() {
                            second = Some(n.parse::<u32>().map_err(|_| "Invalid second")?);
                            parser.advance();
                        }
                    }
                }
            }
        }
    }

    parser.consume(Token::RightParen)?;

    debug!(year = year, month = month, day = day, hour = ?hour, minute = ?minute, second = ?second, "DateTime constructor parsed");

    Ok(SqlExpression::DateTimeConstructor {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

/// Parse NOT expression
fn parse_not_expression<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParsePrimary + ?Sized,
{
    parser.advance(); // consume NOT

    // Check if this is a NOT IN expression
    if let Ok(inner_expr) = parser.parse_comparison() {
        // After parsing the inner expression, check if we're followed by IN
        if matches!(parser.current_token(), Token::In) {
            debug!("NOT IN expression detected");
            parser.advance(); // consume IN
            parser.consume(Token::LeftParen)?;
            let values = parser.parse_expression_list()?;
            parser.consume(Token::RightParen)?;

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
