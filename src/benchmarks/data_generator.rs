use crate::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use std::collections::HashMap;

pub struct BenchmarkDataGenerator;

impl BenchmarkDataGenerator {
    pub fn generate_narrow_table(rows: usize) -> DataTable {
        let mut table = DataTable::new("narrow_bench");

        // Add columns
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("amount").with_type(DataType::Float));
        table.add_column(DataColumn::new("category_id").with_type(DataType::Integer));

        // Add rows
        for i in 1..=rows {
            let row = DataRow::new(vec![
                DataValue::Integer(i as i64),
                DataValue::Float((i as f64) * 1.5 + 100.0),
                DataValue::Integer(((i % 100) + 1) as i64),
            ]);
            table.add_row(row).unwrap();
        }

        table
    }

    pub fn generate_wide_table(rows: usize, columns: usize) -> DataTable {
        let mut table = DataTable::new("wide_bench");

        // Add columns
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        for i in 1..columns {
            let col_type = match i % 3 {
                0 => DataType::Integer,
                1 => DataType::Float,
                _ => DataType::String,
            };
            table.add_column(DataColumn::new(&format!("col_{}", i)).with_type(col_type));
        }

        // Add rows
        for i in 1..=rows {
            let mut row_values = vec![DataValue::Integer(i as i64)];

            for j in 1..columns {
                let value = match j % 3 {
                    0 => DataValue::Integer((i * j) as i64),
                    1 => DataValue::Float((i * j) as f64 * 0.1),
                    _ => DataValue::String(format!("val_{}_{}", i % 10, j % 5)),
                };
                row_values.push(value);
            }

            table.add_row(DataRow::new(row_values)).unwrap();
        }

        table
    }

    pub fn generate_mixed_type_table(rows: usize) -> DataTable {
        let mut table = DataTable::new("mixed_bench");

        // Add columns
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("category").with_type(DataType::String));
        table.add_column(DataColumn::new("price").with_type(DataType::Float));
        table.add_column(DataColumn::new("quantity").with_type(DataType::Integer));
        table.add_column(DataColumn::new("status").with_type(DataType::String));
        table.add_column(DataColumn::new("description").with_type(DataType::String));

        let statuses = vec!["Active", "Inactive", "Pending", "Completed", "Failed"];
        let categories = vec!["Electronics", "Books", "Clothing", "Food", "Toys"];

        // Add rows
        for i in 1..=rows {
            let row = DataRow::new(vec![
                DataValue::Integer(i as i64),
                DataValue::String(categories[i % 5].to_string()),
                DataValue::Float(((i * 13) % 1000) as f64 + 10.0),
                DataValue::Integer(((i * 7) % 100 + 1) as i64),
                DataValue::String(statuses[i % 5].to_string()),
                DataValue::String(format!("Product description for item {}", i)),
            ]);
            table.add_row(row).unwrap();
        }

        table
    }

    pub fn generate_aggregation_table(rows: usize) -> DataTable {
        let mut table = DataTable::new("aggregation_bench");

        // Add columns
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("group_id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("sub_group").with_type(DataType::String));
        table.add_column(DataColumn::new("value1").with_type(DataType::Float));
        table.add_column(DataColumn::new("value2").with_type(DataType::Float));
        table.add_column(DataColumn::new("value3").with_type(DataType::Float));

        // Add rows
        for i in 1..=rows {
            let row = DataRow::new(vec![
                DataValue::Integer(i as i64),
                DataValue::Integer(((i - 1) / 100 + 1) as i64), // Creates groups of ~100
                DataValue::String(format!("sg_{}", (i % 10))),
                DataValue::Float((i as f64) * 0.1),
                DataValue::Float((i as f64) * 0.2 + 50.0),
                DataValue::Float((i as f64) * 0.3 - 25.0),
            ]);
            table.add_row(row).unwrap();
        }

        table
    }

    pub fn generate_window_function_table(rows: usize) -> DataTable {
        let mut table = DataTable::new("window_bench");

        // Add columns
        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("timestamp").with_type(DataType::Integer));
        table.add_column(DataColumn::new("department").with_type(DataType::String));
        table.add_column(DataColumn::new("employee_id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("sales").with_type(DataType::Float));
        table.add_column(DataColumn::new("commission").with_type(DataType::Float));

        let departments = vec!["Sales", "Marketing", "Engineering", "HR", "Finance"];

        // Add rows
        for i in 1..=rows {
            let row = DataRow::new(vec![
                DataValue::Integer(i as i64),
                DataValue::Integer((i * 3600) as i64), // Simulating timestamps
                DataValue::String(departments[i % 5].to_string()),
                DataValue::Integer(((i % 20) + 1) as i64),
                DataValue::Float(((i * 17) % 1000) as f64 + 100.0),
                DataValue::Float(((i * 17) % 100) as f64 * 0.01),
            ]);
            table.add_row(row).unwrap();
        }

        table
    }

    pub fn save_benchmark_data(table: &DataTable, filename: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::Write;

        let csv_content = table.to_csv();
        let mut file =
            File::create(filename).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(csv_content.as_bytes())
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn generate_all_benchmark_tables(base_rows: usize) -> HashMap<String, DataTable> {
        let mut tables = HashMap::new();

        tables.insert("narrow".to_string(), Self::generate_narrow_table(base_rows));
        tables.insert("wide".to_string(), Self::generate_wide_table(base_rows, 20));
        tables.insert(
            "very_wide".to_string(),
            Self::generate_wide_table(base_rows, 50),
        );
        tables.insert(
            "mixed".to_string(),
            Self::generate_mixed_type_table(base_rows),
        );
        tables.insert(
            "aggregation".to_string(),
            Self::generate_aggregation_table(base_rows),
        );
        tables.insert(
            "window".to_string(),
            Self::generate_window_function_table(base_rows),
        );

        tables
    }
}
