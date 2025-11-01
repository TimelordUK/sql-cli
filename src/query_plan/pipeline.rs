//! Query preprocessing pipeline
//!
//! This module provides a structured, extensible pipeline for transforming SQL ASTs
//! before execution. Transformers are applied in a defined order, with logging and
//! debugging support.

use crate::sql::parser::ast::SelectStatement;
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info};

/// Trait for AST transformers that can be added to the preprocessing pipeline
pub trait ASTTransformer: Send + Sync {
    /// Name of this transformer (for logging and debugging)
    fn name(&self) -> &str;

    /// Description of what this transformer does
    fn description(&self) -> &str {
        "No description provided"
    }

    /// Whether this transformer is enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Transform the AST, returning the modified statement
    ///
    /// Transformers should be idempotent where possible and should not
    /// modify the semantic meaning of the query.
    fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement>;

    /// Called before transformation starts (for initialization)
    fn begin(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called after transformation completes (for cleanup)
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Statistics about a single transformation
#[derive(Debug, Clone)]
pub struct TransformStats {
    pub transformer_name: String,
    pub duration_micros: u64,
    pub applied: bool,
    pub modifications: usize,
}

/// Complete preprocessing statistics
#[derive(Debug, Clone, Default)]
pub struct PreprocessingStats {
    pub transformations: Vec<TransformStats>,
    pub total_duration_micros: u64,
    pub transformers_applied: usize,
}

impl PreprocessingStats {
    pub fn add_transform(&mut self, stats: TransformStats) {
        self.total_duration_micros += stats.duration_micros;
        if stats.applied {
            self.transformers_applied += 1;
        }
        self.transformations.push(stats);
    }

    pub fn summary(&self) -> String {
        format!(
            "{} transformer(s) applied in {:.2}ms",
            self.transformers_applied,
            self.total_duration_micros as f64 / 1000.0
        )
    }

    /// Returns true if any transformer actually modified the AST
    pub fn has_modifications(&self) -> bool {
        self.transformations
            .iter()
            .any(|stats| stats.modifications > 0)
    }
}

/// Configuration for the preprocessing pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Whether to enable preprocessing at all
    pub enabled: bool,

    /// Whether to log each transformation
    pub verbose_logging: bool,

    /// Whether to collect detailed statistics
    pub collect_stats: bool,

    /// Whether to show the AST before/after each transformation (Rust Debug format)
    pub debug_ast_changes: bool,

    /// Whether to show SQL before/after each transformation (formatted SQL)
    pub show_sql_transformations: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            verbose_logging: false,
            collect_stats: true,
            debug_ast_changes: false,
            show_sql_transformations: false,
        }
    }
}

/// The main preprocessing pipeline
///
/// Orchestrates the application of multiple AST transformers in sequence.
/// Provides logging, debugging, and statistics collection.
///
/// # Example
///
/// ```ignore
/// use sql_cli::query_plan::{PreprocessingPipeline, PipelineConfig};
/// use sql_cli::query_plan::{CTEHoister, ExpressionLifter};
///
/// let mut pipeline = PreprocessingPipeline::new(PipelineConfig::default());
/// pipeline.add_transformer(Box::new(CTEHoister::new()));
/// pipeline.add_transformer(Box::new(ExpressionLifter::new()));
///
/// let transformed = pipeline.process(statement)?;
/// ```
pub struct PreprocessingPipeline {
    transformers: Vec<Box<dyn ASTTransformer>>,
    config: PipelineConfig,
    stats: PreprocessingStats,
}

