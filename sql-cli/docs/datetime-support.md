# DateTime Support in SQL CLI

## Overview

The SQL CLI now supports DateTime constructors in WHERE clauses, enabling date comparisons similar to Dynamic LINQ syntax used in the trading server.

## Features

### 1. DateTime Constructor Parsing
- Supports syntax: `DateTime(year, month, day)`
- Example: `WHERE createdDate > DateTime(2025, 10, 20)`

### 2. Context-Aware Completion
After typing a comparison operator (`>`, `<`, `>=`, `<=`, `=`, `!=`) following a datetime column, the CLI suggests:
- `DateTime(` - Constructor for specific dates
- `DateTime.Today` - Current date (server-side)
- `DateTime.Now` - Current date and time (server-side)

### 3. DateTime Column Detection
The following columns are recognized as datetime types:
- `tradeDate`
- `settlementDate`
- `createdDate`
- `modifiedDate`
- `valueDate`
- `maturityDate`
- `confirmationDate`
- `executionDate`
- `lastModifiedDate`

## Implementation Details

### Parser Enhancement
- Added `DateTime` token to the lexer
- Created `DateTimeConstructor` AST node
- Implemented parsing logic in `parse_primary`

### Context Detection
- Added `AfterComparisonOp` context for better completion suggestions
- Enhanced `analyze_statement` and `analyze_partial` functions

### Testing
Integration tests verify:
1. DateTime constructor parsing
2. Context detection after comparison operators
3. Completion suggestions for datetime columns

## Usage Examples

```sql
-- Find trades created after a specific date
SELECT * FROM trade_deal WHERE createdDate > DateTime(2025, 10, 20)

-- Find trades settled before a date
SELECT * FROM trade_deal WHERE settlementDate < DateTime(2025, 12, 31)

-- Combined with other conditions
SELECT * FROM trade_deal 
WHERE createdDate >= DateTime(2025, 1, 1) 
  AND status = 'Active'
```

## Future Enhancements
- Support for `DateTime.Today` and `DateTime.Now`
- Time component support: `DateTime(year, month, day, hour, minute, second)`
- Relative date functions: `DateTime.Today.AddDays(-7)`