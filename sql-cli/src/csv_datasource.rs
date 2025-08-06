use crate::api_client::{QueryInfo, QueryResponse};
use crate::csv_fixes::{build_column_lookup, find_column_case_insensitive, parse_column_name};
use crate::recursive_parser::Parser;
use crate::where_ast::evaluate_where_expr;
use crate::where_parser::WhereParser;
use anyhow::Result;
use csv;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CsvDataSource {
    data: Vec<Value>,
    headers: Vec<String>,
    table_name: String,
    column_lookup: HashMap<String, String>,
}

impl CsvDataSource {
    pub fn load_from_file<P: AsRef<Path>>(path: P, table_name: &str) -> Result<Self> {
        let file = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);

        // Get headers
        let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

        // Read all records into JSON values
        let mut data = Vec::new();
        for result in reader.records() {
            let record = result?;
            let mut row = serde_json::Map::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    // Try to parse as number, otherwise store as string
                    let value = if field.is_empty() {
                        Value::Null
                    } else if let Ok(n) = field.parse::<f64>() {
                        json!(n)
                    } else {
                        Value::String(field.to_string())
                    };
                    row.insert(header.clone(), value);
                }
            }

            data.push(Value::Object(row));
        }

        let column_lookup = build_column_lookup(&headers);

        Ok(CsvDataSource {
            data,
            headers,
            table_name: table_name.to_string(),
            column_lookup,
        })
    }

    pub fn load_from_json_file<P: AsRef<Path>>(path: P, table_name: &str) -> Result<Self> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Parse JSON array
        let json_data: Vec<Value> = serde_json::from_reader(reader)?;

        if json_data.is_empty() {
            return Err(anyhow::anyhow!("JSON file contains no data"));
        }

        // Extract headers from the first record
        let headers = if let Some(first_record) = json_data.first() {
            if let Some(obj) = first_record.as_object() {
                obj.keys().cloned().collect()
            } else {
                return Err(anyhow::anyhow!("JSON records must be objects"));
            }
        } else {
            Vec::new()
        };

        // Validate all records have the same structure
        for (i, record) in json_data.iter().enumerate() {
            if !record.is_object() {
                return Err(anyhow::anyhow!("Record {} is not an object", i));
            }
        }

        let column_lookup = build_column_lookup(&headers);

        Ok(CsvDataSource {
            data: json_data,
            headers,
            table_name: table_name.to_string(),
            column_lookup,
        })
    }

    pub fn query(&self, sql: &str) -> Result<Vec<Value>> {
        // Parse SQL using the recursive parser to extract ORDER BY
        let mut parser = Parser::new(sql);
        match parser.parse() {
            Ok(stmt) => {
                let mut results = self.data.clone();

                // Handle WHERE clause using the existing WhereParser
                let sql_lower = sql.to_lowercase();
                if let Some(where_pos) = sql_lower.find(" where ") {
                    // Extract WHERE clause, but stop at ORDER BY if present
                    let where_start = where_pos + 7;
                    let where_end = if let Some(order_pos) = sql_lower.find(" order by ") {
                        order_pos.min(sql.len())
                    } else {
                        sql.len()
                    };
                    let where_clause = sql[where_start..where_end].trim();
                    results = self.filter_results(results, where_clause)?;
                }

                // Handle specific column selection
                if !stmt.columns.contains(&"*".to_string()) {
                    let columns: Vec<&str> = stmt.columns.iter().map(|s| s.as_str()).collect();
                    results = self.select_columns(results, &columns)?;
                }

                // Handle ORDER BY clause
                if let Some(order_by_columns) = &stmt.order_by {
                    results = self.sort_results(results, order_by_columns)?;
                }

                Ok(results)
            }
            Err(_) => {
                // Fallback to simple parsing for backward compatibility
                let sql_lower = sql.to_lowercase();

                if sql_lower.contains("select") {
                    let mut results = self.data.clone();

                    // Handle WHERE clause
                    if let Some(where_pos) = sql_lower.find(" where ") {
                        // Extract WHERE clause, but stop at ORDER BY if present
                        let where_start = where_pos + 7;
                        let where_end = if let Some(order_pos) = sql_lower.find(" order by ") {
                            order_pos.min(sql.len())
                        } else {
                            sql.len()
                        };
                        let where_clause = sql[where_start..where_end].trim();
                        results = self.filter_results(results, where_clause)?;
                    }

                    // Handle specific column selection
                    if !sql_lower.contains("select *") {
                        let select_start = sql_lower.find("select").unwrap() + 6;
                        let from_pos = sql_lower.find("from").unwrap_or(sql.len());
                        let columns_str = sql[select_start..from_pos].trim();

                        if !columns_str.is_empty() && columns_str != "*" {
                            let columns: Vec<&str> = columns_str
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                                .collect();
                            results = self.select_columns(results, &columns)?;
                        }
                    }

                    // Handle ORDER BY clause
                    if let Some(order_pos) = sql_lower.find(" order by ") {
                        let order_start = order_pos + 10; // Skip " order by "
                        let order_clause = sql[order_start..].trim();

                        // Parse ORDER BY columns (simple comma-separated list)
                        let order_columns: Vec<String> = order_clause
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();

                        if !order_columns.is_empty() {
                            results = self.sort_results(results, &order_columns)?;
                        }
                    }

                    Ok(results)
                } else {
                    Err(anyhow::anyhow!(
                        "Only SELECT queries are supported for CSV files"
                    ))
                }
            }
        }
    }

    fn filter_results(&self, data: Vec<Value>, where_clause: &str) -> Result<Vec<Value>> {
        // Parse WHERE clause into AST
        let expr = WhereParser::parse(where_clause)?;

        let mut filtered = Vec::new();
        for row in data {
            if evaluate_where_expr(&expr, &row)? {
                filtered.push(row);
            }
        }

        Ok(filtered)
    }

    fn select_columns(&self, data: Vec<Value>, columns: &[&str]) -> Result<Vec<Value>> {
        let mut results = Vec::new();

        for row in data {
            if let Some(obj) = row.as_object() {
                let mut new_row = serde_json::Map::new();

                for &col in columns {
                    let col_parsed = parse_column_name(col);

                    if let Some((actual_key, value)) =
                        find_column_case_insensitive(obj, col_parsed, &self.column_lookup)
                    {
                        new_row.insert(actual_key.clone(), value.clone());
                    }
                }

                results.push(Value::Object(new_row));
            }
        }

        Ok(results)
    }

    fn sort_results(
        &self,
        mut data: Vec<Value>,
        order_by_columns: &[String],
    ) -> Result<Vec<Value>> {
        if order_by_columns.is_empty() {
            return Ok(data);
        }

        // Sort by multiple columns with proper type-aware comparison
        data.sort_by(|a, b| {
            for column_name in order_by_columns {
                let col_parsed = parse_column_name(column_name);

                let val_a = if let Some(obj_a) = a.as_object() {
                    find_column_case_insensitive(obj_a, col_parsed, &self.column_lookup)
                        .map(|(_, v)| v)
                } else {
                    None
                };

                let val_b = if let Some(obj_b) = b.as_object() {
                    find_column_case_insensitive(obj_b, col_parsed, &self.column_lookup)
                        .map(|(_, v)| v)
                } else {
                    None
                };

                let cmp = match (val_a, val_b) {
                    (Some(Value::Number(a)), Some(Value::Number(b))) => {
                        // Numeric comparison - handles integers and floats properly
                        let a_f64 = a.as_f64().unwrap_or(0.0);
                        let b_f64 = b.as_f64().unwrap_or(0.0);
                        a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
                    }
                    (Some(Value::String(a)), Some(Value::String(b))) => {
                        // String comparison
                        a.cmp(b)
                    }
                    (Some(Value::Bool(a)), Some(Value::Bool(b))) => {
                        // Boolean comparison (false < true)
                        a.cmp(b)
                    }
                    (Some(Value::Null), Some(Value::Null)) => Ordering::Equal,
                    (Some(Value::Null), Some(_)) => {
                        // NULL comes first
                        Ordering::Less
                    }
                    (Some(_), Some(Value::Null)) => {
                        // NULL comes first
                        Ordering::Greater
                    }
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => {
                        // Missing values come first
                        Ordering::Less
                    }
                    (Some(_), None) => {
                        // Missing values come first
                        Ordering::Greater
                    }
                    // Mixed type comparisons - convert to strings
                    (Some(a), Some(b)) => {
                        let a_str = match a {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => a.to_string(),
                        };
                        let b_str = match b {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => b.to_string(),
                        };
                        a_str.cmp(&b_str)
                    }
                };

                // If this column comparison is not equal, return the result
                if cmp != Ordering::Equal {
                    return cmp;
                }

                // Otherwise, continue to the next column for tie-breaking
            }

            // All columns are equal
            Ordering::Equal
        });

        Ok(data)
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    pub fn get_table_name(&self) -> &str {
        &self.table_name
    }

    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }
}

