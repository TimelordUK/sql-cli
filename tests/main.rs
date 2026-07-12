// Single test harness that includes all tests as modules
// This reduces build time from 64 separate executables to just 1

// Include all test files as modules
// The #[path] attribute lets us keep the original file names

#[path = "data_view_tests.rs"]
mod data_view_tests;

#[path = "data_view_trades_test.rs"]
mod data_view_trades_test;

#[path = "datatable_conversion_test.rs"]
mod datatable_conversion_test;

#[path = "datatable_integration_test.rs"]
mod datatable_integration_test;

#[path = "datetime_completion.rs"]
mod datetime_completion;

#[path = "temp_table_qualified_join_tests.rs"]
mod temp_table_qualified_join_tests;

#[path = "join_operand_order_tests.rs"]
mod join_operand_order_tests;

#[path = "history_protection_integration.rs"]
mod history_protection_integration;

#[path = "method_evaluation_test.rs"]
mod method_evaluation_test;

#[path = "parser_regression_tests.rs"]
mod parser_regression_tests;

#[path = "real_query_capture.rs"]
mod real_query_capture;

#[path = "test_arithmetic_operations.rs"]
mod test_arithmetic_operations;

#[path = "test_between_parsing.rs"]
mod test_between_parsing;

#[path = "test_buffer.rs"]
mod test_buffer;

#[path = "test_buffer_state_refactor.rs"]
mod test_buffer_state_refactor;

#[path = "test_cache_query.rs"]
mod test_cache_query;

#[path = "test_case_insensitive.rs"]
mod test_case_insensitive;

#[path = "test_column_search.rs"]
mod test_column_search;

#[path = "test_column_sizing.rs"]
mod test_column_sizing;

#[path = "test_column_suggestions.rs"]
mod test_column_suggestions;

#[path = "test_complex_date_queries.rs"]
mod test_complex_date_queries;

#[path = "test_complex_queries.rs"]
mod test_complex_queries;

#[path = "test_comprehensive_operators.rs"]
mod test_comprehensive_operators;

#[path = "test_datatable_provider.rs"]
mod test_datatable_provider;

#[path = "test_date_formats.rs"]
mod test_date_formats;

#[path = "test_dateadd.rs"]
mod test_dateadd;

#[path = "test_datediff.rs"]
mod test_datediff;

#[path = "test_distinct_parsing.rs"]
mod test_distinct_parsing;

#[path = "test_filter_fix.rs"]
mod test_filter_fix;

#[path = "test_function_parsing_poc.rs"]
mod test_function_parsing_poc;

#[path = "test_function_registry.rs"]
mod test_function_registry;

#[path = "test_indexof_space.rs"]
mod test_indexof_space;

#[path = "test_input_navigation.rs"]
mod test_input_navigation;

#[path = "test_interned_string_sort.rs"]
mod test_interned_string_sort;

#[path = "test_join_parser.rs"]
mod test_join_parser;

#[path = "test_json_loading.rs"]
mod test_json_loading;

#[path = "test_math_functions.rs"]
mod test_math_functions;

#[path = "test_mixed_numeric_sorting.rs"]
mod test_mixed_numeric_sorting;

#[path = "test_multi_buffer.rs"]
mod test_multi_buffer;

#[path = "test_multi_column_interned_sort.rs"]
mod test_multi_column_interned_sort;

#[path = "test_multi_column_order_by.rs"]
mod test_multi_column_order_by;

#[path = "test_null_vs_empty.rs"]
mod test_null_vs_empty;

#[path = "test_numeric_columns.rs"]
mod test_numeric_columns;

#[path = "test_numeric_comparison.rs"]
mod test_numeric_comparison;

#[path = "test_numeric_sorting.rs"]
mod test_numeric_sorting;

#[path = "test_order_by_asc_desc.rs"]
mod test_order_by_asc_desc;

#[path = "test_order_by_expressions.rs"]
mod test_order_by_expressions;

#[path = "test_original_preservation.rs"]
mod test_original_preservation;

#[path = "test_parentheses_errors.rs"]
mod test_parentheses_errors;

#[path = "test_parser_star.rs"]
mod test_parser_star;

#[path = "test_pinned_columns.rs"]
mod test_pinned_columns;

#[path = "test_query_engine.rs"]
mod test_query_engine;

#[path = "test_query_scoping.rs"]
mod test_query_scoping;

#[path = "test_scientific_notation.rs"]
mod test_scientific_notation;

#[path = "test_simple_arithmetic.rs"]
mod test_simple_arithmetic;

#[path = "test_sort_verification.rs"]
mod test_sort_verification;

#[path = "test_star_arithmetic.rs"]
mod test_star_arithmetic;

#[path = "test_textjoin.rs"]
mod test_textjoin;

#[path = "test_trades_queries.rs"]
mod test_trades_queries;

#[path = "test_trim_methods.rs"]
mod test_trim_methods;

#[path = "test_type_coercion_datetime.rs"]
mod test_type_coercion_datetime;

#[path = "test_viewport_buffer_sync.rs"]
mod test_viewport_buffer_sync;

#[path = "test_where_arithmetic.rs"]
mod test_where_arithmetic;

#[path = "test_window_context.rs"]
mod test_window_context;

#[path = "test_yanked_query.rs"]
mod test_yanked_query;

#[path = "tui_integration_test.rs"]
mod tui_integration_test;

#[path = "viewport_manager_test.rs"]
mod viewport_manager_test;
