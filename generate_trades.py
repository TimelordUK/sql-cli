#!/usr/bin/env python3
"""
Trade Data Generator for SQL-CLI Testing
Generates realistic trade data in JSON or CSV format
"""

import json
import csv
import random
import sys
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional

def generate_trades(count: int) -> List[Dict[str, Any]]:
    """Generate realistic trade data"""
    
    # Reference data
    counterparties = [
        "Goldman Sachs", "JP Morgan", "Morgan Stanley", "Bank of America",
        "Citigroup", "Wells Fargo", "Deutsche Bank", "Barclays", 
        "Credit Suisse", "UBS", "BNP Paribas", "HSBC",
        "RBC Capital", "Jefferies", "Nomura", "Mizuho"
    ]
    
    countries = ["US", "UK", "JP", "DE", "FR", "CH", "CA", "AU", "SG", "HK"]
    
    books = [
        "Equity Trading", "Bond Trading", "FX Trading", "Derivatives",
        "Commodities", "Credit Trading", "Emerging Markets", "Prime Brokerage",
        "Structured Products", "ETF Trading", "Options Trading", "Futures Trading"
    ]
    
    clearing_houses = ["lch", "cme", "ice", "eurex", "dtcc", "jscc"]
    
    desks = [
        "Flow Trading", "Prop Trading", "Market Making", "Arbitrage",
        "Delta One", "Exotics", "Structured", "Electronic"
    ]
    
    strategies = [
        "Momentum", "Mean Reversion", "Arbitrage", "Market Making",
        "Hedging", "Directional", "Relative Value", "Event Driven"
    ]
    
    product_types = [
        "Equity", "Bond", "Option", "Future", "Swap", "Forward", 
        "CDS", "ETF", "Index", "Commodity", "FX Spot", "FX Forward"
    ]
    
    currencies = ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD"]
    
    tickers = [
        "AAPL", "GOOGL", "MSFT", "AMZN", "META", "NVDA", "TSLA",
        "JPM", "BAC", "GS", "MS", "C", "WFC", "HSBC",
        "XOM", "CVX", "SHEL", "BP", "TTE",
        "JNJ", "PFE", "UNH", "LLY", "MRK"
    ]
    
    statuses = ["completed", "pending", "cancelled", "failed", "partial"]
    allocation_statuses = ["Allocated", "Unallocated", "Partial", "Pending"]
    confirmation_statuses = ["Confirmed", "Unconfirmed", "Pending", "Rejected"]
    settlement_statuses = ["Settled", "Unsettled", "Pending", "Failed"]
    sides = ["buy", "sell"]
    
    base_date = datetime.now() - timedelta(days=365)
    trades = []
    
    for i in range(count):
        # Pick random values
        counterparty = random.choice(counterparties)
        country = random.choice(countries)
        book = random.choice(books)
        clearing_house = random.choice(clearing_houses)
        desk = random.choice(desks)
        strategy = random.choice(strategies)
        product_type = random.choice(product_types)
        currency = random.choice(currencies)
        ticker = random.choice(tickers)
        status = random.choice(statuses)
        allocation_status = random.choice(allocation_statuses)
        confirmation_status = random.choice(confirmation_statuses)
        settlement_status = random.choice(settlement_statuses)
        side = random.choice(sides)
        
        # Generate correlated values
        notional = round(random.uniform(10_000, 10_000_000), 2)
        price = round(random.uniform(1, 5000), 4)
        quantity = int(notional / price)
        commission = round(notional * random.uniform(0.0001, 0.002), 2)
        fees = round(notional * random.uniform(0.00001, 0.0001), 2)
        
        # Generate dates
        trade_date = base_date + timedelta(days=random.randint(0, 365))
        settlement_date = trade_date + timedelta(days=random.randint(1, 5))
        maturity_date = None
        if random.random() < 0.3:
            maturity_date = trade_date + timedelta(days=random.randint(30, 3650))
        
        # Risk metrics
        dv01 = None
        pv01 = None
        if product_type in ["Bond", "Swap"]:
            dv01 = round(random.uniform(100, 10000), 2)
            pv01 = round(dv01 * 1.1, 2)
        
        delta = None
        gamma = None
        vega = None
        if product_type == "Option":
            delta = round(random.uniform(-1, 1), 4)
            gamma = round(random.uniform(0, 0.1), 6)
            vega = round(random.uniform(0, 100), 2)
        
        duration = None
        yield_val = None
        if product_type == "Bond":
            duration = round(random.uniform(0.5, 30), 2)
            yield_val = round(random.uniform(0.01, 0.10), 4)
        
        trade = {
            # IDs
            "dealId": f"DEAL{i+1:08d}",
            "platformOrderId": f"P{i+1:08d}",
            "parentOrderId": f"PARENT{random.randint(1, count//10):06d}" if random.random() < 0.2 else None,
            "externalOrderId": f"EXT{random.randint(1000000, 99999999):08d}" if random.random() < 0.7 else None,
            
            # Counterparty info
            "counterparty": counterparty,
            "counterpartyId": f"CP{random.randint(1, 100):04d}",
            "counterpartyCountry": country,
            "counterpartyType": "Institution" if random.random() < 0.6 else "Broker",
            
            # Trade details
            "book": book,
            "desk": desk,
            "portfolio": f"{book.replace(' ', '')}-{random.randint(1, 10)}",
            "strategy": strategy,
            "clearingHouse": clearing_house,
            "exchange": "NYSE" if random.random() < 0.8 else "NASDAQ",
            "venue": "Electronic" if random.random() < 0.5 else "Voice",
            
            # Instrument info
            "instrumentId": f"INST{random.randint(1, 10000):06d}",
            "instrumentName": f"{ticker} {product_type}",
            "instrumentType": product_type,
            "productType": product_type,
            "ticker": ticker,
            "isin": f"US{random.randint(1000000000, 9999999999):010d}" if random.random() < 0.8 else None,
            "cusip": f"{random.randint(100000000, 999999999):09d}" if random.random() < 0.5 else None,
            
            # Trade economics
            "side": side,
            "quantity": quantity,
            "price": str(price),
            "notional": str(notional),
            "currency": currency,
            "settlementAmount": str(round(notional + commission + fees, 2)),
            "commission": str(commission),
            "fees": str(fees),
            "accruedInterest": str(round(random.uniform(0, 10000), 2)) if product_type == "Bond" else None,
            
            # Risk metrics
            "dV01": str(dv01) if dv01 else None,
            "pV01": str(pv01) if pv01 else None,
            "delta": str(delta) if delta else None,
            "gamma": str(gamma) if gamma else None,
            "vega": str(vega) if vega else None,
            "duration": str(duration) if duration else None,
            "yield": str(yield_val) if yield_val else None,
            
            # Dates
            "tradeDate": trade_date.strftime("%Y-%m-%d"),
            "settlementDate": settlement_date.strftime("%Y-%m-%d"),
            "valueDate": settlement_date.strftime("%Y-%m-%d"),
            "maturityDate": maturity_date.strftime("%Y-%m-%d") if maturity_date else None,
            "createdDate": trade_date.strftime("%Y-%m-%dT%H:%M:%S"),
            "lastModifiedDate": (trade_date + timedelta(hours=random.randint(1, 24))).strftime("%Y-%m-%dT%H:%M:%S"),
            
            # Statuses
            "status": status,
            "allocationStatus": allocation_status,
            "confirmationStatus": confirmation_status,
            "settlementStatus": settlement_status,
            
            # People
            "trader": f"Trader{random.randint(1, 50):03d}",
            "traderId": f"T{random.randint(1, 50):04d}",
            "prime": random.choice(counterparties) if random.random() < 0.3 else None,
            
            # Flags and metadata
            "isElectronic": random.random() < 0.7,
            "comments": f"Trade {i+1} - Special handling required" if random.random() < 0.1 else None
        }
        
        trades.append(trade)
    
    return trades

