// Main query plan module
mod query_plan;

// Sub-modules
pub mod correlated_subquery_analyzer;
pub mod cte_hoister;
pub mod dependency_analyzer;
pub mod expression_lifter;
pub mod group_by_alias_expander;
pub mod having_alias_transformer;
pub mod ilike_to_like_transformer;
pub mod in_operator_lifter;
pub mod into_clause_remover;
pub mod order_by_alias_transformer;
pub mod pipeline;
pub mod pivot_expander;
pub mod qualify_to_where_transformer;
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
pub use ilike_to_like_transformer::ILikeToLikeTransformer;
pub use in_operator_lifter::{InOperatorLifter, LiftedInExpression};
pub use into_clause_remover::IntoClauseRemover;
pub use order_by_alias_transformer::OrderByAliasTransformer;
pub use pivot_expander::PivotExpander;
pub use qualify_to_where_transformer::QualifyToWhereTransformer;
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
#[derive(Clone, Debug)]
pub struct TransformerConfig {
    pub enable_pivot_expander: bool,
    pub enable_expression_lifter: bool,
    pub enable_where_expansion: bool,
    pub enable_group_by_expansion: bool,
    pub enable_having_expansion: bool,
    pub enable_order_by_expansion: bool,
    pub enable_qualify_to_where: bool,
    pub enable_ilike_to_like: bool,
    pub enable_cte_hoister: bool,
    pub enable_in_lifter: bool,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        // By default, all transformers are enabled for compatibility
        Self::all_enabled()
    }
}

impl TransformerConfig {
    /// Create a config with all transformers enabled
    pub fn all_enabled() -> Self {
        Self {
            enable_pivot_expander: true,
            enable_expression_lifter: true,
            enable_where_expansion: true,
            enable_group_by_expansion: true,
            enable_having_expansion: true,
            enable_order_by_expansion: true,
            enable_qualify_to_where: true,
            enable_ilike_to_like: true,
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
    show_sql_transformations: bool,
    transformer_config: TransformerConfig,
) -> PreprocessingPipeline {
    let config = if verbose || show_sql_transformations {
        PipelineConfig {
            enabled: true,
            verbose_logging: verbose,
            collect_stats: true,
            debug_ast_changes: false,
            show_sql_transformations,
        }
    } else {
        PipelineConfig::default()
    };

    let mut builder = PipelineBuilder::with_config(config);

    // Add transformers in the correct order based on configuration
    // Order matters!
    // 1. PivotExpander must run FIRST to expand PIVOT into standard SQL
    // 2. ExpressionLifter must run before CTEHoister
    // 3. WhereAliasExpander and GroupByAliasExpander run early to expand aliases
    // 4. HavingAliasTransformer runs after GROUP BY to ensure proper aggregate aliases

    // PIVOT expansion must happen first, before any other transformations
    if transformer_config.enable_pivot_expander {
        builder = builder.with_transformer(Box::new(PivotExpander));
    }

    if transformer_config.enable_expression_lifter {
        builder = builder.with_transformer(Box::new(ExpressionLifterTransformer::new()));
    }

    // QualifyToWhereTransformer must run after ExpressionLifter
    // so that window functions are already lifted to CTEs
    if transformer_config.enable_qualify_to_where {
        builder = builder.with_transformer(Box::new(QualifyToWhereTransformer::new()));
    }

    // ILikeToLikeTransformer runs early before WHERE expansion
    // to transform ILIKE operators before WHERE clause processing
    if transformer_config.enable_ilike_to_like {
        builder = builder.with_transformer(Box::new(ILikeToLikeTransformer::new()));
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

    if transformer_config.enable_order_by_expansion {
        builder = builder.with_transformer(Box::new(OrderByAliasTransformer::new()));
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
            show_sql_transformations: false,
        }
    } else {
        PipelineConfig::default()
    };

    let mut builder = PipelineBuilder::with_config(config);

    // Add transformers in the correct order
    // Order matters!
    // 1. PivotExpander MUST run FIRST to expand PIVOT into standard SQL
    // 2. ExpressionLifter must run before CTEHoister
    // 3. ILikeToLikeTransformer runs early to convert ILIKE before other processing
    // 4. WhereAliasExpander and GroupByAliasExpander run early to expand aliases
    // 5. HavingAliasTransformer and OrderByAliasTransformer run after GROUP BY
    builder = builder
        .with_transformer(Box::new(PivotExpander))
        .with_transformer(Box::new(ExpressionLifterTransformer::new()))
        .with_transformer(Box::new(ILikeToLikeTransformer::new()))
        .with_transformer(Box::new(WhereAliasExpander::new()))
        .with_transformer(Box::new(GroupByAliasExpander::new()))
        .with_transformer(Box::new(HavingAliasTransformer::new()))
        .with_transformer(Box::new(OrderByAliasTransformer::new()))
        .with_transformer(Box::new(CTEHoisterTransformer::new()))
        .with_transformer(Box::new(InOperatorLifterTransformer::new()));

    builder.build()
}
