use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub tables: Vec<TableConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub name: String,
    pub columns: Vec<String>,
}

// Load schema from a JSON file if it exists, otherwise use defaults
pub fn load_schema_config() -> SchemaConfig {
    // Check for schema.json in current directory or config directory
    let mut paths = vec![
        String::from("schema.json"),
        String::from(".sql-cli/schema.json"),
    ];

    // Add config directory path if available
    if let Some(config_dir) = dirs::config_dir() {
        if let Some(path_str) = config_dir.join("sql-cli/schema.json").to_str() {
            paths.push(String::from(path_str));
        }
    }

    for path in paths {
        if Path::new(&path).exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<SchemaConfig>(&contents) {
                    eprintln!("Loaded schema from: {}", path);
                    return config;
                }
            }
        }
    }

    // Return default schema
    SchemaConfig {
        tables: vec![
            TableConfig {
                name: String::from("trade_deal"),
                columns: get_full_trade_deal_columns(),
            },
            TableConfig {
                name: String::from("instrument"),
                columns: vec![
                    String::from("instrumentId"),
                    String::from("name"),
                    String::from("type"),
                ],
            },
        ],
    }
}

// Example configuration for full schema with 190+ columns
// This can be loaded from JSON/YAML or your REST API

pub fn get_full_trade_deal_columns() -> Vec<String> {
    vec![
        // Trade identifiers
        "dealId",
        "platformOrderId",
        "externalOrderId",
        "parentOrderId",
        // Dates
        "tradeDate",
        "settlementDate",
        "valueDate",
        "maturityDate",
        "lastModifiedDate",
        "createdDate",
        "confirmationDate",
        "executionDate",
        // Instrument details
        "instrumentId",
        "instrumentName",
        "instrumentType",
        "isin",
        "cusip",
        "sedol",
        "ticker",
        "exchange",
        // Quantities and prices
        "quantity",
        "price",
        "notional",
        "settlementAmount",
        "grossAmount",
        "netAmount",
        "accruedInterest",
        "accrual",
        "commission",
        "fees",
        "tax",
        "spread",
        "currency",
        "baseCurrency",
        "quoteCurrency",
        "settlementCurrency",
        // Counterparty info
        "counterparty",
        "counterpartyId",
        "counterpartyType",
        "counterpartyCountry",
        "counterpartyLei",
        // Internal info
        "trader",
        "traderId",
        "book",
        "bookId",
        "portfolio",
        "portfolioId",
        "strategy",
        "desk",
        "legalEntity",
        "branch",
        "region",
        "side",
        "productType",
        "instrumentClass",
        "assetClass",
        // Trading venue and clearing
        "venue",
        "executionVenue",
        "clearingHouse",
        "clearingBroker",
        "prime",
        "custodian",
        "subCustodian",
        // Status and workflow
        "status",
        "confirmationStatus",
        "settlementStatus",
        "allocationStatus",
        "clearingStatus",
        "bookingStatus",
        // Risk metrics
        "pv01",
        "dv01",
        "delta",
        "gamma",
        "vega",
        "theta",
        "duration",
        "convexity",
        "yield",
        "spread",
        // Compliance
        "regulatoryReporting",
        "mifidClassification",
        "bestExecution",
        "preTradeTransparency",
        "postTradeTransparency",
        // Comments and metadata
        "comments",
        "notes",
        "auditTrail",
        "version",
        "source",
        "sourceSystem",
        "lastUpdatedBy",
        "createdBy",
        // Additional reference fields
        "clientOrderId",
        "brokerOrderId",
        "exchangeOrderId",
        "blockTradeId",
        "allocationId",
        "confirmationId",
        // Add more columns as needed...
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// Function to save the current schema to a file (useful for generating examples)
pub fn save_schema_example(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let schema = load_schema_config();
    let json = serde_json::to_string_pretty(&schema)?;
    fs::write(path, json)?;
    Ok(())
}

// Example of LINQ-style operators that could be supported
pub enum LinqOperator {
    Contains(String),        // field.Contains('value')
    StartsWith(String),      // field.StartsWith('value')
    EndsWith(String),        // field.EndsWith('value')
    GreaterThan(String),     // field > value
    LessThan(String),        // field < value
    Between(String, String), // field >= value1 && field <= value2
    In(Vec<String>),         // field in (value1, value2, ...)
    IsNull,                  // field == null
    IsNotNull,               // field != null
}

// Date constructor support
pub enum DateConstructor {
    Today,               // DateTime.Today
    Now,                 // DateTime.Now
    Date(i32, u32, u32), // DateTime(2024, 1, 15)
    DateOffset(i32),     // DateTime.Today.AddDays(-7)
}
