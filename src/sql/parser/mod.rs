//! Modular SQL Parser
//!
//! This module provides a structured approach to SQL parsing,
//! breaking down the monolithic parser into focused components.

pub mod ast;
pub mod ast_formatter;
pub mod expressions;
pub mod file_cte_parser;
pub mod formatter;
pub mod legacy;
pub mod lexer;
pub mod walk;
pub mod web_cte_parser;

// Re-export commonly used types for convenience
pub use ast::{
    Condition, JoinClause, JoinCondition, JoinOperator, JoinType, LogicalOp, OrderByColumn,
    SelectItem, SelectStatement, SortDirection, SqlExpression, TableFunction, TableSource,
    WhenBranch, WhereClause, WindowSpec, CTE,
};

pub use lexer::{Lexer, LexerMode, Token};

// Re-export legacy types for backward compatibility
pub use legacy::{
    ColumnInfo, ColumnType, ParseContext, ParseState, Schema, SqlParser, SqlToken, TableInfo,
};

// Test modules
#[cfg(test)]
mod tests;

#[cfg(test)]
mod comment_preservation_tests;

// Parser configuration
#[derive(Default)]
pub struct ParserConfig {
    pub case_insensitive: bool,
}
