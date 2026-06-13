// Stream-based data loader that works with any Read source
// This allows the same code to handle files, HTTP responses, or any other data stream

use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use tracing::{debug, info};

use crate::data::advanced_csv_loader::StringInterner;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};

/// Options controlling how a CSV stream is parsed.
///
/// Default is RFC-4180 style: comma delimiter, header row required. Surfaces
/// (CLI flag, `READ_CSV(path, '|')`, WEB CTE `DELIMITER`) populate this struct
/// at the edge; internal layers just pass it through.
#[derive(Debug, Clone)]
pub struct CsvReadOptions {
    pub delimiter: u8,
    pub has_headers: bool,
}

impl Default for CsvReadOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
        }
    }
}

/// Pick a default delimiter from a path's extension.
///
/// `.tsv` → tab, `.psv` → pipe; everything else (including stdin `-`) → comma.
/// Case-insensitive. This is only the *auto-detect* layer — explicit overrides
/// from the CLI flag, `READ_CSV` 2nd arg, or WEB CTE `DELIMITER` win over this.
pub fn detect_delimiter_from_path(path: &str) -> u8 {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".tsv") {
        b'\t'
    } else if lower.ends_with(".psv") {
        b'|'
    } else {
        b','
    }
}

/// Parse a user-supplied delimiter string into a single byte.
///
/// Accepts:
///   - a single ASCII character (e.g. `","`, `"|"`, `";"`)
///   - the two-character escapes `"\t"`, `"\n"`, `"\r"` (typing literal tabs
///     in SQL strings or shell args is awkward, so this is the canonical form)
///
/// Rejects multi-character strings, non-ASCII, and empty strings with a clear
/// error. Caller is expected to wrap the error with context if needed.
pub fn parse_delimiter_arg(s: &str) -> anyhow::Result<u8> {
    match s {
        "\\t" | "\t" => return Ok(b'\t'),
        "\\n" => return Ok(b'\n'),
        "\\r" => return Ok(b'\r'),
        _ => {}
    }
    let bytes = s.as_bytes();
    if bytes.len() == 1 && bytes[0].is_ascii() {
        return Ok(bytes[0]);
    }
    Err(anyhow::anyhow!(
        "delimiter must be a single ASCII character (or '\\t', '\\n', '\\r'); got {:?}",
        s
    ))
}

/// Resolve which delimiter to use for a given path.
///
/// Precedence (highest first):
///   1. `explicit` override (typically from a CLI flag or 2nd arg)
///   2. extension auto-detect (`.tsv` → tab, `.psv` → pipe)
///   3. comma
pub fn resolve_delimiter(path: &str, explicit: Option<u8>) -> u8 {
    explicit.unwrap_or_else(|| detect_delimiter_from_path(path))
}

/// Human-readable form of a delimiter byte for diagnostic metadata.
fn delimiter_label(d: u8) -> String {
    match d {
        b'\t' => "\\t".to_string(),
        b'\n' => "\\n".to_string(),
        b'\r' => "\\r".to_string(),
        b => (b as char).to_string(),
    }
}

/// Column analysis results for determining interning strategy
#[derive(Debug)]
struct ColumnAnalysis {
    index: usize,
    _name: String,
    _cardinality: usize,
    _sample_size: usize,
    _unique_ratio: f64,
    is_categorical: bool,
    _avg_string_length: usize,
}

/// Advanced stream-based CSV loader with string interning
pub struct StreamCsvLoader {
    sample_size: usize,
    cardinality_threshold: f64,
    interners: HashMap<usize, StringInterner>,
}

impl StreamCsvLoader {
    pub fn new() -> Self {
        Self {
            sample_size: 1000,
            cardinality_threshold: 0.3,
            interners: HashMap::new(),
        }
    }

