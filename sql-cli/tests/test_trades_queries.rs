use serde_json::Value;
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::fs;
use std::sync::Arc;

/// Load trades.json into a DataTable
fn load_trades_table() -> Arc<DataTable> {
    let json_str = fs::read_to_string("data/trades.json").expect("Failed to read trades.json");

    let trades: Vec<Value> = serde_json::from_str(&json_str).expect("Failed to parse trades.json");

    let mut table = DataTable::new("trades");

    // Add columns based on first trade structure
    if let Some(first_trade) = trades.first() {
        if let Value::Object(map) = first_trade {
            for key in map.keys() {
                table.add_column(DataColumn::new(key.clone()));
            }
        }
    }

    // Add rows
    for trade in trades {
        if let Value::Object(map) = trade {
            let mut row_values = Vec::new();

            // Get values in same order as columns
            for col in &table.columns {
                let value = map.get(&col.name).unwrap_or(&Value::Null);
                row_values.push(json_to_datavalue(value));
            }

            table.add_row(DataRow::new(row_values)).unwrap();
        }
    }

    Arc::new(table)
}

fn json_to_datavalue(value: &Value) -> DataValue {
    match value {
        Value::String(s) => DataValue::String(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                DataValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                DataValue::Float(f)
            } else {
                DataValue::Null
            }
        }
        Value::Bool(b) => DataValue::Boolean(*b),
        Value::Null => DataValue::Null,
        _ => DataValue::Null,
    }
}

#[test]
fn test_load_trades_table() {
    let table = load_trades_table();

    // Verify we loaded the right number of trades
    assert!(table.row_count() > 0);

    // Verify columns exist
    assert!(table.get_column_index("id").is_some());
    assert!(table.get_column_index("book").is_some());
    assert!(table.get_column_index("trader").is_some());
    assert!(table.get_column_index("currency").is_some());
    assert!(table.get_column_index("quantity").is_some());
    assert!(table.get_column_index("price").is_some());

    println!(
        "Loaded {} trades with {} columns",
        table.row_count(),
        table.column_count()
    );
}

#[test]
fn test_select_all_trades() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades")
        .unwrap();

    assert_eq!(view.row_count(), table.row_count());
    assert_eq!(view.column_count(), table.column_count());
}

#[test]
fn test_select_specific_columns() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT id, trader, currency, quantity, price FROM trades",
        )
        .unwrap();

    assert_eq!(view.row_count(), table.row_count());
    assert_eq!(view.column_count(), 5);

    let columns = view.column_names();
    assert_eq!(
        columns,
        vec!["id", "trader", "currency", "quantity", "price"]
    );
}

#[test]
fn test_filter_by_trader() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE trader = 'Jane Doe'",
        )
        .unwrap();

    // Check all results have the right trader
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let trader_index = view.source().get_column_index("trader").unwrap();
        assert_eq!(
            row.values[trader_index],
            DataValue::String("Jane Doe".to_string())
        );
    }
}

#[test]
fn test_filter_by_currency() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades WHERE currency = 'USD'")
        .unwrap();

    // Check all results have USD currency
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let currency_index = view.source().get_column_index("currency").unwrap();
        assert_eq!(
            row.values[currency_index],
            DataValue::String("USD".to_string())
        );
    }
}

#[test]
fn test_filter_by_quantity_range() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades WHERE quantity > 5000")
        .unwrap();

    // Check all results have quantity > 5000
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let qty_index = view.source().get_column_index("quantity").unwrap();
        if let DataValue::Integer(qty) = &row.values[qty_index] {
            assert!(*qty > 5000);
        }
    }
}

#[test]
fn test_filter_by_price_range() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE price BETWEEN 100 AND 300",
        )
        .unwrap();

    // Check all results have price in range
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let price_index = view.source().get_column_index("price").unwrap();
        if let DataValue::Float(price) = &row.values[price_index] {
            assert!(*price >= 100.0 && *price <= 300.0);
        }
    }
}

#[test]
fn test_filter_by_book() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE book LIKE '%EQUITY%'",
        )
        .unwrap();

    // Check all results have EQUITY in book name
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let book_index = view.source().get_column_index("book").unwrap();
        if let DataValue::String(book) = &row.values[book_index] {
            assert!(book.contains("EQUITY"));
        }
    }
}

#[test]
fn test_filter_by_counterparty_in_list() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades WHERE counterparty IN ('JP_MORGAN', 'GOLDMAN_SACHS', 'DEUTSCHE_BANK')")
        .unwrap();

    // Check all results have one of these counterparties
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let cp_index = view.source().get_column_index("counterparty").unwrap();
        if let DataValue::String(cp) = &row.values[cp_index] {
            assert!(cp == "JP_MORGAN" || cp == "GOLDMAN_SACHS" || cp == "DEUTSCHE_BANK");
        }
    }
}

#[test]
fn test_complex_filter_and() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE currency = 'USD' AND quantity > 5000",
        )
        .unwrap();

    // Check all results match both conditions
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();

        let currency_index = view.source().get_column_index("currency").unwrap();
        assert_eq!(
            row.values[currency_index],
            DataValue::String("USD".to_string())
        );

        let qty_index = view.source().get_column_index("quantity").unwrap();
        if let DataValue::Integer(qty) = &row.values[qty_index] {
            assert!(*qty > 5000);
        }
    }
}

