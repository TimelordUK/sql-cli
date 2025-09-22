// HTTP fetcher for WEB CTEs (now supports file:// URLs too)
use anyhow::{Context, Result};
use regex::Regex;
use serde_json;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};

use crate::data::datatable::DataTable;
use crate::data::stream_loader::{load_csv_from_reader, load_json_from_reader};
use crate::sql::parser::ast::{DataFormat, HttpMethod, WebCTESpec};

/// Fetches data from a URL and converts it to a DataTable
pub struct WebDataFetcher {
    client: reqwest::blocking::Client,
}

impl WebDataFetcher {
    pub fn new() -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("sql-cli/1.0")
            .build()?;

        Ok(Self { client })
    }

    /// Fetch data from a WEB CTE specification (supports http://, https://, and file:// URLs)
    pub fn fetch(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        info!("Fetching data from URL: {}", spec.url);

        // Check if this is a file:// URL
        if spec.url.starts_with("file://") {
            return self.fetch_file(spec, table_name);
        }

        // Regular HTTP/HTTPS handling
        // Build request based on method
        let mut request = match spec.method.as_ref().unwrap_or(&HttpMethod::GET) {
            HttpMethod::GET => self.client.get(&spec.url),
            HttpMethod::POST => self.client.post(&spec.url),
            HttpMethod::PUT => self.client.put(&spec.url),
            HttpMethod::DELETE => self.client.delete(&spec.url),
            HttpMethod::PATCH => self.client.patch(&spec.url),
        };

        // Add headers if provided
        for (key, value) in &spec.headers {
            let resolved_value = self.resolve_env_var(value)?;
            request = request.header(key, resolved_value);
        }

        // Add body if provided (typically for POST/PUT/PATCH)
        if let Some(body) = &spec.body {
            let resolved_body = self.resolve_env_var(body)?;
            request = request.body(resolved_body);
            // Set Content-Type to JSON if not already set and body looks like JSON
            if spec.body.as_ref().unwrap().trim().starts_with('{') {
                request = request.header("Content-Type", "application/json");
            }
        }

        // Execute request
        let response = request
            .send()
            .with_context(|| format!("Failed to fetch from URL: {}", spec.url))?;

        // Check status
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status {}: {}",
                response.status(),
                spec.url
            ));
        }

        // Get content type for format detection
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        debug!("Response content-type: {}", content_type);

        // Read response body
        let bytes = response.bytes()?;

        // Determine format
        let format = match &spec.format {
            Some(fmt) => fmt.clone(),
            None => self.detect_format(&spec.url, &content_type),
        };

        info!("Using format: {:?} for {}", format, spec.url);

        // If JSON_PATH is specified and format is JSON, extract the nested data first
        if let Some(json_path) = &spec.json_path {
            if matches!(format, DataFormat::JSON | DataFormat::Auto) {
                // Parse JSON and extract the path
                let json_value: serde_json::Value = serde_json::from_slice(&bytes)
                    .with_context(|| "Failed to parse JSON for path extraction")?;

                // Navigate to the specified path
                let extracted = self
                    .navigate_json_path(&json_value, json_path)
                    .with_context(|| format!("Failed to extract JSON path: {}", json_path))?;

                // Convert extracted value to bytes and parse as table
                let array_value = match extracted {
                    serde_json::Value::Array(_) => extracted.clone(),
                    _ => serde_json::Value::Array(vec![extracted.clone()]),
                };

                let extracted_bytes = serde_json::to_vec(&array_value)?;
                return self.parse_data(
                    extracted_bytes,
                    DataFormat::JSON,
                    table_name,
                    "web",
                    &spec.url,
                );
            }
        }

        // Parse based on format without JSON path extraction
        self.parse_data(bytes.to_vec(), format, table_name, "web", &spec.url)
    }

    /// Fetch data from a file:// URL
    fn fetch_file(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        // Extract path from file:// URL
        let file_path = if spec.url.starts_with("file://") {
            &spec.url[7..] // Remove "file://" prefix
        } else {
            &spec.url
        };

        info!("Reading local file: {}", file_path);

        // Check if file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }

        // Open file
        let file =
            File::open(path).with_context(|| format!("Failed to open file: {}", file_path))?;

        // Get file size for memory checks
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        debug!("File size: {} bytes", file_size);

        // Determine format from extension or spec
        let format = match &spec.format {
            Some(fmt) => fmt.clone(),
            None => self.detect_format(file_path, ""),
        };

        info!("Using format: {:?} for {}", format, file_path);

        // Parse based on format
        match format {
            DataFormat::CSV => load_csv_from_reader(file, table_name, "file", file_path)
                .with_context(|| format!("Failed to parse CSV from {}", file_path)),
            DataFormat::JSON => load_json_from_reader(file, table_name, "file", file_path)
                .with_context(|| format!("Failed to parse JSON from {}", file_path)),
            DataFormat::Auto => {
                // For files, we can't retry with the same reader, so determine based on extension
                if file_path.ends_with(".json") {
                    let file = File::open(path)?;
                    load_json_from_reader(file, table_name, "file", file_path)
                        .with_context(|| format!("Failed to parse JSON from {}", file_path))
                } else {
                    // Default to CSV for auto-detect with files
                    let file = File::open(path)?;
                    load_csv_from_reader(file, table_name, "file", file_path)
                        .with_context(|| format!("Failed to parse CSV from {}", file_path))
                }
            }
        }
    }

    /// Parse data bytes based on format
    fn parse_data(
        &self,
        bytes: Vec<u8>,
        format: DataFormat,
        table_name: &str,
        source_type: &str,
        source_path: &str,
    ) -> Result<DataTable> {
        match format {
            DataFormat::CSV => {
                let reader = Cursor::new(bytes);
                load_csv_from_reader(reader, table_name, source_type, source_path)
                    .with_context(|| format!("Failed to parse CSV from {}", source_path))
            }
            DataFormat::JSON => {
                let reader = Cursor::new(bytes);
                load_json_from_reader(reader, table_name, source_type, source_path)
                    .with_context(|| format!("Failed to parse JSON from {}", source_path))
            }
            DataFormat::Auto => {
                // Try CSV first, then JSON
                let reader_csv = Cursor::new(bytes.clone());
                match load_csv_from_reader(reader_csv, table_name, source_type, source_path) {
                    Ok(table) => Ok(table),
                    Err(_) => {
                        debug!("CSV parsing failed, trying JSON");
                        let reader_json = Cursor::new(bytes);
                        load_json_from_reader(reader_json, table_name, source_type, source_path)
                            .with_context(|| format!("Failed to parse data from {}", source_path))
                    }
                }
            }
        }
    }

    /// Detect format from URL extension or content type
    fn detect_format(&self, url: &str, content_type: &str) -> DataFormat {
        // Check content type first
        if content_type.contains("json") {
            return DataFormat::JSON;
        }
        if content_type.contains("csv") || content_type.contains("text/plain") {
            return DataFormat::CSV;
        }

        // Check URL extension
        if url.ends_with(".json") {
            DataFormat::JSON
        } else if url.ends_with(".csv") {
            DataFormat::CSV
        } else {
            // Default to auto-detect
            DataFormat::Auto
        }
    }

    /// Extract data from a specific JSON path
    fn extract_json_path(
        &self,
        _table: DataTable,
        json_path: &str,
        bytes: &[u8],
    ) -> Result<DataTable> {
        // Parse the JSON
        let json_value: serde_json::Value = serde_json::from_slice(bytes)
            .with_context(|| "Failed to parse JSON for path extraction")?;

        // Navigate to the specified path
        let extracted = self
            .navigate_json_path(&json_value, json_path)
            .with_context(|| format!("Failed to extract JSON path: {}", json_path))?;

        // If the extracted value is already an array, use it directly
        // Otherwise, wrap it in an array for consistent handling
        let array_value = match extracted {
            serde_json::Value::Array(_) => extracted.clone(),
            _ => serde_json::Value::Array(vec![extracted.clone()]),
        };

        // Convert to bytes and parse as a table
        let extracted_bytes = serde_json::to_vec(&array_value)?;

        // Re-parse the extracted JSON as a DataTable
        let reader = Cursor::new(extracted_bytes);
        load_json_from_reader(reader, "extracted", "web", json_path)
    }

    /// Navigate to a specific path in JSON structure
    fn navigate_json_path<'a>(
        &self,
        value: &'a serde_json::Value,
        path: &str,
    ) -> Result<&'a serde_json::Value> {
        let mut current = value;

        // Split path by dots (simple path navigation for now)
        // Future enhancement: support array indexing like "Result[0]"
        for part in path.split('.') {
            if part.is_empty() {
                continue;
            }

            current = current
                .get(part)
                .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in JSON", part))?;
        }

        Ok(current)
    }

    /// Resolve environment variables in values (${VAR_NAME} or $VAR_NAME syntax)
    fn resolve_env_var(&self, value: &str) -> Result<String> {
        let mut result = value.to_string();

        // Handle ${VAR} syntax - can be embedded in strings
        // Use lazy_static for better performance, but for now just compile inline
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        for cap in re.captures_iter(value) {
            let var_name = &cap[1];
            match std::env::var(var_name) {
                Ok(var_value) => {
                    result = result.replace(&cap[0], &var_value);
                }
                Err(_) => {
                    // For security, don't expose which env vars exist
                    // Just log a debug message and keep the placeholder
                    debug!(
                        "Environment variable {} not found, keeping placeholder",
                        var_name
                    );
                }
            }
        }

        // Also handle simple $VAR syntax at the start of the string
        if result.starts_with('$') && !result.starts_with("${") {
            let var_name = &result[1..];
            if let Ok(var_value) = std::env::var(var_name) {
                return Ok(var_value);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format() {
        let fetcher = WebDataFetcher::new().unwrap();

        // Test URL-based detection
        assert!(matches!(
            fetcher.detect_format("http://example.com/data.csv", ""),
            DataFormat::CSV
        ));
        assert!(matches!(
            fetcher.detect_format("http://example.com/data.json", ""),
            DataFormat::JSON
        ));

        // Test content-type detection
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", "application/json"),
            DataFormat::JSON
        ));
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", "text/csv"),
            DataFormat::CSV
        ));

        // Test auto-detect fallback
        assert!(matches!(
            fetcher.detect_format("http://example.com/data", ""),
            DataFormat::Auto
        ));
    }
}
