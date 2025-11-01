//! Execution configuration for controlling statement execution behavior
//!
//! This module defines configuration options that control how SQL statements
//! are preprocessed and executed.

use crate::query_plan::TransformerConfig;

/// Configuration for statement execution
///
/// Controls preprocessing, case sensitivity, and other execution behaviors
#[derive(Clone, Debug)]
pub struct ExecutionConfig {
    /// Whether to show preprocessing transformations (debug output)
    pub show_preprocessing: bool,

    /// Whether to show SQL before/after each transformation (formatted SQL output)
    pub show_sql_transformations: bool,

    /// Whether to use case-insensitive comparison
    pub case_insensitive: bool,

    /// Whether to automatically hide empty columns
    pub auto_hide_empty: bool,

    /// Configuration for AST transformers (preprocessing pipeline)
    pub transformer_config: TransformerConfig,

    /// Whether to enable debug tracing during execution
    pub debug_trace: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            show_preprocessing: false,
            show_sql_transformations: false,
            case_insensitive: false,
            auto_hide_empty: false,
            transformer_config: TransformerConfig::default(),
            debug_trace: false,
        }
    }
}

impl ExecutionConfig {
    /// Create a new execution config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable preprocessing debug output
    pub fn with_show_preprocessing(mut self, show: bool) -> Self {
        self.show_preprocessing = show;
        self
    }

    /// Enable SQL transformation output (show before/after SQL for each transformer)
    pub fn with_show_sql_transformations(mut self, show: bool) -> Self {
        self.show_sql_transformations = show;
        self
    }

    /// Enable case-insensitive comparison
    pub fn with_case_insensitive(mut self, insensitive: bool) -> Self {
        self.case_insensitive = insensitive;
        self
    }

    /// Enable automatic hiding of empty columns
    pub fn with_auto_hide_empty(mut self, hide: bool) -> Self {
        self.auto_hide_empty = hide;
        self
    }

    /// Set transformer configuration
    pub fn with_transformer_config(mut self, config: TransformerConfig) -> Self {
        self.transformer_config = config;
        self
    }

    /// Enable debug tracing
    pub fn with_debug_trace(mut self, trace: bool) -> Self {
        self.debug_trace = trace;
        self
    }

    /// Disable all preprocessing transformers
    pub fn without_preprocessing(mut self) -> Self {
        self.transformer_config.enable_expression_lifter = false;
        self.transformer_config.enable_where_expansion = false;
        self.transformer_config.enable_group_by_expansion = false;
        self.transformer_config.enable_having_expansion = false;
        self.transformer_config.enable_cte_hoister = false;
        self.transformer_config.enable_in_lifter = false;
        self
    }

    /// Create config from command-line flags (helper for non_interactive.rs)
    pub fn from_cli_flags(
        show_preprocessing: bool,
        show_sql_transformations: bool,
        case_insensitive: bool,
        auto_hide_empty: bool,
        no_expression_lifter: bool,
        no_where_expansion: bool,
        no_group_by_expansion: bool,
        no_having_expansion: bool,
        no_cte_hoister: bool,
        no_in_lifter: bool,
        debug_trace: bool,
    ) -> Self {
        let config = Self {
            show_preprocessing,
            show_sql_transformations,
            case_insensitive,
            auto_hide_empty,
            transformer_config: TransformerConfig {
                enable_expression_lifter: !no_expression_lifter,
                enable_where_expansion: !no_where_expansion,
                enable_group_by_expansion: !no_group_by_expansion,
                enable_having_expansion: !no_having_expansion,
                enable_cte_hoister: !no_cte_hoister,
                enable_in_lifter: !no_in_lifter,
            },
            debug_trace,
        };

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExecutionConfig::default();

        assert!(!config.show_preprocessing);
        assert!(!config.case_insensitive);
        assert!(!config.auto_hide_empty);
        assert!(!config.debug_trace);

        // All transformers enabled by default
        assert!(config.transformer_config.enable_expression_lifter);
        assert!(config.transformer_config.enable_where_expansion);
        assert!(config.transformer_config.enable_group_by_expansion);
        assert!(config.transformer_config.enable_having_expansion);
        assert!(config.transformer_config.enable_cte_hoister);
        assert!(config.transformer_config.enable_in_lifter);
    }

