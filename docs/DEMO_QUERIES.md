# SQL CLI Demo Queries

## Setup

1. Start the API:
```bash
cd TradeApi
dotnet run --urls "http://localhost:5000"
```

2. In another terminal, start the CLI:
```bash
cd sql-cli
TRADE_API_URL=http://localhost:5000 cargo run
```

## Example Queries

### Basic SELECT
```sql
SELECT * FROM trade_deal
SELECT dealId, platformOrderId, price, quantity FROM trade_deal
```

### WHERE Clauses with Comparisons
```sql
SELECT * FROM trade_deal WHERE price > 200
SELECT * FROM trade_deal WHERE quantity < 1000
SELECT * FROM trade_deal WHERE notional > 100000
```

### String Operations (Dynamic LINQ)
```sql
SELECT * FROM trade_deal WHERE ticker = 'AAPL'
SELECT * FROM trade_deal WHERE counterparty.Contains('Goldman')
SELECT * FROM trade_deal WHERE platformOrderId.StartsWith('PO2000')
SELECT * FROM trade_deal WHERE trader.Contains('John')
```

### Date Filtering (Note: Requires date parsing enhancement)
```sql
SELECT * FROM trade_deal WHERE status = 'Executed'
SELECT * FROM trade_deal WHERE side = 'Buy'
```

### ORDER BY
```sql
SELECT * FROM trade_deal ORDER BY price DESC
SELECT * FROM trade_deal ORDER BY quantity ASC
SELECT * FROM trade_deal WHERE price > 100 ORDER BY notional DESC
```

### Complex Queries
```sql
SELECT dealId, ticker, price, quantity, notional FROM trade_deal WHERE ticker = 'MSFT' ORDER BY price DESC
SELECT * FROM trade_deal WHERE counterparty.Contains('Morgan') AND price > 150
```

## Features Demonstrated

1. **Context-Aware Completion**: Press Tab at any point for suggestions
2. **History**: Use Ctrl+P/Ctrl+N to navigate previous queries
3. **Export**: After running a query, use `\export results.csv` to save
4. **Table Display**: Results shown in formatted ASCII table

## API Endpoints

- `GET http://localhost:5000/api/trade/schema/trade_deal` - Get column definitions
- `GET http://localhost:5000/api/trade/sample` - Get 5 sample records
- `POST http://localhost:5000/api/trade/query` - Execute query

### Query API Example:
```json
{
  "select": ["dealId", "price", "quantity"],
  "where": "price > 100",
  "orderBy": "price DESC",
  "take": 10
}
```