    /// Analyze columns to determine which should use string interning
    fn analyze_columns(
        &self,
        rows: &[Vec<String>],
        headers: &csv::StringRecord,
    ) -> Vec<ColumnAnalysis> {
        let mut analyses = Vec::new();

        for (col_idx, header) in headers.iter().enumerate() {
            let mut unique_values = HashSet::new();
            let mut total_length = 0;
            let mut non_empty_count = 0;

            // Sample rows to analyze cardinality
            for row in rows.iter().take(self.sample_size) {
                if let Some(value) = row.get(col_idx) {
                    if !value.is_empty() {
                        unique_values.insert(value.clone());
                        total_length += value.len();
                        non_empty_count += 1;
                    }
                }
            }

            let cardinality = unique_values.len();
            let sample_size = rows.len().min(self.sample_size);
            let unique_ratio = if sample_size > 0 {
                cardinality as f64 / sample_size as f64
            } else {
                1.0
            };

            let avg_string_length = if non_empty_count > 0 {
                total_length / non_empty_count
            } else {
                0
            };

            // Consider categorical if low cardinality ratio or short strings with repetition
            let is_categorical = unique_ratio < self.cardinality_threshold
                || (avg_string_length < 20 && cardinality < sample_size / 2);

            analyses.push(ColumnAnalysis {
                index: col_idx,
                _name: header.to_string(),
                _cardinality: cardinality,
                _sample_size: sample_size,
                _unique_ratio: unique_ratio,
                is_categorical,
                _avg_string_length: avg_string_length,
            });
        }

        analyses
    }

    /// Load CSV data with string interning from any Read source, using default
    /// comma-delimited options. Thin wrapper over [`load_csv_from_reader_with_opts`]
    /// kept so existing callers don't need to touch options.
    pub fn load_csv_from_reader<R: Read>(
        &mut self,
        reader: R,
        table_name: &str,
        source_type: &str,
        source_path: &str,
    ) -> Result<DataTable> {
        self.load_csv_from_reader_with_opts(
            reader,
            table_name,
            source_type,
            source_path,
            &CsvReadOptions::default(),
        )
    }

    /// Load CSV data with string interning, honouring caller-supplied options
    /// (delimiter, headers).
    pub fn load_csv_from_reader_with_opts<R: Read>(
        &mut self,
        mut reader: R,
        table_name: &str,
        source_type: &str,
        source_path: &str,
        opts: &CsvReadOptions,
    ) -> Result<DataTable> {
        info!(
            "Stream CSV load: Loading {} with optimizations (delimiter={})",
            source_path,
            delimiter_label(opts.delimiter)
        );

        // Read all data into memory
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        // First pass: Parse CSV with headers
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(opts.has_headers)
            .delimiter(opts.delimiter)
            .from_reader(&buffer[..]);

        let headers = csv_reader.headers()?.clone();
        let mut table = DataTable::new(table_name);

        // Add metadata about the source
        table
            .metadata
            .insert("source_type".to_string(), source_type.to_string());
        table
            .metadata
            .insert("source_path".to_string(), source_path.to_string());
        table
            .metadata
            .insert("delimiter".to_string(), delimiter_label(opts.delimiter));

        // Create columns from headers
        for header in &headers {
            table.add_column(DataColumn::new(header));
        }

        // Collect all rows as strings
        let mut string_rows = Vec::new();
        for result in csv_reader.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            string_rows.push(row);
        }

        // Analyze columns for string interning
        let analyses = self.analyze_columns(&string_rows, &headers);
        let categorical_columns: HashSet<usize> = analyses
            .iter()
            .filter(|a| a.is_categorical)
            .map(|a| a.index)
            .collect();

        info!(
            "Column analysis: {} of {} columns will use string interning",
            categorical_columns.len(),
            analyses.len()
        );

        // Initialize interners for categorical columns
        for col_idx in &categorical_columns {
            self.interners.insert(*col_idx, StringInterner::new());
        }

        // Second pass: Read raw lines for NULL detection
        let mut line_reader = BufReader::new(&buffer[..]);
        let mut raw_lines = Vec::new();
        let mut raw_line = String::new();

        // Skip header
        line_reader.read_line(&mut raw_line)?;
        raw_line.clear();

        // Read all raw lines
        for _ in 0..string_rows.len() {
            line_reader.read_line(&mut raw_line)?;
            raw_lines.push(raw_line.clone());
            raw_line.clear();
        }

        // Infer column types by sampling
        let mut column_types = vec![DataType::Null; headers.len()];
        let sample_size = string_rows.len().min(100);

