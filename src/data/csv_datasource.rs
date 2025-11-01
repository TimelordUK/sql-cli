use crate::api_client::{QueryInfo, QueryResponse};
use crate::csv_fixes::{build_column_lookup, find_column_case_insensitive, parse_column_name};
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::recursive_parser::{OrderByItem, Parser, SortDirection};
use crate::sql::parser::ast::SqlExpression;
use crate::where_ast::evaluate_where_expr_with_options;
use crate::where_parser::WhereParser;
use anyhow::Result;
use csv;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct CsvDataSource {
    data: Vec<Value>,
    headers: Vec<String>,
    table_name: String,
    column_lookup: HashMap<String, String>,
}

impl CsvDataSource {
    pub fn load_from_file<P: AsRef<Path>>(path: P, table_name: &str) -> Result<Self> {
        use std::io::{BufRead, BufReader as IOBufReader};

        let file = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);

        // Get headers
        let headers: Vec<String> = reader
            .headers()?
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        // Re-open file to read raw lines for null detection
        let file2 = File::open(&path)?;
        let mut line_reader = IOBufReader::new(file2);
        let mut raw_line = String::new();
        // Skip header line
        line_reader.read_line(&mut raw_line)?;

        // Read all records into JSON values
        let mut data = Vec::new();
        for result in reader.records() {
            let record = result?;

            // Read the corresponding raw line
            raw_line.clear();
            line_reader.read_line(&mut raw_line)?;

            let mut row = serde_json::Map::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    // Parse the raw line to check if this field was truly empty (,,)
                    // vs quoted empty ("") - this is a simple heuristic
                    let value = if field.is_empty() {
                        // Check if this was preceded/followed by consecutive commas in raw line
                        // This is a simplified check - for a robust solution we'd need
                        // a full CSV parser that preserves quote information
                        if Self::is_null_field(&raw_line, i) {
                            Value::Null
                        } else {
                            // Quoted empty string
                            Value::String(String::new())
                        }
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

    /// Helper to detect if a field in the raw CSV line is a null (unquoted empty)
    /// This is a heuristic - consecutive commas indicate null
    fn is_null_field(raw_line: &str, field_index: usize) -> bool {
        // Count commas to find the field position
        let mut comma_count = 0;
        let mut in_quotes = false;
        let mut field_start = 0;
        let mut prev_char = ' ';

        for (i, ch) in raw_line.chars().enumerate() {
            if ch == '"' && prev_char != '\\' {
                in_quotes = !in_quotes;
            }

            if ch == ',' && !in_quotes {
                if comma_count == field_index {
                    let field_end = i;
                    let field_content = &raw_line[field_start..field_end].trim();
                    // If empty, check if it was quoted (quoted empty = empty string, unquoted empty = NULL)
                    if field_content.is_empty() {
                        return true; // Unquoted empty field -> NULL
                    }
                    // If it starts and ends with quotes but is empty inside, it's an empty string, not NULL
                    if field_content.starts_with('"')
                        && field_content.ends_with('"')
                        && field_content.len() == 2
                    {
                        return false; // Quoted empty field -> empty string
                    }
                    return false; // Non-empty field -> not NULL
                }
                comma_count += 1;
                field_start = i + 1;
            }
            prev_char = ch;
        }

        // Check last field
        if comma_count == field_index {
            let field_content = raw_line[field_start..]
                .trim()
                .trim_end_matches('\n')
                .trim_end_matches('\r');
            // If empty, check if it was quoted
            if field_content.is_empty() {
                return true; // Unquoted empty field -> NULL
            }
            // If it starts and ends with quotes but is empty inside, it's an empty string, not NULL
            if field_content.starts_with('"')
                && field_content.ends_with('"')
                && field_content.len() == 2
            {
                return false; // Quoted empty field -> empty string
            }
            return false; // Non-empty field -> not NULL
        }

        false // Field not found -> not NULL (shouldn't happen)
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
        self.query_with_options(sql, false)
    }

    pub fn query_with_options(&self, sql: &str, case_insensitive: bool) -> Result<Vec<Value>> {
        // Parse SQL using the recursive parser - NO FALLBACK, recursive parser only
        let mut parser = Parser::new(sql);
        let stmt = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Recursive parser error: {}", e))?;

        let mut results = self.data.clone();

        // Handle WHERE clause using the existing WhereParser
        let sql_lower = sql.to_lowercase();
        if let Some(where_pos) = sql_lower.find(" where ") {
            // Extract WHERE clause, but stop at ORDER BY, LIMIT, or OFFSET if present
            let where_start = where_pos + 7;
            let mut where_end = sql.len();

            // Find the earliest of ORDER BY, LIMIT, or OFFSET to terminate WHERE clause
            if let Some(order_pos) = sql_lower.find(" order by ") {
                where_end = where_end.min(order_pos);
            }
            if let Some(limit_pos) = sql_lower.find(" limit ") {
                where_end = where_end.min(limit_pos);
            }
            if let Some(offset_pos) = sql_lower.find(" offset ") {
                where_end = where_end.min(offset_pos);
            }

            let where_clause = sql[where_start..where_end].trim();
            results = self.filter_results_with_options(results, where_clause, case_insensitive)?;
        }

        // Handle specific column selection
        if !stmt.columns.contains(&"*".to_string()) {
            let columns: Vec<&str> = stmt
                .columns
                .iter()
                .map(std::string::String::as_str)
                .collect();
            results = self.select_columns(results, &columns)?;
        }

        // Handle ORDER BY clause
        if let Some(order_by_columns) = &stmt.order_by {
            results = self.sort_results(results, order_by_columns)?;
        }

        // Handle OFFSET and LIMIT clauses
        if let Some(offset) = stmt.offset {
            results = results.into_iter().skip(offset).collect();
        }

        if let Some(limit) = stmt.limit {
            results = results.into_iter().take(limit).collect();
        }

        Ok(results)
    }

    fn filter_results(&self, data: Vec<Value>, where_clause: &str) -> Result<Vec<Value>> {
        self.filter_results_with_options(data, where_clause, false)
    }

    fn filter_results_with_options(
        &self,
        data: Vec<Value>,
        where_clause: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Value>> {
        // Parse WHERE clause into AST with column context
        let columns = self.headers.clone();
        let expr = WhereParser::parse_with_options(where_clause, columns, case_insensitive)?;

        let mut filtered = Vec::new();
        for row in data {
            if evaluate_where_expr_with_options(&expr, &row, case_insensitive)? {
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
        order_by_columns: &[OrderByItem],
    ) -> Result<Vec<Value>> {
        if order_by_columns.is_empty() {
            return Ok(data);
        }

        // Sort by multiple columns with proper type-aware comparison
        data.sort_by(|a, b| {
            for order_col in order_by_columns {
                // Extract column name from expression (currently only supports simple columns)
                let column_name = match &order_col.expr {
                    SqlExpression::Column(col_ref) => col_ref.name.clone(),
                    _ => continue, // Skip non-column expressions for now (TODO: evaluate expressions)
                };

                let col_parsed = parse_column_name(&column_name);

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
                            Value::Null => String::new(),
                            _ => a.to_string(),
                        };
                        let b_str = match b {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => String::new(),
                            _ => b.to_string(),
                        };
                        a_str.cmp(&b_str)
                    }
                };

                // Apply sort direction (ASC or DESC)
                let final_cmp = match order_col.direction {
                    SortDirection::Asc => cmp,
                    SortDirection::Desc => cmp.reverse(),
                };

                // If this column comparison is not equal, return the result
                if final_cmp != Ordering::Equal {
                    return final_cmp;
                }

                // Otherwise, continue to the next column for tie-breaking
            }

            // All columns are equal
            Ordering::Equal
        });

        Ok(data)
    }

    #[must_use]
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    #[must_use]
    pub fn get_table_name(&self) -> &str {
        &self.table_name
    }

    #[must_use]
    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }

    /// V49: Convert `CsvDataSource` directly to `DataTable`
    /// This avoids the JSON intermediate format
    pub fn to_datatable(&self) -> DataTable {
        debug!(
            "V49: Converting CsvDataSource to DataTable for table '{}'",
            self.table_name
        );

        let mut table = DataTable::new(&self.table_name);

        // Create columns from headers
        for header in &self.headers {
            table.add_column(DataColumn::new(header.clone()));
        }

        // Convert each JSON row to DataRow
        for (row_idx, json_row) in self.data.iter().enumerate() {
            if let Some(obj) = json_row.as_object() {
                let mut values = Vec::new();

                // Get values in the same order as headers
                for header in &self.headers {
                    let value = obj
                        .get(header)
                        .map_or(DataValue::Null, json_value_to_data_value);
                    values.push(value);
                }

                if let Err(e) = table.add_row(DataRow::new(values)) {
                    debug!("V49: Failed to add row {}: {}", row_idx, e);
                }
            }
        }

        // Infer column types from the data
        table.infer_column_types();

        // Add metadata
        table
            .metadata
            .insert("source".to_string(), "csv".to_string());
        table
            .metadata
            .insert("original_count".to_string(), self.data.len().to_string());

        debug!(
            "V49: Created DataTable with {} rows and {} columns directly from CSV",
            table.row_count(),
            table.column_count()
        );

        table
    }
}

/// V49: Helper function to convert JSON value to `DataValue`
fn json_value_to_data_value(json: &Value) -> DataValue {
    match json {
        Value::Null => DataValue::Null,
        Value::Bool(b) => DataValue::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                DataValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                DataValue::Float(f)
            } else {
                DataValue::String(n.to_string())
            }
        }
        Value::String(s) => {
            // Try to detect if it's a date/time
            if s.contains('-') && s.len() >= 8 && s.len() <= 30 {
                // Simple heuristic for dates
                DataValue::DateTime(s.clone())
            } else {
                DataValue::String(s.clone())
            }
        }
        Value::Array(_) | Value::Object(_) => {
            // Store complex types as JSON string
            DataValue::String(json.to_string())
        }
    }
}

// Integration with ApiClient
#[derive(Clone)]
pub struct CsvApiClient {
    datasource: Option<CsvDataSource>,
    case_insensitive: bool,
}

impl Default for CsvApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CsvApiClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            datasource: None,
            case_insensitive: false,
        }
    }

    pub fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
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
            let data = ds.query_with_options(sql, self.case_insensitive)?;
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

    #[must_use]
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
                obj.keys().map(std::string::ToString::to_string).collect()
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

    /// V49: Get `DataTable` directly from the datasource
    /// This avoids JSON intermediate conversion
    #[must_use]
    pub fn get_datatable(&self) -> Option<DataTable> {
        self.datasource.as_ref().map(|ds| {
            debug!("V49: CsvApiClient returning DataTable directly");
            ds.to_datatable()
        })
    }

    /// V49: Check if we have a datasource loaded
    #[must_use]
    pub fn has_data(&self) -> bool {
        self.datasource.is_some()
    }
}
