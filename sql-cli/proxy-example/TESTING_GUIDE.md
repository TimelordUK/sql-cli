# SQL-CLI to KDB+ Testing Guide

## Quick Start

### 1. Test the C# Translator (No kdb+ needed)

```bash
# Option A: Using the build script
./build_and_run.sh

# Option B: Manual with dotnet
dotnet run --project Program.cs

# Option C: Manual with mono/csc
csc Program.cs -out:SqlToQDemo.exe
mono SqlToQDemo.exe
```

This will show you the exact translation from your SQL query to q:

**Input (from sql-cli):**
```sql
SELECT * FROM trades 
WHERE confirmationStatus.StartsWith('pend') 
  AND commission BETWEEN 30 AND 100 
  AND createdDate > DateTime(2025,07,10)
```

**Output (q query):**
```q
select from trades where 
    (lower[confirmationStatus] like lower["pend*"]),
    (commission within 30 100),
    (createdDate>2025.07.10)
```

### 2. Test in KDB+ (Once you have q installed)

```bash
# Start q
q

# Load the sample data
\l create_sample_trades.q

# The script will automatically run test queries and show results
```

## Key Translation Examples

### Case-Insensitive Equality
```sql
SQL: WHERE confirmationStatus = 'pending'
q:   where (lower[confirmationStatus]=lower[`pending])
```

### StartsWith (Case-Insensitive)
```sql
SQL: WHERE confirmationStatus.StartsWith('pend')
q:   where (lower[confirmationStatus] like lower["pend*"])
```

### BETWEEN
```sql
SQL: WHERE commission BETWEEN 30 AND 100
q:   where (commission within 30 100)
```

### Date Comparison
```sql
SQL: WHERE createdDate > DateTime(2025,07,10)
q:   where (createdDate>2025.07.10)
```

### IN Clause (Case-Insensitive)
```sql
SQL: WHERE status IN ('Active', 'Pending', 'Confirmed')
q:   where (lower[status] in (lower[`Active];lower[`Pending];lower[`Confirmed]))
```

### NOT EQUAL
```sql
SQL: WHERE trader != 'John Smith'
q:   where (not trader=`$"John Smith")
```

## Loading Your Real trades.json

If you have your actual trades.json file:

```q
/ In q session
\l json.k  / Load JSON library
tradesJSON:.j.k raze read0 hsym `$"../data/trades.json"

/ Convert to table (adjust types as needed)
trades:flip `confirmationStatus`commission`createdDate`status!
    (`$tradesJSON[;`confirmationStatus];
     `float$tradesJSON[;`commission];
     "D"$tradesJSON[;`createdDate];
     `$tradesJSON[;`status])

/ Test the queries
select from trades where lower[confirmationStatus]=lower[`pending]
```

## Testing the Full Proxy Flow

The complete flow would be:

1. **sql-cli** sends to proxy:
   ```json
   {
     "sqlQuery": "SELECT * FROM trades WHERE confirmationStatus = 'pending'",
     "astTree": { /* AST */ },
     "tokens": ["SELECT", "STAR", "FROM", ...],
     "caseInsensitive": true
   }
   ```

2. **Proxy** translates and sends to kdb+:
   ```q
   select from trades where (lower[confirmationStatus]=lower[`pending])
   ```

3. **kdb+** returns results as q table

4. **Proxy** converts to JSON and returns to sql-cli

## Expected Results

With the sample data:
- `confirmationStatus = 'pending'` (case-insensitive) should find records with:
  - 'Pending' (4 records)
  - 'pending' (3 records)  
  - 'PENDING' (1 record)
  - Total: 8 records

- Complex query should find ~2-3 records depending on exact data

## Performance Notes

In kdb+, case-insensitive operations using `lower[]` are slightly slower than exact matches, but still very fast due to columnar storage. For large datasets, consider:

1. Pre-computing lowercase columns for frequently searched fields
2. Using symbols (`) for categorical data
3. Indexing commonly queried columns

## Troubleshooting

1. **Date format issues**: kdb+ uses `yyyy.mm.dd`, SQL uses `yyyy-mm-dd`
2. **String vs Symbol**: kdb+ distinguishes between strings ("") and symbols (`)
3. **Case sensitivity**: Default kdb+ is case-sensitive, hence the `lower[]` functions
4. **NULL handling**: kdb+ uses different null values per type