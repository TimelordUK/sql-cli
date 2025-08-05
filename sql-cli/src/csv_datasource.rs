use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use csv;
use serde_json::{json, Value};
use std::error::Error;
use anyhow::Result;
use std::collections::HashMap;
use crate::api_client::{QueryResponse, QueryInfo};
use chrono::{Local, NaiveDateTime, Datelike};
use crate::where_parser::WhereParser;
use crate::where_ast::{WhereExpr, evaluate_where_expr};

#[derive(Clone, Debug)]
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
        
        Ok(CsvDataSource {
            data: json_data,
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