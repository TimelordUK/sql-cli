pub mod api_client;
pub mod app_paths;
pub mod cache;
pub mod csv_datasource;
pub mod csv_fixes;
pub mod cursor_aware_parser;
pub mod dynamic_schema;
pub mod history;
pub mod hybrid_parser;
pub mod parser;
pub mod recursive_parser;
pub mod schema_config;
pub mod virtual_table;
pub mod where_ast;
pub mod where_parser;

#[cfg(test)]
mod test_cache_query;
#[cfg(test)]
mod test_column_sizing;
#[cfg(test)]
mod test_comprehensive_operators;
#[cfg(test)]
mod test_filter_fix;
#[cfg(test)]
mod test_json_loading;
#[cfg(test)]
mod test_sort_verification;
