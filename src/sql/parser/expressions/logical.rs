// Logical expression parsing
// Handles logical operators (AND, OR) and NOT operator
// These operators are used for combining boolean expressions

use crate::sql::parser::ast::SqlExpression;
use crate::sql::parser::lexer::Token;
use tracing::debug;

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

/// Parse a logical OR expression
/// OR has the lowest precedence of the logical operators
pub fn parse_logical_or<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseLogical + ?Sized,
{
    trace_parse_entry("parse_logical_or", parser.current_token());

    let mut left = parser.parse_logical_and()?;

    while matches!(parser.current_token(), Token::Or) {
        log_parse_decision(
            "parse_logical_or",
            parser.current_token(),
            "OR operator found, parsing right operand",
        );

        debug!("Processing logical OR operator");
        parser.advance();
        let right = parser.parse_logical_and()?;

        // Use BinaryOp to represent OR operations
        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op: "OR".to_string(),
            right: Box::new(right),
        };

        debug!("Created OR expression");
    }

    let result = Ok(left);
    trace_parse_exit("parse_logical_or", &result);
    result
}

/// Parse a logical AND expression
/// AND has higher precedence than OR but lower than comparison operators
pub fn parse_logical_and<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseLogical + ?Sized,
{
    trace_parse_entry("parse_logical_and", parser.current_token());

    // Parse the base expression (comparison or lower)
    let mut left = parser.parse_base_logical_expression()?;

    while matches!(parser.current_token(), Token::And) {
        log_parse_decision(
            "parse_logical_and",
            parser.current_token(),
            "AND operator found, parsing right operand",
        );

        debug!("Processing logical AND operator");
        parser.advance();
        let right = parser.parse_base_logical_expression()?;

        // Use BinaryOp to represent AND operations
        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op: "AND".to_string(),
            right: Box::new(right),
        };

        debug!("Created AND expression");
    }

    let result = Ok(left);
    trace_parse_exit("parse_logical_and", &result);
    result
}

/// Parse a NOT expression (unary logical operator)
/// This is handled as part of primary expression parsing
pub fn parse_not_expression<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseLogical + ?Sized,
{
    trace_parse_entry("parse_not_expression", parser.current_token());

    if !matches!(parser.current_token(), Token::Not) {
        return Err("Expected NOT token".to_string());
    }

    log_parse_decision(
        "parse_not_expression",
        parser.current_token(),
        "NOT operator found, parsing operand",
    );

    debug!("Processing NOT operator");
    parser.advance(); // consume NOT

    // Check if this is a NOT IN expression
    let inner_expr = parser.parse_comparison()?;

    // After parsing the inner expression, check if we're followed by IN
    if matches!(parser.current_token(), Token::In) {
        debug!("NOT IN expression detected");
        parser.advance(); // consume IN
        parser.consume(Token::LeftParen)?;
        let values = parser.parse_expression_list()?;
        parser.consume(Token::RightParen)?;

        let result = Ok(SqlExpression::NotInList {
            expr: Box::new(inner_expr),
            values,
        });
        trace_parse_exit("parse_not_expression", &result);
        result
    } else {
        // Regular NOT expression
        debug!("Creating NOT expression");
        let result = Ok(SqlExpression::Not {
            expr: Box::new(inner_expr),
        });
        trace_parse_exit("parse_not_expression", &result);
        result
    }
}

/// Trait that parsers must implement to use logical expression parsing
pub trait ParseLogical {
    fn current_token(&self) -> &Token;
    fn advance(&mut self);
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    // These methods are called from logical parsing
    fn parse_logical_and(&mut self) -> Result<SqlExpression, String>;
    fn parse_base_logical_expression(&mut self) -> Result<SqlExpression, String>;
    fn parse_comparison(&mut self) -> Result<SqlExpression, String>;
    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String>;
}
