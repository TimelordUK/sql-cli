# DateTime Support in SQL CLI

## Overview
The SQL CLI supports DateTime constructors for date and time comparisons in WHERE clauses.

## DateTime Constructor Formats

### 1. DateTime() - Today at Midnight
```sql
SELECT * FROM orders WHERE created_date > DateTime()
```
Returns all orders created after today at 00:00:00

### 2. DateTime(year, month, day) - Specific Date at Midnight
```sql
SELECT * FROM orders WHERE created_date > DateTime(2025, 1, 1)
```
Returns all orders created after January 1, 2025 at 00:00:00

### 3. DateTime(year, month, day, hour) - Specific Date and Hour
```sql
SELECT * FROM orders WHERE created_date > DateTime(2025, 1, 1, 9)
```
Returns all orders created after January 1, 2025 at 09:00:00

### 4. DateTime(year, month, day, hour, minute) - Specific Date and Time
```sql
SELECT * FROM orders WHERE created_date > DateTime(2025, 1, 1, 9, 30)
```
Returns all orders created after January 1, 2025 at 09:30:00

### 5. DateTime(year, month, day, hour, minute, second) - Full Precision
```sql
SELECT * FROM orders WHERE created_date > DateTime(2025, 1, 1, 9, 30, 45)
```
Returns all orders created after January 1, 2025 at 09:30:45

## Use Cases

### Time Range for Today
```sql
-- All records from today
SELECT * FROM logs WHERE timestamp >= DateTime() AND timestamp < DateTime(2025, 8, 6)

-- Business hours today (9 AM to 5 PM)
SELECT * FROM transactions 
WHERE created_at >= DateTime() 
  AND created_at >= DateTime(2025, 8, 5, 9) 
  AND created_at < DateTime(2025, 8, 5, 17)
```

### Specific Time Ranges
```sql
-- Last 30 days (approximate)
SELECT * FROM events WHERE event_date > DateTime(2025, 7, 5)

-- Specific day's transactions
SELECT * FROM trades 
WHERE trade_date >= DateTime(2025, 7, 15) 
  AND trade_date < DateTime(2025, 7, 16)
```

## Notes
- All DateTime values are interpreted in the local timezone
- When time components are omitted, they default to 0 (midnight)
- DateTime() without arguments always returns today's date at midnight
- The parser recognizes DateTime as a keyword and provides proper syntax highlighting