// Arithmetic expression parsing
// Handles additive (+, -) and multiplicative (*, /, %) expressions
// Also handles method calls (e.g., column.upper()) as part of multiplicative precedence

use crate::sql::parser::ast::{ColumnRef, SqlExpression};
use crate::sql::parser::lexer::Token;
use tracing::debug;

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

/// Parse an additive expression (+ and - operators, and || for string concatenation)
/// This handles left-associative binary operators at the additive precedence level
pub fn parse_additive<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseArithmetic + ?Sized,
{
    trace_parse_entry("parse_additive", parser.current_token());

    let mut left = parser.parse_multiplicative()?;

    while matches!(
        parser.current_token(),
        Token::Plus | Token::Minus | Token::Concat
    ) {
        let op = match parser.current_token() {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Concat => {
                // Handle || as syntactic sugar for TEXTJOIN
                log_parse_decision(
                    "parse_additive",
                    parser.current_token(),
                    "String concatenation '||' found, converting to TEXTJOIN",
                );

                parser.advance();
                let right = parser.parse_multiplicative()?;

                // Convert left || right to TEXTJOIN('', 1, left, right)
                // TEXTJOIN needs: delimiter, ignore_empty flag, then values
                left = SqlExpression::FunctionCall {
                    name: "TEXTJOIN".to_string(),
                    args: vec![
                        SqlExpression::StringLiteral("".to_string()), // Empty separator
                        SqlExpression::NumberLiteral("1".to_string()), // ignore_empty = true
                        left,
                        right,
                    ],
                    distinct: false,
                };
                continue; // Process next operator if any
            }
            _ => unreachable!(),
        };

        log_parse_decision(
            "parse_additive",
            parser.current_token(),
            &format!("Binary operator '{}' found, parsing right operand", op),
        );

        parser.advance();
        let right = parser.parse_multiplicative()?;

        debug!(operator = op, "Creating additive binary operation");

        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        };
    }

    let result = Ok(left);
    trace_parse_exit("parse_additive", &result);
    result
}

/// Parse a multiplicative expression (*, /, % operators and method calls)
/// This handles left-associative binary operators at the multiplicative precedence level
/// Also handles method calls (.) which have the same precedence as multiplication
pub fn parse_multiplicative<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseArithmetic + ?Sized,
{
    trace_parse_entry("parse_multiplicative", parser.current_token());

    let mut left = parser.parse_primary()?;

    // Handle method calls and qualified column names
    // Method calls have the same precedence as multiplication
    while matches!(parser.current_token(), Token::Dot) {
        debug!("Found dot operator");
        parser.advance();

        if let Token::Identifier(name) = parser.current_token() {
            let name_str = name.clone();
            parser.advance();

            if matches!(parser.current_token(), Token::LeftParen) {
                // This is a method call
                log_parse_decision(
                    "parse_multiplicative",
                    parser.current_token(),
                    &format!("Method call '{}' detected", name_str),
                );

                parser.advance();
                let args = parser.parse_method_args()?;
                parser.consume(Token::RightParen)?;

                // Support chained method calls
                left = match left {
                    SqlExpression::Column(obj) => {
                        // First method call on a column
                        debug!(
                            column = %obj,
                            method = %name_str,
                            "Creating method call on column"
                        );
                        SqlExpression::MethodCall {
                            object: obj.name,
                            method: name_str,
                            args,
                        }
                    }
                    SqlExpression::MethodCall { .. } | SqlExpression::ChainedMethodCall { .. } => {
                        // Chained method call on a previous method call
                        debug!(
                            method = %name_str,
                            "Creating chained method call"
                        );
                        SqlExpression::ChainedMethodCall {
                            base: Box::new(left),
                            method: name_str,
                            args,
                        }
                    }
                    _ => {
                        // Method call on any other expression
                        debug!(
                            method = %name_str,
                            "Creating method call on expression"
                        );
                        SqlExpression::ChainedMethodCall {
                            base: Box::new(left),
                            method: name_str,
                            args,
                        }
                    }
                };
            } else {
                // This is a qualified column name (table.column or alias.column)
                // Combine the left expression with the column name
                left = match left {
                    SqlExpression::Column(table_or_alias) => {
                        debug!(
                            table = %table_or_alias,
                            column = %name_str,
                            "Creating qualified column reference"
                        );
                        SqlExpression::Column(ColumnRef::unquoted(format!(
                            "{}.{}",
                            table_or_alias, name_str
                        )))
                    }
                    _ => {
                        // If left is not a simple column, this is an error
                        return Err(format!(
                            "Invalid qualified column reference with expression"
                        ));
                    }
                };
            }
        } else {
            return Err("Expected identifier after '.'".to_string());
        }
    }

    // Handle multiplicative binary operators
    while matches!(
        parser.current_token(),
        Token::Star | Token::Divide | Token::Modulo
    ) {
        let op = match parser.current_token() {
            Token::Star => "*",
            Token::Divide => "/",
            Token::Modulo => "%",
            _ => unreachable!(),
        };

        log_parse_decision(
            "parse_multiplicative",
            parser.current_token(),
            &format!("Binary operator '{}' found, parsing right operand", op),
        );

        parser.advance();
        let right = parser.parse_primary()?;

        debug!(operator = op, "Creating multiplicative binary operation");

        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        };
    }

    let result = Ok(left);
    trace_parse_exit("parse_multiplicative", &result);
    result
}

/// Trait that parsers must implement to use arithmetic expression parsing
pub trait ParseArithmetic {
    fn current_token(&self) -> &Token;
    fn advance(&mut self);
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    // These methods are called from arithmetic parsing
    fn parse_primary(&mut self) -> Result<SqlExpression, String>;
    fn parse_multiplicative(&mut self) -> Result<SqlExpression, String>;
    fn parse_method_args(&mut self) -> Result<Vec<SqlExpression>, String>;
}