    #[test]
    fn test_builder_pattern() {
        let config = ExecutionConfig::new()
            .with_show_preprocessing(true)
            .with_case_insensitive(true)
            .with_auto_hide_empty(true)
            .with_debug_trace(true);

        assert!(config.show_preprocessing);
        assert!(config.case_insensitive);
        assert!(config.auto_hide_empty);
        assert!(config.debug_trace);
    }

    #[test]
    fn test_without_preprocessing() {
        let config = ExecutionConfig::new().without_preprocessing();

        assert!(!config.transformer_config.enable_expression_lifter);
        assert!(!config.transformer_config.enable_where_expansion);
        assert!(!config.transformer_config.enable_group_by_expansion);
        assert!(!config.transformer_config.enable_having_expansion);
        assert!(!config.transformer_config.enable_cte_hoister);
        assert!(!config.transformer_config.enable_in_lifter);
    }

    #[test]
    fn test_from_cli_flags_all_disabled() {
        let config = ExecutionConfig::from_cli_flags(
            false, // show_preprocessing
            false, // show_sql_transformations
            false, // case_insensitive
            false, // auto_hide_empty
            true,  // no_expression_lifter
            true,  // no_where_expansion
            true,  // no_group_by_expansion
            true,  // no_having_expansion
            true,  // no_cte_hoister
            true,  // no_in_lifter
            false, // debug_trace
        );

        assert!(!config.show_preprocessing);
        assert!(!config.show_sql_transformations);
        assert!(!config.case_insensitive);
        assert!(!config.transformer_config.enable_expression_lifter);
        assert!(!config.transformer_config.enable_where_expansion);
        assert!(!config.transformer_config.enable_group_by_expansion);
        assert!(!config.transformer_config.enable_having_expansion);
        assert!(!config.transformer_config.enable_cte_hoister);
        assert!(!config.transformer_config.enable_in_lifter);
    }

    #[test]
    fn test_from_cli_flags_all_enabled() {
        let config = ExecutionConfig::from_cli_flags(
            true,  // show_preprocessing
            true,  // show_sql_transformations
            true,  // case_insensitive
            true,  // auto_hide_empty
            false, // no_expression_lifter
            false, // no_where_expansion
            false, // no_group_by_expansion
            false, // no_having_expansion
            false, // no_cte_hoister
            false, // no_in_lifter
            true,  // debug_trace
        );

        assert!(config.show_preprocessing);
        assert!(config.show_sql_transformations);
        assert!(config.case_insensitive);
        assert!(config.auto_hide_empty);
        assert!(config.debug_trace);
        assert!(config.transformer_config.enable_expression_lifter);
        assert!(config.transformer_config.enable_where_expansion);
        assert!(config.transformer_config.enable_group_by_expansion);
        assert!(config.transformer_config.enable_having_expansion);
        assert!(config.transformer_config.enable_cte_hoister);
        assert!(config.transformer_config.enable_in_lifter);
    }

    #[test]
    fn test_custom_transformer_config() {
        let custom_transformer = TransformerConfig {
            enable_expression_lifter: true,
            enable_where_expansion: false,
            enable_group_by_expansion: true,
            enable_having_expansion: false,
            enable_cte_hoister: true,
            enable_in_lifter: false,
        };

        let config = ExecutionConfig::new().with_transformer_config(custom_transformer.clone());

        assert!(config.transformer_config.enable_expression_lifter);
        assert!(!config.transformer_config.enable_where_expansion);
        assert!(config.transformer_config.enable_group_by_expansion);
        assert!(!config.transformer_config.enable_having_expansion);
        assert!(config.transformer_config.enable_cte_hoister);
        assert!(!config.transformer_config.enable_in_lifter);
    }
}