        for row in string_rows.iter().take(sample_size) {
            for (col_idx, value) in row.iter().enumerate() {
                if !value.is_empty() {
                    let inferred = DataType::infer_from_string(value);
                    column_types[col_idx] = column_types[col_idx].merge(&inferred);
                }
            }
        }

        // Update column types
        for (col_idx, column) in table.columns.iter_mut().enumerate() {
            column.data_type = column_types[col_idx].clone();
        }

        // Convert strings to typed values and add rows
        for (row_idx, string_row) in string_rows.iter().enumerate() {
            let mut values = Vec::new();
            let raw_line = &raw_lines[row_idx];

            for (col_idx, value) in string_row.iter().enumerate() {
                let data_value = if value.is_empty() {
                    // Check if this is NULL (,,) vs empty string ("")
                    if is_null_field(raw_line, col_idx, opts.delimiter as char) {
                        DataValue::Null
                    } else if categorical_columns.contains(&col_idx) {
                        // Use interned string for empty categorical values
                        if let Some(interner) = self.interners.get_mut(&col_idx) {
                            DataValue::InternedString(interner.intern(""))
                        } else {
                            DataValue::String(String::new())
                        }
                    } else {
                        DataValue::String(String::new())
                    }
                } else if categorical_columns.contains(&col_idx)
                    && column_types[col_idx] == DataType::String
                {
                    // Use string interning for categorical columns
                    if let Some(interner) = self.interners.get_mut(&col_idx) {
                        DataValue::InternedString(interner.intern(value))
                    } else {
                        DataValue::from_string(value, &column_types[col_idx])
                    }
                } else {
                    DataValue::from_string(value, &column_types[col_idx])
                };
                values.push(data_value);
            }
            table
                .add_row(DataRow::new(values))
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        // Print interner statistics
        for (col_idx, interner) in &self.interners {
            let stats = interner.stats();
            if stats.memory_saved_bytes > 0 {
                debug!(
                    "Column {} interning: {} unique strings, {} references, {} bytes saved",
                    headers.get(*col_idx).unwrap_or(&String::new()),
                    stats.unique_strings,
                    stats.total_references,
                    stats.memory_saved_bytes
                );
            }
        }

        // Update column statistics
        table.infer_column_types();

        Ok(table)
    }
}

/// Simple wrapper for loading CSV without advanced features. Defaults to
/// comma delimiter; for other delimiters use [`load_csv_from_reader_with_opts`].
pub fn load_csv_from_reader<R: Read>(
    reader: R,
    table_name: &str,
    source_type: &str,
    source_path: &str,
) -> Result<DataTable> {
    let mut loader = StreamCsvLoader::new();
    loader.load_csv_from_reader(reader, table_name, source_type, source_path)
}

/// As [`load_csv_from_reader`], but honouring caller-supplied [`CsvReadOptions`]
/// (delimiter, headers).
pub fn load_csv_from_reader_with_opts<R: Read>(
    reader: R,
    table_name: &str,
    source_type: &str,
    source_path: &str,
    opts: &CsvReadOptions,
) -> Result<DataTable> {
    let mut loader = StreamCsvLoader::new();
    loader.load_csv_from_reader_with_opts(reader, table_name, source_type, source_path, opts)
}

/// Parse JSON content as either a JSON array of objects or JSONL
/// (newline-delimited JSON, one object per line). The format is detected by
/// peeking at the first non-whitespace byte: `[` starts an array, anything
/// else is parsed line-by-line.
///
/// Empty and whitespace-only lines are skipped in JSONL mode. Parse errors
/// in JSONL mode include the source line number.
pub fn parse_json_records(content: &str) -> Result<Vec<JsonValue>> {
    let trimmed = content.trim_start();
    if trimmed.starts_with('[') {
        return serde_json::from_str(content).with_context(|| "Failed to parse JSON array");
    }

    let mut out = Vec::new();
    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(line)
            .with_context(|| format!("Failed to parse JSONL at line {}", idx + 1))?;
        out.push(value);
    }
    Ok(out)
}

