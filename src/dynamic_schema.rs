use crate::app_paths::AppPaths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub description: Option<String>,
    pub row_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub methods: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaResponse {
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSchemasResponse {
    pub schemas: HashMap<String, TableSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSchema {
    pub schemas: HashMap<String, TableSchema>,
    pub last_updated: DateTime<Utc>,
    pub server_url: String,
}

pub struct SchemaManager {
    cache_path: PathBuf,
    cached_schema: Option<CachedSchema>,
    api_client: crate::api_client::ApiClient,
}

impl SchemaManager {
    pub fn new(api_client: crate::api_client::ApiClient) -> Self {
        let cache_path = AppPaths::schemas_file().unwrap_or_else(|_| PathBuf::from("schemas.json"));

        Self {
            cache_path,
            cached_schema: None,
            api_client,
        }
    }

    pub fn load_schema(
        &mut self,
    ) -> Result<HashMap<String, TableSchema>, Box<dyn std::error::Error>> {
        // Try to fetch from server first
        if let Ok(schemas) = self.fetch_from_server() {
            self.save_cache(&schemas)?;
            return Ok(schemas);
        }

        // Fall back to cache
        if let Ok(schemas) = self.load_from_cache() {
            // Using cached schema (server unavailable)
            return Ok(schemas);
        }

        // Last resort: use default schema
        // Using default schema (no server or cache available)
        Ok(self.get_default_schema())
    }

    fn fetch_from_server(
        &self,
    ) -> Result<HashMap<String, TableSchema>, Box<dyn std::error::Error>> {
        // For now, return error to trigger fallback
        // TODO: Implement actual API call when server endpoint is ready
        Err("Schema API not yet implemented".into())
    }

    fn load_from_cache(
        &mut self,
    ) -> Result<HashMap<String, TableSchema>, Box<dyn std::error::Error>> {
        if !self.cache_path.exists() {
            return Err("No cache file found".into());
        }

        let content = fs::read_to_string(&self.cache_path)?;
        let cached: CachedSchema = serde_json::from_str(&content)?;

        // Check if cache is less than 24 hours old
        let age = Utc::now() - cached.last_updated;
        if age.num_hours() > 24 {
            // Warning: Schema cache is old
        }

        self.cached_schema = Some(cached.clone());
        Ok(cached.schemas)
    }

    fn save_cache(
        &self,
        schemas: &HashMap<String, TableSchema>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cached = CachedSchema {
            schemas: schemas.clone(),
            last_updated: Utc::now(),
            server_url: self.api_client.base_url.clone(),
        };

        // Create directory if it doesn't exist
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&cached)?;
        fs::write(&self.cache_path, json)?;

        Ok(())
    }

    fn get_default_schema(&self) -> HashMap<String, TableSchema> {
        // Convert from existing schema.json format
        let config = crate::schema_config::load_schema_config();
        let mut schemas = HashMap::new();

        for table_config in config.tables {
            let columns: Vec<ColumnInfo> = table_config
                .columns
                .iter()
                .map(|name| {
                    ColumnInfo {
                        name: name.clone(),
                        data_type: self.infer_type(name),
                        nullable: true, // Conservative default
                        description: None,
                    }
                })
                .collect();

            let mut methods = HashMap::new();
            methods.insert(
                "string".to_string(),
                vec![
                    "Contains".to_string(),
                    "StartsWith".to_string(),
                    "EndsWith".to_string(),
                ],
            );
            methods.insert("datetime".to_string(), vec!["DateTime".to_string()]);

            schemas.insert(
                table_config.name.clone(),
                TableSchema {
                    table_name: table_config.name,
                    columns,
                    methods,
                },
            );
        }

        schemas
    }

    fn infer_type(&self, column_name: &str) -> String {
        // Simple type inference based on column name
        if column_name.ends_with("Date") || column_name.ends_with("Time") {
            "datetime".to_string()
        } else if column_name.ends_with("Id") || column_name.ends_with("Name") {
            "string".to_string()
        } else if column_name == "price"
            || column_name == "quantity"
            || column_name == "commission"
            || column_name.ends_with("Amount")
        {
            "decimal".to_string()
        } else {
            "string".to_string()
        }
    }

    pub fn get_tables(&self) -> Vec<String> {
        if let Some(ref cached) = self.cached_schema {
            cached.schemas.keys().cloned().collect()
        } else {
            vec!["trade_deal".to_string()]
        }
    }

    pub fn get_columns(&self, table: &str) -> Vec<String> {
        if let Some(ref cached) = self.cached_schema {
            if let Some(schema) = cached.schemas.get(table) {
                return schema.columns.iter().map(|c| c.name.clone()).collect();
            }
        }

        // Fallback to default
        self.get_default_schema()
            .get(table)
            .map(|s| s.columns.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn refresh_schema(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let schemas = self.fetch_from_server()?;
        self.save_cache(&schemas)?;
        Ok(())
    }
}
