// Main query plan module
mod query_plan;

// Sub-modules
pub mod cte_hoister;
pub mod expression_lifter;
pub mod in_operator_lifter;
pub mod into_clause_remover;

// Re-export main types
pub use query_plan::{
    DependencyGraph, PlanMetadata, QueryAnalyzer, QueryPlan, WorkUnit, WorkUnitExpression,
    WorkUnitType,
};

// Re-export commonly used items
pub use cte_hoister::CTEHoister;
pub use expression_lifter::{ExpressionLifter, LiftableExpression};
pub use in_operator_lifter::{InOperatorLifter, LiftedInExpression};
pub use into_clause_remover::IntoClauseRemover;