/// Navigate to a sub-value of a JSON document using a dotted path.
///
/// This is the shared "find the rows" step used by both the WEB CTE
/// `JSON_PATH` clause and `READ_JSON(path, json_path)`. It only *locates* a
/// value; it deliberately does not filter, transform, or pluck scalars — that
/// is SQL's job (use a WHERE clause / column list once the rows are loaded).
/// Anything beyond locating the row set is out of scope; pipe through `jq`.
///
/// Supports two forms per dotted segment:
///   - `name`     — descend into an object key
///   - `name[]`   — descend into `name` (must be an array), then map the
///                  remainder of the path across every element. A bare `[]`
///                  projects over the current value when it is already an array.
///
/// Example for an Elasticsearch response:
///   `hits.hits[]._source`
/// returns an array of `_source` objects, one per hit — i.e. the `_source`
/// fields become the top-level row shape consumed by the loader.
///
/// An empty path (or one that is only dots) returns the value unchanged.
pub fn navigate_json_path(value: &JsonValue, path: &str) -> Result<JsonValue> {
    let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
    walk_json_path(value, &parts)
}

fn walk_json_path(value: &JsonValue, parts: &[&str]) -> Result<JsonValue> {
    let Some((head, tail)) = parts.split_first() else {
        return Ok(value.clone());
    };

    // Array projection: `name[]` (or bare `[]`) maps the rest of the path
    // across each element of an array.
    if let Some(name) = head.strip_suffix("[]") {
        let array_val = if name.is_empty() {
            value
        } else {
            value
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in JSON", name))?
        };
        let arr = array_val.as_array().ok_or_else(|| {
            anyhow::anyhow!(
                "Expected array at '{}' for [] projection, got {}",
                if name.is_empty() { "<root>" } else { name },
                json_kind(array_val)
            )
        })?;
        let mut projected = Vec::with_capacity(arr.len());
        for el in arr {
            projected.push(walk_json_path(el, tail)?);
        }
        return Ok(JsonValue::Array(projected));
    }

    let next = value
        .get(head)
        .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in JSON", head))?;
    walk_json_path(next, tail)
}

/// Human-readable JSON type name, for error messages.
fn json_kind(value: &JsonValue) -> &'static str {
    match value {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

/// Compute the ordered union of object keys across the first `sample_size`
/// records. Order of first occurrence is preserved so the column layout is
/// stable. Non-object records are skipped.
pub fn collect_column_names(records: &[JsonValue], sample_size: usize) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut names: Vec<String> = Vec::new();
    for record in records.iter().take(sample_size) {
        if let Some(obj) = record.as_object() {
            for key in obj.keys() {
                if seen.insert(key.clone()) {
                    names.push(key.clone());
                }
            }
        }
    }
    names
}

/// Load JSON data from any Read source into a DataTable.
///
/// Accepts either a JSON array of objects (`[{...}, {...}]`) or JSONL
/// (one JSON object per line). Format is auto-detected.
pub fn load_json_from_reader<R: Read>(
    mut reader: R,
    table_name: &str,
    source_type: &str,
    source_path: &str,
) -> Result<DataTable> {
    let mut json_str = String::new();
    reader.read_to_string(&mut json_str)?;

    let json_data: Vec<JsonValue> = parse_json_records(&json_str)?;

    if json_data.is_empty() {
        return Ok(DataTable::new(table_name));
    }

    // Schema is the union of keys across the first 100 records so heterogeneous
    // JSONL streams (where later records may carry fields the first one did
    // not) don't silently drop columns.
    let column_names = collect_column_names(&json_data, 100);
    if column_names.is_empty() {
        return Err(anyhow::anyhow!(
            "JSON data must contain objects (got non-object records)"
        ));
    }

    let mut table = DataTable::new(table_name);

    // Add metadata
    table
        .metadata
        .insert("source_type".to_string(), source_type.to_string());
    table
        .metadata
        .insert("source_path".to_string(), source_path.to_string());

    for name in &column_names {
        table.add_column(DataColumn::new(name));
    }

    // Collect values for type inference
    let mut string_rows = Vec::new();
    for json_obj in &json_data {
        if let Some(obj) = json_obj.as_object() {
            let mut row = Vec::new();
            for col_name in &column_names {
                let value = obj
                    .get(col_name)
                    .map(|v| json_value_to_string(v))
                    .unwrap_or_default();
                row.push(value);
            }
            string_rows.push(row);
        }
    }

    // Infer column types
    let mut column_types = vec![DataType::Null; column_names.len()];
    let sample_size = string_rows.len().min(100);

    for row in string_rows.iter().take(sample_size) {
        for (col_idx, value) in row.iter().enumerate() {
            if !value.is_empty() && value != "null" {
                let inferred = DataType::infer_from_string(value);
                column_types[col_idx] = column_types[col_idx].merge(&inferred);
            }
        }
    }

    // Update column types
    for (col_idx, column) in table.columns.iter_mut().enumerate() {
        column.data_type = column_types[col_idx].clone();
    }

    // Convert to typed values and add rows
    for string_row in &string_rows {
        let mut values = Vec::new();
        for (col_idx, value) in string_row.iter().enumerate() {
            let data_value = if value.is_empty() || value == "null" {
                DataValue::Null
            } else {
                DataValue::from_string(value, &column_types[col_idx])
            };
            values.push(data_value);
        }
        table
            .add_row(DataRow::new(values))
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    // Update statistics
    table.infer_column_types();

    Ok(table)
}

/// Helper to convert JSON value to string for type inference
fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        JsonValue::Array(arr) => format!("{:?}", arr),
        JsonValue::Object(obj) => format!("{:?}", obj),
    }
}