impl PreprocessingPipeline {
    /// Create a new empty pipeline with the given configuration
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            transformers: Vec::new(),
            config,
            stats: PreprocessingStats::default(),
        }
    }

    /// Add a transformer to the pipeline
    ///
    /// Transformers are applied in the order they are added.
    pub fn add_transformer(&mut self, transformer: Box<dyn ASTTransformer>) {
        self.transformers.push(transformer);
    }

    /// Get the collected statistics
    pub fn stats(&self) -> &PreprocessingStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PreprocessingStats::default();
    }

    /// Process a SQL statement through the pipeline
    ///
    /// Applies each enabled transformer in sequence, collecting statistics
    /// and logging as configured.
    pub fn process(&mut self, mut stmt: SelectStatement) -> Result<SelectStatement> {
        if !self.config.enabled {
            debug!("Preprocessing pipeline is disabled");
            return Ok(stmt);
        }

        let pipeline_start = Instant::now();
        self.reset_stats();

        info!(
            "Starting preprocessing pipeline with {} transformer(s)",
            self.transformers.len()
        );

        for transformer in &mut self.transformers {
            if !transformer.enabled() {
                debug!("Transformer '{}' is disabled, skipping", transformer.name());
                continue;
            }

            let transform_start = Instant::now();

            if self.config.verbose_logging {
                info!(
                    "Applying transformer: {} - {}",
                    transformer.name(),
                    transformer.description()
                );
            }

            // Store original for comparison if debugging
            let original_ast = if self.config.debug_ast_changes {
                Some(format!("{:#?}", stmt))
            } else {
                None
            };

            // Store original SQL for comparison if showing transformations
            let original_sql = if self.config.show_sql_transformations {
                Some(crate::sql::parser::ast_formatter::format_select_statement(
                    &stmt,
                ))
            } else {
                None
            };

            // Call begin hook
            transformer.begin()?;

            // Apply transformation
            let transformed = transformer.transform(stmt)?;

            // Call end hook
            transformer.end()?;

            let duration = transform_start.elapsed();

            // Check if anything changed (AST debug format)
            let mut modifications = 0;
            if let Some(original_ast_str) = original_ast {
                let new_ast_str = format!("{:#?}", transformed);
                if original_ast_str != new_ast_str {
                    if self.config.debug_ast_changes {
                        debug!("AST changed by '{}'", transformer.name());
                        debug!("Before:\n{}", original_ast_str);
                        debug!("After:\n{}", new_ast_str);
                    }
                    modifications = 1;
                }
            }

            // Show SQL transformations if enabled
            if let Some(original_sql_str) = original_sql {
                let new_sql_str =
                    crate::sql::parser::ast_formatter::format_select_statement(&transformed);
                if original_sql_str != new_sql_str {
                    eprintln!(
                        "\n╔════════════════════════════════════════════════════════════════╗"
                    );
                    eprintln!("║ Transformer: {:<51} ║", transformer.name());
                    eprintln!("╠════════════════════════════════════════════════════════════════╣");
                    eprintln!("║ BEFORE:                                                        ║");
                    eprintln!("╠════════════════════════════════════════════════════════════════╣");
                    for line in original_sql_str.lines() {
                        eprintln!("  {}", line);
                    }
                    eprintln!("╠════════════════════════════════════════════════════════════════╣");
                    eprintln!("║ AFTER:                                                         ║");
                    eprintln!("╠════════════════════════════════════════════════════════════════╣");
                    for line in new_sql_str.lines() {
                        eprintln!("  {}", line);
                    }
                    eprintln!(
                        "╚════════════════════════════════════════════════════════════════╝\n"
                    );
                    modifications = 1;
                }
            }

            // Record statistics
            let stats = TransformStats {
                transformer_name: transformer.name().to_string(),
                duration_micros: duration.as_micros() as u64,
                applied: true,
                modifications,
            };

            self.stats.add_transform(stats);

            stmt = transformed;
        }

        let total_duration = pipeline_start.elapsed();
        self.stats.total_duration_micros = total_duration.as_micros() as u64;

        if self.config.verbose_logging {
            info!("Preprocessing complete: {}", self.stats.summary());
        }

        Ok(stmt)
    }

    /// Get a summary of what transformers are in the pipeline
    pub fn transformer_summary(&self) -> String {
        let enabled_count = self.transformers.iter().filter(|t| t.enabled()).count();
        let total_count = self.transformers.len();

        let names: Vec<String> = self
            .transformers
            .iter()
            .map(|t| {
                let status = if t.enabled() { "✓" } else { "✗" };
                format!("{} {}", status, t.name())
            })
            .collect();

        format!(
            "{}/{} transformers enabled:\n{}",
            enabled_count,
            total_count,
            names.join("\n")
        )
    }
}

