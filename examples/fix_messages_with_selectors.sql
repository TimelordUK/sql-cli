-- #! data/fix_messages.json
-- [TEST:SKIP] needs the message-query server on localhost:5050
-- Example: Querying FIX messages with custom selectors
-- This shows how to specify the actual selector configuration in the SQL query

-- Query 1: Using the pre-configured example endpoint
-- The service has built-in selectors for common FIX message fields
-- Selectors are defined in the service at: FixQueryExamples.CreateExecutionAndAllocationQuery()
WITH WEB fix_trades AS (
    URL 'http://localhost:5050/api/messagequery/example/fix'
    METHOD POST
    FORM_FILE 'file' 'data/fix_messages.json'
    FORMAT CSV
)
SELECT
    msg_type,
    symbol,
    side,
    quantity,
    price,
    trader,
    fund_allocations,
    total_allocated
FROM fix_trades
WHERE quantity IS NOT NULL
ORDER BY symbol;
GO

-- Query 2: Custom selectors with message-type-specific extraction
-- This demonstrates how to pass selector configuration via FORM_FIELD
-- Different message types (8=Execution, J=Allocation) have different field mappings:
-- - MsgType=8: LastQty → quantity, LastPx → price, Parties[PartyRole=11] → trader
-- - MsgType=J: Quantity → quantity, AvgPx → price, null → trader
WITH
    WEB fix_custom AS (
        URL 'http://localhost:5050/api/messagequery/upload' METHOD POST FORMAT CSV
        FORM_FILE 'file' 'data/fix_messages.json'
        FORM_FIELD 'query' '{
          "MessageTypeField": "header.MsgType",
          "MessageTypes": {
            "8": {
              "Select": {
                "msg_type": "header.MsgType",
                "price": "body.LastPx",
                "quantity": "body.LastQty",
                "side": "body.Side",
                "symbol": "body.Symbol",
                "trader": "Parties[PartyRole=11].PartyID"
              },
              "Where": {
                "body.ExecType": "F"
              }
            },
            "J": {
              "Select": {
                "msg_type": "header.MsgType",
                "price": "body.AvgPx",
                "quantity": "body.Quantity",
                "side": "body.Side",
                "symbol": "body.Symbol",
                "trader": "null"
              }
            }
          },
          "OutputColumns": [
            "msg_type",
            "symbol",
            "side",
            "quantity",
            "price",
            "trader"
          ],
          "OutputFormat": "csv"
        }'

    )
SELECT
    msg_type,
    CASE
        WHEN msg_type = '8' THEN 'Execution'
        WHEN msg_type = 'J' THEN 'Allocation'
    END AS message_type,
    symbol,
    quantity,
    price,
    trader
FROM fix_custom
WHERE quantity > 500
ORDER BY price DESC;
GO

-- Query 3: Simplified custom selector
-- Shows a minimal configuration for extracting just a few fields
WITH
    WEB fix_simple AS (
        URL 'http://localhost:5050/api/messagequery/upload' METHOD POST FORMAT CSV
        FORM_FILE 'file' 'data/fix_messages.json'
        FORM_FIELD 'query' '{
          "MessageTypeField": "header.MsgType",
          "MessageTypes": {
            "8": {
              "Select": {
                "qty": "body.LastQty",
                "symbol": "body.Symbol",
                "type": "header.MsgType"
              }
            },
            "J": {
              "Select": {
                "qty": "body.Quantity",
                "symbol": "body.Symbol",
                "type": "header.MsgType"
              }
            }
          },
          "OutputColumns": [
            "type",
            "symbol",
            "qty"
          ],
          "OutputFormat": "csv"
        }'

    )
SELECT
    type,
    symbol,
    SUM(qty) AS total_quantity
FROM fix_simple
WHERE qty IS NOT NULL
GROUP BY type, symbol
ORDER BY symbol ASC, type ASC;
GO

-- Selector Configuration Structure:
-- {
--   "MessageTypeField": "header.MsgType",           // Field that identifies message type
--   "OutputColumns": ["col1", "col2", ...],         // Unified output schema
--   "MessageTypes": {
--     "8": {                                       // Configuration for MsgType=8
--       "Select": {
--         "col1": "body.LastQty",                  // Map column to field path
--         "col2": "Parties[PartyRole=11].PartyID"  // Array filter selector
--       },
--       "Where": {                                 // Optional filters
--         "body.ExecType": "F"                     // Only filled trades
--       }
--     },
--     "J": {                                       // Configuration for MsgType=J
--       "Select": {
--         "col1": "body.Quantity",                 // Different field for same column
--         "col2": "null"                           // Field doesn't exist in this type
--       }
--     }
--   },
--   "OutputFormat": "csv"                          // Response format
-- }

-- Selector Syntax:
-- - Dot notation:      "body.Symbol" or "header.MsgType"
-- - Array index:       "AllocGrp[0].AllocAccount"
-- - Array filter:      "Parties[PartyRole=11].PartyID"
-- - Array wildcard:    "AllocGrp[*].AllocQty"
-- - Aggregations:      "AllocGrp:count()" or "AllocGrp:sum(AllocQty)"
-- - Null value:        "null" (for non-existent fields)

-- Note: The JSON selector service must be running on port 5050
-- Start with: dotnet run --project proxy/json-selector/JsonSelector/JsonSelector.csproj
