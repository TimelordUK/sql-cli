#[cfg(test)]
mod tests {
    use crate::csv_datasource::CsvApiClient;
    use serde_json::json;

    #[test]
    fn test_cache_mode_query_filtering() {
        // Create test data
        let test_data = vec![
            json!({
                "id": 1,
                "commission": 100,
                "counterparty": "Bank of America",
                "counterpartyCountry": "US"
            }),
            json!({
                "id": 2,
                "commission": 30,
                "counterparty": "JP Morgan Chase Bank",
                "counterpartyCountry": "US"
            }),
            json!({
                "id": 3,
                "commission": 75,
                "counterparty": "Deutsche Bank",
                "counterpartyCountry": "DE"
            }),
            json!({
                "id": 4,
                "commission": 25,
                "counterparty": "Credit Suisse",
                "counterpartyCountry": "CH"
            }),
        ];

        // Test 1: Simple numeric filter
        let mut csv_client = CsvApiClient::new();
        csv_client
            .load_from_json(test_data.clone(), "cached_data")
            .unwrap();

        let result = csv_client
            .query_csv("SELECT * FROM cached_data WHERE commission > 50")
            .unwrap();
        assert_eq!(
            result.data.len(),
            2,
            "Should return 2 rows with commission > 50"
        );

        // Test 2: Contains filter
        let result = csv_client
            .query_csv("SELECT * FROM cached_data WHERE counterparty.Contains(\"Bank\")")
            .unwrap();
        assert_eq!(
            result.data.len(),
            3,
            "Should return 3 rows with 'Bank' in counterparty"
        );

        // Test 3: Combined filters
        let result = csv_client.query_csv("SELECT * FROM cached_data WHERE commission > 50 AND counterparty.Contains(\"Bank\")").unwrap();
        assert_eq!(
            result.data.len(),
            2,
            "Should return 2 rows matching both conditions"
        );

        // Test 4: IN clause
        let result = csv_client
            .query_csv("SELECT * FROM cached_data WHERE counterpartyCountry IN (\"US\", \"DE\")")
            .unwrap();
        assert_eq!(
            result.data.len(),
            3,
            "Should return 3 rows with countries US or DE"
        );

        // Test 5: Complex query with IN clause
        let result = csv_client.query_csv("SELECT * FROM cached_data WHERE commission > 50 AND counterparty.Contains(\"Bank\") AND counterpartyCountry IN (\"US\", \"DE\")").unwrap();
        assert_eq!(
            result.data.len(),
            2,
            "Should return 2 rows matching all conditions"
        );
    }

    #[test]
    fn test_user_scenario_with_trade_deal() {
        // Create test data similar to user's trade_deal table
        let mut test_data = vec![];
        for i in 1..=100 {
            let commission = (i as f64) * 1.5;
            let counterparty = if i % 3 == 0 {
                format!("Bank {}", i)
            } else if i % 5 == 0 {
                format!("Financial Corp {}", i)
            } else {
                format!("Trading Company {}", i)
            };

            test_data.push(json!({
                "id": i,
                "commission": commission,
                "counterparty": counterparty,
                "amount": i * 1000
            }));
        }

        let mut csv_client = CsvApiClient::new();
        csv_client
            .load_from_json(test_data.clone(), "trade_deal")
            .unwrap();

        // Test the exact query the user mentioned
        let query =
            "select * from trade_deal where commission > 50 and counterparty.Contains(\"Bank\")";
        let result = csv_client.query_csv(query).unwrap();

        // Count expected results manually
        let expected_count = test_data
            .iter()
            .filter(|row| {
                let commission = row["commission"].as_f64().unwrap_or(0.0);
                let counterparty = row["counterparty"].as_str().unwrap_or("");
                commission > 50.0 && counterparty.contains("Bank")
            })
            .count();

        assert_eq!(
            result.data.len(),
            expected_count,
            "Query should return only rows with commission > 50 AND counterparty containing 'Bank'"
        );
        assert!(result.data.len() < 100, "Should not return all 100 rows");
    }

    #[test]
    fn test_complex_query_with_in_clause() {
        // Create test data with various countries
        let test_data = vec![
            json!({
                "id": 1,
                "commission": 100,
                "counterparty": "Bank of Tokyo",
                "counterpartyCountry": "JP"
            }),
            json!({
                "id": 2,
                "commission": 30,
                "counterparty": "Bank of America",
                "counterpartyCountry": "US"
            }),
            json!({
                "id": 3,
                "commission": 75,
                "counterparty": "BNP Paribas Bank",
                "counterpartyCountry": "FR"
            }),
            json!({
                "id": 4,
                "commission": 25,
                "counterparty": "Royal Bank of Canada",
                "counterpartyCountry": "CA"
            }),
            json!({
                "id": 5,
                "commission": 60,
                "counterparty": "Westpac Bank",
                "counterpartyCountry": "AU"
            }),
            json!({
                "id": 6,
                "commission": 80,
                "counterparty": "Societe Generale Bank",
                "counterpartyCountry": "FR"
            }),
            json!({
                "id": 7,
                "commission": 90,
                "counterparty": "Mizuho Bank",
                "counterpartyCountry": "JP"
            }),
        ];

        let mut csv_client = CsvApiClient::new();
        csv_client
            .load_from_json(test_data.clone(), "trade_deal")
            .unwrap();

        // Test the exact user query
        let query = r#"select * from trade_deal where commission > 50 and counterparty.Contains("Bank") and counterpartyCountry in ("JP","FR")"#;
        let result = csv_client.query_csv(query).unwrap();

        // Should only return rows with:
        // - commission > 50 (excludes id 2, 4)
        // - counterparty contains "Bank" (all have Bank)
        // - country in JP or FR (excludes id 5 with AU)
        // Expected: id 1 (JP, 100), id 3 (FR, 75), id 6 (FR, 80), id 7 (JP, 90)
        assert_eq!(
            result.data.len(),
            4,
            "Should return 4 rows matching all conditions"
        );

        // Verify the countries in results
        for row in &result.data {
            let country = row["counterpartyCountry"].as_str().unwrap();
            assert!(
                country == "JP" || country == "FR",
                "Country should be JP or FR, got {}",
                country
            );

            let commission = row["commission"].as_f64().unwrap();
            assert!(
                commission > 50.0,
                "Commission should be > 50, got {}",
                commission
            );

            let counterparty = row["counterparty"].as_str().unwrap();
            assert!(
                counterparty.contains("Bank"),
                "Counterparty should contain 'Bank', got {}",
                counterparty
            );
        }
    }
}
