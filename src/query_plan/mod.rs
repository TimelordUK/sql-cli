// Main query plan module
mod query_plan;

// Sub-modules
pub mod correlated_subquery_analyzer;
pub mod cte_hoister;
pub mod dependency_analyzer;
pub mod expression_lifter;
pub mod group_by_alias_expander;
pub mod having_alias_transformer;
pub mod in_operator_lifter;
pub mod into_clause_remover;
pub mod pipeline;
pub mod transformer_adapters;
pub mod where_alias_expander;

// Re-export main types
pub use query_plan::{
    DependencyGraph, PlanMetadata, QueryAnalyzer, QueryPlan, WorkUnit, WorkUnitExpression,
    WorkUnitType,
};

// Re-export commonly used items
pub use correlated_subquery_analyzer::{
    CorrelatedSubqueryAnalyzer, CorrelationAnalysis, SubqueryInfo, SubqueryLocation, SubqueryType,
};
pub use cte_hoister::CTEHoister;
pub use dependency_analyzer::{ScriptDependencyGraph, StatementNode};
pub use expression_lifter::{ExpressionLifter, LiftableExpression};
pub use group_by_alias_expander::GroupByAliasExpander;
pub use having_alias_transformer::HavingAliasTransformer;
pub use in_operator_lifter::{InOperatorLifter, LiftedInExpression};
pub use into_clause_remover::IntoClauseRemover;
pub use where_alias_expander::WhereAliasExpander;

// Re-export pipeline types
pub use pipeline::{
    ASTTransformer, PipelineBuilder, PipelineConfig, PreprocessingPipeline, PreprocessingStats,
    TransformStats,
};

// Re-export transformer adapters
pub use transformer_adapters::{
    CTEHoisterTransformer, ExpressionLifterTransformer, InOperatorLifterTransformer,
};

/// Configuration for selective transformer enabling/disabling
#[derive(Default)]
pub struct TransformerConfig {
    pub enable_expression_lifter: bool,
    pub enable_where_expansion: bool,
    pub enable_group_by_expansion: bool,
    pub enable_having_expansion: bool,
    pub enable_cte_hoister: bool,
    pub enable_in_lifter: bool,
}

impl TransformerConfig {
    /// Create a config with all transformers enabled
    pub fn all_enabled() -> Self {
        Self {
            enable_expression_lifter: true,
            enable_where_expansion: true,
            enable_group_by_expansion: true,
            enable_having_expansion: true,
            enable_cte_hoister: true,
            enable_in_lifter: true,
        }
    }
}

/// Create a preprocessing pipeline with configurable transformers
///
/// # Arguments
/// * `verbose` - Whether to enable verbose logging
/// * `transformer_config` - Configuration for which transformers to enable
///
/// # Example
/// ```ignore
/// let config = TransformerConfig::all_enabled();
/// let mut pipeline = create_pipeline_with_config(false, config);
/// let transformed = pipeline.process(statement)?;
/// ```
pub fn create_pipeline_with_config(
    verbose: bool,
    transformer_config: TransformerConfig,
) -> PreprocessingPipeline {
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

    // Add transformers in the correct order based on configuration
    // Order matters! ExpressionLifter must run before CTEHoister
    // WhereAliasExpander and GroupByAliasExpander run early to expand aliases
    // HavingAliasTransformer runs after GROUP BY to ensure proper aggregate aliases

    if transformer_config.enable_expression_lifter {
        builder = builder.with_transformer(Box::new(ExpressionLifterTransformer::new()));
    }

    if transformer_config.enable_where_expansion {
        builder = builder.with_transformer(Box::new(WhereAliasExpander::new()));
    }

    if transformer_config.enable_group_by_expansion {
        builder = builder.with_transformer(Box::new(GroupByAliasExpander::new()));
    }

    if transformer_config.enable_having_expansion {
        builder = builder.with_transformer(Box::new(HavingAliasTransformer::new()));
    }

    if transformer_config.enable_cte_hoister {
        builder = builder.with_transformer(Box::new(CTEHoisterTransformer::new()));
    }

    if transformer_config.enable_in_lifter {
        builder = builder.with_transformer(Box::new(InOperatorLifterTransformer::new()));
    }

    builder.build()
}

/// Create a standard preprocessing pipeline with all default transformers
///
/// The transformers are applied in this order:
/// 1. ExpressionLifter - Lifts column alias dependencies and window functions
/// 2. WhereAliasExpander - Expands SELECT aliases in WHERE clauses
/// 3. GroupByAliasExpander - Expands SELECT aliases in GROUP BY clauses
/// 4. HavingAliasTransformer - Adds aliases to aggregates and rewrites HAVING
/// 5. CTEHoister - Hoists nested CTEs to top level
/// 6. InOperatorLifter - Optimizes large IN expressions
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
    // WhereAliasExpander and GroupByAliasExpander run early to expand aliases
    // HavingAliasTransformer runs after GROUP BY to ensure proper aggregate aliases
    builder = builder
        .with_transformer(Box::new(ExpressionLifterTransformer::new()))
        .with_transformer(Box::new(WhereAliasExpander::new()))
        .with_transformer(Box::new(GroupByAliasExpander::new()))
        .with_transformer(Box::new(HavingAliasTransformer::new()))
        .with_transformer(Box::new(CTEHoisterTransformer::new()))
        .with_transformer(Box::new(InOperatorLifterTransformer::new()));

    builder.build()
}
