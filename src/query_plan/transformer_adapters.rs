//! Adapter wrappers for existing transformers to implement ASTTransformer trait
//!
//! This module provides wrappers around our existing transformation code
//! (CTEHoister, ExpressionLifter, etc.) so they can be used in the pipeline.

use super::pipeline::ASTTransformer;
use super::{CTEHoister, ExpressionLifter, InOperatorLifter};
use crate::sql::parser::ast::SelectStatement;
use anyhow::Result;

/// Wrapper for CTEHoister
pub struct CTEHoisterTransformer;

impl CTEHoisterTransformer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CTEHoisterTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for CTEHoisterTransformer {
    fn name(&self) -> &str {
        "CTEHoister"
    }

    fn description(&self) -> &str {
        "Hoists nested WITH clauses to top level for efficient execution"
    }

    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
        Ok(CTEHoister::hoist_ctes(stmt))
    }
}

/// Wrapper for ExpressionLifter
pub struct ExpressionLifterTransformer {
    lifter: ExpressionLifter,
}

impl ExpressionLifterTransformer {
    pub fn new() -> Self {
        Self {
            lifter: ExpressionLifter::new(),
        }
    }
}

impl Default for ExpressionLifterTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for ExpressionLifterTransformer {
    fn name(&self) -> &str {
        "ExpressionLifter"
    }

    fn description(&self) -> &str {
        "Lifts window functions and column alias dependencies to CTEs"
    }

    fn transform(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        let lifted_ctes = self.lifter.lift_expressions(&mut stmt);

        if !lifted_ctes.is_empty() {
            tracing::info!("ExpressionLifter generated {} CTE(s)", lifted_ctes.len());
        }

        Ok(stmt)
    }
}

/// Wrapper for InOperatorLifter
pub struct InOperatorLifterTransformer {
    lifter: InOperatorLifter,
}

impl InOperatorLifterTransformer {
    pub fn new() -> Self {
        Self {
            lifter: InOperatorLifter::new(),
        }
    }
}

impl Default for InOperatorLifterTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransformer for InOperatorLifterTransformer {
    fn name(&self) -> &str {
        "InOperatorLifter"
    }

    fn description(&self) -> &str {
        "Optimizes IN expressions with large value lists by lifting to CTEs"
    }

    fn transform(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        if self.lifter.rewrite_query(&mut stmt) {
            tracing::info!("InOperatorLifter applied - query rewritten with CTEs");
        }
        Ok(stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cte_hoister_transformer() {
        let mut transformer = CTEHoisterTransformer::new();
        assert_eq!(transformer.name(), "CTEHoister");
        assert!(transformer.enabled());

        let stmt = SelectStatement::default();
        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_lifter_transformer() {
        let mut transformer = ExpressionLifterTransformer::new();
        assert_eq!(transformer.name(), "ExpressionLifter");

        let stmt = SelectStatement::default();
        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_in_operator_lifter_transformer() {
        let mut transformer = InOperatorLifterTransformer::new();
        assert_eq!(transformer.name(), "InOperatorLifter");

        let stmt = SelectStatement::default();
        let result = transformer.transform(stmt);
        assert!(result.is_ok());
    }
}
