//! SQL parsing, execution, and optimization
//!
//! This module handles all SQL-related functionality including
//! parsing, query optimization, execution, and caching.

pub mod cache;
pub mod cursor_aware_parser;
pub mod hybrid_parser;
pub mod parser;
pub mod recursive_parser;
pub mod smart_parser;
pub mod sql_highlighter;
pub mod where_ast;
pub mod where_parser;