// Integration with ApiClient
#[derive(Clone)]
pub struct CsvApiClient {
    datasource: Option<CsvDataSource>,
}

impl CsvApiClient {
    pub fn new() -> Self {
        Self { datasource: None }
    }

    pub fn load_csv<P: AsRef<Path>>(&mut self, path: P, table_name: &str) -> Result<()> {
        self.datasource = Some(CsvDataSource::load_from_file(path, table_name)?);
        Ok(())
    }

    pub fn load_json<P: AsRef<Path>>(&mut self, path: P, table_name: &str) -> Result<()> {
        self.datasource = Some(CsvDataSource::load_from_json_file(path, table_name)?);
        Ok(())
    }

    pub fn query_csv(&self, sql: &str) -> Result<QueryResponse> {
        if let Some(ref ds) = self.datasource {
            let data = ds.query(sql)?;
            let count = data.len();

            Ok(QueryResponse {
                data,
                count,
                query: QueryInfo {
                    select: vec!["*".to_string()],
                    where_clause: None,
                    order_by: None,
                },
                source: Some("file".to_string()),
                table: Some(ds.table_name.clone()),
                cached: Some(false),
            })
        } else {
            Err(anyhow::anyhow!("No CSV file loaded"))
        }
    }

    pub fn get_schema(&self) -> Option<HashMap<String, Vec<String>>> {
        self.datasource.as_ref().map(|ds| {
            let mut schema = HashMap::new();
            schema.insert(ds.get_table_name().to_string(), ds.get_headers().to_vec());
            schema
        })
    }

    pub fn load_from_json(&mut self, data: Vec<Value>, table_name: &str) -> Result<()> {
        // Extract headers from the first row
        let headers: Vec<String> = if let Some(first_row) = data.first() {
            if let Some(obj) = first_row.as_object() {
                obj.keys().map(|k| k.to_string()).collect()
            } else {
                return Err(anyhow::anyhow!("Invalid JSON data format"));
            }
        } else {
            return Err(anyhow::anyhow!("Empty data set"));
        };

        let column_lookup = build_column_lookup(&headers);

        self.datasource = Some(CsvDataSource {
            data: data.clone(),
            headers,
            table_name: table_name.to_string(),
            column_lookup,
        });

        Ok(())
    }
}
