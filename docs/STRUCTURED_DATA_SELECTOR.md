# Structured Data Selector Design

## Problem Statement
Transform structured/hierarchical objects (like parsed FIX messages) into flat tabular data for SQL analysis. Need a simple yet powerful selector syntax that can:
1. Navigate nested objects
2. Select from arrays conditionally
3. Handle missing fields gracefully (NULL)
4. Be efficient enough for processing large files

## Example Structure
```json
{
  "header": {
    "MsgType": "8",
    "SendingTime": "20240115-09:30:00",
    "SenderCompID": "BROKER1"
  },
  "body": {
    "ClOrdID": "ORD123",
    "Symbol": "EURUSD",
    "Side": "1",
    "OrderQty": 1000000,
    "Price": 1.0925,
    "OrdStatus": "2",
    "ExecType": "F",
    "LastQty": 500000,
    "LastPx": 1.0926
  },
  "Parties": [
    {
      "PartyID": "TRADER1",
      "PartyIDSource": "D",
      "PartyRole": 11
    },
    {
      "PartyID": "ACCOUNT1",
      "PartyIDSource": "1",
      "PartyRole": 1
    }
  ],
  "AllocGrp": [
    {
      "AllocAccount": "FUND1",
      "AllocQty": 300000
    },
    {
      "AllocAccount": "FUND2",
      "AllocQty": 200000
    }
  ]
}
```

## Proposed Selector Syntaxes

### Option 1: Dot-Path with Array Filters
```
Syntax: path.to.field[filter].property

Examples:
header.MsgType                           -> "8"
body.Symbol                               -> "EURUSD"
Parties[PartyRole=11].PartyID           -> "TRADER1"
Parties[PartyRole=1].PartyID            -> "ACCOUNT1"
AllocGrp[0].AllocAccount                 -> "FUND1"
AllocGrp[].AllocQty                      -> 500000 (sum all)
AllocGrp[AllocAccount="FUND1"].AllocQty -> 300000
```

### Option 2: SQL-like Nested Selectors
```
Syntax: field FROM path WHERE condition

Examples:
MsgType FROM header                      -> "8"
Symbol FROM body                          -> "EURUSD"
PartyID FROM Parties WHERE PartyRole=11  -> "TRADER1"
AllocQty FROM AllocGrp WHERE AllocAccount="FUND1" -> 300000
SUM(AllocQty) FROM AllocGrp              -> 500000
```

### Option 3: Simplified JSONPath
```
Syntax: $.path.field or $.array[?filter].field

Examples:
$.header.MsgType                         -> "8"
$.body.Symbol                             -> "EURUSD"
$.Parties[?(@.PartyRole==11)].PartyID   -> "TRADER1"
$.AllocGrp[0].AllocAccount               -> "FUND1"
$.AllocGrp[*].AllocQty                   -> [300000, 200000]
```

### Option 4: Tagged Field Extraction (Most FIX-like)
```
Syntax: path:field@condition

Examples:
header:MsgType                           -> "8"
body:Symbol                               -> "EURUSD"
Parties:PartyID@PartyRole=11            -> "TRADER1"
Parties:PartyID@PartyRole=1             -> "ACCOUNT1"
AllocGrp:AllocQty@AllocAccount=FUND1    -> 300000
AllocGrp:SUM(AllocQty)                  -> 500000
```

## Proposed Solution: Hybrid Selector Language

### Basic Syntax
```
selector := path ['[' filter ']'] ['.' property] [':' aggregate]
path := identifier ('.' identifier)*
filter := expression | index | '*'
aggregate := 'sum' | 'count' | 'first' | 'last' | 'concat'
```

### Core Features

#### 1. Simple Path Navigation
```
header.MsgType        // Navigate nested objects
body.LastPx          // Direct property access
```

#### 2. Array Indexing
```
Parties[0]           // First element
Parties[-1]          // Last element
Parties[*]           // All elements (returns array)
```

#### 3. Array Filtering
```
Parties[PartyRole=11]              // Simple equality
Parties[PartyRole>10]              // Comparison
Parties[PartyIDSource="D"]         // String equality
Parties[PartyRole=11].PartyID      // Navigate after filter
```

#### 4. Aggregations
```
AllocGrp:sum(AllocQty)             // Sum all quantities
AllocGrp:count()                   // Count allocations
Parties[PartyRole=11]:first()      // First matching element
```

#### 5. Null Handling
```
body.MissingField                  // Returns NULL
Parties[PartyRole=99].PartyID     // Returns NULL if no match
```

## Query Definition Format

