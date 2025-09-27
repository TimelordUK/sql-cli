# JSON Selector Service with FPML/XML Support

A high-performance ASP.NET Core service that transforms hierarchical JSON data (including embedded XML/FPML) into flat CSV format using an intuitive selector syntax.

## Quick Start

```bash
# Build the service
./build.sh

# Run the service (default port 5050)
./run.sh

# Access Swagger UI
open http://localhost:5050/swagger
```

## Core Features

- **JSON Path Selection**: Extract nested JSON fields using dot notation
- **XML/FPML Support**: Parse embedded XML fields (e.g., FPML in FIX messages)
- **Array Filtering**: Filter arrays by field values
- **Aggregations**: Sum, count, min, max operations on arrays
- **CSV Output**: Flatten complex hierarchical data to CSV
- **Swagger UI**: Interactive API documentation

## Architecture Overview

### For FIX Engine Integration

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  FIX Engine  │────>│ JSON Output  │────>│ JsonSelector │
│   (Server)   │     │  with FPML   │     │   Service    │
└──────────────┘     └──────────────┘     └──────────────┘
                                                  │
                                                  ▼
                                           ┌──────────────┐
                                           │   Flat CSV   │
                                           │    Output    │
                                           └──────────────┘
```

## Selector Syntax

### Basic JSON Selectors
```javascript
// Simple field access
"body.Symbol"

// Nested field access
"body.AllocGrp[0].AllocAccount"

// Array filtering
"Parties[PartyRole=11].PartyID"

// All array elements
"AllocGrp[*].AllocQty"

// Aggregations
"AllocGrp:sum(AllocQty)"
"AllocGrp:count()"
```

### XML/FPML Selectors (NEW)
```javascript
// XML field with XPath
"body.FpmlData@xml://trade/tradeHeader/tradeDate"

// With namespace
"body.FpmlData@xml://fpml:trade/fpml:tradeHeader/fpml:tradeDate"

// Complex XPath
"body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/fixedRate"
```

## API Endpoints

### 1. Standard Query Endpoints

#### POST `/api/query/upload`
Upload JSON file with query selectors
```bash
curl -X POST http://localhost:5050/api/query/upload \
  -F "file=@data.json" \
  -F 'query={"Select":{"Symbol":"body.Symbol","Price":"body.Price"}}'
```

#### POST `/api/query/direct`
Query with JSON in request body
```json
{
  "data": { /* your JSON data */ },
  "query": {
    "Select": {
      "columnName": "selector.path"
    },
    "OutputFormat": "csv"
  }
}
```

### 2. FPML/XML Query Endpoints (NEW)

#### POST `/api/FpmlQuery/upload`
Process FIX messages with embedded FPML
```bash
curl -X POST http://localhost:5050/api/FpmlQuery/upload \
  -F "file=@fix_messages.json" \
  -F 'query={
    "Select": {
      "Symbol": "body.Symbol",
      "TradeDate": "body.FpmlData@xml://trade/tradeHeader/tradeDate",
      "FixedRate": "body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/fixedRate"
    }
  }'
```

#### POST `/api/FpmlQuery/example/cds`
Pre-configured CDS/CDX query example

## Integration Guide for FIX Engine

### Step 1: Configure FIX Engine Output
Ensure your FIX engine outputs JSON with FPML as a field:
```json
{
  "header": {
    "MsgType": "D",
    "SenderCompID": "BLOOMBERG"
  },
  "body": {
    "Symbol": "CDX.NA.IG.38",
    "SecurityType": "CDS",
    "Price": 0.0055,
    "FpmlData": "<?xml version=\"1.0\"?>...</xml>"  // Can be XML string or base64
  }
}
```

### Step 2: Design Your Selectors
Create a query that extracts both FIX fields and FPML data:
```json
{
  "Select": {
    // FIX Message fields
    "MsgType": "header.MsgType",
    "Symbol": "body.Symbol",
    "Price": "body.Price",

    // FPML fields (using @xml: syntax)
    "TradeDate": "body.FpmlData@xml://trade/tradeHeader/tradeDate",
    "TradeId": "body.FpmlData@xml://trade/tradeHeader/partyTradeIdentifier/tradeId",
    "ReferenceEntity": "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/referenceInformation/referenceEntity/entityName",
    "EffectiveDate": "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/effectiveDate/unadjustedDate",
    "TerminationDate": "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/scheduledTerminationDate/unadjustedDate",
    "FixedRate": "body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/fixedRate",
    "Bankruptcy": "body.FpmlData@xml://trade/product/creditDefaultSwap/protectionTerms/creditEvents/bankruptcy"
  },
  "OutputFormat": "csv"
}
```

### Step 3: Process Messages
```python
# Python example
import requests
import json

