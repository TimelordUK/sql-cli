# Advanced Parsing and Query Optimization Ideas

## Overview
This document captures ideas for advanced parsing and query optimization improvements to be implemented after the current refactoring and Redux migration is complete (estimated 1+ month timeline).

## Current State (as of 2025-01-15)
- Successfully optimized WHERE clause evaluation from 5s to 8.5ms for 100k rows (588x improvement)
- Achieved by reducing debug logging in hot paths
- Parser and evaluator are tightly coupled but functional

## Proposed Architecture: Expression Context Stack

### Core Concept
Detach parsing from evaluation entirely by introducing an intermediate "Expression Context" layer:

```
SQL Query → Parser → Expression Context → Evaluator → Results
                           ↑
                     (Reusable Stack)
```

### Benefits
1. **Parse Once, Evaluate Many**: Parse WHERE clause into reusable context
2. **Pre-computation**: Capture and cache constant expressions
3. **Type Safety**: Know types at parse time, eliminate runtime checks
4. **Optimization Opportunities**: Reorder conditions, short-circuit evaluation

## Specific Optimizations

### 1. Expression Context Stack
```rust
struct ExpressionContext {
    // Pre-computed values
    constants: HashMap<String, DataValue>,
    
    // Method results cache
    method_cache: HashMap<(String, String), DataValue>, // (column, method) -> result
    
    // Type information
    column_types: HashMap<String, DataType>,
    
    // Optimized expression tree
    expression: OptimizedExpression,
}
```

### 2. Constant Folding
- Pre-compute expressions like `"ABC".Length()` → `3`
- Cache regex patterns for LIKE operations
- Pre-lowercase strings for case-insensitive comparisons

### 3. Method Result Caching
- Cache `Length()` results for string columns
- Cache `Contains()` lowercase conversions
- Reuse results across row evaluations when possible

### 4. Expression Tree Optimization
- Reorder AND conditions: cheapest/most selective first
- Identify contradictions: `x > 5 AND x < 3` → always false
- Simplify redundant conditions: `x > 5 AND x > 3` → `x > 5`

### 5. Type-Aware Evaluation
- Generate type-specific evaluation code
- Eliminate runtime type checks
- Use specialized comparison functions

### 6. Parallel Evaluation
- Split independent OR branches for parallel evaluation
- Use Rust's rayon for data parallelism
- Similar to PLINQ in C#

### 7. GPU Acceleration
- Offload simple filtering to GPU for massive datasets
- Use CUDA/WebGPU for parallel row evaluation
- Particularly effective for numeric comparisons

## Implementation Phases

### Phase 1: Expression Context (Post-Redux)
- Implement basic expression context
- Separate parsing from evaluation
- Add constant folding

### Phase 2: Caching Layer
- Add method result caching
- Implement expression tree optimization
- Profile and measure improvements

### Phase 3: Advanced Optimizations
- Add parallel evaluation with rayon
- Implement type-specific code generation
- Explore GPU acceleration for large datasets

## Prerequisites
Before implementing these optimizations, complete:
1. ✅ DataView refactoring (DONE)
2. ✅ Performance baseline established (DONE - 8.5ms/100k rows)
3. ⏳ Redux state management migration
4. ⏳ Key handling extraction from TUI
5. ⏳ Filter state migration to DataView

## Related Documents
- `/docs/MATH_EXPRESSIONS.md` - Math expression support
- `/docs/REFACTOR_NOTES.md` - Current refactoring progress
- `/docs/REDUX_ARCHITECTURE.md` - Redux migration plan

## Performance Targets
- Current: 8.5ms for 100k rows with Contains()
- Target Phase 1: < 5ms for 100k rows
- Target Phase 2: < 2ms for 100k rows
- Target Phase 3: < 1ms for 100k rows (with GPU: < 0.5ms)

## Notes
- Focus on migration and refactoring first
- These optimizations become much easier with clean architecture
- Consider benchmarking against DuckDB/DataFusion for comparison