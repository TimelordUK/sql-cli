use crate::data::advanced_csv_loader::AdvancedCsvLoader;
use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::data::stream_loader::{
    collect_column_names, detect_delimiter_from_path, parse_delimiter_arg as parse_delim_byte,
    CsvReadOptions,
};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value as JsonValue;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, IsTerminal};
use std::sync::{Arc, OnceLock};

/// Hard cap on rows any file reader will return. Users who need more can raise
/// it via a session setting (future work); for now this protects against
/// accidentally pulling a multi-GB log into memory.
const MAX_LINES_PER_FILE: usize = 1_000_000;

/// Extract a string argument, erroring if the arg is missing/NULL/non-string.
fn require_string(args: &[DataValue], idx: usize, name: &str) -> Result<String> {
    match args.get(idx) {
        Some(DataValue::String(s)) => Ok(s.clone()),
        Some(DataValue::InternedString(s)) => Ok(s.as_str().to_string()),
        Some(DataValue::Null) | None => Err(anyhow!("{} requires argument {}", name, idx + 1)),
        Some(v) => Err(anyhow!(
            "{} argument {} must be a string, got {:?}",
            name,
            idx + 1,
            v
        )),
    }
}

/// Extract an optional string argument. Returns None for missing or NULL.
fn optional_string(args: &[DataValue], idx: usize) -> Option<String> {
    match args.get(idx) {
        Some(DataValue::String(s)) => Some(s.clone()),
        Some(DataValue::InternedString(s)) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// Parse a delimiter arg for READ_CSV, wrapping the shared parser's error
/// with the function name for clearer messages.
fn parse_delimiter_arg(s: &str, fn_name: &str) -> Result<u8> {
    parse_delim_byte(s).map_err(|e| anyhow!("{}: {}", fn_name, e))
}

/// Open a file and stream its lines, applying an optional include-regex filter
/// and the global truncation cap. Emits a stderr warning when truncation kicks in.
///
/// Returns (line_num, line) pairs where `line_num` is the original 1-based line
/// number in the source file — so numbers are preserved through filtering.
fn read_filtered_lines(path: &str, match_regex: Option<&Regex>) -> Result<Vec<(i64, String)>> {
    let file = File::open(path).map_err(|e| anyhow!("Failed to open '{}': {}", path, e))?;
    let reader = BufReader::new(file);

    let mut out = Vec::new();
    let mut truncated = false;

    for (idx, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| anyhow!("Error reading '{}': {}", path, e))?;
        let line_num = (idx + 1) as i64;

        if let Some(re) = match_regex {
            if !re.is_match(&line) {
                continue;
            }
        }

        if out.len() >= MAX_LINES_PER_FILE {
            truncated = true;
            break;
        }
        out.push((line_num, line));
    }

    if truncated {
        eprintln!(
            "WARNING: truncated to {} rows (max_lines_per_file cap) when reading '{}'",
            MAX_LINES_PER_FILE, path
        );
    }

    Ok(out)
}

/// Read lines from a file path, or from stdin if `path == "-"` (Unix convention).
///
/// Stdin reads are cached for the process — same buffer is reused across multiple
/// reader invocations in the same query. The optional regex filter is applied to
/// both sources uniformly.
fn read_lines_from_path_or_stdin(
    path: &str,
    match_regex: Option<&Regex>,
) -> Result<Vec<(i64, String)>> {
    if path == "-" {
        let cached = cached_stdin_lines()?;
        return Ok(cached
            .iter()
            .filter(|(_, line)| match_regex.map_or(true, |re| re.is_match(line)))
            .cloned()
            .collect());
    }
    read_filtered_lines(path, match_regex)
}

/// READ_TEXT(path [, match_regex]) - Read a text file line by line.
///
/// Emits `(line_num, line)` rows. Optional `match_regex` filters source lines
/// *before* materializing them, which is the primary fast path for large logs.
pub struct ReadText;

impl TableGenerator for ReadText {
    fn name(&self) -> &str {
        "READ_TEXT"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("line_num"), DataColumn::new("line")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!(
                "READ_TEXT expects 1 or 2 arguments: (path [, match_regex])"
            ));
        }

        let path = require_string(&args, 0, "READ_TEXT")?;
        let match_regex = optional_string(&args, 1)
            .map(|s| Regex::new(&s).map_err(|e| anyhow!("Invalid match_regex: {}", e)))
            .transpose()?;

        let lines = read_lines_from_path_or_stdin(&path, match_regex.as_ref())?;

        let mut table = DataTable::new("read_text");
        table.add_column(DataColumn::new("line_num"));
        table.add_column(DataColumn::new("line"));

        for (line_num, line) in lines {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(line_num),
                    DataValue::String(line),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read a text file line-by-line. Pass '-' as path to read from stdin. Optional second arg is a regex that filters lines at read time."
    }

    fn arg_count(&self) -> usize {
        2
    }
}

