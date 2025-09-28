#!/usr/bin/env python3
"""Generate a large trades CSV for testing cache performance."""

import csv
import random
import datetime
from decimal import Decimal

def generate_trades_csv(filename="data/large_trades.csv", num_rows=5000, num_cols=20):
    """Generate a large trades CSV file for testing."""

    print(f"Generating {num_rows} trades with {num_cols} columns...")

    # Define column names
    base_columns = [
        'trade_id', 'source', 'trade_date', 'trade_time', 'symbol', 'isin', 'cusip',
        'side', 'quantity', 'price', 'amount', 'currency', 'counterparty',
        'trader', 'book', 'strategy', 'venue', 'settlement_date', 'status'
    ]

    # Add extra columns to reach desired count
    extra_cols = [f'attribute_{i}' for i in range(1, num_cols - len(base_columns) + 1)]
    columns = base_columns + extra_cols

    sources = ['Bloomberg', 'Reuters', 'Barclays', 'JPMorgan', 'GoldmanSachs']
    symbols = ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA', 'IBM', 'NVDA', 'AMD']
    sides = ['BUY', 'SELL']
    currencies = ['USD', 'EUR', 'GBP', 'JPY', 'CHF']
    counterparties = ['BANK_A', 'BANK_B', 'FUND_X', 'FUND_Y', 'HEDGE_Z']
    statuses = ['SETTLED', 'PENDING', 'CONFIRMED', 'ALLOCATED']

    with open(filename, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=columns)
        writer.writeheader()

        base_date = datetime.date(2025, 9, 1)

        for i in range(1, num_rows + 1):
            row = {
                'trade_id': f'TRD{i:08d}',
                'source': random.choice(sources),
                'trade_date': str(base_date - datetime.timedelta(days=random.randint(0, 30))),
                'trade_time': f'{random.randint(9,16):02d}:{random.randint(0,59):02d}:{random.randint(0,59):02d}',
                'symbol': random.choice(symbols),
                'isin': f'US{random.randint(100000000,999999999)}',
                'cusip': f'{random.randint(100000000,999999999)}',
                'side': random.choice(sides),
                'quantity': random.randint(100, 10000) * 100,
                'price': round(random.uniform(10.0, 500.0), 2),
                'amount': round(random.uniform(10000.0, 5000000.0), 2),
                'currency': random.choice(currencies),
                'counterparty': random.choice(counterparties),
                'trader': f'TRADER_{random.randint(1,20)}',
                'book': f'BOOK_{random.randint(1,10)}',
                'strategy': f'STRAT_{random.randint(1,5)}',
                'venue': random.choice(['NYSE', 'NASDAQ', 'LSE', 'EUREX']),
                'settlement_date': str(base_date + datetime.timedelta(days=2)),
                'status': random.choice(statuses),
            }

            # Fill extra columns with random data
            for col in extra_cols:
                row[col] = f'VAL_{random.randint(1000,9999)}'

            writer.writerow(row)

            if i % 1000 == 0:
                print(f"  Generated {i} rows...")

    # Check file size
    import os
    size_mb = os.path.getsize(filename) / (1024 * 1024)
    print(f"Created {filename}: {num_rows} rows, {num_cols} columns, {size_mb:.2f} MB")

if __name__ == "__main__":
    generate_trades_csv(num_rows=5000, num_cols=25)
    print("\nYou can now test with:")
    print("  ./target/release/sql-cli data/large_trades.csv -q \"SELECT * FROM large_trades\" -o csv | head")