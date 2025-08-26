#[cfg(test)]
mod mixed_numeric_sorting_tests {
    use sql_cli::data::csv_datasource::CsvDataSource;
    use sql_cli::data::data_view::DataView;
    use sql_cli::data::datatable::DataValue;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mixed_integer_float_sorting() {
        // Create a CSV that mimics the user's issue:
        // Mix of large integers and small floats in same column
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "name,mixed_values
Item1,142
Item2,0.0006
Item3,16257
Item4,0.0007
Item5,148
Item6,0.5
Item7,1000
Item8,0.001"
        )
        .unwrap();
        file.flush().unwrap();

        // Load CSV
        let csv_source = CsvDataSource::load_from_file(file.path(), "test").unwrap();
        let datatable = csv_source.to_datatable();

        // Create DataView and sort
        let mut view = DataView::new(Arc::new(datatable.clone()));
        view.apply_sort(1, true).unwrap(); // Sort mixed_values column ascending

        // Get sorted values
        let mut sorted_values = Vec::new();
        let mut numeric_values = Vec::new();

        for i in 0..view.row_count() {
            if let Some(row) = view.get_row(i) {
                if let Some(val) = row.values.get(1) {
                    sorted_values.push(val.clone());

                    // Convert to f64 for comparison
                    let num = match val {
                        DataValue::Integer(i) => *i as f64,
                        DataValue::Float(f) => *f,
                        _ => panic!("Unexpected non-numeric value: {:?}", val),
                    };
                    numeric_values.push(num);
                }
            }
        }

        println!("Sorted values (should be in numeric order):");
        for (i, (val, num)) in sorted_values.iter().zip(numeric_values.iter()).enumerate() {
            println!("  {}: {:?} ({})", i, val, num);
        }

        // Verify correct numeric ordering
        // Expected order: 0.0006, 0.0007, 0.001, 0.5, 142, 148, 1000, 16257
        assert_eq!(numeric_values.len(), 8);

        // Check specific values are in correct positions
        assert!(numeric_values[0] < 0.001, "First value should be 0.0006");
        assert!(numeric_values[1] < 0.001, "Second value should be 0.0007");
        assert!(numeric_values[2] < 0.01, "Third value should be 0.001");
        assert!(numeric_values[3] < 1.0, "Fourth value should be 0.5");
        assert!(
            numeric_values[4] > 100.0 && numeric_values[4] < 143.0,
            "Fifth should be 142"
        );
        assert!(
            numeric_values[5] > 147.0 && numeric_values[5] < 149.0,
            "Sixth should be 148"
        );
        assert!(
            numeric_values[6] > 999.0 && numeric_values[6] < 1001.0,
            "Seventh should be 1000"
        );
        assert!(numeric_values[7] > 16000.0, "Last value should be 16257");

        // Most importantly: verify proper ascending order
        for i in 1..numeric_values.len() {
            assert!(
                numeric_values[i] >= numeric_values[i - 1],
                "Values not in ascending order at index {}: {} should be >= {}",
                i,
                numeric_values[i],
                numeric_values[i - 1]
            );
        }
    }

    #[test]
    fn test_type_detection_for_mixed_column() {
        // Test that a column with both integers and floats is detected properly
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "name,values
Item1,100
Item2,0.5
Item3,200"
        )
        .unwrap();
        file.flush().unwrap();

        let csv_source = CsvDataSource::load_from_file(file.path(), "test").unwrap();
        let datatable = csv_source.to_datatable();

        // Check column type
        println!("Column types:");
        for col in &datatable.columns {
            println!("  {}: {:?}", col.name, col.data_type);
        }

        // Check actual value types
        println!("Value types in 'values' column:");
        for (i, row) in datatable.rows.iter().enumerate() {
            if let Some(val) = row.values.get(1) {
                let type_str = match val {
                    DataValue::Integer(_) => "Integer",
                    DataValue::Float(_) => "Float",
                    _ => "Other",
                };
                println!("  Row {}: {:?} ({})", i, val, type_str);
            }
        }
    }
}
