# Trades Query Tests

This test file (`test_trades_queries.rs`) loads real trade data from `data/trades.json` (100 records) into a DataTable and tests various SQL queries using the QueryEngine.

## Test File Location
- Test file: `tests/test_trades_queries.rs`
- Data file: `data/trades.json`
- Run tests: `cargo test --test test_trades_queries`

## Available Tests

### Basic Operations
- `test_load_trades_table` - Verifies loading JSON into DataTable
- `test_select_all_trades` - SELECT * FROM trades
- `test_select_specific_columns` - Column projection

### Filtering Tests
- `test_filter_by_trader` - WHERE trader = 'Jane Doe'
- `test_filter_by_currency` - WHERE currency = 'USD'
- `test_filter_by_quantity_range` - WHERE quantity > 5000
- `test_filter_by_price_range` - WHERE price BETWEEN 100 AND 300
- `test_filter_by_book` - WHERE book LIKE '%EQUITY%'
- `test_filter_by_counterparty_in_list` - WHERE counterparty IN (...)
- `test_filter_by_date` - WHERE createdDate > '2024-06-01'
- `test_filter_by_confirmation_status` - WHERE confirmationStatus = 'confirmed'
- `test_filter_counterparty_type` - WHERE counterpartyType = 'BANK'

### Complex Queries
- `test_complex_filter_and` - WHERE currency = 'USD' AND quantity > 5000
- `test_complex_filter_or` - WHERE currency = 'EUR' OR currency = 'GBP'
- `test_projection_with_filter` - SELECT columns with WHERE filter

### Pagination & Sorting
- `test_select_with_limit` - LIMIT 10
- `test_select_with_limit_offset` - LIMIT 5 OFFSET 10
- `test_order_by_price` - ORDER BY price ASC

## Trade Data Structure
The trades.json file contains records with these fields:
- id: Integer
- book: String (e.g., "EQUITY_DESK_2", "BOND_DESK_1")
- commission: Float
- confirmationStatus: String
- instrumentId: String
- platformOrderId: String
- counterparty: String (e.g., "JP_MORGAN", "GOLDMAN_SACHS")
- instrumentName: String
- counterpartyCountry: String
- counterpartyType: String (e.g., "BANK", "BROKER")
- createdDate: String (YYYY-MM-DD format)
- currency: String (e.g., "USD", "EUR", "GBP")
- quantity: Integer
- price: Float
- trader: String

## Adding New Tests

To add a new test case:

1. Add a new test function in `test_trades_queries.rs`
2. Use the `load_trades_table()` helper to get the DataTable
3. Execute your SQL query using `QueryEngine`
4. Assert the results

Example:
```rust
#[test]
fn test_your_new_query() {
    let table = load_trades_table();
    let engine = QueryEngine::new();
    
    let view = engine
        .execute(table.clone(), "YOUR SQL QUERY HERE")
        .unwrap();
    
    // Add your assertions
    assert_eq!(view.row_count(), expected_count);
}
```

## Supported SQL Features
✅ SELECT * and column projection
✅ WHERE with =, !=, >, <, >=, <=
✅ WHERE with LIKE patterns
✅ WHERE with IN lists
✅ WHERE with BETWEEN ranges
✅ AND/OR conditions
✅ ORDER BY ASC/DESC
✅ LIMIT and OFFSET

## Not Yet Supported
- GROUP BY
- Aggregate functions (COUNT, SUM, AVG, etc.)
- JOINs
- Nested subqueries
- HAVING clauses