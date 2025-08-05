use std::path::Path;
use std::fs::File;
use csv;
use serde_json::{json, Value};
use std::error::Error;
use anyhow::Result;
use std::collections::HashMap;
use crate::api_client::{QueryResponse, QueryInfo};
use chrono::{Local, NaiveDateTime, Datelike};

#[derive(Clone)]
pub struct CsvDataSource {
    data: Vec<Value>,
    headers: Vec<String>,
    table_name: String,
}

impl CsvDataSource {
    pub fn load_from_file<P: AsRef<Path>>(path: P, table_name: &str) -> Result<Self> {
        let file = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);
        
        // Get headers
        let headers: Vec<String> = reader.headers()?
            .iter()
            .map(|h| h.to_string())
            .collect();
        
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
        
        Ok(CsvDataSource {
            data,
            headers,
            table_name: table_name.to_string(),
        })
    }
    
    pub fn query(&self, sql: &str) -> Result<Vec<Value>> {
        // Simple SQL parsing for basic queries
        let sql_lower = sql.to_lowercase();
        
        // For now, support simple SELECT * or SELECT cols queries
        if sql_lower.contains("select") {
            let mut results = self.data.clone();
            
            // Handle WHERE clause
            if let Some(where_pos) = sql_lower.find(" where ") {
                let where_clause = &sql[where_pos + 7..];  // Skip " where "
                results = self.filter_results(results, where_clause)?;
            }
            
            // Handle specific column selection
            if !sql_lower.contains("select *") {
                let select_start = sql_lower.find("select").unwrap() + 6;
                let from_pos = sql_lower.find("from").unwrap_or(sql.len());
                let columns_str = sql[select_start..from_pos].trim();
                
                if !columns_str.is_empty() && columns_str != "*" {
                    let columns: Vec<&str> = columns_str.split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                        .collect();
                    results = self.select_columns(results, &columns)?;
                }
            }
            
            Ok(results)
        } else {
            Err(anyhow::anyhow!("Only SELECT queries are supported for CSV files"))
        }
    }
    
    fn filter_results(&self, data: Vec<Value>, where_clause: &str) -> Result<Vec<Value>> {
        let mut filtered = Vec::new();
        
        for row in data {
            if self.evaluate_where_clause(&row, where_clause)? {
                filtered.push(row);
            }
        }
        
        Ok(filtered)
    }
    
    fn process_datetime_in_clause(&self, clause: &str) -> String {
        let mut processed = clause.to_string();
        
        // Process DateTime() - today at midnight
        if processed.contains("DateTime()") {
            let today = Local::now();
            let today_str = format!("{:04}-{:02}-{:02} 00:00:00", 
                today.year(), today.month(), today.day());
            processed = processed.replace("DateTime()", &format!("\"{}\"", today_str));
        }
        
        // Process DateTime(year, month, day, ...) with regex
        let datetime_pattern = regex::Regex::new(r"DateTime\((\d+),\s*(\d+),\s*(\d+)(?:,\s*(\d+))?(?:,\s*(\d+))?(?:,\s*(\d+))?\)").unwrap();
        
        processed = datetime_pattern.replace_all(&processed, |caps: &regex::Captures| {
            let year: i32 = caps[1].parse().unwrap();
            let month: u32 = caps[2].parse().unwrap();
            let day: u32 = caps[3].parse().unwrap();
            let hour: u32 = caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let minute: u32 = caps.get(5).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let second: u32 = caps.get(6).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            
            format!("\"{:04}-{:02}-{:02} {:02}:{:02}:{:02}\"", 
                year, month, day, hour, minute, second)
        }).to_string();
        
        processed
    }
    
    fn evaluate_where_clause(&self, row: &Value, clause: &str) -> Result<bool> {
        // Simple WHERE clause evaluation
        // Support basic patterns like: column = "value", column > number, column.Contains("text")
        
        // First process any DateTime constructs
        let processed_clause = self.process_datetime_in_clause(clause);
        
        // Handle AND conditions
        if processed_clause.contains(" AND ") || processed_clause.contains(" and ") {
            let parts: Vec<&str> = if processed_clause.contains(" AND ") {
                processed_clause.split(" AND ").collect()
            } else {
                processed_clause.split(" and ").collect()
            };
            for part in parts {
                if !self.evaluate_where_clause(row, part.trim())? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        
        // Handle OR conditions
        if processed_clause.contains(" OR ") || processed_clause.contains(" or ") {
            let parts: Vec<&str> = if processed_clause.contains(" OR ") {
                processed_clause.split(" OR ").collect()
            } else {
                processed_clause.split(" or ").collect()
            };
            for part in parts {
                if self.evaluate_where_clause(row, part.trim())? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        
        // Handle .Length() method - e.g., column.Length() > 5
        if processed_clause.contains(".Length()") {
            // Split by comparison operators to handle cases like column.Length() > 5
            let comparison_ops = vec![" > ", " < ", " >= ", " <= ", " = ", " == "];
            for op in comparison_ops {
                if clause.contains(op) {
                    let parts: Vec<&str> = clause.split(op).collect();
                    if parts.len() == 2 && parts[0].contains(".Length()") {
                        let column_part = parts[0].replace(".Length()", "").trim().to_string();
                        let column = column_part.trim_matches('"').trim_matches('\'');
                        
                        if let Ok(compare_value) = parts[1].trim().parse::<usize>() {
                            if let Some(field_value) = row.get(column) {
                                if let Some(s) = field_value.as_str() {
                                    let len = s.len();
                                    return Ok(match op {
                                        " > " => len > compare_value,
                                        " < " => len < compare_value,
                                        " >= " => len >= compare_value,
                                        " <= " => len <= compare_value,
                                        " = " | " == " => len == compare_value,
                                        _ => false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Handle .StartsWith() method
        if clause.contains(".StartsWith(") {
            let parts: Vec<&str> = clause.split(".StartsWith(").collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value_part = parts[1].trim_end_matches(')');
                let search_value = value_part.trim_matches('"').trim_matches('\'');
                
                if let Some(field_value) = row.get(column) {
                    if let Some(s) = field_value.as_str() {
                        return Ok(s.starts_with(search_value));
                    }
                }
            }
        }
        
        // Handle .EndsWith() method
        if clause.contains(".EndsWith(") {
            let parts: Vec<&str> = clause.split(".EndsWith(").collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value_part = parts[1].trim_end_matches(')');
                let search_value = value_part.trim_matches('"').trim_matches('\'');
                
                if let Some(field_value) = row.get(column) {
                    if let Some(s) = field_value.as_str() {
                        return Ok(s.ends_with(search_value));
                    }
                }
            }
        }
        
        if processed_clause.contains(".Contains(") {
            // Handle .Contains() method
            let parts: Vec<&str> = processed_clause.split(".Contains(").collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value_part = parts[1].trim_end_matches(')');
                let search_value = value_part.trim_matches('"').trim_matches('\'');
                
                if let Some(field_value) = row.get(column) {
                    if let Some(s) = field_value.as_str() {
                        return Ok(s.contains(search_value));
                    }
                }
            }
        } else if clause.contains('=') {
            // Handle equality
            let parts: Vec<&str> = clause.split('=').collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                
                if let Some(field_value) = row.get(column) {
                    if let Some(s) = field_value.as_str() {
                        return Ok(s == value);
                    } else if let Some(n) = field_value.as_f64() {
                        if let Ok(search_num) = value.parse::<f64>() {
                            return Ok(n == search_num);
                        }
                    }
                }
            }
        } else if processed_clause.contains('>') {
            // Handle greater than
            let parts: Vec<&str> = processed_clause.split('>').collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value = parts[1].trim();
                
                if let Some(field_value) = row.get(column) {
                    // Try numeric comparison first
                    if let Some(n) = field_value.as_f64() {
                        if let Ok(search_num) = value.parse::<f64>() {
                            return Ok(n > search_num);
                        }
                    }
                    // Try string/date comparison
                    else if let Some(s) = field_value.as_str() {
                        let compare_value = value.trim_matches('"').trim_matches('\'');
                        // For date strings, lexicographic comparison works if in ISO format
                        return Ok(s > compare_value);
                    }
                }
            }
        }
        
        // Handle IN clause - e.g., column IN ("value1", "value2")
        if processed_clause.contains(" IN (") || processed_clause.contains(" in (") {
            let in_pos = if processed_clause.contains(" IN (") {
                processed_clause.find(" IN (").unwrap()
            } else {
                processed_clause.find(" in (").unwrap()
            };
            
            let column = processed_clause[..in_pos].trim().trim_matches('"').trim_matches('\'');
            let values_part = &processed_clause[in_pos + 5..]; // Skip " IN ("
            
            if let Some(end_pos) = values_part.find(')') {
                let values_str = &values_part[..end_pos];
                let values: Vec<&str> = values_str.split(',')
                    .map(|v| v.trim().trim_matches('"').trim_matches('\''))
                    .collect();
                
                if let Some(field_value) = row.get(column) {
                    if let Some(s) = field_value.as_str() {
                        return Ok(values.contains(&s));
                    } else if let Some(n) = field_value.as_f64() {
                        let n_str = n.to_string();
                        return Ok(values.iter().any(|v| v == &n_str));
                    }
                }
                return Ok(false);
            }
        }
        
        // If we can't parse the clause, include the row
        Ok(true)
    }
    
    fn select_columns(&self, data: Vec<Value>, columns: &[&str]) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        
        for row in data {
            if let Some(obj) = row.as_object() {
                let mut new_row = serde_json::Map::new();
                
                for &col in columns {
                    if let Some(value) = obj.get(col) {
                        new_row.insert(col.to_string(), value.clone());
                    }
                }
                
                results.push(Value::Object(new_row));
            }
        }
        
        Ok(results)
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
        let headers = if let Some(first_row) = data.first() {
            if let Some(obj) = first_row.as_object() {
                obj.keys().map(|k| k.to_string()).collect()
            } else {
                return Err(anyhow::anyhow!("Invalid JSON data format"));
            }
        } else {
            return Err(anyhow::anyhow!("Empty data set"));
        };
        
        self.datasource = Some(CsvDataSource {
            data: data.clone(),
            headers,
            table_name: table_name.to_string(),
        });
        
        Ok(())
    }
}