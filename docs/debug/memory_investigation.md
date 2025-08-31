# Memory Investigation

## The Problem
- 10k rows = ~350MB
- 20k rows = ~680MB  
- Linear scaling suggests ~35KB per row!
- A typical trade row should be <1KB

## Where Memory Could Be Going

### 1. Data Storage (Multiple Copies)
Currently we have:
- **CsvDataSource**: `Vec<Value>` (JSON)
- **Buffer.results**: `QueryResponse` with `Vec<Value>` 
- **Buffer.datatable**: `DataTable` with typed values
- **Buffer.filtered_data**: `Option<Vec<Vec<String>>>` (if filtering)

So we're storing data 3-4x!

### 2. Ratatui Rendering
Every frame, we create:
- `Row` objects for visible rows (30-50)
- `Cell` objects for each cell
- String clones for each value
- Style information per cell

Even though we only show 30 rows, we might be creating more.

### 3. String Allocations
- Field names duplicated in every JSON object
- String values stored multiple times
- UTF-8 overhead

## Experiment Plan

### Test 1: Load CSV but Don't Render
```rust
// Load 20k rows
// Don't create any TUI
// Measure memory
```

### Test 2: Load CSV with Limited DataTable
```rust
// Load 20k rows into CsvDataSource
// Create DataTable with only 5k rows
// Measure memory difference
```

### Test 3: Render Only Subset
```rust
// Load all data
// But only pass 1k rows to BufferAdapter
// See if memory usage drops
```

## Quick Test Code
Let's add a memory reporting function that shows:
- CsvDataSource size
- DataTable size  
- Buffer overhead
- Ratatui allocations (if possible)