#[cfg(test)]
mod numeric_sorting_tests {
    use sql_cli::data::csv_datasource::CsvDataSource;
    use sql_cli::data::data_view::DataView;
    use sql_cli::data::datatable::DataValue;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "name,small_num,big_num,float_val
Item1,100,1000000,1.5
Item2,20,500000,10.99
Item3,3,999999,100.00
Item4,1000,100,0.99
Item5,5,50000,50.50
Item6,1,10,1000.0
Item7,200,1,0.01
Item8,999999,999999999,999999.99"
        )
        .unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_numeric_column_sorting() {
        // Create test CSV file
        let temp_file = create_test_csv();
        let path = temp_file.path();

        // Load CSV
        let csv_source = CsvDataSource::load_from_file(path, "test").unwrap();
        let datatable = csv_source.to_datatable();

        // Create DataView
        let mut view = DataView::new(Arc::new(datatable.clone()));

        // Test sorting the small_num column (index 1)
        view.apply_sort(1, true).unwrap(); // Sort ascending

        // Get the sorted values
        let mut sorted_values = Vec::new();
        for i in 0..view.row_count() {
            if let Some(row) = view.get_row(i) {
                // small_num is at index 1 in the row values
                if let Some(val) = row.values.get(1) {
                    sorted_values.push(val.clone());
                }
            }
        }

        println!("Sorted small_num values (ascending):");
        for (i, val) in sorted_values.iter().enumerate() {
            println!("  {}: {:?}", i, val);
        }

        // Check if values are properly sorted numerically
        // Expected order: 1, 3, 5, 20, 100, 200, 1000, 999999
        assert_eq!(sorted_values.len(), 8);

        // Extract numeric values for verification
        let numeric_values: Vec<f64> = sorted_values
            .iter()
            .map(|v| match v {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => 0.0,
            })
            .collect();

        println!("Numeric values: {:?}", numeric_values);

        // Verify they are in ascending order
        for i in 1..numeric_values.len() {
            assert!(
                numeric_values[i] >= numeric_values[i - 1],
                "Values not in ascending order: {} should be >= {}",
                numeric_values[i],
                numeric_values[i - 1]
            );
        }

        // Also test that specific values are in the right positions
        assert!(numeric_values[0] <= 1.0, "First value should be 1");
        assert!(
            numeric_values[numeric_values.len() - 1] >= 999999.0,
            "Last value should be 999999"
        );
    }

    #[test]
    fn test_float_column_sorting() {
        // Create test CSV file
        let temp_file = create_test_csv();
        let path = temp_file.path();

        // Load CSV
        let csv_source = CsvDataSource::load_from_file(path, "test").unwrap();
        let datatable = csv_source.to_datatable();

        // Create DataView
        let mut view = DataView::new(Arc::new(datatable.clone()));

        // Test sorting the float_val column (index 3)
        view.apply_sort(3, true).unwrap(); // Sort ascending

        // Get the sorted values
        let mut sorted_values = Vec::new();
        for i in 0..view.row_count() {
            if let Some(row) = view.get_row(i) {
                // float_val is at index 3 in the row values
                if let Some(val) = row.values.get(3) {
                    sorted_values.push(val.clone());
                }
            }
        }

        println!("Sorted float_val values (ascending):");
        for (i, val) in sorted_values.iter().enumerate() {
            println!("  {}: {:?}", i, val);
        }

        // Extract numeric values for verification
        let numeric_values: Vec<f64> = sorted_values
            .iter()
            .map(|v| match v {
                DataValue::Integer(i) => *i as f64,
                DataValue::Float(f) => *f,
                DataValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => 0.0,
            })
            .collect();

        println!("Float numeric values: {:?}", numeric_values);

        // Verify they are in ascending order
        for i in 1..numeric_values.len() {
            assert!(
                numeric_values[i] >= numeric_values[i - 1],
                "Float values not in ascending order: {} should be >= {}",
                numeric_values[i],
                numeric_values[i - 1]
            );
        }

        // Check specific values
        assert!(numeric_values[0] <= 0.01, "First float should be 0.01");
        assert!(
            numeric_values[numeric_values.len() - 1] >= 999999.0,
            "Last float should be 999999.99"
        );
    }

    #[test]
    fn test_values_are_numeric_not_strings() {
        // Create test CSV file
        let temp_file = create_test_csv();
        let path = temp_file.path();

        // Load CSV
        let csv_source = CsvDataSource::load_from_file(path, "test").unwrap();
        let datatable = csv_source.to_datatable();

        // Check that numeric columns are properly typed
        println!("Column types:");
        for (i, col) in datatable.columns.iter().enumerate() {
            println!("  Column {}: {} - Type: {:?}", i, col.name, col.data_type);
        }

        // Get first row to check actual value types
        if let Some(row) = datatable.rows.get(0) {
            println!("First row value types:");
            for (i, val) in row.values.iter().enumerate() {
                let type_name = match val {
                    DataValue::String(_) => "String",
                    DataValue::Integer(_) => "Integer",
                    DataValue::Float(_) => "Float",
                    DataValue::InternedString(_) => "InternedString",
                    _ => "Other",
                };
                println!("  Value {}: {:?} ({})", i, val, type_name);
            }
        }

        // Verify that numeric columns are NOT strings
        for row in &datatable.rows {
            // Check small_num column (index 1)
            if let Some(val) = row.values.get(1) {
                assert!(
                    !matches!(val, DataValue::String(_) | DataValue::InternedString(_)),
                    "small_num column should be numeric, not string: {:?}",
                    val
                );
            }
            // Check float_val column (index 3)
            if let Some(val) = row.values.get(3) {
                assert!(
                    !matches!(val, DataValue::String(_) | DataValue::InternedString(_)),
                    "float_val column should be numeric, not string: {:?}",
                    val
                );
            }
        }
    }
}
