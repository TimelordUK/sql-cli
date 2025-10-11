use std::io;

use crate::data::csv_datasource;
use crate::refactoring::banding::CaseGenerator;

/// Handle --generate-bands flag to generate banding CASE statements
pub fn handle_banding_generation(args: &[String]) -> io::Result<()> {
    // Parse column and bands arguments
    let column_pos = args.iter().position(|arg| arg == "--column");
    let bands_pos = args.iter().position(|arg| arg == "--bands");

    let column = column_pos.and_then(|pos| args.get(pos + 1));
    let bands = bands_pos.and_then(|pos| args.get(pos + 1));

    match (column, bands) {
        (Some(col), Some(bands_spec)) => {
            // Generate the CASE statement
            let case_sql = generate_banding_case(col, bands_spec);
            println!("{}", case_sql);
            Ok(())
        }
        _ => {
            eprintln!("Error: --generate-bands requires --column <name> and --bands <spec>");
            eprintln!("Example: --generate-bands --column age --bands \"0-24,25-49,50-74,75+\"");
            std::process::exit(1);
        }
    }
}

/// Handle --generate-case flag to generate CASE statements from data analysis
pub fn handle_case_generation(args: &[String]) -> io::Result<()> {
    // Find file argument position
    let case_pos = args
        .iter()
        .position(|arg| arg == "--generate-case")
        .unwrap();
    let file_path = args.get(case_pos + 1);

    // Parse other arguments
    let column_pos = args.iter().position(|arg| arg == "--column");
    let column = column_pos.and_then(|pos| args.get(pos + 1));

    let style_pos = args.iter().position(|arg| arg == "--style");
    let style = style_pos
        .and_then(|pos| args.get(pos + 1))
        .map(|s| s.as_str())
        .unwrap_or("values");

    let labels_pos = args.iter().position(|arg| arg == "--labels");
    let labels = labels_pos.and_then(|pos| args.get(pos + 1)).map(|l| {
        l.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    match (file_path, column) {
        (Some(path), Some(col)) => {
            // Load the data file
            let datasource = match csv_datasource::CsvDataSource::load_from_file(path, "data") {
                Ok(ds) => ds,
                Err(e) => {
                    eprintln!("Error loading file {}: {}", path, e);
                    std::process::exit(1);
                }
            };

            let datatable = datasource.to_datatable();

            // Find the column
            let col_index = datatable
                .columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(col))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Column '{}' not found", col),
                    )
                })?;

            // Get distinct values or analyze range
            match style {
                "values" => {
                    // Get distinct values
                    let mut distinct_values = std::collections::BTreeSet::new();
                    for row in &datatable.rows {
                        if let Some(value) = row.get(col_index) {
                            if !value.is_null() {
                                distinct_values.insert(value.to_string());
                            }
                        }
                    }

                    // Generate value mappings
                    let value_mappings: Vec<(String, String)> = if let Some(ref labels) = labels {
                        distinct_values
                            .into_iter()
                            .zip(labels.iter())
                            .map(|(v, l)| (v, l.clone()))
                            .collect()
                    } else {
                        distinct_values
                            .into_iter()
                            .map(|v| {
                                let label = v.replace('_', " ").replace('-', " ");
                                let label = label
                                    .split_whitespace()
                                    .map(|word| {
                                        let mut chars = word.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => {
                                                first.to_uppercase().collect::<String>()
                                                    + chars.as_str()
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                (v, label)
                            })
                            .collect()
                    };

                    let generator = CaseGenerator::from_values(col, value_mappings);
                    println!("{}", generator.to_sql());
                }
                "ranges" => {
                    // Analyze numeric range
                    let mut min_val = f64::MAX;
                    let mut max_val = f64::MIN;
                    let mut count = 0;

                    for row in &datatable.rows {
                        if let Some(value) = row.get(col_index) {
                            if let Ok(num) = value.to_string().parse::<f64>() {
                                min_val = min_val.min(num);
                                max_val = max_val.max(num);
                                count += 1;
                            }
                        }
                    }

                    if count == 0 {
                        eprintln!("No numeric values found in column '{}'", col);
                        std::process::exit(1);
                    }

                    // Generate smart ranges based on data distribution
                    let range = max_val - min_val;
                    let bands_spec = if range <= 100.0 {
                        // Small range - use 10-unit bands
                        let num_bands = ((range / 10.0).ceil() as usize).min(10);
                        let mut bands = Vec::new();
                        for i in 0..num_bands {
                            let start = min_val + (i as f64 * 10.0);
                            let end = (min_val + ((i + 1) as f64 * 10.0)).min(max_val);
                            if i == num_bands - 1 {
                                bands.push(format!("{:.0}+", start));
                            } else {
                                bands.push(format!("{:.0}-{:.0}", start, end));
                            }
                        }
                        bands.join(",")
                    } else {
                        // Large range - use quartiles or quintiles
                        let step = range / 5.0;
                        let mut bands = Vec::new();
                        for i in 0..5 {
                            let start = min_val + (i as f64 * step);
                            let end = min_val + ((i + 1) as f64 * step);
                            if i == 4 {
                                bands.push(format!("{:.0}+", start));
                            } else {
                                bands.push(format!("{:.0}-{:.0}", start, end));
                            }
                        }
                        bands.join(",")
                    };

                    let generator = CaseGenerator::from_ranges(col, &bands_spec, labels).unwrap();
                    println!("{}", generator.to_sql());
                }
                _ => {
                    eprintln!("Unknown style: {}. Use 'values' or 'ranges'", style);
                    std::process::exit(1);
                }
            }

            Ok(())
        }
        _ => {
            eprintln!("Error: --generate-case requires a file path and --column <name>");
            eprintln!("Example: --generate-case data.csv --column ocean_proximity --style values");
            std::process::exit(1);
        }
    }
}

/// Handle --generate-case-range flag to generate CASE statements for numeric ranges
pub fn handle_case_range_generation(args: &[String]) -> io::Result<()> {
    // Parse arguments
    let column_pos = args.iter().position(|arg| arg == "--column");
    let column = column_pos.and_then(|pos| args.get(pos + 1));

    let min_pos = args.iter().position(|arg| arg == "--min");
    let min_val = min_pos
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let max_pos = args.iter().position(|arg| arg == "--max");
    let max_val = max_pos
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(100.0);

    let bands_pos = args.iter().position(|arg| arg == "--bands");
    let num_bands = bands_pos
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5);

    let labels_pos = args.iter().position(|arg| arg == "--labels");
    let labels = labels_pos.and_then(|pos| args.get(pos + 1)).map(|l| {
        l.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    match column {
        Some(col) => {
            // Generate equal-width bands
            let width = (max_val - min_val) / num_bands as f64;
            let mut bands_spec = Vec::new();

            for i in 0..num_bands {
                let start = min_val + (i as f64 * width);
                let end = if i == num_bands - 1 {
                    max_val
                } else {
                    min_val + ((i + 1) as f64 * width)
                };

                if i == num_bands - 1 {
                    bands_spec.push(format!("{:.0}+", start));
                } else {
                    bands_spec.push(format!("{:.0}-{:.0}", start, end));
                }
            }

            let bands_str = bands_spec.join(",");
            let generator = CaseGenerator::from_ranges(col, &bands_str, labels).unwrap();
            println!("{}", generator.to_sql());

            Ok(())
        }
        _ => {
            eprintln!("Error: --generate-case-range requires --column <name>");
            eprintln!("Example: --generate-case-range --column value --min 0 --max 100 --bands 5");
            std::process::exit(1);
        }
    }
}

/// Generate a CASE statement from column name and bands specification
fn generate_banding_case(column: &str, bands_spec: &str) -> String {
    let mut sql = String::from("CASE");
    let bands: Vec<&str> = bands_spec.split(',').map(|s| s.trim()).collect();

    for (i, band) in bands.iter().enumerate() {
        sql.push('\n');

        if band.ends_with('+') {
            // Handle "75+" format - everything above the minimum
            let min = band.trim_end_matches('+').trim();
            sql.push_str(&format!("    WHEN {} >= {} THEN '{}'", column, min, band));
        } else if band.contains('-') {
            // Handle range format like "25-49"
            let parts: Vec<&str> = band.split('-').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let min = parts[0];
                let max = parts[1];

                if i == 0 {
                    // First band, from start up to max
                    sql.push_str(&format!("    WHEN {} <= {} THEN '{}'", column, max, band));
                } else {
                    // Subsequent bands, check both boundaries
                    sql.push_str(&format!(
                        "    WHEN {} BETWEEN {} AND {} THEN '{}'",
                        column, min, max, band
                    ));
                }
            }
        }
    }

    sql.push_str(&format!("\nEND AS {}_band", column));
    sql
}