### For Web CTE Integration
```json
{
  "source": "fix_messages",
  "select": {
    "msg_type": "header.MsgType",
    "symbol": "body.Symbol",
    "side": "body.Side",
    "trader": "Parties[PartyRole=11].PartyID",
    "account": "Parties[PartyRole=1].PartyID",
    "exec_qty": "body.LastQty",
    "exec_px": "body.LastPx",
    "total_alloc": "AllocGrp:sum(AllocQty)",
    "alloc_count": "AllocGrp:count()"
  },
  "where": {
    "header.MsgType": ["8", "F"],
    "body.ExecType": "F",
    "body.Symbol": { "regex": "EUR.*" }
  }
}
```

### Result Table
```
msg_type | symbol | side | trader  | account  | exec_qty | exec_px | total_alloc | alloc_count
---------|--------|------|---------|----------|----------|---------|-------------|------------
8        | EURUSD | 1    | TRADER1 | ACCOUNT1 | 500000   | 1.0926  | 500000      | 2
```

## Implementation Strategy

### Phase 1: Parser
1. Tokenize selector expressions
2. Build AST for each selector
3. Validate against schema (optional)

### Phase 2: Evaluator
```csharp
public interface ISelector
{
    object Evaluate(JObject source);
    Type ResultType { get; }
}

public class PathSelector : ISelector
{
    public string Path { get; set; }
    public IFilter Filter { get; set; }
    public string Property { get; set; }
    public AggregateFunction Aggregate { get; set; }

    public object Evaluate(JObject source)
    {
        // Navigate path
        // Apply filter
        // Extract property
        // Apply aggregate
    }
}
```

### Phase 3: Compiler (Optimization)
```csharp
// Compile selector to expression tree for performance
Expression<Func<JObject, object>> CompileSelector(string selector)
{
    var param = Expression.Parameter(typeof(JObject), "obj");
    // Build expression tree from selector
    return Expression.Lambda<Func<JObject, object>>(body, param);
}
```

## Examples

### Trade Execution Query
```
SELECT:
  header.MsgType as msg_type,
  body.ClOrdID as order_id,
  body.Symbol as symbol,
  body.Side as side,
  body.LastQty as qty,
  body.LastPx as price,
  Parties[PartyRole=11].PartyID as trader

WHERE:
  header.MsgType = "8"
  body.ExecType = "F"
```

### Allocation Summary
```
SELECT:
  body.Symbol as symbol,
  AllocGrp:count() as num_allocations,
  AllocGrp:sum(AllocQty) as total_allocated,
  AllocGrp[AllocAccount="FUND1"].AllocQty as fund1_qty

WHERE:
  header.MsgType = "J"
```

### Complex Party Selection
```
SELECT:
  Parties[PartyRole=11].PartyID as executing_trader,
  Parties[PartyRole=12].PartyID as executing_firm,
  Parties[PartyRole=1].PartyID as account,
  Parties[PartyRole=3].PartyID as client_id
```

## Comparison with Existing Solutions

### vs JQ
```bash
# JQ (complex)
.Parties[] | select(.PartyRole == 11) | .PartyID

# Our syntax (simple)
Parties[PartyRole=11].PartyID
```

### vs JSONPath
```
# JSONPath (verbose)
$.Parties[?(@.PartyRole == 11)].PartyID

# Our syntax (cleaner)
Parties[PartyRole=11].PartyID
```

### vs XPath
```xml
# XPath (XML-specific)
//Parties[PartyRole=11]/PartyID

# Our syntax (JSON-friendly)
Parties[PartyRole=11].PartyID
```

## Benefits

1. **Simple to learn** - Intuitive dot notation with SQL-like filters
2. **Powerful enough** - Handles nested objects, arrays, filtering, aggregation
3. **Efficient** - Can be compiled to expression trees in C#
4. **Type-aware** - Can infer result types for validation
5. **Null-safe** - Gracefully handles missing fields

## Open Questions

1. **Array flattening**: How to handle nested arrays?
   - `Orders[].Fills[].Qty` - Should this flatten?

2. **Multiple matches**: What if filter returns multiple results?
   - Return first? Array? Comma-separated?

3. **Type coercion**: How to handle mixed types?
   - Strings vs numbers in comparisons

4. **Functions**: Support for string functions?
   - `UPPER(body.Symbol)`, `SUBSTRING(ClOrdID, 0, 3)`

5. **Joins**: Can we reference other objects?
   - `Parties[PartyID=@body.TraderId]`

## Next Steps

1. Prototype the parser for basic path navigation
2. Add array filtering support
3. Implement in C# with JObject/JArray
4. Test with real FIX message structures
5. Optimize with expression compilation
6. Integrate with FIX engine query endpoint