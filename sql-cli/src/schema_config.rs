// Example configuration for full schema with 190+ columns
// This can be loaded from JSON/YAML or your REST API

pub fn get_full_trade_deal_columns() -> Vec<String> {
    vec![
        // Trade identifiers
        "dealId", "platformOrderId", "externalOrderId", "parentOrderId",
        
        // Dates
        "tradeDate", "settlementDate", "valueDate", "maturityDate",
        "lastModifiedDate", "createdDate", "confirmationDate",
        
        // Instrument details
        "instrumentId", "instrumentName", "instrumentType", "isin",
        "cusip", "sedol", "ticker", "exchange",
        
        // Quantities and prices
        "quantity", "price", "notional", "settlementAmount",
        "accruedInterest", "commission", "fees", "tax",
        
        // Counterparty info
        "counterparty", "counterpartyId", "counterpartyType",
        "counterpartyCountry", "counterpartyLei",
        
        // Internal info
        "trader", "traderId", "book", "bookId", "portfolio",
        "strategy", "desk", "legalEntity", "branch",
        
        // Status and workflow
        "status", "confirmationStatus", "settlementStatus",
        "allocationStatus", "clearingStatus",
        
        // Risk metrics
        "pv01", "dv01", "delta", "gamma", "vega", "theta",
        "duration", "convexity", "yield", "spread",
        
        // Compliance
        "regulatoryReporting", "mifidClassification", "bestExecution",
        "preTradeTransparency", "postTradeTransparency",
        
        // Add more columns as needed...
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

// Example of LINQ-style operators that could be supported
pub enum LinqOperator {
    Contains(String),           // field.Contains('value')
    StartsWith(String),         // field.StartsWith('value')
    EndsWith(String),           // field.EndsWith('value')
    GreaterThan(String),        // field > value
    LessThan(String),           // field < value
    Between(String, String),    // field >= value1 && field <= value2
    In(Vec<String>),           // field in (value1, value2, ...)
    IsNull,                    // field == null
    IsNotNull,                 // field != null
}

// Date constructor support
pub enum DateConstructor {
    Today,                     // DateTime.Today
    Now,                       // DateTime.Now
    Date(i32, u32, u32),      // DateTime(2024, 1, 15)
    DateOffset(i32),          // DateTime.Today.AddDays(-7)
}