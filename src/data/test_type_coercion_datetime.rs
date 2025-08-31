// Test for type coercion issues with ID columns that look like dates
#[cfg(test)]
mod tests {
    use crate::data::data_analyzer::{ColumnType, DataAnalyzer};
    use crate::data::datatable::DataType;

    #[test]
    fn test_json_id_not_detected_as_datetime() {
        // Test that ID strings from JSON are not incorrectly detected as DateTime
        let id_values = vec![
            "BQ-812674123",
            "BQ-81198596",
            "ID-12345678",
            "AB-99999999",
            "XY-00000001",
        ];

        for id in &id_values {
            let inferred_type = DataType::infer_from_string(id);
            assert_eq!(
                inferred_type,
                DataType::String,
                "ID '{}' should be inferred as String, not DateTime",
                id
            );
        }
    }

    #[test]
    fn test_real_dates_still_detected() {
        // Real dates should still be detected correctly
        let date_values = vec!["2024-01-15", "2024-12-31", "01/15/2024", "31-12-2024"];

        for date in &date_values {
            let inferred_type = DataType::infer_from_string(date);
            assert_eq!(
                inferred_type,
                DataType::DateTime,
                "Date '{}' should be inferred as DateTime",
                date
            );
        }
    }

    #[test]
    fn test_type_coercion_contains() {
        // This test demonstrates the issue where ID columns like "BQ-81198596"
        // are being incorrectly detected as datetime values

        let analyzer = DataAnalyzer::new();

        // Test various ID formats that should NOT be detected as dates
        let id_values = vec![
            "BQ-81198596",
            "ID-12345678",
            "AB-99999999",
            "XY-00000001",
            "ZZ-87654321",
        ];

        let column_type =
            analyzer.detect_column_type(&id_values.iter().map(|s| *s).collect::<Vec<_>>());

        // The problem: These are being detected as Date when they should be String
        println!("Column type detected for ID values: {:?}", column_type);

        // This test will currently FAIL because the regex pattern
        // ^\d{2}-\d{2}-\d{4} matches strings like "81-19-8596"
        // (the numeric part after "BQ-")
        assert_eq!(
            column_type,
            ColumnType::String,
            "ID columns like 'BQ-81198596' should be detected as String, not Date"
        );
    }

    #[test]
    fn test_real_dates_still_work() {
        let analyzer = DataAnalyzer::new();

        // Real dates should still be detected correctly
        let date_values = vec![
            "2024-01-15",
            "2024-02-20",
            "2024-03-25",
            "2024-04-30",
            "2024-05-10",
        ];

        let column_type =
            analyzer.detect_column_type(&date_values.iter().map(|s| *s).collect::<Vec<_>>());
        assert_eq!(
            column_type,
            ColumnType::Date,
            "Actual dates like '2024-01-15' should still be detected as Date"
        );
    }
}
