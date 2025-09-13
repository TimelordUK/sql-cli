// Arithmetic expression parsing
// Handles additive (+, -) and multiplicative (*, /, %) expressions
// Also handles method calls (e.g., column.upper()) as part of multiplicative precedence

use crate::sql::parser::ast::SqlExpression;
use crate::sql::parser::lexer::Token;
use tracing::debug;

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

/// Parse an additive expression (+ and - operators)
/// This handles left-associative binary operators at the additive precedence level
pub fn parse_additive<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseArithmetic + ?Sized,
{
    trace_parse_entry("parse_additive", parser.current_token());

    let mut left = parser.parse_multiplicative()?;

    while matches!(parser.current_token(), Token::Plus | Token::Minus) {
        let op = match parser.current_token() {
            Token::Plus => "+",
            Token::Minus => "-",
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

    // Handle method calls on the primary expression
    // Method calls have the same precedence as multiplication
    while matches!(parser.current_token(), Token::Dot) {
        debug!("Found dot operator, parsing method call");
        parser.advance();

        if let Token::Identifier(method) = parser.current_token() {
            let method_name = method.clone();
            parser.advance();

            if matches!(parser.current_token(), Token::LeftParen) {
                log_parse_decision(
                    "parse_multiplicative",
                    parser.current_token(),
                    &format!("Method call '{}' detected", method_name),
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
                            method = %method_name,
                            "Creating method call on column"
                        );
                        SqlExpression::MethodCall {
                            object: obj,
                            method: method_name,
                            args,
                        }
                    }
                    SqlExpression::MethodCall { .. } | SqlExpression::ChainedMethodCall { .. } => {
                        // Chained method call on a previous method call
                        debug!(
                            method = %method_name,
                            "Creating chained method call"
                        );
                        SqlExpression::ChainedMethodCall {
                            base: Box::new(left),
                            method: method_name,
                            args,
                        }
                    }
                    _ => {
                        // Method call on any other expression
                        debug!(
                            method = %method_name,
                            "Creating method call on expression"
                        );
                        SqlExpression::ChainedMethodCall {
                            base: Box::new(left),
                            method: method_name,
                            args,
                        }
                    }
                };
            } else {
                return Err(format!("Expected '(' after method name '{method_name}'"));
            }
        } else {
            return Err("Expected method name after '.'".to_string());
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
