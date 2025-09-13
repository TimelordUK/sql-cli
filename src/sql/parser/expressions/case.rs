// CASE expression parsing
// Handles CASE/WHEN/THEN/ELSE/END expressions for conditional logic

use crate::sql::parser::ast::{SqlExpression, WhenBranch};
use crate::sql::parser::lexer::Token;
use tracing::debug;

use super::{log_parse_decision, trace_parse_entry, trace_parse_exit};

/// Parse a CASE expression
/// CASE [expr] WHEN condition THEN result [WHEN ...] [ELSE result] END
pub fn parse_case_expression<P>(parser: &mut P) -> Result<SqlExpression, String>
where
    P: ParseCase + ?Sized,
{
    trace_parse_entry("parse_case_expression", parser.current_token());

    // Consume CASE keyword
    parser.consume(Token::Case)?;

    debug!("Starting CASE expression parsing");

    // Check for simple CASE (CASE expr WHEN value1 THEN ...)
    // vs searched CASE (CASE WHEN condition1 THEN ...)
    // For now, we only support searched CASE

    let mut when_branches = Vec::new();

    // Parse WHEN clauses
    while matches!(parser.current_token(), Token::When) {
        log_parse_decision(
            "parse_case_expression",
            parser.current_token(),
            "WHEN clause found, parsing condition and result",
        );

        parser.advance(); // consume WHEN

        // Parse the condition
        debug!("Parsing WHEN condition");
        let condition = parser.parse_expression()?;

        // Expect THEN
        parser.consume(Token::Then)?;

        // Parse the result expression
        debug!("Parsing THEN result");
        let result = parser.parse_expression()?;

        when_branches.push(WhenBranch {
            condition: Box::new(condition),
            result: Box::new(result),
        });

        debug!("Added WHEN branch #{}", when_branches.len());
    }

    // Check for at least one WHEN clause
    if when_branches.is_empty() {
        return Err("CASE expression must have at least one WHEN clause".to_string());
    }

    // Parse optional ELSE clause
    let else_branch = if matches!(parser.current_token(), Token::Else) {
        log_parse_decision(
            "parse_case_expression",
            parser.current_token(),
            "ELSE clause found, parsing default result",
        );

        parser.advance(); // consume ELSE
        debug!("Parsing ELSE branch");
        Some(Box::new(parser.parse_expression()?))
    } else {
        debug!("No ELSE branch");
        None
    };

    // Expect END keyword
    parser.consume(Token::End)?;

    debug!(
        "Completed CASE expression with {} WHEN branches{}",
        when_branches.len(),
        if else_branch.is_some() {
            " and ELSE"
        } else {
            ""
        }
    );

    let result = Ok(SqlExpression::CaseExpression {
        when_branches,
        else_branch,
    });

    trace_parse_exit("parse_case_expression", &result);
    result
}

/// Trait that parsers must implement to use CASE expression parsing
pub trait ParseCase {
    fn current_token(&self) -> &Token;
    fn advance(&mut self);
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    // Parse a general expression (for conditions and results)
    fn parse_expression(&mut self) -> Result<SqlExpression, String>;
}
