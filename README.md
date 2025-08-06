# SQL CLI with Dynamic LINQ Support

A fast, context-aware SQL command-line interface with intelligent tab completion and a sophisticated TUI for querying REST API-based database engines.

## Features

### ✨ Smart SQL Completion
- **Context-aware tab completion** - understands SQL syntax, not just naive string matching
- **Cursor position tracking** - provides completions even mid-query (e.g., deleting `*` in `SELECT * FROM trade_deal` and typing `plat<tab>`)
- **Column-aware suggestions** - knows about your 190+ columns and suggests relevant ones

### 🎯 Dynamic LINQ Query Support
- **String methods**: `platformOrderId.Contains("E")`, `ticker.StartsWith("AA")`, `ticker.IndexOf("abc") > 10`
- **Complex expressions**: `Price > 100 AND Ticker == "AAPL"`
- **Property name normalization** - automatically converts `platformOrderId` → `PlatformOrderId`
- **Rich filtering** - supports all .NET string methods and comparison operators

### 🖥️ Professional TUI Interface  
- **Split-view design** - command input at top, results grid below
- **Scrollable data grid** - handles large result sets efficiently
- **Status bar** - shows query status and navigation hints
- **Mode switching** - Command mode for input, Results mode for navigation

### ⚡ High Performance
- **Rust-based client** - fast startup and efficient memory usage
- **Streaming results** - handles large datasets without blocking
- **Cross-platform** - works on Linux and Windows terminals

## Architecture

```
┌─────────────────┐    HTTP/JSON    ┌──────────────────┐
│   Rust CLI      │◄──────────────►│   C# REST API    │
│                 │                 │                  │
│ • ratatui TUI   │                 │ • ASP.NET Core   │
│ • reedline      │                 │ • Dynamic LINQ   │
│ • SQL parser    │                 │ • Query processor│
│ • Completions   │                 │ • 190+ columns  │
└─────────────────┘                 └──────────────────┘
```

## Installation

### Download Pre-built Binaries

Download the latest release from the [Releases](https://github.com/YOUR_USERNAME/sql-cli/releases) page:

- **Linux x64**: `sql-cli-linux-x64.tar.gz`
- **Windows x64**: `sql-cli-windows-x64.zip`
- **macOS x64 (Intel)**: `sql-cli-macos-x64.tar.gz`
- **macOS ARM64 (Apple Silicon)**: `sql-cli-macos-arm64.tar.gz`

Extract and run:
```bash
# Linux/macOS
tar xzf sql-cli-*.tar.gz
chmod +x sql-cli
./sql-cli

# Windows
# Extract the zip file and run sql-cli.exe
```

### Build from Source

```bash
git clone https://github.com/YOUR_USERNAME/sql-cli.git
cd sql-cli/sql-cli
cargo build --release
./target/release/sql-cli
```

## Quick Start

### Standalone Mode (CSV/JSON Files)
```bash
# Query a CSV file
sql-cli data/customers.csv

# Query a JSON file  
sql-cli data/trades.json
```

### API Server Mode
```bash
cd TradeApi
dotnet run
# Server starts on http://localhost:5073

# In another terminal
sql-cli --api http://localhost:5073
```

### Running the CLI
```bash
cd sql-cli
cargo run
# TUI interface launches
```

### Example Queries
```sql
SELECT * FROM trade_deal WHERE platformOrderId.Contains("200000")
SELECT DealId, Ticker, Price FROM trade_deal WHERE Price > 100 AND Ticker.StartsWith("AA")
SELECT * FROM trade_deal WHERE Ticker IN ("AAPL", "MSFT") ORDER BY Price DESC
```

## Development

### Testing
```bash
# Run C# API tests
cd TradeApi.Tests
dotnet test

# Run Rust CLI tests  
cd sql-cli
cargo test
```

### Key Components

#### Rust CLI (`sql-cli/src/`)
- `main.rs` - Entry point and TUI initialization
- `tui_app.rs` - Main TUI application with split-view interface
- `parser.rs` - SQL syntax parser for context awareness
- `cursor_aware_parser.rs` - Advanced completion with cursor tracking
- `api_client.rs` - HTTP client for REST API communication
- `completer.rs` - Tab completion logic

#### C# API (`TradeApi/`)
- `Controllers/TradeController.cs` - REST endpoints for queries and schema
- `Services/QueryProcessor.cs` - Dynamic LINQ query processing and property normalization
- `Services/TradeDataService.cs` - Mock data service with 190+ columns
- `Models/TradeDeal.cs` - Trade entity with comprehensive field set

## Roadmap

- [ ] csvlens-style table features (column sorting, filtering)
- [ ] Vim-like search and navigation
- [ ] Grammar tree visualization
- [ ] Separate diagnostic console
- [ ] Query history and persistence
- [ ] Multiple database connection support

## License

Private repository - All rights reserved.