impl Default for PreprocessingPipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::default())
    }
}

/// Builder for creating a preprocessing pipeline with common transformers
pub struct PipelineBuilder {
    pipeline: PreprocessingPipeline,
}

impl PipelineBuilder {
    /// Start building a new pipeline
    pub fn new() -> Self {
        Self {
            pipeline: PreprocessingPipeline::default(),
        }
    }

    /// Start with a specific configuration
    pub fn with_config(config: PipelineConfig) -> Self {
        Self {
            pipeline: PreprocessingPipeline::new(config),
        }
    }

    /// Enable verbose logging
    pub fn verbose(mut self) -> Self {
        self.pipeline.config.verbose_logging = true;
        self
    }

    /// Enable AST change debugging
    pub fn debug_ast(mut self) -> Self {
        self.pipeline.config.debug_ast_changes = true;
        self
    }

    /// Add a custom transformer to the pipeline
    pub fn with_transformer(mut self, transformer: Box<dyn ASTTransformer>) -> Self {
        self.pipeline.add_transformer(transformer);
        self
    }

    /// Build the final pipeline
    pub fn build(self) -> PreprocessingPipeline {
        self.pipeline
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::ast::SelectStatement;

    // Mock transformer for testing
    struct NoOpTransformer {
        name: String,
        enabled: bool,
    }

    impl ASTTransformer for NoOpTransformer {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Test transformer that does nothing"
        }

        fn enabled(&self) -> bool {
            self.enabled
        }

        fn transform(&mut self, stmt: SelectStatement) -> Result<SelectStatement> {
            Ok(stmt)
        }
    }

    #[test]
    fn test_empty_pipeline() {
        let mut pipeline = PreprocessingPipeline::default();
        let stmt = SelectStatement::default();
        let result = pipeline.process(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_disabled_pipeline() {
        let mut config = PipelineConfig::default();
        config.enabled = false;

        let mut pipeline = PreprocessingPipeline::new(config);
        pipeline.add_transformer(Box::new(NoOpTransformer {
            name: "test".to_string(),
            enabled: true,
        }));

        let stmt = SelectStatement::default();
        let result = pipeline.process(stmt);
        assert!(result.is_ok());
        assert_eq!(pipeline.stats().transformers_applied, 0);
    }

    #[test]
    fn test_disabled_transformer() {
        let mut pipeline = PreprocessingPipeline::default();
        pipeline.add_transformer(Box::new(NoOpTransformer {
            name: "disabled".to_string(),
            enabled: false,
        }));

        let stmt = SelectStatement::default();
        let result = pipeline.process(stmt);
        assert!(result.is_ok());
        assert_eq!(pipeline.stats().transformers_applied, 0);
    }

    #[test]
    fn test_stats_collection() {
        let mut pipeline = PreprocessingPipeline::default();
        pipeline.add_transformer(Box::new(NoOpTransformer {
            name: "test1".to_string(),
            enabled: true,
        }));
        pipeline.add_transformer(Box::new(NoOpTransformer {
            name: "test2".to_string(),
            enabled: true,
        }));

        let stmt = SelectStatement::default();
        let result = pipeline.process(stmt);
        assert!(result.is_ok());
        assert_eq!(pipeline.stats().transformers_applied, 2);
        assert_eq!(pipeline.stats().transformations.len(), 2);
    }

    #[test]
    fn test_builder() {
        let pipeline = PipelineBuilder::new()
            .verbose()
            .debug_ast()
            .with_transformer(Box::new(NoOpTransformer {
                name: "test".to_string(),
                enabled: true,
            }))
            .build();

        assert!(pipeline.config.verbose_logging);
        assert!(pipeline.config.debug_ast_changes);
        assert_eq!(pipeline.transformers.len(), 1);
    }
}
