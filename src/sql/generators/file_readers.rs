use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use crate::sql::generators::TableGenerator;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

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

        let lines = read_filtered_lines(&path, match_regex.as_ref())?;

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
        "Read a text file line-by-line. Optional second arg is a regex that filters lines at read time."
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

        // When not inverted we can push the filter down into the file reader for
        // the fast path. When inverted we still iterate every line.
        let lines = if invert {
            let all = read_filtered_lines(&path, None)?;
            all.into_iter()
                .filter(|(_, line)| !pattern.is_match(line))
                .collect::<Vec<_>>()
        } else {
            read_filtered_lines(&path, Some(&pattern))?
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
        "Read only lines matching a regex (third arg inverts the match, like grep -v)"
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

        let lines = read_filtered_lines(&path, None)?;

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
}
