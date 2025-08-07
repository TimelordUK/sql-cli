# SQL-CLI Scripts

## generate_trades.py

Generates realistic trade data for testing SQL-CLI with large datasets.

### Usage

```bash
# Generate 10,000 trades in JSON format
python scripts/generate_trades.py 10000

# Generate 100,000 trades in CSV format
python scripts/generate_trades.py 100000 csv

# Generate with custom output file
python scripts/generate_trades.py 50000 json data/my_trades.json
```

### Output Files

Generated files are placed in `data/` directory by default:
- `data/trades_10k.json` - 10K rows (~14.5 MB)
- `data/trades_100k.json` - 100K rows (~145 MB)

These files are excluded from git via `.gitignore` pattern `data/trades_*.json`

### Features

- 53 columns of realistic financial trade data
- Correlated values (price, notional, quantity)
- Risk metrics for appropriate instruments
- Realistic distributions of counterparties, books, currencies
- Date ranges and proper trade lifecycle fields