/// GREP(path, pattern [, invert]) - Read only lines matching a regex.
///
/// Thin composable wrapper around READ_TEXT's filter path. Third argument
/// (boolean or integer truthy value) inverts the match, matching `grep -v`.
pub struct Grep;

impl TableGenerator for Grep {
    fn name(&self) -> &str {
        "GREP"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("line_num"), DataColumn::new("line")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() < 2 || args.len() > 3 {
            return Err(anyhow!(
                "GREP expects 2 or 3 arguments: (path, pattern [, invert])"
            ));
        }

        let path = require_string(&args, 0, "GREP")?;
        let pattern_str = require_string(&args, 1, "GREP")?;
        let pattern =
            Regex::new(&pattern_str).map_err(|e| anyhow!("Invalid GREP pattern: {}", e))?;

        let invert = match args.get(2) {
            Some(DataValue::Boolean(b)) => *b,
            Some(DataValue::Integer(n)) => *n != 0,
            Some(DataValue::Null) | None => false,
            Some(v) => return Err(anyhow!("GREP invert flag must be boolean, got {:?}", v)),
        };

        // When not inverted we can push the filter down into the line reader for
        // the fast path. When inverted we still iterate every line.
        let lines = if invert {
            let all = read_lines_from_path_or_stdin(&path, None)?;
            all.into_iter()
                .filter(|(_, line)| !pattern.is_match(line))
                .collect::<Vec<_>>()
        } else {
            read_lines_from_path_or_stdin(&path, Some(&pattern))?
        };

        let mut table = DataTable::new("grep");
        table.add_column(DataColumn::new("line_num"));
        table.add_column(DataColumn::new("line"));

        for (line_num, line) in lines {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(line_num),
                    DataValue::String(line),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read only lines matching a regex (third arg inverts the match, like grep -v). Pass '-' as path to read from stdin."
    }

    fn arg_count(&self) -> usize {
        3
    }
}

/// READ_WORDS(path [, min_length [, case]]) - Read a text file and emit one row per word.
///
/// Emits `(word_num, word, line_num, word_pos)` rows where:
///   - `word_num` is a global 1-based word counter across the whole file
///   - `word` is the extracted token (punctuation stripped)
///   - `line_num` is the original 1-based line number the word came from
///   - `word_pos` is the 1-based position of the word within its line
///
/// Optional `min_length` (default 1) filters out short words.
/// Optional `case` ('lower' or 'upper') normalises word casing.
pub struct ReadWords;

impl TableGenerator for ReadWords {
    fn name(&self) -> &str {
        "READ_WORDS"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![
            DataColumn::new("word_num"),
            DataColumn::new("word"),
            DataColumn::new("line_num"),
            DataColumn::new("word_pos"),
        ]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() || args.len() > 3 {
            return Err(anyhow!(
                "READ_WORDS expects 1 to 3 arguments: (path [, min_length [, case]])"
            ));
        }

        let path = require_string(&args, 0, "READ_WORDS")?;

        let min_length: usize = match args.get(1) {
            Some(DataValue::Integer(n)) => {
                if *n < 1 {
                    return Err(anyhow!("READ_WORDS min_length must be >= 1"));
                }
                *n as usize
            }
            Some(DataValue::Float(f)) => *f as usize,
            Some(DataValue::Null) | None => 1,
            Some(v) => {
                return Err(anyhow!(
                    "READ_WORDS min_length must be an integer, got {:?}",
                    v
                ))
            }
        };

        let case_option = optional_string(&args, 2);

        let lines = read_lines_from_path_or_stdin(&path, None)?;

        let mut table = DataTable::new("read_words");
        table.add_column(DataColumn::new("word_num"));
        table.add_column(DataColumn::new("word"));
        table.add_column(DataColumn::new("line_num"));
        table.add_column(DataColumn::new("word_pos"));

        let mut word_num: i64 = 0;

        for (line_num, line) in &lines {
            let mut word_pos: i64 = 0;

            for token in line.split(|c: char| !c.is_alphanumeric()) {
                if token.is_empty() || token.len() < min_length {
                    continue;
                }

                word_pos += 1;
                word_num += 1;

                let word = match case_option.as_deref() {
                    Some("lower") | Some("lowercase") => token.to_lowercase(),
                    Some("upper") | Some("uppercase") => token.to_uppercase(),
                    _ => token.to_string(),
                };

                table
                    .add_row(DataRow::new(vec![
                        DataValue::Integer(word_num),
                        DataValue::String(word),
                        DataValue::Integer(*line_num),
                        DataValue::Integer(word_pos),
                    ]))
                    .map_err(|e| anyhow!(e))?;
            }
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read a text file and emit one row per word, with optional min length and case normalisation"
    }

    fn arg_count(&self) -> usize {
        3
    }
}

/// READ_JSONL(path [, match_regex]) - Read newline-delimited JSON.
///
/// Each non-blank line is parsed as a self-contained JSON object. Schema is
/// the union of object keys across the first 100 records, so heterogeneous
/// log streams (later events introducing new fields) don't drop columns.
/// Optional `match_regex` filters source lines *before* JSON parsing — the
/// fast path for grepping large log files.
pub struct ReadJsonl;

impl TableGenerator for ReadJsonl {
    fn name(&self) -> &str {
        "READ_JSONL"
    }

