# Styled Table Output - Implementation TODO

## Status: 70% Complete

We've started implementing terminal-colored table output with YAML configuration. Here's what's done and what remains:

## ✅ Completed

1. **Added serde_yaml dependency** to Cargo.toml
2. **Created `src/output/styled_table.rs`** module with:
   - `StyleConfig` struct for YAML configuration
   - `ColumnRule`, `NumericRule`, `PatternRule` for different styling rules
   - `apply_styles_to_table()` function to apply colors to comfy_table
   - Color parsing (`parse_color()`)
   - Condition evaluation (`evaluate_condition()`)
   - Comprehensive tests for condition parsing
3. **Added output module** to lib.rs
4. **Updated NonInteractiveConfig** with `styled` and `style_file` fields
5. **Updated `output_results()` signature** to pass style parameters

## 🚧 Remaining Work

### 1. Update `output_table()` Function Signature
File: `src/non_interactive.rs` line 1395

**Current:**
```rust
fn output_table<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    max_col_width: Option<usize>,
    _col_sample_rows: usize,
    style: TableStyle,
) -> Result<()>
```

**Change to:**
```rust
fn output_table<W: Write>(
    dataview: &DataView,
    writer: &mut W,
    max_col_width: Option<usize>,
    _col_sample_rows: usize,
    style: TableStyle,
    styled: bool,
    style_file: Option<&str>,
) -> Result<()>
```

### 2. Add Style Application Logic to `output_table()`
After building the table (line ~1489), add:

```rust
// Apply color styling if requested
if styled {
    use crate::output::styled_table::{StyleConfig, apply_styles_to_table};
    use std::path::PathBuf;

    // Load style configuration
    let style_config = if let Some(file_path) = style_file {
        let path = PathBuf::from(file_path);
        StyleConfig::from_file(&path).ok()
    } else {
        StyleConfig::load_default()
    };

    if let Some(config) = style_config {
        // Convert DataView rows to Vec<Vec<String>> for styling
        let rows: Vec<Vec<String>> = (0..dataview.row_count())
            .filter_map(|i| {
                dataview.get_row(i).map(|row| {
                    row.values.iter().map(|v| format_value(v)).collect()
                })
            })
            .collect();

        let columns = dataview.column_names();
        if let Err(e) = apply_styles_to_table(&mut table, &columns, &rows, &config) {
            eprintln!("Warning: Failed to apply styles: {}", e);
        }
    }
}
```

### 3. Update Call Sites
Need to update all `output_table()` calls to include new parameters:

**Line ~802 (execute_script):**
```rust
output_table(
    &final_view,
    &mut statement_output,
    config.max_col_width,
    config.col_sample_rows,
    config.table_style,
    config.styled,  // ADD
    config.style_file.as_deref(),  // ADD
)?;
```

**Lines ~617 and ~632 (execute_non_interactive):**
```rust
output_results(
    &final_view,
    config.output_format,
    &mut file,  // or &mut io::stdout()
    config.max_col_width,
    config.col_sample_rows,
    exec_time_ms,
    config.table_style,
    config.styled,  // ADD
    config.style_file.as_deref(),  // ADD
)?;
```

### 4. Add CLI Flags to main.rs
Add these arguments to the CLI parser:

```rust
.arg(
    Arg::new("styled")
        .long("styled")
        .action(ArgAction::SetTrue)
        .help("Apply color styling rules to table output (uses ~/.config/sql-cli/styles.yaml or --style-file)"),
)
.arg(
    Arg::new("style-file")
        .long("style-file")
        .value_name("PATH")
        .help("Path to YAML style configuration file"),
)
```

Then in the config building section:

```rust
styled: matches.get_flag("styled"),
style_file: matches.get_one::<String>("style-file").cloned(),
```

### 5. Create Example Style File
File: `~/.config/sql-cli/styles.yaml`

```yaml
version: 1

# Financial data styling
columns:
  Side:
    - value: "Buy"
      fg_color: blue
      bold: true
    - value: "Sell"
      fg_color: red
      bold: true

  Status:
    - value: "Active"
      fg_color: green
    - value: "Inactive"
      fg_color: dark_grey

numeric_ranges:
  LatencyMs:
    - condition: "< 100"
      fg_color: green
    - condition: ">= 100 AND < 300"
      fg_color: yellow
    - condition: ">= 300"
      fg_color: red
      bold: true

  ExecutionPrice:
    - condition: "> 400"
      fg_color: cyan
      bold: true

patterns:
  - regex: "^ERROR"
    fg_color: red
    bold: true
  - regex: "^WARN"
    fg_color: yellow

defaults:
  header_color: white
  header_bold: true
```

### 6. Test with Hedge Fund Example

```bash
# Build first
cargo build --release

# Test with styling
./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql \
  --execute-statement 6 -o table --styled

# With custom style file
./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql \
  --execute-statement 6 -o table --style-file examples/financial-styles.yaml
```

## Implementation Time Estimate

- Update function signatures and call sites: **15 minutes**
- Add CLI flags to main.rs: **10 minutes**
- Test compilation and fix any issues: **15 minutes**
- Create example style files: **10 minutes**
- Test with hedge fund example: **10 minutes**

**Total: ~1 hour**

## Benefits Once Complete

1. **Terminal-native colored output** for any user running sql-cli
2. **YAML configuration** - easy to edit, version control, share
3. **Flexible rules** - column values, numeric ranges, regex patterns
4. **Composable** - multiple rules can apply to same cell
5. **Financial workflows** - perfect for buy/sell, latency, pricing visualization

## Example Output (After Completion)

```
Stage 6: Execution Quality Analysis by Sector
┌────────────────────────┬──────┬─────────────────┐
│         Sector         │ Side │ total_volume    │
├────────────────────────┼──────┼─────────────────┤
│ Technology             │ Buy  │ 13993           │  <- "Buy" in BLUE+BOLD
│ Technology             │ Sell │ 2960            │  <- "Sell" in RED+BOLD
└────────────────────────┴──────┴─────────────────┘

Stage 8: Latency Distribution
┌───────────────┬─────────────────┐
│ latency_bucket│ execution_count │
├───────────────┼─────────────────┤
│ < 100ms       │ 6               │  <- GREEN (good latency)
│ 100-200ms     │ 6               │  <- YELLOW
│ 300-400ms     │ 14              │  <- RED
│ > 400ms       │ 11              │  <- RED+BOLD (bad latency)
└───────────────┴─────────────────┘
```

## Files Modified

- ✅ `Cargo.toml` - Added serde_yaml
- ✅ `src/lib.rs` - Added output module
- ✅ `src/output/mod.rs` - New module declaration
- ✅ `src/output/styled_table.rs` - New styling implementation
- ✅ `src/non_interactive.rs` - Added styled fields to config
- 🚧 `src/non_interactive.rs` - Need to update output_table() and call sites
- 🚧 `src/main.rs` - Need to add CLI flags
- 🚧 `~/.config/sql-cli/styles.yaml` - Need to create example

## Next Session Plan

1. Complete the remaining implementation (1 hour)
2. Test with hedge fund example
3. Create example style files for different use cases
4. Add to CHANGELOG.md
5. Commit and push

---

**Created:** 2025-10-12
**Status:** In Progress
**Priority:** Medium (Nice to have for next release)
