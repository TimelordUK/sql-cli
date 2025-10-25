// Main query plan module
mod query_plan;

// Sub-modules
pub mod cte_hoister;
pub mod dependency_analyzer;
pub mod expression_lifter;
pub mod having_alias_transformer;
pub mod in_operator_lifter;
pub mod into_clause_remover;
pub mod pipeline;
pub mod transformer_adapters;

// Re-export main types
pub use query_plan::{
    DependencyGraph, PlanMetadata, QueryAnalyzer, QueryPlan, WorkUnit, WorkUnitExpression,
    WorkUnitType,
};

// Re-export commonly used items
pub use cte_hoister::CTEHoister;
pub use dependency_analyzer::{ScriptDependencyGraph, StatementNode};
pub use expression_lifter::{ExpressionLifter, LiftableExpression};
pub use having_alias_transformer::HavingAliasTransformer;
pub use in_operator_lifter::{InOperatorLifter, LiftedInExpression};
pub use into_clause_remover::IntoClauseRemover;

// Re-export pipeline types
pub use pipeline::{
    ASTTransformer, PipelineBuilder, PipelineConfig, PreprocessingPipeline, PreprocessingStats,
    TransformStats,
};

// Re-export transformer adapters
pub use transformer_adapters::{
    CTEHoisterTransformer, ExpressionLifterTransformer, InOperatorLifterTransformer,
};

/// Create a standard preprocessing pipeline with all default transformers
///
/// The transformers are applied in this order:
/// 1. ExpressionLifter - Lifts column alias dependencies and window functions
/// 2. HavingAliasTransformer - Adds aliases to aggregates and rewrites HAVING
/// 3. CTEHoister - Hoists nested CTEs to top level
/// 4. InOperatorLifter - Optimizes large IN expressions
///
/// # Arguments
/// * `verbose` - Whether to enable verbose logging
///
/// # Example
/// ```ignore
/// let mut pipeline = create_standard_pipeline(false);
/// let transformed = pipeline.process(statement)?;
/// ```
pub fn create_standard_pipeline(verbose: bool) -> PreprocessingPipeline {
    let config = if verbose {
        PipelineConfig {
            enabled: true,
            verbose_logging: true,
            collect_stats: true,
            debug_ast_changes: false,
        }
    } else {
        PipelineConfig::default()
    };

    let mut builder = PipelineBuilder::with_config(config);

    // Add transformers in the correct order
    // Order matters! ExpressionLifter must run before CTEHoister
    // HavingAliasTransformer runs after ExpressionLifter to ensure proper aliases
    builder = builder
        .with_transformer(Box::new(ExpressionLifterTransformer::new()))
        .with_transformer(Box::new(HavingAliasTransformer::new()))
        .with_transformer(Box::new(CTEHoisterTransformer::new()))
        .with_transformer(Box::new(InOperatorLifterTransformer::new()));

    builder.build()
}
