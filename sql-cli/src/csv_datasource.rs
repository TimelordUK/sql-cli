use std::path::Path;
use std::fs::File;
use csv;
use serde_json::{json, Value};
use std::error::Error;
use anyhow::Result;
use std::collections::HashMap;
use crate::api_client::{QueryResponse, QueryInfo};

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
            if let Some(where_pos) = sql_lower.find("where") {
                let where_clause = &sql[where_pos + 5..];
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
    
    fn evaluate_where_clause(&self, row: &Value, clause: &str) -> Result<bool> {
        // Simple WHERE clause evaluation
        // Support basic patterns like: column = "value", column > number, column.Contains("text")
        
        // Handle AND conditions
        if clause.contains(" AND ") {
            let parts: Vec<&str> = clause.split(" AND ").collect();
            for part in parts {
                if !self.evaluate_where_clause(row, part.trim())? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        
        // Handle OR conditions
        if clause.contains(" OR ") {
            let parts: Vec<&str> = clause.split(" OR ").collect();
            for part in parts {
                if self.evaluate_where_clause(row, part.trim())? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        
        // Handle .Length() method - e.g., column.Length() > 5
        if clause.contains(".Length()") {
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
        
        if clause.contains(".Contains(") {
            // Handle .Contains() method
            let parts: Vec<&str> = clause.split(".Contains(").collect();
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
        } else if clause.contains('>') {
            // Handle greater than
            let parts: Vec<&str> = clause.split('>').collect();
            if parts.len() == 2 {
                let column = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value = parts[1].trim();
                
                if let Some(field_value) = row.get(column) {
                    if let Some(n) = field_value.as_f64() {
                        if let Ok(search_num) = value.parse::<f64>() {
                            return Ok(n > search_num);
                        }
                    }
                }
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