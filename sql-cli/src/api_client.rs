use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Serialize)]
pub struct QueryRequest {
    pub select: Vec<String>,
    pub where_clause: Option<String>,
    pub order_by: Option<String>,
    pub skip: Option<usize>,
    pub take: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryResponse {
    pub data: Vec<Value>,
    pub count: usize,
    pub query: QueryInfo,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryInfo {
    pub select: Vec<String>,
    pub where_clause: Option<String>,
    pub order_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SchemaResponse {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
    pub is_nullable: bool,
}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }
    
    pub fn query_trades(&self, sql: &str) -> Result<QueryResponse, Box<dyn Error>> {
        // Parse the SQL to extract components
        let (select_fields, where_clause, order_by) = self.parse_sql(sql)?;
        
        let request = QueryRequest {
            select: select_fields,
            where_clause,
            order_by,
            skip: None,
            take: Some(100),
        };
        
        // Build JSON request, only including non-null fields
        let mut json_request = serde_json::json!({
            "select": request.select,
            "skip": request.skip,
            "take": request.take,
        });
        
        if let Some(where_clause) = &request.where_clause {
            json_request["where"] = serde_json::Value::String(where_clause.clone());
        }
        
        if let Some(order_by) = &request.order_by {
            json_request["orderBy"] = serde_json::Value::String(order_by.clone());
        }
        
        println!("[DEBUG] Sending request: {}", serde_json::to_string_pretty(&json_request)?);
        
        let response = self.client
            .post(format!("{}/api/trade/query", self.base_url))
            .json(&json_request)
            .send()?;
            
        if !response.status().is_success() {
            let error_text = response.text()?;
            return Err(format!("API Error: {}", error_text).into());
        }
        
        let result: QueryResponse = response.json()?;
        Ok(result)
    }
    
    pub fn get_schema(&self) -> Result<SchemaResponse, Box<dyn Error>> {
        let response = self.client
            .get(format!("{}/api/trade/schema/trade_deal", self.base_url))
            .send()?;
            
        if !response.status().is_success() {
            return Err("Failed to fetch schema".into());
        }
        
        let schema: SchemaResponse = response.json()?;
        Ok(schema)
    }
    
    fn parse_sql(&self, sql: &str) -> Result<(Vec<String>, Option<String>, Option<String>), Box<dyn Error>> {
        let sql_lower = sql.to_lowercase();
        
        // Extract SELECT fields
        let select_start = sql_lower.find("select").ok_or("SELECT not found")? + 6;
        let from_pos = sql_lower.find("from").ok_or("FROM not found")?;
        let select_part = sql[select_start..from_pos].trim();
        
        let select_fields: Vec<String> = if select_part == "*" {
            vec!["*".to_string()]
        } else {
            select_part.split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };
        
        // Extract WHERE clause
        let where_clause = if let Some(where_pos) = sql_lower.find("where") {
            let where_start = where_pos + 5;
            let order_pos = sql_lower.find("order by");
            let where_end = order_pos.unwrap_or(sql.len());
            Some(sql[where_start..where_end].trim().to_string())
        } else {
            None
        };
        
        // Extract ORDER BY
        let order_by = if let Some(order_pos) = sql_lower.find("order by") {
            let order_start = order_pos + 8;
            Some(sql[order_start..].trim().to_string())
        } else {
            None
        };
        
        Ok((select_fields, where_clause, order_by))
    }
}