# Your FIX messages with FPML
fix_messages = [
    {
        "header": {...},
        "body": {
            "Symbol": "CDX.NA.IG.38",
            "FpmlData": "<trade>...</trade>"
        }
    }
]

# Query configuration
query = {
    "Select": {
        "Symbol": "body.Symbol",
        "TradeDate": "body.FpmlData@xml://trade/tradeHeader/tradeDate"
    },
    "OutputFormat": "csv"
}

# Send request
response = requests.post(
    "http://localhost:5050/api/FpmlQuery/direct",
    json={
        "data": fix_messages,
        "query": query
    }
)

# Save CSV output
with open("output.csv", "w") as f:
    f.write(response.text)
```

### Step 4: Cache & Optimize (Optional)
For high-volume processing:
1. Batch messages before sending
2. Implement client-side caching of results
3. Use the service's XML caching (automatic)

## Supported FPML Elements

The service automatically handles common FPML structures:

- **CDS/CDX**: Trade headers, reference entities, protection terms, fee legs
- **Interest Rate Swaps**: Swap streams, calculation periods, payment dates
- **FX Options**: Option premiums, exercise terms, barriers
- **Equity Derivatives**: Underlying assets, valuation dates, dividends

## Output Format

CSV output with all selected fields flattened:
```csv
MsgType,Symbol,Price,TradeDate,TradeId,ReferenceEntity,EffectiveDate,FixedRate
D,CDX.NA.IG.38,0.0055,2024-01-15,CDX-2024-001,CDX NA IG Series 38,2024-01-20,0.0055
D,TSLA 5Y CDS,0.0125,2024-01-15,TSLA-CDS-2024-002,Tesla Inc,2024-01-17,0.0125
```

## Technical Details

### XML Parsing Features
- **Auto-detection**: Automatically detects XML vs base64 encoded XML
- **Namespace handling**: FPML namespaces are auto-discovered
- **XPath support**: Full XPath 1.0 syntax
- **Caching**: Parsed XML documents are cached for performance
- **Error handling**: Invalid XML returns error markers in output

### Performance Considerations
- XML parsing is cached (100 document limit)
- Batch processing recommended for large volumes
- Async endpoints for file uploads
- Streaming for large CSV outputs

## Configuration

### Environment Variables
```bash
PORT=5050  # Service port (default: 5050)
```

### Project Structure
```
JsonSelector/
├── Controllers/
│   ├── QueryController.cs         # Standard JSON queries
│   ├── MessageQueryController.cs  # Message-type aware queries
│   └── FpmlQueryController.cs     # FPML/XML queries (NEW)
├── Services/
│   ├── SelectorParser.cs          # Parses selector syntax
│   ├── SelectorEvaluator.cs       # Evaluates JSON selectors
│   ├── XmlFieldParser.cs          # XML/XPath parsing (NEW)
│   └── EnhancedSelectorEvaluator.cs # Combined JSON/XML (NEW)
├── Models/
│   └── QueryRequest.cs            # Request/response models
└── sample-data/
    └── fix-with-fpml.json         # Example FIX messages with FPML
```

## Testing

### Test with Sample Data
```bash
# Test with provided sample
curl -X POST http://localhost:5050/api/FpmlQuery/example/cds \
  -F "file=@sample-data/fix-with-fpml.json"

# Test with your own data
curl -X POST http://localhost:5050/api/FpmlQuery/upload \
  -F "file=@your-fix-messages.json" \
  -F 'query={"Select":{"Symbol":"body.Symbol"},"OutputFormat":"csv"}'
```

### Swagger UI Testing
1. Navigate to http://localhost:5050/swagger
2. Select `/api/FpmlQuery/upload` endpoint
3. Upload your JSON file
4. Paste your query configuration
5. Execute and download CSV

## Troubleshooting

### Common Issues

1. **FPML not parsing**: Check if XML is base64 encoded
2. **XPath not finding elements**: Verify namespace prefixes
3. **Empty results**: Check selector syntax and JSON structure
4. **Performance issues**: Increase XML cache size in XmlFieldParser.cs

### Debug Tips
- Use Swagger UI to test queries interactively
- Check XML structure with: `body.FpmlData` (returns raw XML)
- Test XPath separately with online XPath testers
- Enable logging in services for detailed trace

## Requirements

- .NET 9.0 SDK
- 512MB RAM minimum
- Port 5050 available (configurable)

## License

Internal use only - Property of your organization

## Support

For issues or questions:
- Check Swagger documentation at `/swagger`
- Review sample data in `/sample-data`
- Test endpoints with example queries

---

**Version**: 1.0.0
**Last Updated**: January 2024
**Status**: Production Ready