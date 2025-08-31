# IN Clause vs Contains Method

## Understanding the Difference

### Contains Method
The `.Contains()` method performs a substring search:
```sql
-- This will match any counterparty that has "US" anywhere in the string
SELECT * FROM trade_deal WHERE counterparty.Contains("US")
```
Matches:
- "US Bank"
- "Bank of US"  
- "HSBC US Division"
- "US"

### IN Clause
The `IN` clause performs exact matching:
```sql
-- This will only match if counterparty equals exactly "US"
SELECT * FROM trade_deal WHERE counterparty IN ("US")
```
Matches:
- "US" (exact match only)

Does NOT match:
- "US Bank"
- "Bank of US"
- "HSBC US Division"

## Examples

### To find exact matches from a list:
```sql
-- Find trades with specific counterparties
SELECT * FROM trade_deal WHERE counterparty IN ("US Bank", "JP Morgan", "HSBC")
```

### To find partial matches:
```sql
-- Find any counterparty containing "Bank"
SELECT * FROM trade_deal WHERE counterparty.Contains("Bank")
```

### Combining both:
```sql
-- Find specific banks OR any counterparty containing "US"
SELECT * FROM trade_deal WHERE counterparty IN ("JP Morgan", "HSBC") OR counterparty.Contains("US")
```

## Debugging Tips

1. First check what values exist in your data:
```sql
SELECT DISTINCT counterparty FROM trade_deal
```

2. If IN returns 0 rows, the exact values don't exist. Try Contains instead:
```sql
-- Instead of:
SELECT * FROM trade_deal WHERE counterparty IN ("US")

-- Try:
SELECT * FROM trade_deal WHERE counterparty.Contains("US")
```

3. For multiple partial matches, use multiple Contains with OR:
```sql
SELECT * FROM trade_deal WHERE counterparty.Contains("US") OR counterparty.Contains("UK")
```