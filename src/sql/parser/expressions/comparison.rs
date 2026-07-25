// Comparison expression parsing
// Handles comparison operators, BETWEEN, IN/NOT IN, LIKE, IS NULL/IS NOT NULL

use crate::sql::parser::ast::SqlExpression;
use crate::sql::parser::lexer::Token;
use tracing::debug;

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

/// Parse a comparison expression
/// This handles comparison operators and special SQL operators
pub fn parse_comparison<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseComparison + ?Sized,
{
    trace_parse_entry("parse_comparison", parser.current_token());

    let mut left = parser.parse_additive()?;

    // Handle BETWEEN operator
    if matches!(parser.current_token(), Token::Between) {
        debug!("BETWEEN operator detected");
        log_parse_decision(
            "parse_comparison",
            parser.current_token(),
            "BETWEEN operator - parsing range bounds",
        );

        parser.advance(); // consume BETWEEN
        let lower = parser.parse_additive()?;
        parser.consume(Token::And)?; // BETWEEN requires AND
        let upper = parser.parse_additive()?;

        let result = Ok(SqlExpression::Between {
            expr: Box::new(left),
            lower: Box::new(lower),
            upper: Box::new(upper),
        });
        trace_parse_exit("parse_comparison", &result);
        return result;
    }

    // Handle NOT IN operator
    if matches!(parser.current_token(), Token::Not) {
        // Peek ahead to see if this is NOT IN
        parser.advance(); // consume NOT
        if matches!(parser.current_token(), Token::In) {
            debug!("NOT IN operator detected");
            log_parse_decision(
                "parse_comparison",
                parser.current_token(),
                "NOT IN operator - parsing value list",
            );

            parser.advance(); // consume IN
            parser.consume(Token::LeftParen)?;

            // Check if this is a subquery (starts with SELECT, or WITH for a
            // CTE in expression position — P12).
            if matches!(parser.current_token(), Token::Select | Token::With) {
                debug!("Detected NOT IN subquery");
                let subquery = parser.parse_subquery()?;
                parser.consume(Token::RightParen)?;

                let result = Ok(SqlExpression::NotInSubquery {
                    expr: Box::new(left),
                    subquery: Box::new(subquery),
                });
                trace_parse_exit("parse_comparison", &result);
                return result;
            } else {
                // Regular NOT IN with value list
                let values = parser.parse_expression_list()?;
                parser.consume(Token::RightParen)?;

                let result = Ok(SqlExpression::NotInList {
                    expr: Box::new(left),
                    values,
                });
                trace_parse_exit("parse_comparison", &result);
                return result;
            }
        } else {
            return Err("Expected IN after NOT".to_string());
        }
    }

    // Handle IS NULL / IS NOT NULL
    if matches!(parser.current_token(), Token::Is) {
        parser.advance(); // consume IS

        if matches!(parser.current_token(), Token::Not) {
            parser.advance(); // consume NOT
            if matches!(parser.current_token(), Token::Null) {
                debug!("IS NOT NULL operator detected");
                log_parse_decision(
                    "parse_comparison",
                    parser.current_token(),
                    "IS NOT NULL operator",
                );

                parser.advance(); // consume NULL
                left = SqlExpression::BinaryOp {
                    left: Box::new(left),
                    op: "IS NOT NULL".to_string(),
                    right: Box::new(SqlExpression::Null),
                };
            } else {
                return Err("Expected NULL after IS NOT".to_string());
            }
        } else if matches!(parser.current_token(), Token::Null) {
            debug!("IS NULL operator detected");
            log_parse_decision(
                "parse_comparison",
                parser.current_token(),
                "IS NULL operator",
            );

            parser.advance(); // consume NULL
            left = SqlExpression::BinaryOp {
                left: Box::new(left),
                op: "IS NULL".to_string(),
                right: Box::new(SqlExpression::Null),
            };
        } else {
            return Err("Expected NULL or NOT after IS".to_string());
        }
    }
    // Handle comparison operators
    else if let Some(op) = get_comparison_op(parser.current_token()) {
        log_parse_decision(
            "parse_comparison",
            parser.current_token(),
            &format!("Comparison operator '{}' found", op),
        );

        debug!(operator = %op, "Processing comparison operator");

        parser.advance();
        let right = parser.parse_additive()?;
        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    let result = Ok(left);
    trace_parse_exit("parse_comparison", &result);
    result
}

/// Parse an expression that may contain IN operator
/// This is called from parse_expression to handle IN after other comparisons
pub fn parse_in_operator<P>(parser: &mut P, expr: SqlExpression) -> Result<SqlExpression, String>
where
    P: ParseComparison + ?Sized,
{
    trace_parse_entry("parse_in_operator", parser.current_token());

    if matches!(parser.current_token(), Token::In) {
        debug!("IN operator detected");
        log_parse_decision(
            "parse_in_operator",
            parser.current_token(),
            "IN operator - parsing value list",
        );

        parser.advance(); // consume IN
        parser.consume(Token::LeftParen)?;

        // Check if this is a subquery (starts with SELECT, or WITH for a CTE
        // in expression position — P12).
        if matches!(parser.current_token(), Token::Select | Token::With) {
            debug!("Detected IN subquery");
            let subquery = parser.parse_subquery()?;
            parser.consume(Token::RightParen)?;

            let result = Ok(SqlExpression::InSubquery {
                expr: Box::new(expr),
                subquery: Box::new(subquery),
            });
            trace_parse_exit("parse_in_operator", &result);
            return result;
        } else {
            // Regular IN with value list
            let values = parser.parse_expression_list()?;
            parser.consume(Token::RightParen)?;

            let result = Ok(SqlExpression::InList {
                expr: Box::new(expr),
                values,
            });
            trace_parse_exit("parse_in_operator", &result);
            return result;
        }
    } else {
        Ok(expr)
    }
}

/// Get comparison operator from token
fn get_comparison_op(token: &Token) -> Option<String> {
    match token {
        Token::Equal => Some("=".to_string()),
        Token::NotEqual => Some("!=".to_string()),
        Token::LessThan => Some("<".to_string()),
        Token::GreaterThan => Some(">".to_string()),
        Token::LessThanOrEqual => Some("<=".to_string()),
        Token::GreaterThanOrEqual => Some(">=".to_string()),
        Token::Like => Some("LIKE".to_string()),
        Token::ILike => Some("ILIKE".to_string()),
        _ => None,
    }
}

/// Trait that parsers must implement to use comparison expression parsing
pub trait ParseComparison {
    fn current_token(&self) -> &Token;
    fn advance(&mut self);
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    // These methods are called from comparison parsing
    fn parse_primary(&mut self) -> Result<SqlExpression, String>;
    fn parse_additive(&mut self) -> Result<SqlExpression, String>;
    fn parse_expression_list(&mut self) -> Result<Vec<SqlExpression>, String>;

    // For subquery parsing (without parenthesis balance validation)
    fn parse_subquery(&mut self) -> Result<crate::sql::parser::ast::SelectStatement, String>;
}
