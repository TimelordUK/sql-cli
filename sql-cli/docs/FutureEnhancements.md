# Future Enhancements

## Enhanced Statistics Visualization (Post-DataView)

### Overview
Transform the Statistics page (S key) into a comprehensive data visualization dashboard using Ratatui's charting capabilities.

### Planned Features

#### 1. Automatic Column Analysis
- Detect column types (numeric, datetime, categorical)
- Auto-generate appropriate visualizations
- Smart defaults based on data distribution

#### 2. Numeric Column Visualizations
- **Sparklines**: Inline trend visualization
- **Histograms**: Distribution analysis
- **Box plots**: Quartile analysis
- **Scatter plots**: Correlation between columns
- **Time series**: For temporal data

#### 3. Trade Data Specific Charts
- **Price Charts**:
  - Line charts for price movement
  - Candlestick charts (using Canvas widget)
  - Volume bars overlay
  - Moving averages (SMA, EMA)
  
- **Volume Analysis**:
  - Cumulative volume profiles
  - Volume distribution histograms
  - Time-based volume charts
  
- **Statistical Overlays**:
  - Bollinger Bands
  - Standard deviation channels
  - Min/max indicators
  - Percentile markers

#### 4. Interactive Features
- Zoom/pan with keyboard navigation
- Toggle between chart types
- Adjustable time windows
- Export chart data

#### 5. Performance Metrics
- Real-time calculation (all in-memory)
- Incremental updates for streaming data
- Efficient data sampling for large datasets

### Implementation Notes

#### Available Ratatui Widgets
```rust
// Basic charts
Chart::new(datasets)
    .x_axis(axis)
    .y_axis(axis)
    
// Compact visualizations
Sparkline::default()
    .data(&data)
    
// Bar charts
BarChart::default()
    .data(&data)
    
// Custom drawing
Canvas::default()
    .paint(|ctx| {
        // Draw candlesticks, custom indicators
    })
```

#### Data Flow
1. DataView provides filtered/sorted data
2. Statistics analyzer calculates metrics
3. Chart renderer creates visualizations
4. All calculations in-memory for speed

### Benefits
- **Lightning fast**: All in-memory operations
- **No dependencies**: Pure terminal rendering
- **Consistent UX**: Same navigation as rest of TUI
- **Export ready**: Charts can be exported as data

### Examples

#### Trade Price Analysis
```
┌─────────────────────────────────────┐
│ AAPL Price (Last 100 trades)       │
│     $150 ┤ ╭─╮                      │
│     $149 ┤╭╯ ╰─╮    ╭─╮            │
│     $148 ┼╯    ╰────╯ ╰─╮          │
│     $147 ┤              ╰──        │
│          └────────────────────────   │
│ Volume   ▁▃▅▇▅▃▁▂▄▆▇▅▃▁           │
└─────────────────────────────────────┘
```

#### Distribution Analysis
```
┌─────────────────────────────────────┐
│ Trade Size Distribution             │
│                                     │
│ 0-100    ████████████████ 45%      │
│ 100-500  ████████ 20%              │
│ 500-1K   ██████ 15%                │
│ 1K-5K    ████ 10%                  │
│ 5K+      ██ 10%                    │
└─────────────────────────────────────┘
```

### Prerequisites
- Complete DataView architecture
- All state in AppStateContainer
- Statistics calculation engine
- Efficient data sampling algorithms

### Related Tools
Similar to terminal tools like:
- `btm` (bottom) - system monitor with charts
- `gtop` - graphical top with sparklines
- `bandwhich` - network monitor with graphs

But focused on data analysis rather than system monitoring.