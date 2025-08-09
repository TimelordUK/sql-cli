#!/usr/bin/env python3
"""Generate 100 realistic trading records for testing the parser fix"""

import json
import random
from datetime import datetime, timedelta

# Base data for generating realistic trades
books = ["EQUITY_DESK_1", "EQUITY_DESK_2", "BOND_DESK_1", "FOREX_DESK_1", "COMMODITY_DESK", "DERIVATIVES_DESK"]
counterparties = ["BANK_A", "BANK_B", "BANK_C", "CREDIT_SUISSE", "GOLDMAN_SACHS", "JP_MORGAN", "BARCLAYS", "UBS", "DEUTSCHE_BANK", "HSBC"]
countries = ["US", "UK", "JP", "DE", "CH", "FR", "SG", "HK"]
counterparty_types = ["BANK", "BROKER", "HEDGE_FUND", "PENSION_FUND", "INSURANCE"]
currencies = ["USD", "EUR", "GBP", "JPY", "CHF"]
instruments = [
    "Apple Inc", "Microsoft Corp", "Google Inc", "Amazon Inc", "Tesla Inc", "Meta Inc", "Netflix Inc",
    "US Treasury 10Y", "German Bund 10Y", "UK Gilt 10Y", "Corporate Bond AAA", "Municipal Bond",
    "EUR/USD", "GBP/USD", "USD/JPY", "EUR/GBP", "AUD/USD", "USD/CHF",
    "Gold Futures", "Oil Futures", "Copper Futures", "Silver Futures",
    "S&P 500 Option", "FTSE 100 Option", "Nikkei Option"
]

# Different confirmation statuses including ones with 'pend' for testing NOT Contains
confirmation_statuses = [
    "confirmed", "settled", "cleared", "processed", 
    "pending_confirmation", "pending_settlement", "pending_review", "pending_clearance",
    "under_review", "rejected", "cancelled", "expired"
]

def generate_trade(trade_id):
    """Generate a single realistic trade record"""
    base_date = datetime(2024, 1, 1)
    random_days = random.randint(0, 365)
    trade_date = base_date + timedelta(days=random_days)
    
    return {
        "id": trade_id,
        "book": random.choice(books),
        "commission": round(random.uniform(10.0, 150.0), 2),
        "confirmationStatus": random.choice(confirmation_statuses),
        "instrumentId": f"INST{trade_id:03d}",
        "platformOrderId": f"ORDER-2024-{trade_id:03d}",
        "counterparty": random.choice(counterparties),
        "instrumentName": random.choice(instruments),
        "counterpartyCountry": random.choice(countries),
        "counterpartyType": random.choice(counterparty_types),
        "createdDate": trade_date.strftime("%Y-%m-%d"),
        "currency": random.choice(currencies),
        "quantity": random.randint(100, 10000),
        "price": round(random.uniform(50.0, 500.0), 2),
        "trader": random.choice(["John Smith", "Jane Doe", "Alice Johnson", "Bob Wilson", "Carol Brown", "David Lee", "Emma Davis", "Frank Miller", "Grace Chen", "Henry Taylor"])
    }

def main():
    """Generate 100 trades and save to data/trades.json"""
    trades = []
    
    for i in range(1, 101):
        trades.append(generate_trade(i))
    
    # Write to file
    with open('data/trades.json', 'w') as f:
        json.dump(trades, f, indent=2)
    
    print("Generated 100 trades in data/trades.json")
    
    # Print some statistics for verification
    pending_count = len([t for t in trades if 'pend' in t['confirmationStatus'].lower()])
    confirmed_count = len([t for t in trades if 'confirmed' in t['confirmationStatus'].lower()])
    commission_20_50 = len([t for t in trades if 20.0 <= t['commission'] <= 50.0])
    
    print(f"Statistics:")
    print(f"  - Trades with 'pend' in confirmationStatus: {pending_count}")
    print(f"  - Trades with 'confirmed' status: {confirmed_count}")  
    print(f"  - Trades with commission 20-50: {commission_20_50}")
    print(f"  - Trades where NOT Contains('pend') AND commission 20-50: {len([t for t in trades if 'pend' not in t['confirmationStatus'].lower() and 20.0 <= t['commission'] <= 50.0])}")

if __name__ == "__main__":
    main()