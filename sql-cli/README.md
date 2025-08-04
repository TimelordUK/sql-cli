# SQL CLI - Syntax-Aware SQL Editor

A fast, terminal-based SQL editor with intelligent tab completion that understands SQL syntax context.

## Features

- **Context-Aware Completion**: Only suggests relevant completions based on current SQL syntax position
- **Fast Performance**: Built in Rust for instant response times
- **Persistent History**: Remembers your previous queries across sessions
- **Cross-Platform**: Works on Linux and Windows (PowerShell)
- **Simple Syntax**: Supports SELECT, FROM, WHERE, and ORDER BY clauses

## Installation

```bash
cargo build --release
```

The binary will be in `target/release/sql-cli`

## Usage

Run the CLI:
```bash
./target/release/sql-cli
```

### Key Bindings

- **Tab**: Context-aware SQL completion
- **Enter**: Execute query (currently mock implementation)
- **Ctrl+P**: Previous command from history
- **Ctrl+N**: Next command from history
- **Ctrl+R**: Search command history
- **Ctrl+D**: Exit
- **\help**: Show help
- **\clear**: Clear screen

### Example Queries

```sql
SELECT dealId, price FROM trade_deal
SELECT * FROM trade_deal WHERE price > 100
SELECT * FROM trade_deal ORDER BY tradeDate DESC
```

## How It Works

The CLI uses a state machine parser that tracks your position in the SQL statement:

1. **Start**: Only suggests "SELECT"
2. **After SELECT**: Suggests column names or "*"
3. **In Column List**: Suggests more columns or "FROM"
4. **After FROM**: Suggests table names (trade_deal, instrument)
5. **After Table**: Suggests "WHERE" or "ORDER BY"
6. **In WHERE**: Suggests columns and logical operators
7. **In ORDER BY**: Suggests columns and "ASC"/"DESC"

## Architecture

- **parser.rs**: SQL syntax parser and state machine
- **completer.rs**: Reedline completer integration
- **main.rs**: Terminal UI and command handling

## Extending

To add more columns or tables:

1. Edit `Schema::new()` in `parser.rs`
2. Add column names to the `trade_deal_columns` vector
3. Add new tables to the `tables` vector

## Next Steps

- Integrate with your REST API for actual query execution
- Add support for LINQ-style operations (Contains, DateTime constructors)
- Extend parser for more complex SQL features
- Add configuration file for API endpoint and schema