    fn columns(&self) -> Vec<DataColumn> {
        // Schema is dynamic; the actual columns are inferred at generate() time
        // from the JSON keys in the file.
        vec![DataColumn::new("(inferred from JSON keys)")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!(
                "READ_JSONL expects 1 or 2 arguments: (path [, match_regex])"
            ));
        }

        let path = require_string(&args, 0, "READ_JSONL")?;
        let match_regex = optional_string(&args, 1)
            .map(|s| Regex::new(&s).map_err(|e| anyhow!("Invalid match_regex: {}", e)))
            .transpose()?;

        let lines = read_lines_from_path_or_stdin(&path, match_regex.as_ref())?;

        let mut records: Vec<JsonValue> = Vec::with_capacity(lines.len());
        for (line_num, line) in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let value: JsonValue = serde_json::from_str(trimmed)
                .map_err(|e| anyhow!("READ_JSONL parse error at line {}: {}", line_num, e))?;
            records.push(value);
        }

        if records.is_empty() {
            return Ok(Arc::new(DataTable::new("read_jsonl")));
        }

        let column_names = collect_column_names(&records, 100);
        if column_names.is_empty() {
            return Err(anyhow!(
                "READ_JSONL: no JSON objects found (records must be objects, not arrays/scalars)"
            ));
        }

        // Stringify, infer types from the first 100 rows, then convert to typed
        // DataValues — same pipeline the file-level JSON loader uses.
        let mut string_rows: Vec<Vec<String>> = Vec::with_capacity(records.len());
        for record in &records {
            let obj = match record.as_object() {
                Some(o) => o,
                None => continue,
            };
            let mut row = Vec::with_capacity(column_names.len());
            for col_name in &column_names {
                let value = obj
                    .get(col_name)
                    .map(json_value_to_string)
                    .unwrap_or_default();
                row.push(value);
            }
            string_rows.push(row);
        }

