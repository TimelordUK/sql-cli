// Expression parsing module
// This module contains all expression parsing logic extracted from the main parser

use crate::sql::parser::ast::SqlExpression;
use crate::sql::parser::lexer::Token;
use tracing::{debug, trace};

pub mod primary;

/// Trait for expression parsers
/// This allows us to modularize expression parsing while maintaining consistency
pub trait ExpressionParser {
    /// Get the current token
    fn current_token(&self) -> &Token;

    /// Advance to the next token
    fn advance(&mut self);

    /// Peek at the next token without consuming it
    fn peek(&self) -> Option<&Token>;

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool;

    /// Consume a specific token or return error
    fn consume(&mut self, expected: Token) -> Result<(), String>;

    /// Parse an identifier and return its name
    fn parse_identifier(&mut self) -> Result<String, String>;
}

/// Helper function to log expression parsing decisions
#[inline]
pub fn log_parse_decision(context: &str, token: &Token, decision: &str) {
    debug!(
        context = context,
        token = ?token,
        decision = decision,
        "Parser decision point"
    );
}

/// Helper function for detailed trace logging
#[inline]
pub fn trace_parse_entry(function: &str, token: &Token) {
    trace!(
        function = function,
        token = ?token,
        "Entering parse function"
    );
}

/// Helper function for trace exit logging
#[inline]
pub fn trace_parse_exit(function: &str, result: &Result<SqlExpression, String>) {
    trace!(
        function = function,
        success = result.is_ok(),
        "Exiting parse function"
    );
}