def write_json(trades: List[Dict], filename: str):
    """Write trades to JSON file"""
    with open(filename, 'w') as f:
        json.dump(trades, f, indent=2, default=str)

def write_csv(trades: List[Dict], filename: str):
    """Write trades to CSV file"""
    if not trades:
        return
    
    # Get all unique keys across all trades
    all_keys = set()
    for trade in trades:
        all_keys.update(trade.keys())
    
    headers = sorted(all_keys)
    
    with open(filename, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=headers)
        writer.writeheader()
        writer.writerows(trades)

def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        print("Trade Data Generator")
        print("Usage: python generate_trades.py <rows> [format] [output_file]")
        print("  rows: Number of rows to generate (e.g., 100, 1000, 10000)")
        print("  format: 'json' or 'csv' (default: json)")
        print("  output_file: Output filename (default: trades_<rows>.<format>)")
        print("\nExamples:")
        print("  python generate_trades.py 1000")
        print("  python generate_trades.py 10000 csv")
        print("  python generate_trades.py 5000 json my_trades.json")
        sys.exit(0)
    
    row_count = int(sys.argv[1])
    format_type = sys.argv[2] if len(sys.argv) > 2 else "json"
    output_file = sys.argv[3] if len(sys.argv) > 3 else f"trades_{row_count}.{format_type}"
    
    print(f"Generating {row_count:,} trade records in {format_type} format...")
    
    trades = generate_trades(row_count)
    
    if format_type == "csv":
        write_csv(trades, output_file)
    elif format_type == "json":
        write_json(trades, output_file)
    else:
        print(f"Invalid format: {format_type}. Use 'json' or 'csv'")
        sys.exit(1)
    
    # Get file size
    import os
    file_size = os.path.getsize(output_file) / (1024 * 1024)  # MB
    
    print(f"✅ Generated {row_count:,} trades in {output_file}")
    print(f"📁 File size: {file_size:.2f} MB")
    
    # Show sample query suggestions
    print("\n📊 Sample queries to try:")
    print("  - SELECT * FROM trades WHERE counterparty.Contains('Morgan')")
    print("  - SELECT * FROM trades WHERE notional > 1000000 ORDER BY notional DESC")
    print("  - SELECT * FROM trades WHERE clearingHouse IN ('lch', 'cme') AND status = 'completed'")
    print("  - SELECT book, COUNT(*) FROM trades GROUP BY book")

if __name__ == "__main__":
    main()