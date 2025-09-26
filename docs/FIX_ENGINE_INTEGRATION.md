# FIX Engine Integration for SQL CLI

## Overview
Integrate FIX (Financial Information eXchange) protocol message parsing with SQL CLI by adding a query interface to the existing C# FIX engine server. This will enable SQL queries directly against FIX message logs for trade analysis and reconciliation.

## Current Architecture
- **FIX Engine**: C# server that can parse entire FIX message files
- **React GUI**: Sophisticated interface for detailed message inspection
- **SQL CLI**: Already supports Web CTEs for fetching external data
- **Flask Proxy**: Handles NTLM authentication for database connections

## Proposed FIX Query Interface

### Goal
Add a lightweight query endpoint to the FIX engine server that:
1. Accepts SELECT-like queries for FIX tags
2. Filters messages based on criteria
3. Returns normalized columnar data (JSON/CSV)
4. Integrates seamlessly with SQL CLI's Web CTE

### Data Model
```
Map Structure: (tag, occurrence) => value
- Handles multiple occurrences of the same tag
- Flattens nested FIX structure for SQL analysis
- Normalizes column set across all messages
```

### Query Grammar (Proposed)

```sql
-- Basic FIX query syntax
SELECT
    Tag35,           -- MsgType
    Tag11,           -- ClOrdID
    Tag55,           -- Symbol
    Tag54,           -- Side (1=Buy, 2=Sell)
    Tag38,           -- OrderQty
    Tag32,           -- LastQty
    Tag31,           -- LastPx
    Tag6,            -- AvgPx
    Tag14,           -- CumQty
    Tag151,          -- LeavesQty
    Tag39,           -- OrdStatus
    Tag150,          -- ExecType
    Tag17,           -- ExecID
    Tag37,           -- OrderID
    Tag1,            -- Account
    Tag75,           -- TradeDate
    Tag60,           -- TransactTime
    Tag79,           -- AllocAccount (for allocations)
    Tag80,           -- AllocQty
    Tag661           -- AllocAcctIDSource
FROM fix_messages
WHERE Tag35 IN ('8', 'D', 'J')  -- Execution Report, New Order, Allocation
  AND Tag150 = 'F'               -- Trade executions only
  AND TransactTime >= '20240101'
  AND Symbol MATCHES 'EUR.*'     -- Regex support
```

### FIX Engine Extensions

#### 1. Query Endpoint
```http
POST /fix/query
Content-Type: application/json

{
  "select": ["Tag35", "Tag11", "Tag55", "Tag38", "Tag32", "Tag31"],
  "where": {
    "Tag35": ["8", "D"],
    "Tag150": "F",
    "regex": {
      "Tag55": "EUR.*"
    },
    "dateRange": {
      "Tag75": {
        "from": "20240101",
        "to": "20240131"
      }
    }
  },
  "limit": 10000
}
```

#### 2. Response Format
```json
{
  "columns": ["MsgType", "ClOrdID", "Symbol", "OrderQty", "LastQty", "LastPx"],
  "rows": [
    ["8", "ORD123", "EURUSD", "1000000", "1000000", "1.0925"],
    ["8", "ORD124", "EURGBP", "500000", "500000", "0.8435"],
    ["D", "ORD125", "EURJPY", "2000000", null, null]
  ]
}
```

### SQL CLI Integration

#### Web CTE Usage
```sql
WITH fix_executions AS WEB (
    URL 'http://fix-engine:8080/fix/query',
    METHOD 'POST',
    HEADERS '{"Content-Type": "application/json"}',
    BODY '{
        "select": ["Tag11", "Tag55", "Tag54", "Tag32", "Tag31", "Tag6"],
        "where": {"Tag35": "8", "Tag150": "F"}
    }'
),
trades_booked AS (
    SELECT * FROM trades_database
    WHERE trade_date = TODAY()
),
reconciliation AS (
    SELECT
        f.Tag11 as fix_order_id,
        f.Tag55 as symbol,
        CASE f.Tag54
            WHEN '1' THEN 'Buy'
            WHEN '2' THEN 'Sell'
        END as side,
        f.Tag32 as executed_qty,
        f.Tag31 as last_px,
        f.Tag6 as avg_px,
        t.order_id as booked_order_id,
        t.quantity as booked_qty,
        t.price as booked_px
    FROM fix_executions f
    LEFT JOIN trades_booked t ON f.Tag11 = t.order_id
)
SELECT
    symbol,
    side,
    COUNT(*) as execution_count,
    SUM(executed_qty) as total_executed,
    SUM(booked_qty) as total_booked,
    SUM(executed_qty) - SUM(booked_qty) as qty_diff,
    AVG(avg_px) as avg_execution_px,
    AVG(booked_px) as avg_booked_px
FROM reconciliation
GROUP BY symbol, side
HAVING qty_diff != 0;
```

