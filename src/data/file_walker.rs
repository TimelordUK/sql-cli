use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use globset::{Glob, GlobMatcher};
use walkdir::WalkDir;

use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use crate::sql::parser::ast::FileCTESpec;

const DEFAULT_MAX_FILES: usize = 500_000;

/// Walk the filesystem according to a FileCTESpec and return a DataTable.
pub fn walk_filesystem(spec: &FileCTESpec, cte_name: &str) -> Result<DataTable> {
    // Resolve path
    let root = std::fs::canonicalize(&spec.path)
        .map_err(|e| anyhow!("FILE CTE: cannot resolve path '{}': {}", spec.path, e))?;

    if !root.is_dir() {
        return Err(anyhow!("FILE CTE: path '{}' is not a directory", spec.path));
    }

    // Compile glob if present
    let glob_matcher: Option<GlobMatcher> = match &spec.glob {
        Some(pattern) => {
            let g = Glob::new(pattern)
                .map_err(|e| anyhow!("FILE CTE: invalid GLOB pattern '{}': {}", pattern, e))?;
            Some(g.compile_matcher())
        }
        None => None,
    };

    // Configure walker
    let max_depth = if spec.recursive {
        spec.max_depth // None means unlimited
    } else {
        Some(1) // Non-recursive: root + immediate children only
    };

    let mut walker = WalkDir::new(&root).follow_links(spec.follow_links);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    // Build table schema
    let mut table = build_schema(cte_name);

    // Walk and collect rows
    let max_files = spec.max_files.unwrap_or(DEFAULT_MAX_FILES);
    let mut file_count: usize = 0;
    let mut permission_errors: usize = 0;

    for entry_result in walker {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                permission_errors += 1;
                tracing::debug!("FILE CTE: skipping entry: {}", e);
                continue;
            }
        };

        // Hidden file filtering
        if !spec.include_hidden && is_hidden(&entry) {
            continue;
        }

        // Glob filtering — apply to file name, let directories through
        if let Some(ref matcher) = glob_matcher {
            if !entry.file_type().is_dir()
                && !matcher.is_match(entry.file_name().to_string_lossy().as_ref())
            {
                continue;
            }
        }

        // MAX_FILES check
        file_count += 1;
        if file_count > max_files {
            return Err(anyhow!(
                "FILE CTE: exceeded MAX_FILES limit of {}. \
                 Use MAX_FILES <n> or GLOB to constrain the walk.",
                max_files
            ));
        }

        let row = build_row(&entry);
        table
            .add_row(row)
            .map_err(|e| anyhow!("FILE CTE: failed to add row: {}", e))?;
    }

    if permission_errors > 0 {
        tracing::warn!(
            "FILE CTE: {} entries skipped due to permission errors",
            permission_errors
        );
    }

    Ok(table)
}

fn build_schema(cte_name: &str) -> DataTable {
    let mut table = DataTable::new(cte_name);
    table.add_column(DataColumn::new("path").with_type(DataType::String));
    table.add_column(DataColumn::new("parent").with_type(DataType::String));
    table.add_column(DataColumn::new("name").with_type(DataType::String));
    table.add_column(DataColumn::new("stem").with_type(DataType::String));
    table.add_column(
        DataColumn::new("ext")
            .with_type(DataType::String)
            .with_nullable(true),
    );
    table.add_column(DataColumn::new("size").with_type(DataType::Integer));
    table.add_column(
        DataColumn::new("modified")
            .with_type(DataType::DateTime)
            .with_nullable(true),
    );
    table.add_column(
        DataColumn::new("created")
            .with_type(DataType::DateTime)
            .with_nullable(true),
    );
    table.add_column(
        DataColumn::new("accessed")
            .with_type(DataType::DateTime)
            .with_nullable(true),
    );
    table.add_column(DataColumn::new("is_dir").with_type(DataType::Boolean));
    table.add_column(DataColumn::new("is_symlink").with_type(DataType::Boolean));
    table.add_column(DataColumn::new("depth").with_type(DataType::Integer));
    table
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    // Never treat root (depth 0) as hidden
    if entry.depth() == 0 {
        return false;
    }
    entry.file_name().to_string_lossy().starts_with('.')
}

fn build_row(entry: &walkdir::DirEntry) -> DataRow {
    let path = entry.path();

    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let path_str = canonical.to_string_lossy().to_string();
    let parent_str = canonical
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let name_str = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem_str = canonical
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext_val = canonical
        .extension()
        .map(|e| DataValue::String(e.to_string_lossy().to_lowercase()))
        .unwrap_or(DataValue::Null);

    let metadata = entry.metadata().ok();

    let size = metadata
        .as_ref()
        .map(|m| DataValue::Integer(m.len() as i64))
        .unwrap_or(DataValue::Integer(0));

    let modified = system_time_to_datavalue(metadata.as_ref().and_then(|m| m.modified().ok()));
    let created = system_time_to_datavalue(metadata.as_ref().and_then(|m| m.created().ok()));
    let accessed = system_time_to_datavalue(metadata.as_ref().and_then(|m| m.accessed().ok()));

    let is_dir = DataValue::Boolean(entry.file_type().is_dir());
    let is_symlink = DataValue::Boolean(entry.file_type().is_symlink());
    let depth = DataValue::Integer(entry.depth() as i64);

    DataRow::new(vec![
        DataValue::String(path_str),
        DataValue::String(parent_str),
        DataValue::String(name_str),
        DataValue::String(stem_str),
        ext_val,
        size,
        modified,
        created,
        accessed,
        is_dir,
        is_symlink,
        depth,
    ])
}