/// Helper to detect NULL fields in raw CSV lines. `delimiter` is the field
/// separator character used in the source (`,` for plain CSV, `\t` for TSV, etc.).
fn is_null_field(raw_line: &str, field_index: usize, delimiter: char) -> bool {
    let mut delim_count = 0;
    let mut in_quotes = false;
    let mut field_start = 0;
    let mut prev_char = ' ';

    for (i, ch) in raw_line.char_indices() {
        if ch == '"' && prev_char != '\\' {
            in_quotes = !in_quotes;
        } else if ch == delimiter && !in_quotes {
            if delim_count == field_index {
                // Found the field - check if it's empty
                return i == field_start
                    || (i == field_start + 1
                        && raw_line.chars().nth(field_start) == Some(delimiter));
            }
            delim_count += 1;
            field_start = i + 1;
        }
        prev_char = ch;
    }

    // Check last field
    if delim_count == field_index {
        let remaining = raw_line[field_start..].trim_end();
        return remaining.is_empty() || remaining.chars().next() == Some(delimiter);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ---- navigate_json_path tests ----

    #[test]
    fn test_navigate_json_path_descends_object_key() {
        // The TeamCity-style case: object with a nested array one level down.
        let doc = serde_json::json!({
            "count": 2,
            "project": [{"id": "a"}, {"id": "b"}]
        });
        let extracted = navigate_json_path(&doc, "project").unwrap();
        assert!(extracted.is_array());
        assert_eq!(extracted.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_navigate_json_path_nested_descent() {
        // TeamCity actually wraps as { projects: { project: [...] } }.
        let doc = serde_json::json!({
            "projects": {"project": [{"id": "a"}, {"id": "b"}, {"id": "c"}]}
        });
        let extracted = navigate_json_path(&doc, "projects.project").unwrap();
        assert_eq!(extracted.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_navigate_json_path_array_projection() {
        // Elasticsearch-style: project _source out of each hit.
        let doc = serde_json::json!({
            "hits": {"hits": [
                {"_source": {"id": 1}},
                {"_source": {"id": 2}}
            ]}
        });
        let extracted = navigate_json_path(&doc, "hits.hits[]._source").unwrap();
        let arr = extracted.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[1]["id"], serde_json::json!(2));
    }

    #[test]
    fn test_navigate_json_path_bare_projection_over_root_array() {
        let doc = serde_json::json!([{"v": {"x": 1}}, {"v": {"x": 2}}]);
        let extracted = navigate_json_path(&doc, "[].v").unwrap();
        let arr = extracted.as_array().unwrap();
        assert_eq!(arr[0]["x"], serde_json::json!(1));
    }

    #[test]
    fn test_navigate_json_path_empty_path_is_identity() {
        let doc = serde_json::json!({"a": 1});
        let extracted = navigate_json_path(&doc, "").unwrap();
        assert_eq!(extracted, doc);
    }

    #[test]
    fn test_navigate_json_path_missing_key_errors() {
        let doc = serde_json::json!({"a": 1});
        let err = navigate_json_path(&doc, "b").unwrap_err();
        assert!(err.to_string().contains("not found"), "{}", err);
    }

    #[test]
    fn test_navigate_json_path_projection_on_non_array_errors() {
        let doc = serde_json::json!({"a": {"not": "an array"}});
        let err = navigate_json_path(&doc, "a[]").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Expected array"), "{}", msg);
        assert!(msg.contains("object"), "{}", msg);
    }

    #[test]
    fn test_csv_from_reader() {
        let csv_data = "id,name,value\n1,Alice,100\n2,Bob,200\n3,,300";
        let reader = Cursor::new(csv_data);

        let table =
            load_csv_from_reader(reader, "test", "stream", "memory").expect("Failed to load CSV");

        assert_eq!(table.name, "test");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that empty field is NULL
        let value = table.get_value(2, 1).unwrap();
        assert!(matches!(value, DataValue::Null));
    }

    #[test]
    fn test_json_from_reader() {
        let json_data = r#"[
            {"id": 1, "name": "Alice", "value": 100},
            {"id": 2, "name": "Bob", "value": 200},
            {"id": 3, "name": null, "value": 300}
        ]"#;
        let reader = Cursor::new(json_data);

        let table =
            load_json_from_reader(reader, "test", "stream", "memory").expect("Failed to load JSON");

        assert_eq!(table.name, "test");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);

        // Check that null is handled
        let value = table.get_value(2, 1).unwrap();
        assert!(matches!(value, DataValue::Null));
    }

    #[test]
    fn test_jsonl_from_reader() {
        let jsonl_data = "{\"id\":1,\"name\":\"Alice\"}\n{\"id\":2,\"name\":\"Bob\"}\n";
        let reader = Cursor::new(jsonl_data);

        let table = load_json_from_reader(reader, "test", "stream", "memory")
            .expect("Failed to load JSONL");

        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_jsonl_heterogeneous_schema_unioned() {
        // Second record adds an "extra" field; loader should pick it up via the
        // union, and row 0 should have Null for it.
        let jsonl_data = "{\"id\":1}\n{\"id\":2,\"extra\":\"hi\"}\n";
        let reader = Cursor::new(jsonl_data);
        let table = load_json_from_reader(reader, "test", "stream", "memory").expect("load");
        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_jsonl_skips_blank_lines() {
        let jsonl_data = "{\"id\":1}\n\n\n{\"id\":2}\n";
        let reader = Cursor::new(jsonl_data);
        let table = load_json_from_reader(reader, "test", "stream", "memory").expect("load");
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_parse_json_records_array_form() {
        let recs = parse_json_records(r#"[{"a":1},{"a":2}]"#).unwrap();
        assert_eq!(recs.len(), 2);
    }

    #[test]
    fn test_parse_json_records_jsonl_form() {
        let recs = parse_json_records("{\"a\":1}\n{\"a\":2}\n").unwrap();
        assert_eq!(recs.len(), 2);
    }

    #[test]
    fn test_parse_json_records_jsonl_error_cites_line() {
        let err = parse_json_records("{\"a\":1}\nnot json\n").unwrap_err();
        assert!(err.to_string().contains("line 2"));
    }

    // ---- CsvReadOptions / delimiter detection ----

    #[test]
    fn test_csv_options_default_is_comma() {
        let opts = CsvReadOptions::default();
        assert_eq!(opts.delimiter, b',');
        assert!(opts.has_headers);
    }

    #[test]
    fn test_detect_delimiter_from_path() {
        assert_eq!(detect_delimiter_from_path("data.tsv"), b'\t');
        assert_eq!(detect_delimiter_from_path("data.TSV"), b'\t');
        assert_eq!(detect_delimiter_from_path("/tmp/foo.psv"), b'|');
        assert_eq!(detect_delimiter_from_path("data.PSV"), b'|');
        assert_eq!(detect_delimiter_from_path("data.csv"), b',');
        assert_eq!(detect_delimiter_from_path("noext"), b',');
        assert_eq!(detect_delimiter_from_path("-"), b',');
    }

    #[test]
    fn test_load_csv_with_pipe_delimiter() {
        let data = "id|name|score\n1|alice|10\n2|bob|20\n";
        let reader = Cursor::new(data);
        let opts = CsvReadOptions {
            delimiter: b'|',
            has_headers: true,
        };
        let table = load_csv_from_reader_with_opts(reader, "psv", "test", "memory", &opts)
            .expect("load failed");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.get_value(0, 0).unwrap(), &DataValue::Integer(1));
        assert_eq!(
            table.get_value(1, 1).unwrap(),
            &DataValue::String("bob".to_string())
        );
    }

    #[test]
    fn test_load_csv_with_tab_delimiter() {
        let data = "id\tname\tscore\n1\talice\t10\n2\tbob\t20\n";
        let reader = Cursor::new(data);
        let opts = CsvReadOptions {
            delimiter: b'\t',
            has_headers: true,
        };
        let table = load_csv_from_reader_with_opts(reader, "tsv", "test", "memory", &opts)
            .expect("load failed");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.get_value(0, 0).unwrap(), &DataValue::Integer(1));
    }

    #[test]
    fn test_metadata_records_delimiter() {
        // Comma -> stored as ","
        let table = load_csv_from_reader(Cursor::new("a,b\n1,2\n"), "t", "test", "memory").unwrap();
        assert_eq!(
            table.metadata.get("delimiter").map(String::as_str),
            Some(",")
        );

        // Tab -> stored as "\t"
        let opts = CsvReadOptions {
            delimiter: b'\t',
            has_headers: true,
        };
        let table = load_csv_from_reader_with_opts(
            Cursor::new("a\tb\n1\t2\n"),
            "t",
            "test",
            "memory",
            &opts,
        )
        .unwrap();
        assert_eq!(
            table.metadata.get("delimiter").map(String::as_str),
            Some("\\t")
        );
    }

    #[test]
    fn test_parse_delimiter_arg_accepts_single_char() {
        assert_eq!(parse_delimiter_arg(",").unwrap(), b',');
        assert_eq!(parse_delimiter_arg("|").unwrap(), b'|');
        assert_eq!(parse_delimiter_arg(";").unwrap(), b';');
    }

    #[test]
    fn test_parse_delimiter_arg_accepts_backslash_escapes() {
        assert_eq!(parse_delimiter_arg("\\t").unwrap(), b'\t');
        assert_eq!(parse_delimiter_arg("\t").unwrap(), b'\t');
        assert_eq!(parse_delimiter_arg("\\n").unwrap(), b'\n');
        assert_eq!(parse_delimiter_arg("\\r").unwrap(), b'\r');
    }

    #[test]
    fn test_parse_delimiter_arg_rejects_multi_char() {
        let err = parse_delimiter_arg("||").unwrap_err();
        assert!(err.to_string().contains("single ASCII character"));
    }

    #[test]
    fn test_parse_delimiter_arg_rejects_non_ascii() {
        let err = parse_delimiter_arg("ö").unwrap_err();
        assert!(err.to_string().contains("single ASCII character"));
    }

    #[test]
    fn test_resolve_delimiter_explicit_wins() {
        assert_eq!(resolve_delimiter("data.psv", Some(b',')), b',');
        assert_eq!(resolve_delimiter("data.tsv", Some(b';')), b';');
        assert_eq!(resolve_delimiter("data.csv", Some(b'|')), b'|');
    }

    #[test]
    fn test_resolve_delimiter_falls_back_to_extension() {
        assert_eq!(resolve_delimiter("data.psv", None), b'|');
        assert_eq!(resolve_delimiter("data.tsv", None), b'\t');
        assert_eq!(resolve_delimiter("data.csv", None), b',');
        assert_eq!(resolve_delimiter("data.dat", None), b',');
    }

    #[test]
    fn test_null_detection_works_with_pipe_delimiter() {
        // Middle column is unquoted-empty -> NULL, not empty string.
        let data = "id|name|score\n1||10\n";
        let opts = CsvReadOptions {
            delimiter: b'|',
            has_headers: true,
        };
        let table =
            load_csv_from_reader_with_opts(Cursor::new(data), "psv", "test", "memory", &opts)
                .expect("load failed");
        assert!(matches!(table.get_value(0, 1).unwrap(), DataValue::Null));
    }
}
