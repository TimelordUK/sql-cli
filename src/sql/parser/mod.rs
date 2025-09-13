//! Modular SQL Parser
//!
//! This module provides a structured approach to SQL parsing,
//! breaking down the monolithic parser into focused components.

pub mod ast;
pub mod expressions;
pub mod legacy;
pub mod lexer;

// Re-export commonly used types for convenience
pub use ast::{
    Condition, JoinClause, JoinCondition, JoinOperator, JoinType, LogicalOp, OrderByColumn,
    SelectItem, SelectStatement, SortDirection, SqlExpression, TableFunction, TableSource,
    WhenBranch, WhereClause, WindowSpec, CTE,
};

pub use lexer::{Lexer, Token};

// Re-export legacy types for backward compatibility
pub use legacy::{ParseContext, ParseState, Schema, SqlParser, SqlToken, TableInfo};

// Parser configuration
#[derive(Default)]
pub struct ParserConfig {
    pub case_insensitive: bool,
}