        let mut column_types: Vec<DataType> = vec![DataType::Null; column_names.len()];
        let sample_size = string_rows.len().min(100);
        for row in string_rows.iter().take(sample_size) {
            for (col_idx, value) in row.iter().enumerate() {
                if !value.is_empty() && value != "null" {
                    let inferred = DataType::infer_from_string(value);
                    column_types[col_idx] = column_types[col_idx].merge(&inferred);
                }
            }
        }

        let mut table = DataTable::new("read_jsonl");
        for (name, dtype) in column_names.iter().zip(column_types.iter()) {
            let mut col = DataColumn::new(name);
            col.data_type = dtype.clone();
            table.add_column(col);
        }

        for string_row in &string_rows {
            let mut values = Vec::with_capacity(string_row.len());
            for (col_idx, value) in string_row.iter().enumerate() {
                let dv = if value.is_empty() || value == "null" {
                    DataValue::Null
                } else {
                    DataValue::from_string(value, &column_types[col_idx])
                };
                values.push(dv);
            }
            table
                .add_row(DataRow::new(values))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read newline-delimited JSON (one object per line). Pass '-' as path to read JSONL from stdin. Optional second arg is a regex that filters lines at read time."
    }

    fn arg_count(&self) -> usize {
        2
    }
}

/// READ_CSV(path [, delimiter]) - Read a delimited text file and emit one row per record.
///
/// Columns are inferred from the header row. Pass `-` as the path to read CSV
/// from stdin (shares the same cached-once buffer with READ_STDIN / READ_JSONL).
///
/// Delimiter resolution:
///   1. Explicit 2nd arg wins (single ASCII char or `\t` escape).
///   2. Otherwise, `.tsv` → tab, `.psv` → pipe, everything else → comma.
///   3. Stdin (`-`) with no explicit delimiter defaults to comma.
///
/// Type inference, string interning, and other optimisations are inherited
/// from the main CSV loader so behaviour matches `sql-cli file.csv -q ...`.
pub struct ReadCsv;

impl TableGenerator for ReadCsv {
    fn name(&self) -> &str {
        "READ_CSV"
    }

    fn columns(&self) -> Vec<DataColumn> {
        // Schema is inferred from the CSV header at generate() time.
        vec![DataColumn::new("(inferred from CSV header)")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!(
                "READ_CSV expects 1 or 2 arguments: (path [, delimiter])"
            ));
        }

        let path = require_string(&args, 0, "READ_CSV")?;

        // Resolve delimiter: explicit 2nd arg > extension auto-detect > comma.
        // Stdin (`-`) skips the extension layer since there's no extension to read.
        let delimiter = if let Some(s) = optional_string(&args, 1) {
            parse_delimiter_arg(&s, "READ_CSV")?
        } else if path == "-" {
            b','
        } else {
            detect_delimiter_from_path(&path)
        };

        let opts = CsvReadOptions {
            delimiter,
            has_headers: true,
        };

        let mut loader = AdvancedCsvLoader::new();

        let table = if path == "-" {
            // Reconstruct a CSV byte stream from the cached stdin lines so other
            // stdin readers in the same query keep seeing the same buffer.
            let lines = cached_stdin_lines()?;
            let mut buffer = String::with_capacity(lines.iter().map(|(_, l)| l.len() + 1).sum());
            for (i, (_, line)) in lines.iter().enumerate() {
                if i > 0 {
                    buffer.push('\n');
                }
                buffer.push_str(line);
            }
            let cursor = Cursor::new(buffer.into_bytes());
            loader
                .load_csv_from_reader_with_opts(cursor, "read_csv", "<stdin>", &opts)
                .map_err(|e| anyhow!("READ_CSV parse error reading stdin: {}", e))?
        } else {
            let file = File::open(&path)
                .map_err(|e| anyhow!("READ_CSV failed to open '{}': {}", path, e))?;
            loader
                .load_csv_from_reader_with_opts(file, "read_csv", &path, &opts)
                .map_err(|e| anyhow!("READ_CSV parse error reading '{}': {}", path, e))?
        };

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read a delimited text file (header row required). Pass '-' as path to read from stdin. \
         Second arg overrides the delimiter; otherwise '.tsv' → tab, '.psv' → pipe, else comma."
    }

