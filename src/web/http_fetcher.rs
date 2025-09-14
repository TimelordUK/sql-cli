// HTTP fetcher for WEB CTEs
use anyhow::{Context, Result};
use std::io::Cursor;
use std::time::Duration;
use tracing::{debug, info};

use crate::data::datatable::DataTable;
use crate::data::stream_loader::{load_csv_from_reader, load_json_from_reader};
use crate::sql::parser::ast::{DataFormat, WebCTESpec};

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

    /// Fetch data from a WEB CTE specification
    pub fn fetch(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        info!("Fetching data from URL: {}", spec.url);

        // Build request
        let mut request = self.client.get(&spec.url);

        // Add headers if provided
        for (key, value) in &spec.headers {
            let resolved_value = self.resolve_env_var(value)?;
            request = request.header(key, resolved_value);
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

        // Parse based on format
        match format {
            DataFormat::CSV => {
                let reader = Cursor::new(bytes);
                load_csv_from_reader(reader, table_name, "web", &spec.url)
                    .with_context(|| format!("Failed to parse CSV from {}", spec.url))
            }
            DataFormat::JSON => {
                let reader = Cursor::new(bytes);
                load_json_from_reader(reader, table_name, "web", &spec.url)
                    .with_context(|| format!("Failed to parse JSON from {}", spec.url))
            }
            DataFormat::Auto => {
                // Try CSV first, then JSON
                let reader_csv = Cursor::new(bytes.clone());
                match load_csv_from_reader(reader_csv, table_name, "web", &spec.url) {
                    Ok(table) => Ok(table),
                    Err(_) => {
                        debug!("CSV parsing failed, trying JSON");
                        let reader_json = Cursor::new(bytes);
                        load_json_from_reader(reader_json, table_name, "web", &spec.url)
                            .with_context(|| format!("Failed to parse data from {}", spec.url))
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

    /// Resolve environment variables in values (${VAR_NAME} syntax)
    fn resolve_env_var(&self, value: &str) -> Result<String> {
        if value.starts_with("${") && value.ends_with("}") {
            let var_name = &value[2..value.len() - 1];
            std::env::var(var_name)
                .with_context(|| format!("Environment variable {} not set", var_name))
        } else {
            Ok(value.to_string())
        }
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
