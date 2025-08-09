#!/usr/bin/env python3
"""Generate large trade datasets for performance testing"""

import json
import random
import sys
from datetime import datetime, timedelta

# Base data for generating realistic trades
books = ["EQUITY_DESK_1", "EQUITY_DESK_2", "BOND_DESK_1", "FOREX_DESK_1", "COMMODITY_DESK", "DERIVATIVES_DESK"]
counterparties = ["BANK_A", "BANK_B", "BANK_C", "CREDIT_SUISSE", "GOLDMAN_SACHS", "JP_MORGAN", "BARCLAYS", "UBS", "DEUTSCHE_BANK", "HSBC", 
                  "MORGAN_STANLEY", "CITI", "BNP_PARIBAS", "SOCIETE_GENERALE", "NOMURA", "RBC", "WELLS_FARGO", "BANK_OF_AMERICA"]
countries = ["US", "UK", "JP", "DE", "CH", "FR", "SG", "HK", "CA", "AU", "NL", "IT", "ES", "SE", "NO"]
counterparty_types = ["BANK", "BROKER", "HEDGE_FUND", "PENSION_FUND", "INSURANCE", "MUTUAL_FUND", "SOVEREIGN_WEALTH", "FAMILY_OFFICE"]
currencies = ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD", "SEK", "NOK", "DKK", "SGD", "HKD"]
instruments = [
    "Apple Inc", "Microsoft Corp", "Google Inc", "Amazon Inc", "Tesla Inc", "Meta Inc", "Netflix Inc",
    "NVIDIA Corp", "AMD Inc", "Intel Corp", "Oracle Corp", "IBM Corp", "Salesforce Inc", "Adobe Inc",
    "US Treasury 10Y", "German Bund 10Y", "UK Gilt 10Y", "Corporate Bond AAA", "Municipal Bond",
    "High Yield Bond", "Emerging Market Bond", "Japanese Govt Bond", "French OAT", "Italian BTP",
    "EUR/USD", "GBP/USD", "USD/JPY", "EUR/GBP", "AUD/USD", "USD/CHF", "NZD/USD", "USD/CAD",
    "Gold Futures", "Oil Futures", "Copper Futures", "Silver Futures", "Natural Gas", "Wheat Futures",
    "S&P 500 Option", "FTSE 100 Option", "Nikkei Option", "DAX Option", "VIX Futures"
]
traders = [
    "John Smith", "Jane Doe", "Alice Johnson", "Bob Wilson", "Carol Brown", 
    "David Lee", "Emma Davis", "Frank Miller", "Grace Chen", "Henry Taylor",
    "Ivan Petrov", "Julia Wong", "Kevin Park", "Lisa Anderson", "Michael Chen",
    "Nancy Williams", "Oliver James", "Patricia Garcia", "Quinn Roberts", "Rachel Kim"
]

# Different confirmation statuses including ones with 'pend' for testing NOT Contains
confirmation_statuses = [
    "confirmed", "settled", "cleared", "processed", "matched", "executed",
    "pending_confirmation", "pending_settlement", "pending_review", "pending_clearance",
    "under_review", "rejected", "cancelled", "expired", "failed", "disputed"
]

def generate_trade(trade_id):
    """Generate a single realistic trade record"""
    base_date = datetime(2023, 1, 1)
    random_days = random.randint(0, 730)  # 2 years of data
    trade_date = base_date + timedelta(days=random_days)
    
    # Generate correlated price and quantity
    price = round(random.uniform(10.0, 1000.0), 2)
    # Higher priced instruments tend to have lower quantities
    max_quantity = int(100000 / (price / 10))
    quantity = random.randint(10, max(100, min(max_quantity, 50000)))
    
    # Commission based on trade value
    trade_value = price * quantity
    commission = round(trade_value * random.uniform(0.0001, 0.002), 2)
    
    return {
        "id": trade_id,
        "book": random.choice(books),
        "commission": commission,
        "confirmationStatus": random.choice(confirmation_statuses),
        "instrumentId": f"INST{trade_id % 1000:03d}",  # Reuse instrument IDs for realistic data
        "platformOrderId": f"ORDER-{trade_date.year}-{trade_id:07d}",
        "counterparty": random.choice(counterparties),
        "instrumentName": random.choice(instruments),
        "counterpartyCountry": random.choice(countries),
        "counterpartyType": random.choice(counterparty_types),
        "createdDate": trade_date.strftime("%Y-%m-%d"),
        "currency": random.choice(currencies),
        "quantity": quantity,
        "price": price,
        "trader": random.choice(traders),
        "tradeValue": round(trade_value, 2),
        "settlementDate": (trade_date + timedelta(days=random.randint(1, 5))).strftime("%Y-%m-%d"),
        "side": random.choice(["BUY", "SELL"]),
        "venue": random.choice(["NYSE", "NASDAQ", "LSE", "TSE", "EUREX", "CME", "ICE", "SGX"])
    }

def main():
    """Generate trades based on command line argument"""
    if len(sys.argv) != 2:
        print("Usage: python generate_large_trades.py <number_of_trades>")
        print("Example: python generate_large_trades.py 100000")
        sys.exit(1)
    
    try:
        num_trades = int(sys.argv[1])
    except ValueError:
        print(f"Error: '{sys.argv[1]}' is not a valid number")
        sys.exit(1)
    
    if num_trades <= 0:
        print("Error: Number of trades must be positive")
        sys.exit(1)
    
    output_file = f'trades_{num_trades // 1000}k.json' if num_trades >= 1000 else f'trades_{num_trades}.json'
    
    print(f"Generating {num_trades:,} trades...")
    trades = []
    
    # Generate trades in batches for memory efficiency
    batch_size = 10000
    for batch_start in range(0, num_trades, batch_size):
        batch_end = min(batch_start + batch_size, num_trades)
        batch_trades = [generate_trade(i) for i in range(batch_start + 1, batch_end + 1)]
        trades.extend(batch_trades)
        
        # Progress indicator
        if batch_end % 10000 == 0 or batch_end == num_trades:
            print(f"  Generated {batch_end:,} / {num_trades:,} trades...")
    
    # Write to file
    print(f"Writing to {output_file}...")
    with open(output_file, 'w') as f:
        json.dump(trades, f, indent=2)
    
    print(f"\nSuccessfully generated {num_trades:,} trades in {output_file}")
    
    # Print some statistics for verification
    pending_count = sum(1 for t in trades if 'pend' in t['confirmationStatus'].lower())
    confirmed_count = sum(1 for t in trades if 'confirmed' in t['confirmationStatus'].lower())
    commission_20_50 = sum(1 for t in trades if 20.0 <= t['commission'] <= 50.0)
    commission_100_500 = sum(1 for t in trades if 100.0 <= t['commission'] <= 500.0)
    
    print(f"\nStatistics:")
    print(f"  - Trades with 'pend' in confirmationStatus: {pending_count:,}")
    print(f"  - Trades with 'confirmed' status: {confirmed_count:,}")  
    print(f"  - Trades with commission 20-50: {commission_20_50:,}")
    print(f"  - Trades with commission 100-500: {commission_100_500:,}")
    print(f"  - Unique instruments: {len(set(t['instrumentName'] for t in trades[:min(10000, len(trades))]))}")
    print(f"  - Unique counterparties: {len(set(t['counterparty'] for t in trades[:min(10000, len(trades))]))}")
    print(f"  - File size: {len(json.dumps(trades)) / (1024*1024):.1f} MB")

if __name__ == "__main__":
    main()