    fn arg_count(&self) -> usize {
        2
    }
}

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

/// READ_STDIN([match_regex]) - Read lines piped on standard input.
///
/// Emits `(line_num, line)` rows. Optional regex pre-filters lines before
/// materialisation. Erroring on TTY input prevents the engine from blocking
/// forever waiting on a keyboard when the user forgets the pipe.
///
/// Stdin is consumable, so it is read **once per process** and cached. This
/// keeps the function composable: multiple `READ_STDIN()` references in the
/// same query (CTEs, self-joins) see the same rows. The regex filter is
/// applied per call against the cached buffer.
pub struct ReadStdin;

static STDIN_CACHE: OnceLock<Result<Vec<(i64, String)>, String>> = OnceLock::new();

fn cached_stdin_lines() -> Result<&'static Vec<(i64, String)>> {
    let cached = STDIN_CACHE.get_or_init(|| {
        let stdin = std::io::stdin();
        if stdin.is_terminal() {
            return Err("READ_STDIN() requires data piped on stdin; got an interactive terminal. Try: cat file | sql-cli -q '...'".to_string());
        }
        let handle = stdin.lock();
        let reader = BufReader::new(handle);
        let mut out = Vec::new();
        let mut truncated = false;
        for (idx, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => return Err(format!("Error reading stdin: {}", e)),
            };
            if out.len() >= MAX_LINES_PER_FILE {
                truncated = true;
                break;
            }
            out.push(((idx + 1) as i64, line));
        }
        if truncated {
            eprintln!(
                "WARNING: truncated to {} rows (max_lines_per_file cap) when reading stdin",
                MAX_LINES_PER_FILE
            );
        }
        Ok(out)
    });
    cached.as_ref().map_err(|e| anyhow!(e.clone()))
}

