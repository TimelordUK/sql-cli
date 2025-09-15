// Main query plan module
mod query_plan;

// Sub-modules
pub mod expression_lifter;

// Re-export main types
pub use query_plan::{
    WorkUnit, WorkUnitType, WorkUnitExpression,
    QueryPlan, PlanMetadata, DependencyGraph,
    QueryAnalyzer
};

// Re-export commonly used items
pub use expression_lifter::{ExpressionLifter, LiftableExpression};