## Key FIX Tags for Trading

### Execution Reports (Tag35=8)
- **Tag11**: ClOrdID (Client Order ID)
- **Tag37**: OrderID (Exchange Order ID)
- **Tag17**: ExecID (Execution ID)
- **Tag55**: Symbol
- **Tag54**: Side (1=Buy, 2=Sell)
- **Tag38**: OrderQty (Original order quantity)
- **Tag32**: LastQty (Executed quantity this fill)
- **Tag31**: LastPx (Execution price)
- **Tag14**: CumQty (Cumulative executed quantity)
- **Tag151**: LeavesQty (Remaining quantity)
- **Tag6**: AvgPx (Average price of all fills)
- **Tag39**: OrdStatus (Order status)
- **Tag150**: ExecType (Execution type, F=Trade)
- **Tag1**: Account
- **Tag75**: TradeDate
- **Tag60**: TransactTime

### Allocation Messages (Tag35=J)
- **Tag79**: AllocAccount
- **Tag80**: AllocQty
- **Tag661**: AllocAcctIDSource
- **Tag736**: AllocPrice

## Implementation Plan

### Phase 1: FIX Engine Query Interface
1. Add HTTP endpoint to C# FIX engine
2. Implement tag selection and filtering
3. Create response normalizer for columnar output
4. Add regex support for flexible filtering

### Phase 2: Column Normalization
1. Define standard column superset
2. Handle NULL values for missing tags
3. Implement tag-to-column name mapping
4. Support repeating groups (flatten or array)

### Phase 3: Performance Optimization
1. Index FIX messages by key tags (35, 150, 75)
2. Cache parsed messages in memory
3. Implement query result pagination
4. Add compression for large result sets

### Phase 4: SQL CLI Enhancement
1. Add FIX-specific functions:
   - `FIX_SIDE(tag54)` - Convert side codes to names
   - `FIX_STATUS(tag39)` - Convert status codes
   - `FIX_MSGTYPE(tag35)` - Convert message types
2. Create example queries for common use cases
3. Add FIX tag documentation to help system

## Use Cases

### 1. Trade Reconciliation
Compare FIX executions with booked trades to find discrepancies.

### 2. Volume Analysis
```sql
SELECT
    symbol,
    DATE(transact_time) as trade_date,
    SUM(last_qty) as daily_volume,
    COUNT(*) as trade_count,
    AVG(last_px) as vwap
FROM fix_executions
WHERE msg_type = '8' AND exec_type = 'F'
GROUP BY symbol, DATE(transact_time)
ORDER BY trade_date, daily_volume DESC;
```

### 3. Allocation Tracking
```sql
WITH allocations AS (
    SELECT * FROM fix_messages
    WHERE msg_type = 'J'
)
SELECT
    alloc_account,
    symbol,
    SUM(alloc_qty) as total_allocated,
    COUNT(*) as allocation_count
FROM allocations
GROUP BY alloc_account, symbol;
```

### 4. Order Fill Analysis
```sql
SELECT
    cl_ord_id,
    order_qty,
    cum_qty,
    leaves_qty,
    ROUND(100.0 * cum_qty / order_qty, 2) as fill_pct,
    avg_px
FROM fix_executions
WHERE msg_type = '8'
  AND leaves_qty > 0
ORDER BY fill_pct DESC;
```

## Benefits

1. **Direct SQL queries on FIX logs** - No intermediate parsing required
2. **Powerful aggregations** - GROUP BY, window functions on trade data
3. **Cross-system reconciliation** - Join FIX data with database records
4. **Rapid analysis** - Interactive exploration without coding
5. **Audit trail** - Query historical FIX messages for compliance

## Technical Considerations

1. **Message Volume**: FIX logs can be massive - need efficient filtering
2. **Tag Normalization**: Same semantic data may use different tags across venues
3. **Binary Fields**: Handle binary data in FIX messages appropriately
4. **Performance**: Cache frequently accessed messages in memory
5. **Security**: Ensure proper authentication for FIX data access

## Next Steps

1. Define minimal query grammar for Phase 1
2. Implement prototype endpoint in FIX engine
3. Test with SQL CLI Web CTE integration
4. Iterate based on real-world query patterns
5. Add specialized FIX functions to SQL CLI