impl TableGenerator for ReadStdin {
    fn name(&self) -> &str {
        "READ_STDIN"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("line_num"), DataColumn::new("line")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() > 1 {
            return Err(anyhow!(
                "READ_STDIN expects 0 or 1 arguments: ([match_regex])"
            ));
        }

        let match_regex = optional_string(&args, 0)
            .map(|s| Regex::new(&s).map_err(|e| anyhow!("Invalid match_regex: {}", e)))
            .transpose()?;

        let lines = cached_stdin_lines()?;

        let mut table = DataTable::new("read_stdin");
        table.add_column(DataColumn::new("line_num"));
        table.add_column(DataColumn::new("line"));

        for (line_num, line) in lines {
            if let Some(ref re) = match_regex {
                if !re.is_match(line) {
                    continue;
                }
            }
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(*line_num),
                    DataValue::String(line.clone()),
                ]))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(Arc::new(table))
    }

    fn description(&self) -> &str {
        "Read lines piped on stdin (cached once per process). Optional regex filters lines at read time. Yields (line_num, line)."
    }

    fn arg_count(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_tmp(contents: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        f
    }

    #[test]
    fn test_read_text_returns_all_lines() {
        let f = write_tmp("one\ntwo\nthree\n");
        let table = ReadText
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 3);
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("one".to_string())
        );
        assert_eq!(table.get_value(2, 0).unwrap(), &DataValue::Integer(3));
    }

    #[test]
    fn test_read_text_with_match_regex_filters_lines() {
        let f = write_tmp("INFO boot\nERROR disk full\nINFO shutdown\nERROR oom\n");
        let table = ReadText
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("ERROR".to_string()),
            ])
            .unwrap();
        assert_eq!(table.row_count(), 2);
        // Line numbers preserve original file positions (2 and 4), not 1 and 2.
        assert_eq!(table.get_value(0, 0).unwrap(), &DataValue::Integer(2));
        assert_eq!(table.get_value(1, 0).unwrap(), &DataValue::Integer(4));
    }

    #[test]
    fn test_read_text_requires_path() {
        assert!(ReadText.generate(vec![]).is_err());
    }

    #[test]
    fn test_read_text_invalid_regex_errors_early() {
        let f = write_tmp("hello\n");
        let err = ReadText
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("(unclosed".to_string()),
            ])
            .unwrap_err();
        assert!(err.to_string().contains("match_regex"));
    }

    #[test]
    fn test_grep_matches_like_grep() {
        let f = write_tmp("apple\nbanana\ncherry\napricot\n");
        let table = Grep
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("^ap".to_string()),
            ])
            .unwrap();
        assert_eq!(table.row_count(), 2);
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("apple".to_string())
        );
        assert_eq!(
            table.get_value(1, 1).unwrap(),
            &DataValue::String("apricot".to_string())
        );
    }

    #[test]
    fn test_grep_invert_like_grep_v() {
        let f = write_tmp("apple\nbanana\ncherry\napricot\n");
        let table = Grep
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("^ap".to_string()),
                DataValue::Boolean(true),
            ])
            .unwrap();
        assert_eq!(table.row_count(), 2);
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("banana".to_string())
        );
    }

    // ---- ReadWords tests ----

    #[test]
    fn test_read_words_basic() {
        let f = write_tmp("hello world\ngoodbye moon\n");
        let table = ReadWords
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        // 4 words total
        assert_eq!(table.row_count(), 4);
        // Columns: word_num, word, line_num, word_pos
        // First word
        assert_eq!(table.get_value(0, 0).unwrap(), &DataValue::Integer(1)); // word_num
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("hello".to_string())
        );
        assert_eq!(table.get_value(0, 2).unwrap(), &DataValue::Integer(1)); // line_num
        assert_eq!(table.get_value(0, 3).unwrap(), &DataValue::Integer(1)); // word_pos
                                                                            // Third word (first on line 2)
        assert_eq!(table.get_value(2, 0).unwrap(), &DataValue::Integer(3));
        assert_eq!(
            table.get_value(2, 1).unwrap(),
            &DataValue::String("goodbye".to_string())
        );
        assert_eq!(table.get_value(2, 2).unwrap(), &DataValue::Integer(2));
        assert_eq!(table.get_value(2, 3).unwrap(), &DataValue::Integer(1));
    }

    #[test]
    fn test_read_words_min_length() {
        let f = write_tmp("I am a big dog\n");
        let table = ReadWords
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::Integer(3),
            ])
            .unwrap();
        // Only "big" and "dog" have length >= 3
        assert_eq!(table.row_count(), 2);
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("big".to_string())
        );
        assert_eq!(
            table.get_value(1, 1).unwrap(),
            &DataValue::String("dog".to_string())
        );
    }

    #[test]
    fn test_read_words_case_lower() {
        let f = write_tmp("Hello World\n");
        let table = ReadWords
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::Integer(1),
                DataValue::String("lower".to_string()),
            ])
            .unwrap();
        assert_eq!(
            table.get_value(0, 1).unwrap(),
            &DataValue::String("hello".to_string())
        );
        assert_eq!(
            table.get_value(1, 1).unwrap(),
            &DataValue::String("world".to_string())
        );
    }

    #[test]
    fn test_read_words_strips_punctuation() {
        let f = write_tmp("hello, world! foo-bar.\n");
        let table = ReadWords
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        let words: Vec<String> = (0..table.row_count())
            .map(|i| match table.get_value(i, 1).unwrap() {
                DataValue::String(s) => s.clone(),
                _ => panic!("expected string"),
            })
            .collect();
        assert_eq!(words, vec!["hello", "world", "foo", "bar"]);
    }

    #[test]
    fn test_read_words_requires_path() {
        assert!(ReadWords.generate(vec![]).is_err());
    }

    #[test]
    fn test_read_words_empty_lines_skipped() {
        let f = write_tmp("hello\n\n\nworld\n");
        let table = ReadWords
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 2);
        // word_num is contiguous
        assert_eq!(table.get_value(0, 0).unwrap(), &DataValue::Integer(1));
        assert_eq!(table.get_value(1, 0).unwrap(), &DataValue::Integer(2));
        // line_num preserves original positions
        assert_eq!(table.get_value(0, 2).unwrap(), &DataValue::Integer(1));
        assert_eq!(table.get_value(1, 2).unwrap(), &DataValue::Integer(4));
    }

    // ---- ReadJsonl tests ----

    fn col_index(table: &DataTable, name: &str) -> usize {
        table
            .columns
            .iter()
            .position(|c| c.name == name)
            .unwrap_or_else(|| panic!("column '{}' not found", name))
    }

    #[test]
    fn test_read_jsonl_basic() {
        let f = write_tmp(
            r#"{"id":1,"name":"alice","score":10}
{"id":2,"name":"bob","score":20}
{"id":3,"name":"carol","score":30}
"#,
        );
        let table = ReadJsonl
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.column_count(), 3);

        let id_col = col_index(&table, "id");
        let name_col = col_index(&table, "name");
        assert_eq!(table.get_value(0, id_col).unwrap(), &DataValue::Integer(1));
        assert_eq!(
            table.get_value(2, name_col).unwrap(),
            &DataValue::String("carol".to_string())
        );
    }

    #[test]
    fn test_read_jsonl_heterogeneous_schema_unioned() {
        // Later records introduce fields the first one didn't have. Schema
        // should union them; missing values become Null.
        let f = write_tmp(
            r#"{"id":1,"name":"alice"}
{"id":2,"name":"bob","extra":"hello"}
{"id":3,"name":"carol","other":42}
"#,
        );
        let table = ReadJsonl
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.column_count(), 4);
        let extra = col_index(&table, "extra");
        let other = col_index(&table, "other");
        // Row 0 has neither extra nor other -> Null
        assert_eq!(table.get_value(0, extra).unwrap(), &DataValue::Null);
        assert_eq!(table.get_value(0, other).unwrap(), &DataValue::Null);
        // Row 1 has extra="hello"
        assert_eq!(
            table.get_value(1, extra).unwrap(),
            &DataValue::String("hello".to_string())
        );
        // Row 2 has other=42
        assert_eq!(table.get_value(2, other).unwrap(), &DataValue::Integer(42));
    }

    #[test]
    fn test_read_jsonl_blank_lines_skipped() {
        let f = write_tmp(
            r#"{"id":1}

{"id":2}

"#,
        );
        let table = ReadJsonl
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_read_jsonl_match_regex_pre_filters() {
        let f = write_tmp(
            r#"{"level":"INFO","msg":"boot"}
{"level":"ERROR","msg":"disk"}
{"level":"INFO","msg":"shutdown"}
{"level":"ERROR","msg":"oom"}
"#,
        );
        let table = ReadJsonl
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("ERROR".to_string()),
            ])
            .unwrap();
        assert_eq!(table.row_count(), 2);
        let msg = col_index(&table, "msg");
        assert_eq!(
            table.get_value(0, msg).unwrap(),
            &DataValue::String("disk".to_string())
        );
    }

    #[test]
    fn test_read_jsonl_invalid_line_errors_with_line_number() {
        let f = write_tmp(
            r#"{"id":1}
not json at all
{"id":3}
"#,
        );
        let err = ReadJsonl
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("line 2"),
            "error should cite line number: {}",
            msg
        );
    }

    #[test]
    fn test_read_jsonl_requires_path() {
        assert!(ReadJsonl.generate(vec![]).is_err());
    }

    #[test]
    fn test_read_jsonl_empty_file_returns_empty_table() {
        let f = write_tmp("");
        let table = ReadJsonl
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.row_count(), 0);
    }

    // ReadStdin: argument-validation only (stdin is process-global and we cannot
    // safely inject test data without refactoring to inject a Reader). The
    // happy-path is covered by manual smoke tests in commit messages and the
    // examples/ folder.

    #[test]
    fn test_read_stdin_rejects_too_many_args() {
        let err = ReadStdin
            .generate(vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
            ])
            .unwrap_err();
        assert!(
            err.to_string().contains("0 or 1 arguments"),
            "should mention arg count: {}",
            err
        );
    }

    #[test]
    fn test_read_stdin_rejects_invalid_regex() {
        let err = ReadStdin
            .generate(vec![DataValue::String("[invalid(regex".to_string())])
            .unwrap_err();
        assert!(
            err.to_string().contains("Invalid match_regex"),
            "should mention regex: {}",
            err
        );
    }

    // ---- ReadCsv tests ----

    /// Helper to write a temp CSV-like file with a given extension and content.
    fn write_tmp_with_ext(ext: &str, contents: &str) -> tempfile::NamedTempFile {
        let f = tempfile::Builder::new()
            .suffix(&format!(".{}", ext))
            .tempfile()
            .unwrap();
        std::fs::write(f.path(), contents).unwrap();
        f
    }

    #[test]
    fn test_read_csv_default_comma() {
        let f = write_tmp_with_ext("csv", "id,name\n1,alice\n2,bob\n");
        let table = ReadCsv
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[1].name, "name");
    }

    #[test]
    fn test_read_csv_psv_extension_auto_detects_pipe() {
        let f = write_tmp_with_ext("psv", "id|name|score\n1|alice|10\n2|bob|20\n");
        let table = ReadCsv
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[2].name, "score");
        assert_eq!(
            table.metadata.get("delimiter").map(String::as_str),
            Some("|")
        );
    }

    #[test]
    fn test_read_csv_tsv_extension_auto_detects_tab() {
        let f = write_tmp_with_ext("tsv", "id\tname\n1\talice\n2\tbob\n");
        let table = ReadCsv
            .generate(vec![DataValue::String(
                f.path().to_string_lossy().to_string(),
            )])
            .unwrap();
        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 2);
        assert_eq!(
            table.metadata.get("delimiter").map(String::as_str),
            Some("\\t")
        );
    }

    #[test]
    fn test_read_csv_explicit_delimiter_overrides_extension() {
        // .psv extension would auto-detect pipe, but explicit comma wins.
        // Content is comma-delimited, so if extension auto-detect ran, it
        // would parse as a single column.
        let f = write_tmp_with_ext("psv", "id,name\n1,alice\n");
        let table = ReadCsv
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String(",".to_string()),
            ])
            .unwrap();
        assert_eq!(table.column_count(), 2);
    }

    #[test]
    fn test_read_csv_explicit_pipe_on_unrecognised_extension() {
        let f = write_tmp_with_ext("dat", "a|b\n1|2\n");
        let table = ReadCsv
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("|".to_string()),
            ])
            .unwrap();
        assert_eq!(table.column_count(), 2);
    }

    #[test]
    fn test_read_csv_backslash_t_parses_as_tab() {
        let f = write_tmp_with_ext("dat", "a\tb\n1\t2\n");
        let table = ReadCsv
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("\\t".to_string()),
            ])
            .unwrap();
        assert_eq!(table.column_count(), 2);
        assert_eq!(
            table.metadata.get("delimiter").map(String::as_str),
            Some("\\t")
        );
    }

    #[test]
    fn test_read_csv_rejects_multi_char_delimiter() {
        let f = write_tmp_with_ext("dat", "a,b\n1,2\n");
        let err = ReadCsv
            .generate(vec![
                DataValue::String(f.path().to_string_lossy().to_string()),
                DataValue::String("||".to_string()),
            ])
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("single ASCII character"),
            "should reject multi-char delimiter: {}",
            msg
        );
    }

    #[test]
    fn test_read_csv_rejects_too_many_args() {
        let err = ReadCsv
            .generate(vec![
                DataValue::String("a".to_string()),
                DataValue::String("b".to_string()),
                DataValue::String("c".to_string()),
            ])
            .unwrap_err();
        assert!(err.to_string().contains("1 or 2 arguments"));
    }

    #[test]
    fn test_read_csv_requires_path() {
        assert!(ReadCsv.generate(vec![]).is_err());
    }
}
