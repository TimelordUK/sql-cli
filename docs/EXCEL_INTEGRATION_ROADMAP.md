# Excel Integration & Monetization Roadmap

## Vision
Transform sql-cli from a powerful CSV/JSON viewer into a professional-grade data analysis tool for financial professionals, bridging the gap between Excel workflows and modern data processing.

## Target Audience
- Quantitative analysts
- Traders
- Risk managers  
- Financial analysts
- Anyone working with large Excel datasets who needs more power

## The Problem We're Solving
Financial professionals live in Excel but constantly hit its limits:
- Excel chokes on files > 100MB
- No real-time filtering on large datasets
- Formulas recalculate slowly on big sheets
- No SQL-like query capabilities
- Poor multi-file handling

## Free Tier (Core Features)
The free version remains extremely powerful:
- CSV/JSON file viewing and analysis
- SQL queries on local data
- Advanced filtering and sorting
- Multi-file operations
- Export to CSV
- Full TUI with keyboard navigation
- Basic Excel import (data only, no formulas)

## Professional Tier ($20-30/month)
Worth every penny for financial professionals:

### 1. Advanced Excel Integration
```rust
// Import Excel with full formula support
let workbook = DataTable::from_excel("trading_book.xlsx")?;

// Preserves:
// - Multiple sheets
// - Formulas and computed columns  
// - Cell formatting information
// - Named ranges
```

### 2. Formula Engine
```rust
pub enum Formula {
    // Basic Operations
    Add(ColumnRef, ColumnRef),        // =A1+B1
    Multiply(ColumnRef, ColumnRef),   // =A1*B1
    Divide(ColumnRef, ColumnRef),     // =A1/B1
    
    // Financial Functions
    NPV(rate: f64, CashFlows),        // =NPV(0.1, A1:A10)
    IRR(CashFlows),                   // =IRR(A1:A10)
    VWAP(prices: Column, volumes: Column),
    
    // Lookups
    VLookup(value, table, col_index),
    Index(array, row, col),
    
    // Custom trader formulas
    PnL(entry: Column, exit: Column, quantity: Column),
    Sharpe(returns: Column, risk_free_rate: f64),
}
```

### 3. Real-Time Computed Columns
```sql
-- Mix SQL with Excel-like formulas
SELECT 
    Symbol,
    Quantity,
    EntryPrice,
    CurrentPrice,
    Quantity * (CurrentPrice - EntryPrice) as PnL,  -- Computed
    PnL / (EntryPrice * Quantity) as ReturnPct      -- Computed
FROM positions
WHERE PnL > 10000
```

### 4. Charting Integration
```rust
// ASCII charts in TUI
// Export to HTML with interactive charts
// Sparklines in table cells
impl DataTable {
    fn render_sparkline(&self, column: &str) -> String {
        // ▁▂▃▄▅▆▇█ for quick in-table visualization
    }
}
```

### 5. Multi-Sheet Workflows
```rust
// Work with multiple Excel sheets like database tables
SELECT 
    t.Symbol,
    t.Quantity,
    p.LastPrice,
    t.Quantity * p.LastPrice as MarketValue
FROM trades t
JOIN prices p ON t.Symbol = p.Symbol
```

### 6. Export Back to Excel
```rust
// Not just CSV export - full Excel with formulas
let output = ExcelWriter::new("analysis_results.xlsx");
output.add_sheet("Results", &data_table);
output.add_formula_column("PnL", "=D2-C2");
output.add_chart("PnL Distribution", ChartType::Histogram);
```

## Implementation Phases

### Phase 1: Basic Excel Reading (Q1 2025)
- [ ] Integrate `calamine` crate
- [ ] Read .xlsx files into DataTable
- [ ] Support multiple sheets
- [ ] Handle basic data types

### Phase 2: Formula Detection (Q2 2025)
- [ ] Detect formula columns
- [ ] Display formula in TUI
- [ ] Parse simple formulas (A1*B1)
- [ ] Evaluate basic math operations

### Phase 3: Advanced Formulas (Q3 2025)
- [ ] Financial functions (NPV, IRR, etc.)
- [ ] Lookup functions
- [ ] Date/time operations
- [ ] Array formulas

### Phase 4: Monetization (Q4 2025)
- [ ] License key system
- [ ] Feature flags for pro features
- [ ] Stripe/Paddle integration
- [ ] Trial period implementation

## Technical Architecture

```rust
// Core trait remains free
pub trait DataProvider {
    fn get_row(&self, index: usize) -> Option<Vec<String>>;
    // ... existing methods
}

// Pro features extend the trait
pub trait ComputedDataProvider: DataProvider {
    fn get_computed_column(&self, name: &str) -> Option<Vec<DataValue>>;
    fn add_formula_column(&mut self, name: &str, formula: Formula);
    fn recalculate(&mut self);
}

// Excel-specific implementation (Pro only)
pub struct ExcelDataTable {
    base: DataTable,
    formulas: HashMap<String, Formula>,
    cached_results: HashMap<String, Vec<DataValue>>,
}
```

## Revenue Model Justification

### Why $20-30/month is a No-Brainer:
1. **Replaces multiple tools**: Excel plugins ($50+), SQL clients ($100+)
2. **Time savings**: 10x faster on large datasets
3. **Prevents Excel crashes**: No more lost work
4. **Professional edge**: SQL queries on Excel data
5. **Single binary**: No installation hassles

### Market Size:
- 500,000+ financial analysts globally
- 50,000+ quant traders
- 100,000+ risk managers
- If we capture 0.1% at $25/month = $40K MRR

## Competition Analysis

| Tool | Price | Strengths | Our Advantage |
|------|-------|-----------|---------------|
| Excel | $70/year | Ubiquitous | We complement, not replace |
| Tableau | $70/month | Visualization | We're faster, keyboard-driven |
| DBeaver | Free/$22 | Database focus | We handle files better |
| pandas/Jupyter | Free | Powerful | We're instant, no coding |

## Marketing Angles

### For Traders:
"Analyze your trading book 100x faster than Excel. SQL queries on your positions. Never wait for recalculation again."

### For Quants:
"Your Excel models, turbocharged. Import sheets with formulas, query with SQL, export back to Excel."

### For Risk Managers:
"Process risk reports that crash Excel. Real-time filtering on million-row datasets. PnL calculations that don't freeze."

## Success Metrics

### Free Tier:
- 10,000 active users
- 5-star ratings on GitHub
- Active community

### Pro Tier:
- 100 paying customers in Year 1
- $2,500 MRR by month 6
- 80% retention rate
- 5% free-to-paid conversion

## Why This Will Work

1. **Real Pain Point**: Every financial professional has cursed at Excel
2. **Low Friction**: Works with existing Excel files
3. **Immediate Value**: 10x performance improvement visible in first use
4. **Fair Pricing**: $25/month is nothing for the time saved
5. **Moat**: The TUI expertise and performance optimization is hard to replicate

## Next Steps

1. Complete DataTable refactoring (current)
2. Add basic Excel import to free tier
3. Build formula parser prototype
4. Survey target users for feature priorities
5. Implement licensing system
6. Beta test with 10 financial professionals
7. Launch on HN/Reddit with "Show HN: SQL for your Excel files"

## Open Source Strategy

- Core remains MIT licensed
- Pro features in separate crate
- Clear distinction in documentation
- Community can contribute to both
- Pro features fund core development

This is not just a tool - it's a bridge between the Excel world financial professionals live in and the modern data processing they need.