fn system_time_to_datavalue(time: Option<std::time::SystemTime>) -> DataValue {
    match time {
        Some(t) => {
            let dt: DateTime<Local> = t.into();
            DataValue::DateTime(dt.to_rfc3339())
        }
        None => DataValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_spec(path: &str) -> FileCTESpec {
        FileCTESpec {
            path: path.to_string(),
            recursive: false,
            glob: None,
            max_depth: None,
            max_files: None,
            follow_links: false,
            include_hidden: false,
        }
    }

    #[test]
    fn test_basic_walk() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), "hello").unwrap();
        fs::write(tmp.path().join("b.csv"), "1,2,3").unwrap();
        fs::create_dir(tmp.path().join("subdir")).unwrap();

        let spec = make_spec(tmp.path().to_str().unwrap());
        let table = walk_filesystem(&spec, "files").unwrap();

        // root + a.txt + b.csv + subdir = 4 entries
        assert_eq!(table.row_count(), 4);
        assert_eq!(table.columns.len(), 12);
    }

    #[test]
    fn test_non_recursive_excludes_nested() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("sub/deep")).unwrap();
        fs::write(tmp.path().join("top.txt"), "").unwrap();
        fs::write(tmp.path().join("sub/nested.txt"), "").unwrap();
        fs::write(tmp.path().join("sub/deep/buried.txt"), "").unwrap();

        let spec = make_spec(tmp.path().to_str().unwrap());
        let table = walk_filesystem(&spec, "files").unwrap();

        // Non-recursive: root + top.txt + sub = 3 (no nested.txt, no deep/, no buried.txt)
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_recursive_walk() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("sub/deep")).unwrap();
        fs::write(tmp.path().join("top.txt"), "").unwrap();
        fs::write(tmp.path().join("sub/nested.txt"), "").unwrap();
        fs::write(tmp.path().join("sub/deep/buried.txt"), "").unwrap();

        let mut spec = make_spec(tmp.path().to_str().unwrap());
        spec.recursive = true;

        let table = walk_filesystem(&spec, "files").unwrap();

        // root + top.txt + sub + nested.txt + deep + buried.txt = 6
        assert_eq!(table.row_count(), 6);
    }

    #[test]
    fn test_glob_filter() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.csv"), "").unwrap();
        fs::write(tmp.path().join("b.csv"), "").unwrap();
        fs::write(tmp.path().join("c.txt"), "").unwrap();

        let mut spec = make_spec(tmp.path().to_str().unwrap());
        spec.glob = Some("*.csv".to_string());

        let table = walk_filesystem(&spec, "files").unwrap();

        // root dir + 2 csv files (txt excluded, root dir passes as directory)
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_max_files_enforcement() {
        let tmp = TempDir::new().unwrap();
        for i in 0..10 {
            fs::write(tmp.path().join(format!("file_{i}.txt")), "").unwrap();
        }

        let mut spec = make_spec(tmp.path().to_str().unwrap());
        spec.max_files = Some(5);

        let result = walk_filesystem(&spec, "files");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MAX_FILES"));
    }

    #[test]
    fn test_hidden_files_excluded_by_default() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("visible.txt"), "").unwrap();
        fs::write(tmp.path().join(".hidden"), "").unwrap();

        let spec = make_spec(tmp.path().to_str().unwrap());
        let table = walk_filesystem(&spec, "files").unwrap();

        // root + visible.txt = 2 (hidden excluded)
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_hidden_files_included() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("visible.txt"), "").unwrap();
        fs::write(tmp.path().join(".hidden"), "").unwrap();

        let mut spec = make_spec(tmp.path().to_str().unwrap());
        spec.include_hidden = true;

        let table = walk_filesystem(&spec, "files").unwrap();

        // root + visible.txt + .hidden = 3
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_max_depth() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("a/b/c")).unwrap();
        fs::write(tmp.path().join("a/b/c/deep.txt"), "").unwrap();

        let mut spec = make_spec(tmp.path().to_str().unwrap());
        spec.recursive = true;
        spec.max_depth = Some(2);

        let table = walk_filesystem(&spec, "files").unwrap();

        // root(0) + a(1) + b(2) = 3. c(3) and deep.txt(4) excluded by max_depth 2
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn test_invalid_path() {
        let spec = make_spec("/nonexistent/path/that/does/not/exist");
        let result = walk_filesystem(&spec, "files");
        assert!(result.is_err());
    }

    #[test]
    fn test_column_values() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("test.csv"), "hello world").unwrap();

        let spec = make_spec(tmp.path().to_str().unwrap());
        let table = walk_filesystem(&spec, "files").unwrap();

        // Find the test.csv row
        let csv_row = table
            .rows
            .iter()
            .find(|r| matches!(&r.values[2], DataValue::String(s) if s == "test.csv"))
            .expect("should find test.csv row");

        // Check ext
        assert_eq!(csv_row.values[4], DataValue::String("csv".to_string()));
        // Check size (11 bytes for "hello world")
        assert_eq!(csv_row.values[5], DataValue::Integer(11));
        // Check is_dir
        assert_eq!(csv_row.values[9], DataValue::Boolean(false));
        // Check is_symlink
        assert_eq!(csv_row.values[10], DataValue::Boolean(false));
        // Check depth (1 for immediate child)
        assert_eq!(csv_row.values[11], DataValue::Integer(1));
    }
}
