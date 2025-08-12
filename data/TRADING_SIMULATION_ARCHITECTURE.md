# Trading System Simulation Architecture

## Overview

This document describes the production-quality trading system simulation used to generate realistic test data for the SQL-CLI TUI. The simulation models a complete VWAP (Volume Weighted Average Price) execution flow from client order through algo engine, smart order router, to venue execution.

## Architecture Components

### 1. Order Hierarchy

```
CLIENT_001 (Level 0) - Original client order
    ↓
ALGO_001 (Level 1) - Algo parent order (VWAP strategy)
    ↓
SLICE_00001...N (Level 2) - Algo child slices
    ↓
SOR_000001...N (Level 3) - SOR routes to venues
```

**Key Design Decisions:**
- `client_order_id` preserved throughout entire hierarchy for traceability
- Each level maintains its own order state
- Fill propagation cascades up immediately (5ms delays between levels)

### 2. Fill Propagation Model

Every fill follows this cascade pattern:
```
Venue Fill (T+0ms)
    → SOR Route Update (T+0ms)
    → Algo Slice Update (T+10ms)
    → Algo Parent Update (T+15ms)
    → Client Order Update (T+20ms)
```

**Rationale:** This mirrors production systems where:
- Venue sends execution report
- SOR aggregates venue fills
- Algo engine updates parent order
- FIX engine sends client update

### 3. Participation Monitoring & Urgency

The algo engine monitors participation rate against VWAP schedule:

```python
Participation Rate = (Filled Quantity / Expected Quantity) * 100

if participation < 70%:    urgency = CRITICAL  # Sweep all venues
elif participation < 85%:  urgency = URGENT    # Take liquidity aggressively
elif participation < 95%:  urgency = NORMAL    # Standard execution
else:                      urgency = PASSIVE   # Post liquidity
```

**Impact on Execution:**
- **CRITICAL**: Large slices, market orders, accept slippage
- **URGENT**: Medium slices, immediate-or-cancel orders
- **NORMAL**: Standard slicing, limit orders
- **PASSIVE**: Small slices, post-only orders

### 4. Market Microstructure Modeling

#### Current Implementation

**Venue Responses:**
- **FILLED** (83%): Complete fill at venue
- **PARTIAL** (10%): Partial fill due to limited liquidity
- **FADE** (5%): Liquidity taken by competitor
- **REJECT** (2%): Connection issues or price protection

**Price Formation:**
```python
Base Price + Urgency Spread + Random Walk
- CRITICAL: +2-4 bps (paying for immediacy)
- URGENT: +1-2 bps (crossing spread)
- NORMAL: -1 to +1 bps (at mid)
- PASSIVE: -1 bps (earning spread)
```

#### Future Enhancements (Not Yet Implemented)

1. **Internal Liquidity Matching**
   - SOR checks internal crossing engine before routing external
   - Risk desk provides liquidity from inventory
   - Internalization rate typically 10-20% for large firms

2. **Dark Pool Aggregation**
   - Multiple dark pool venues with different liquidity profiles
   - Conditional orders based on minimum quantity
   - Mid-point crossing logic

3. **Advanced SOR Logic**
   - Spray ordering across venues based on historical fill rates
   - Venue toxicity scoring (avoid venues with high fade rates)
   - Dynamic routing based on real-time market data

4. **Regulatory Considerations**
   - Best execution validation
   - Reg NMS compliance (trade-through protection)
   - MiFID II reporting fields

## Data Generation Strategy

### File Size Management

Target: < 100K rows for TUI performance

**Approach:**
- **Full Detail**: All orders and routes (~3-5K rows for 2M shares)
- **Summary Mode**: Slice summaries + client updates (~500-1K rows)
- **Client Only**: Just client order updates (~10-50 rows)

### Realistic Volumes

**Production Benchmarks:**
- Large institutional order: 1-5M shares
- Slices: 500-5000 shares each
- Routes: 2-5 venues per slice
- Daily volume: 10K-100K orders

**Our Simulation:**
- Default: 2M shares, 2000 share slices
- Generates ~1000 slices, ~3000 routes
- Results in ~3-5K database snapshots

## Key Metrics Tracked

### Execution Quality
- Fill rate (% of order completed)
- VWAP performance (slippage in bps)
- Participation rate (actual vs planned)
- Venue performance (fill rates, fade rates)

### Microstructure Analysis
- Fade events (lost liquidity to competitors)
- Partial fills (liquidity constraints)
- Rejects (technical/connectivity issues)
- Retry attempts (recovery from failures)

## SQL Queries for Analysis

### Client Perspective
```sql
-- What the client sees
SELECT * FROM production_vwap_final 
WHERE order_level = 0 
ORDER BY snapshot_time
```

### Algo Performance
```sql
-- Participation tracking
SELECT snapshot_time, filled_quantity, participation_pct, urgency
FROM production_vwap_final
WHERE order_id = 'ALGO_001' AND event_type = 'ALGO_UPDATE'
```

### Microstructure Issues
```sql
-- Find problem venues
SELECT venue, 
       COUNT(*) as attempts,
       SUM(CASE WHEN state = 'FADE' THEN 1 ELSE 0 END) as fades,
       SUM(CASE WHEN state = 'PARTIAL' THEN 1 ELSE 0 END) as partials
FROM production_vwap_final
WHERE order_level = 3
GROUP BY venue
```

## Future Roadmap

### Phase 2: Internal Liquidity
- Implement crossing engine
- Add risk desk liquidity provision
- Model internalization benefits

### Phase 3: Advanced Market Models
- Multi-asset support (futures, options)
- Cross-asset hedging flows
- Market impact modeling

### Phase 4: Real-time Simulation
- WebSocket feed simulation
- Streaming position updates
- Live market data integration

### Phase 5: Machine Learning Integration
- Venue selection optimization
- Fill rate prediction
- Optimal slice sizing

## File Structure

```
data/
├── production_vwap_final.csv    # Main dataset with fill propagation
├── instruments.csv               # Reference data
├── generate_production_vwap_fixed.py  # Generator script
└── TRADING_SIMULATION_ARCHITECTURE.md # This document
```

## Testing Recommendations

1. **Load Test**: Start with summary mode for large orders
2. **Propagation Test**: Verify every fill cascades up
3. **Urgency Test**: Check aggression increases when behind
4. **Microstructure Test**: Analyze fade/partial patterns

## Configuration Parameters

```python
# Current defaults
ORDER_SIZE = 2,000,000      # 2M shares
AVG_SLICE_SIZE = 2,000       # 2K shares per slice
FADE_RATE = 0.05             # 5% fade probability
PARTIAL_RATE = 0.10          # 10% partial fill probability
PROPAGATION_DELAY = 5ms      # Between cascade levels
```

## Validation Checklist

- [ ] Client order quantity remains constant
- [ ] Filled quantity only increases
- [ ] Every slice fill propagates to parent
- [ ] Urgency changes with participation rate
- [ ] Venue statistics are realistic
- [ ] File size < 100K rows
- [ ] All orders preserve client_order_id

## Contact & Maintenance

This simulation was designed to provide realistic test data for SQL-CLI development. The modular Python architecture allows easy enhancement for additional scenarios.

Key principles:
1. **Realism over complexity** - Model what matters for testing
2. **Audit trail completeness** - Every event is captured
3. **Performance awareness** - Keep data volumes manageable
4. **Extensibility** - Easy to add new scenarios

---

*Last Updated: 2024-12-16*
*Version: 1.0 - Production VWAP with fill propagation*