#[test]
fn test_complex_filter_or() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE currency = 'EUR' OR currency = 'GBP'",
        )
        .unwrap();

    // Check all results have EUR or GBP
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let currency_index = view.source().get_column_index("currency").unwrap();
        if let DataValue::String(currency) = &row.values[currency_index] {
            assert!(currency == "EUR" || currency == "GBP");
        }
    }
}

#[test]
fn test_select_with_limit() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades LIMIT 10")
        .unwrap();

    assert_eq!(view.row_count(), 10);
}

#[test]
fn test_select_with_limit_offset() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades LIMIT 5 OFFSET 10")
        .unwrap();

    assert_eq!(view.row_count(), 5);

    // Verify it's actually offset by checking IDs
    let first_row = view.get_row(0).unwrap();
    let id_index = view.source().get_column_index("id").unwrap();
    if let DataValue::Integer(id) = &first_row.values[id_index] {
        assert_eq!(*id, 11); // Should be 11th record (offset by 10)
    }
}

#[test]
fn test_order_by_price() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades ORDER BY price ASC LIMIT 10",
        )
        .unwrap();

    // Check that prices are in ascending order
    let price_index = view.source().get_column_index("price").unwrap();
    let mut last_price = 0.0;

    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        if let DataValue::Float(price) = &row.values[price_index] {
            assert!(*price >= last_price);
            last_price = *price;
        }
    }
}

#[test]
fn test_filter_by_date() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE createdDate > '2024-06-01'",
        )
        .unwrap();

    // Check all dates are after June 1, 2024
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let date_index = view.source().get_column_index("createdDate").unwrap();
        if let DataValue::String(date) = &row.values[date_index] {
            assert!(date.as_str() > "2024-06-01");
        }
    }
}

#[test]
fn test_filter_by_confirmation_status() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE confirmationStatus = 'confirmed'",
        )
        .unwrap();

    // Check all have confirmed status
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let status_index = view
            .source()
            .get_column_index("confirmationStatus")
            .unwrap();
        assert_eq!(
            row.values[status_index],
            DataValue::String("confirmed".to_string())
        );
    }
}

#[test]
fn test_projection_with_filter() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT trader, currency, quantity, price FROM trades WHERE price > 500",
        )
        .unwrap();

    // Check we only have 4 columns
    assert_eq!(view.column_count(), 4);

    // Check all prices are > 500
    let price_index = 3; // price is 4th column in our projection
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        if let DataValue::Float(price) = &row.values[price_index] {
            assert!(*price > 500.0);
        }
    }
}

#[test]
fn test_filter_counterparty_type() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE counterpartyType = 'BANK'",
        )
        .unwrap();

    // Check all have BANK type
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let type_index = view.source().get_column_index("counterpartyType").unwrap();
        assert_eq!(
            row.values[type_index],
            DataValue::String("BANK".to_string())
        );
    }
}

// LINQ-style method tests

#[test]
fn test_linq_contains_method() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE trader.Contains('John')",
        )
        .unwrap();

    // Check all results contain 'John' in trader name
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let trader_index = view.source().get_column_index("trader").unwrap();
        if let DataValue::String(trader) = &row.values[trader_index] {
            assert!(
                trader.contains("John"),
                "Trader {} should contain 'John'",
                trader
            );
        }
    }
}

#[test]
fn test_linq_startswith_method() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE book.StartsWith('EQUITY')",
        )
        .unwrap();

    // Check all results start with 'EQUITY'
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let book_index = view.source().get_column_index("book").unwrap();
        if let DataValue::String(book) = &row.values[book_index] {
            assert!(
                book.starts_with("EQUITY"),
                "Book {} should start with 'EQUITY'",
                book
            );
        }
    }
}

#[test]
fn test_linq_endswith_method() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE instrumentName.EndsWith('Option')",
        )
        .unwrap();

    // Check all results end with 'Option'
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let inst_index = view.source().get_column_index("instrumentName").unwrap();
        if let DataValue::String(inst) = &row.values[inst_index] {
            assert!(
                inst.ends_with("Option"),
                "Instrument {} should end with 'Option'",
                inst
            );
        }
    }
}

#[test]
fn test_linq_method_with_and() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM trades WHERE trader.Contains('John') AND currency = 'USD'",
        )
        .unwrap();

    // Check all results have John in trader name AND USD currency
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();

        let trader_index = view.source().get_column_index("trader").unwrap();
        if let DataValue::String(trader) = &row.values[trader_index] {
            assert!(trader.contains("John"));
        }

        let currency_index = view.source().get_column_index("currency").unwrap();
        assert_eq!(
            row.values[currency_index],
            DataValue::String("USD".to_string())
        );
    }
}

#[test]
fn test_linq_method_complex() {
    let table = load_trades_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM trades WHERE counterparty.StartsWith('GOLD') OR counterparty.Contains('MORGAN')")
        .unwrap();

    // Check all results have counterparty starting with GOLD or containing MORGAN
    for i in 0..view.row_count() {
        let row = view.get_row(i).unwrap();
        let cp_index = view.source().get_column_index("counterparty").unwrap();
        if let DataValue::String(cp) = &row.values[cp_index] {
            assert!(
                cp.starts_with("GOLD") || cp.contains("MORGAN"),
                "Counterparty {} should start with GOLD or contain MORGAN",
                cp
            );
        }
    }
}

// Add more test cases as needed...
