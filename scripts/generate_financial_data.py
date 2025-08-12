#!/usr/bin/env python3
"""
Advanced Financial Data Generator for SQL-CLI Testing
Generates realistic trade flows, instrument reference data, and tick data
Supports VWAP algo execution, SOR routing, and parent-child order hierarchies
"""

import json
import csv
import random
import sys
import argparse
import math
from datetime import datetime, timedelta, time
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum

# ============== Data Models ==============

class OrderState(Enum):
    """Order lifecycle states"""
    PENDING = "Pending"
    NEW = "New"
    ACCEPTED = "Accepted"
    REJECTED = "Rejected"
    WORKING = "Working"
    PARTIALLY_FILLED = "PartiallyFilled"
    FILLED = "Filled"
    CANCELLED = "Cancelled"
    EXPIRED = "Expired"

class Venue(Enum):
    """Trading venues"""
    NYSE = "NYSE"
    NASDAQ = "NASDAQ"
    BATS = "BATS"
    ARCA = "ARCA"
    EDGX = "EDGX"
    CHX = "CHX"
    IEX = "IEX"
    DARK_POOL = "DarkPool"
    LSE = "LSE"
    EUREX = "EUREX"
    XETRA = "XETRA"

class AlgoType(Enum):
    """Algo trading strategies"""
    VWAP = "VWAP"
    TWAP = "TWAP"
    POV = "POV"  # Percentage of Volume
    IS = "IS"    # Implementation Shortfall
    CLOSE = "Close"
    ICEBERG = "Iceberg"
    SNIPER = "Sniper"

@dataclass
class Order:
    """Order data structure"""
    order_id: str
    parent_order_id: Optional[str]
    client_order_id: str
    ticker: str
    side: str
    quantity: int
    filled_quantity: int
    price: Optional[float]
    order_type: str
    tif: str  # Time in Force
    state: str
    venue: Optional[str]
    algo_type: Optional[str]
    timestamp: datetime
    update_timestamp: datetime
    
    # Additional fields
    remaining_quantity: int = 0
    average_price: float = 0.0
    commission: float = 0.0
    client_name: Optional[str] = None
    trader: Optional[str] = None
    desk: Optional[str] = None
    strategy: Optional[str] = None
    
    def __post_init__(self):
        self.remaining_quantity = self.quantity - self.filled_quantity

# ============== Market Data ==============

EU_LARGE_CAP_STOCKS = [
    {"ticker": "ASML.AS", "name": "ASML Holding", "price": 650.0, "daily_volume": 2500000},
    {"ticker": "NESN.VX", "name": "Nestle", "price": 105.0, "daily_volume": 3000000},
    {"ticker": "ROG.VX", "name": "Roche", "price": 280.0, "daily_volume": 1800000},
    {"ticker": "SAP.DE", "name": "SAP", "price": 140.0, "daily_volume": 2200000},
    {"ticker": "SAN.MC", "name": "Santander", "price": 3.5, "daily_volume": 45000000},
    {"ticker": "TOTF.PA", "name": "TotalEnergies", "price": 55.0, "daily_volume": 8000000},
    {"ticker": "SIE.DE", "name": "Siemens", "price": 165.0, "daily_volume": 2000000},
    {"ticker": "NOVN.VX", "name": "Novartis", "price": 95.0, "daily_volume": 2500000},
    {"ticker": "AZN.L", "name": "AstraZeneca", "price": 110.0, "daily_volume": 3500000},
    {"ticker": "SHEL.L", "name": "Shell", "price": 28.0, "daily_volume": 15000000},
]

CLIENTS = [
    "Blackrock Asset Management", "Vanguard Group", "State Street Global",
    "Fidelity Investments", "JP Morgan Asset Mgmt", "Capital Group",
    "BNY Mellon", "Goldman Sachs Asset Mgmt", "PIMCO", "Invesco",
    "Wellington Management", "T. Rowe Price", "Northern Trust",
    "Millennium Management", "Citadel Securities", "Two Sigma",
    "Renaissance Technologies", "Bridgewater Associates", "AQR Capital"
]

TRADERS = [f"TRD{i:03d}" for i in range(1, 21)]
DESKS = ["Equity Trading", "Program Trading", "Electronic Trading", "Cash Equity", "Delta One"]

# ============== VWAP Profile ==============

def get_vwap_profile() -> List[float]:
    """
    Generate a realistic VWAP volume distribution profile for a trading day
    Returns hourly participation rates (9:00 to 17:30 CET)
    """
    # Typical European equity market volume distribution
    # Higher at open, lunch dip, pickup in afternoon, spike at close
    hourly_percentages = [
        0.12,  # 9:00-10:00 - Opening volume
        0.09,  # 10:00-11:00
        0.08,  # 11:00-12:00
        0.06,  # 12:00-13:00 - Lunch dip
        0.07,  # 13:00-14:00
        0.08,  # 14:00-15:00
        0.09,  # 15:00-16:00
        0.10,  # 16:00-17:00
        0.15,  # 17:00-17:30 - Closing auction preparation
        0.16,  # 17:30 - Closing auction
    ]
    return hourly_percentages

def get_intraday_volatility_pattern() -> List[float]:
    """Get typical intraday volatility pattern"""
    # Higher volatility at open and close
    return [1.5, 1.2, 1.0, 0.8, 0.9, 1.0, 1.1, 1.3, 1.6, 2.0]

# ============== Order Generation ==============

class VWAPAlgoSimulator:
    """Simulates VWAP algo execution"""
    
    def __init__(self, parent_order: Order, stock_info: Dict, start_time: datetime):
        self.parent_order = parent_order
        self.stock_info = stock_info
        self.start_time = start_time
        self.vwap_profile = get_vwap_profile()
        self.volatility_pattern = get_intraday_volatility_pattern()
        self.current_filled = 0
        self.orders = []
        self.fills = []
        
    def generate_execution(self) -> Tuple[List[Dict], List[Dict]]:
        """Generate child orders and fills for VWAP execution"""
        
        # Calculate slice sizes based on VWAP profile
        total_quantity = self.parent_order.quantity
        current_time = self.start_time
        
        # Add parent order to orders list
        parent_dict = asdict(self.parent_order)
        parent_dict['state'] = OrderState.ACCEPTED.value
        parent_dict['timestamp'] = current_time.isoformat()
        parent_dict['update_timestamp'] = current_time.isoformat()
        self.orders.append(parent_dict)
        
        # Generate child orders throughout the day
        for hour_idx, participation_rate in enumerate(self.vwap_profile):
            hour_quantity = int(total_quantity * participation_rate)
            
            if hour_quantity == 0:
                continue
                
            # Split hour quantity into multiple child orders (3-8 per hour)
            num_slices = random.randint(3, 8)
            slice_size = hour_quantity // num_slices
            
            for slice_idx in range(num_slices):
                if self.current_filled >= total_quantity:
                    break
                    
                # Calculate timing within the hour
                minutes_offset = (60 // num_slices) * slice_idx + random.randint(0, 5)
                order_time = current_time + timedelta(hours=hour_idx, minutes=minutes_offset)
                
                # Determine slice quantity
                remaining = total_quantity - self.current_filled
                slice_qty = min(slice_size + random.randint(-slice_size//4, slice_size//4), remaining)
                
                if slice_qty <= 0:
                    continue
                
                # Create child order
                child_order = self._create_child_order(slice_qty, order_time)
                self.orders.append(asdict(child_order))
                
                # Route to SOR and generate fills
                sor_orders, sor_fills = self._route_to_sor(child_order, order_time)
                self.orders.extend(sor_orders)
                self.fills.extend(sor_fills)
                
                self.current_filled += slice_qty
        
        # Update parent order status
        parent_dict['state'] = OrderState.FILLED.value if self.current_filled >= total_quantity else OrderState.PARTIALLY_FILLED.value
        parent_dict['filled_quantity'] = self.current_filled
        parent_dict['remaining_quantity'] = total_quantity - self.current_filled
        parent_dict['update_timestamp'] = (current_time + timedelta(hours=8, minutes=30)).isoformat()
        
        return self.orders, self.fills
    
    def _create_child_order(self, quantity: int, timestamp: datetime) -> Order:
        """Create a child order from parent"""
        return Order(
            order_id=f"ALGO_{self.parent_order.order_id}_{len(self.orders):04d}",
            parent_order_id=self.parent_order.order_id,
            client_order_id=f"CLI_{timestamp.timestamp():.0f}",
            ticker=self.parent_order.ticker,
            side=self.parent_order.side,
            quantity=quantity,
            filled_quantity=0,
            price=None,  # Market order
            order_type="Market",
            tif="IOC",
            state=OrderState.NEW.value,
            venue=None,
            algo_type=AlgoType.VWAP.value,
            timestamp=timestamp,
            update_timestamp=timestamp,
            client_name=self.parent_order.client_name,
            trader=self.parent_order.trader,
            desk=self.parent_order.desk,
            strategy="VWAP"
        )
    
    def _route_to_sor(self, child_order: Order, timestamp: datetime) -> Tuple[List[Dict], List[Dict]]:
        """Smart Order Router - splits order across venues"""
        sor_orders = []
        sor_fills = []
        
        # Venue selection based on liquidity and fees
        venue_distribution = {
            Venue.NYSE: 0.35,
            Venue.NASDAQ: 0.25,
            Venue.ARCA: 0.15,
            Venue.BATS: 0.10,
            Venue.DARK_POOL: 0.10,
            Venue.IEX: 0.05,
        }
        
        remaining_qty = child_order.quantity
        venues = list(venue_distribution.keys())
        random.shuffle(venues)
        
        for venue in venues[:random.randint(1, 4)]:  # Use 1-4 venues
            if remaining_qty <= 0:
                break
                
            # Calculate venue slice
            venue_qty = int(remaining_qty * venue_distribution[venue] * random.uniform(0.8, 1.2))
            venue_qty = min(venue_qty, remaining_qty)
            
            if venue_qty <= 0:
                continue
            
            # Create SOR child order
            sor_order = Order(
                order_id=f"SOR_{child_order.order_id}_{venue.value}",
                parent_order_id=child_order.order_id,
                client_order_id=f"SOR_{timestamp.timestamp():.0f}",
                ticker=child_order.ticker,
                side=child_order.side,
                quantity=venue_qty,
                filled_quantity=venue_qty,  # Assume immediate fill
                price=self._calculate_execution_price(timestamp),
                order_type="Market",
                tif="IOC",
                state=OrderState.FILLED.value,
                venue=venue.value,
                algo_type=None,
                timestamp=timestamp + timedelta(milliseconds=random.randint(1, 100)),
                update_timestamp=timestamp + timedelta(milliseconds=random.randint(100, 500)),
                client_name=child_order.client_name,
                trader=child_order.trader,
                desk=child_order.desk,
                strategy="SOR"
            )
            
            sor_orders.append(asdict(sor_order))
            
            # Generate fills (potentially multiple per SOR order)
            fill_count = random.randint(1, 3)
            fill_qty_per = venue_qty // fill_count
            
            for fill_idx in range(fill_count):
                fill_qty = fill_qty_per if fill_idx < fill_count - 1 else venue_qty - (fill_qty_per * fill_idx)
                
                fill = {
                    "fill_id": f"FILL_{sor_order.order_id}_{fill_idx:02d}",
                    "order_id": sor_order.order_id,
                    "parent_order_id": child_order.order_id,
                    "root_order_id": self.parent_order.order_id,
                    "ticker": sor_order.ticker,
                    "side": sor_order.side,
                    "quantity": fill_qty,
                    "price": sor_order.price + random.uniform(-0.01, 0.01),
                    "venue": venue.value,
                    "timestamp": (timestamp + timedelta(milliseconds=random.randint(100, 1000))).isoformat(),
                    "counterparty": random.choice(["MM01", "MM02", "HFT01", "BANK01", "FLOW01"]),
                    "commission": round(fill_qty * sor_order.price * 0.0002, 2),
                    "fees": round(fill_qty * sor_order.price * 0.00003, 2),
                }
                sor_fills.append(fill)
            
            remaining_qty -= venue_qty
        
        # Update child order
        child_order.filled_quantity = child_order.quantity - remaining_qty
        child_order.state = OrderState.FILLED.value if remaining_qty == 0 else OrderState.PARTIALLY_FILLED.value
        child_order.update_timestamp = timestamp + timedelta(seconds=1)
        
        return sor_orders, sor_fills
    
    def _calculate_execution_price(self, timestamp: datetime) -> float:
        """Calculate realistic execution price with slippage and spread"""
        base_price = self.stock_info['price']
        
        # Time-based volatility
        hour = timestamp.hour - 9  # Market opens at 9
        if hour < 0 or hour >= len(self.volatility_pattern):
            hour = 0
        volatility = self.volatility_pattern[hour]
        
        # Price movement with volatility
        price_move = random.gauss(0, 0.002 * volatility)  # 0.2% std dev * volatility
        
        # Add spread
        spread = base_price * 0.0005  # 5 bps spread
        
        # Side-based adjustment
        if self.parent_order.side == "Buy":
            price = base_price * (1 + price_move) + spread/2
        else:
            price = base_price * (1 + price_move) - spread/2
            
        return round(price, 2)

# ============== Data Generation Functions ==============

def generate_vwap_execution(
    ticker: str = None,
    quantity: int = None,
    side: str = None,
    client: str = None,
    start_time: datetime = None
) -> Tuple[List[Dict], List[Dict]]:
    """Generate a complete VWAP execution flow"""
    
    # Select random values if not provided
    stock = random.choice(EU_LARGE_CAP_STOCKS)
    if ticker:
        stock = next((s for s in EU_LARGE_CAP_STOCKS if s['ticker'] == ticker), stock)
    
    quantity = quantity or random.randint(50000, 500000)
    side = side or random.choice(["Buy", "Sell"])
    client = client or random.choice(CLIENTS)
    start_time = start_time or datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    
    # Create parent order
    parent_order = Order(
        order_id=f"ORD_{start_time.timestamp():.0f}_{random.randint(1000, 9999)}",
        parent_order_id=None,
        client_order_id=f"CLIENT_{random.randint(100000, 999999)}",
        ticker=stock['ticker'],
        side=side,
        quantity=quantity,
        filled_quantity=0,
        price=None,  # Market order
        order_type="Market",
        tif="Day",
        state=OrderState.PENDING.value,
        venue=None,
        algo_type=AlgoType.VWAP.value,
        timestamp=start_time,
        update_timestamp=start_time,
        client_name=client,
        trader=random.choice(TRADERS),
        desk=random.choice(DESKS),
        strategy="VWAP All Day"
    )
    
    # Run VWAP simulation
    simulator = VWAPAlgoSimulator(parent_order, stock, start_time)
    orders, fills = simulator.generate_execution()
    
    return orders, fills

def generate_instrument_reference_data(count: int = 100) -> List[Dict]:
    """Generate detailed instrument reference data"""
    
    instruments = []
    
    # Asset classes and their properties
    asset_classes = {
        "Equity": {
            "subtypes": ["Common Stock", "Preferred Stock", "ADR", "ETF"],
            "exchanges": ["NYSE", "NASDAQ", "LSE", "XETRA", "TSE"],
            "currencies": ["USD", "EUR", "GBP", "JPY", "CHF"],
        },
        "Fixed Income": {
            "subtypes": ["Government Bond", "Corporate Bond", "Municipal Bond", "MBS", "ABS"],
            "exchanges": ["OTC", "NYSE Bonds", "LSE", "EuroTLX"],
            "currencies": ["USD", "EUR", "GBP"],
        },
        "Derivative": {
            "subtypes": ["Option", "Future", "Swap", "Forward", "Swaption"],
            "exchanges": ["CME", "ICE", "EUREX", "LME", "OTC"],
            "currencies": ["USD", "EUR", "GBP", "JPY"],
        },
        "Commodity": {
            "subtypes": ["Energy", "Metal", "Agriculture"],
            "exchanges": ["CME", "ICE", "LME", "SHFE"],
            "currencies": ["USD", "EUR"],
        },
        "FX": {
            "subtypes": ["Spot", "Forward", "NDF", "Option"],
            "exchanges": ["OTC", "CME", "ICE"],
            "currencies": ["USD", "EUR", "GBP", "JPY", "CHF", "AUD", "CAD"],
        }
    }
    
    issuers = [
        "US Treasury", "German Bund", "UK Gilt", "Apple Inc", "Microsoft Corp",
        "JP Morgan Chase", "Goldman Sachs", "Deutsche Bank", "BNP Paribas",
        "Toyota Motor", "Nestle SA", "Royal Dutch Shell", "HSBC Holdings"
    ]
    
    for i in range(count):
        asset_class = random.choice(list(asset_classes.keys()))
        asset_info = asset_classes[asset_class]
        
        instrument = {
            # Identifiers
            "instrument_id": f"INST{i+1:06d}",
            "isin": f"{random.choice(['US', 'GB', 'DE', 'FR', 'JP'])}{random.randint(1000000000, 9999999999):010d}",
            "cusip": f"{random.randint(100000000, 999999999):09d}" if asset_class in ["Equity", "Fixed Income"] else None,
            "sedol": f"{random.randint(1000000, 9999999):07d}" if random.random() < 0.5 else None,
            "bloomberg_ticker": f"{random.choice(['AAPL', 'MSFT', 'JPM', 'GS', 'DBK', 'NESN', 'SHEL'])} {random.choice(['US', 'LN', 'GY', 'FP'])} {asset_class[:3].upper()}",
            "reuters_ric": f"{random.choice(['AAPL', 'MSFT', 'JPM', 'GS', 'DBK'])}{'.' + random.choice(['N', 'L', 'DE', 'PA']) if random.random() < 0.7 else ''}",
            
            # Classification
            "asset_class": asset_class,
            "instrument_type": random.choice(asset_info["subtypes"]),
            "sector": random.choice(["Technology", "Financials", "Healthcare", "Energy", "Consumer", "Industrials"]),
            "industry": random.choice(["Software", "Banking", "Pharma", "Oil & Gas", "Retail", "Aerospace"]),
            
            # Description
            "name": f"{random.choice(issuers)} {random.choice(asset_info['subtypes'])}",
            "description": f"{random.choice(asset_info['subtypes'])} issued by {random.choice(issuers)}",
            "issuer": random.choice(issuers),
            "issue_date": (datetime.now() - timedelta(days=random.randint(30, 3650))).strftime("%Y-%m-%d"),
            
            # Trading info
            "exchange": random.choice(asset_info["exchanges"]),
            "currency": random.choice(asset_info["currencies"]),
            "trading_currency": random.choice(asset_info["currencies"]),
            "settlement_currency": random.choice(asset_info["currencies"]),
            "tick_size": random.choice([0.01, 0.001, 0.0001, 0.00001]),
            "lot_size": random.choice([1, 10, 100, 1000]),
            "min_trade_size": random.choice([1, 10, 100, 1000]),
            
            # Market data
            "last_price": round(random.uniform(1, 1000), 2),
            "bid_price": round(random.uniform(1, 1000), 2),
            "ask_price": round(random.uniform(1, 1000), 2),
            "volume": random.randint(10000, 10000000),
            "market_cap": random.randint(1000000, 1000000000000) if asset_class == "Equity" else None,
            
            # Fixed Income specific
            "maturity_date": (datetime.now() + timedelta(days=random.randint(30, 10950))).strftime("%Y-%m-%d") if asset_class == "Fixed Income" else None,
            "coupon_rate": round(random.uniform(0, 10), 3) if asset_class == "Fixed Income" else None,
            "coupon_frequency": random.choice(["Annual", "Semi-Annual", "Quarterly"]) if asset_class == "Fixed Income" else None,
            "yield_to_maturity": round(random.uniform(0, 8), 3) if asset_class == "Fixed Income" else None,
            "duration": round(random.uniform(0.5, 30), 2) if asset_class == "Fixed Income" else None,
            "credit_rating": random.choice(["AAA", "AA+", "AA", "AA-", "A+", "A", "BBB+", "BBB"]) if asset_class == "Fixed Income" else None,
            
            # Derivative specific
            "underlying": f"INST{random.randint(1, count):06d}" if asset_class == "Derivative" else None,
            "strike_price": round(random.uniform(50, 200), 2) if "Option" in str(asset_info.get("subtypes", [])) else None,
            "expiry_date": (datetime.now() + timedelta(days=random.randint(1, 365))).strftime("%Y-%m-%d") if asset_class == "Derivative" else None,
            "contract_size": random.choice([100, 1000, 10000]) if asset_class == "Derivative" else None,
            
            # Risk metrics
            "var_95": round(random.uniform(1000, 100000), 2),
            "var_99": round(random.uniform(2000, 200000), 2),
            "beta": round(random.uniform(0.5, 2.0), 3) if asset_class == "Equity" else None,
            "sharpe_ratio": round(random.uniform(-1, 3), 3) if asset_class == "Equity" else None,
            
            # Status
            "status": random.choice(["Active", "Active", "Active", "Suspended", "Delisted"]),
            "is_tradeable": random.random() < 0.95,
            "last_updated": datetime.now().isoformat(),
        }
        
        instruments.append(instrument)
    
    return instruments

def generate_tick_data(
    ticker: str,
    hours: float = 1.0,
    ticks_per_second: int = 10
) -> List[Dict]:
    """Generate realistic tick data for a given ticker"""
    
    ticks = []
    
    # Get stock info
    stock = next((s for s in EU_LARGE_CAP_STOCKS if s['ticker'] == ticker), EU_LARGE_CAP_STOCKS[0])
    base_price = stock['price']
    
    # Start time
    start_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    end_time = start_time + timedelta(hours=hours)
    
    # Generate ticks
    current_time = start_time
    current_price = base_price
    bid_price = base_price - 0.01
    ask_price = base_price + 0.01
    
    while current_time < end_time:
        # Random walk with mean reversion
        price_change = random.gauss(0, 0.0001) - (current_price - base_price) * 0.001
        current_price += price_change
        
        # Update bid/ask
        spread = random.uniform(0.01, 0.03)
        bid_price = current_price - spread/2
        ask_price = current_price + spread/2
        
        # Generate trades at this price level
        num_trades = random.randint(0, 5)
        
        for _ in range(num_trades):
            tick = {
                "timestamp": current_time.isoformat(),
                "ticker": ticker,
                "price": round(current_price + random.uniform(-0.005, 0.005), 3),
                "bid": round(bid_price, 3),
                "ask": round(ask_price, 3),
                "bid_size": random.randint(100, 10000),
                "ask_size": random.randint(100, 10000),
                "volume": random.randint(100, 5000),
                "trade_type": random.choice(["Buy", "Sell", "Cross"]),
                "condition": random.choice(["Regular", "Block", "Odd Lot"]),
                "exchange": random.choice(["NYSE", "NASDAQ", "ARCA", "BATS"])
            }
            ticks.append(tick)
            
            current_time += timedelta(milliseconds=random.randint(10, 100))
        
        # Move to next tick interval
        current_time += timedelta(milliseconds=1000 // ticks_per_second)
    
    return ticks

# ============== Main Entry Point ==============

def main():
    parser = argparse.ArgumentParser(
        description="Advanced Financial Data Generator",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate VWAP algo execution
  python generate_financial_data.py --mode vwap --ticker ASML.AS --quantity 100000
  
  # Generate instrument reference data
  python generate_financial_data.py --mode instruments --count 500
  
  # Generate tick data
  python generate_financial_data.py --mode ticks --ticker ASML.AS --hours 2
  
  # Generate general trades (legacy mode)
  python generate_financial_data.py --mode general --count 1000
        """
    )
    
    parser.add_argument(
        "--mode",
        choices=["vwap", "instruments", "ticks", "general"],
        default="vwap",
        help="Data generation mode"
    )
    
    parser.add_argument("--count", type=int, default=100, help="Number of records to generate")
    parser.add_argument("--ticker", type=str, help="Ticker symbol for VWAP/tick generation")
    parser.add_argument("--quantity", type=int, help="Order quantity for VWAP")
    parser.add_argument("--side", choices=["Buy", "Sell"], help="Order side")
    parser.add_argument("--client", type=str, help="Client name")
    parser.add_argument("--hours", type=float, default=1.0, help="Hours of tick data")
    parser.add_argument("--format", choices=["json", "csv"], default="json", help="Output format")
    parser.add_argument("--output", type=str, help="Output filename")
    
    args = parser.parse_args()
    
    # Generate data based on mode
    if args.mode == "vwap":
        print(f"Generating VWAP execution flow...")
        orders, fills = generate_vwap_execution(
            ticker=args.ticker,
            quantity=args.quantity,
            side=args.side,
            client=args.client
        )
        
        # Save orders and fills
        output_base = args.output or f"vwap_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        
        # Save orders
        orders_file = f"{output_base}_orders.{args.format}"
        if args.format == "json":
            with open(orders_file, 'w') as f:
                json.dump(orders, f, indent=2, default=str)
        else:
            with open(orders_file, 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=orders[0].keys())
                writer.writeheader()
                writer.writerows(orders)
        
        # Save fills
        fills_file = f"{output_base}_fills.{args.format}"
        if args.format == "json":
            with open(fills_file, 'w') as f:
                json.dump(fills, f, indent=2, default=str)
        else:
            with open(fills_file, 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=fills[0].keys())
                writer.writeheader()
                writer.writerows(fills)
        
        print(f"✅ Generated {len(orders)} orders in {orders_file}")
        print(f"✅ Generated {len(fills)} fills in {fills_file}")
        
        # Summary
        total_filled = sum(f['quantity'] for f in fills)
        avg_price = sum(f['quantity'] * f['price'] for f in fills) / total_filled if total_filled > 0 else 0
        print(f"\n📊 Execution Summary:")
        print(f"  Total Filled: {total_filled:,}")
        print(f"  Average Price: {avg_price:.2f}")
        print(f"  Child Orders: {len([o for o in orders if o.get('parent_order_id')])}")
        print(f"  Venues Used: {len(set(f['venue'] for f in fills))}")
        
    elif args.mode == "instruments":
        print(f"Generating {args.count} instrument reference records...")
        instruments = generate_instrument_reference_data(args.count)
        
        output_file = args.output or f"instruments_{args.count}.{args.format}"
        
        if args.format == "json":
            with open(output_file, 'w') as f:
                json.dump(instruments, f, indent=2, default=str)
        else:
            with open(output_file, 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=instruments[0].keys())
                writer.writeheader()
                writer.writerows(instruments)
        
        print(f"✅ Generated {len(instruments)} instruments in {output_file}")
        
        # Summary by asset class
        asset_classes = {}
        for inst in instruments:
            ac = inst['asset_class']
            asset_classes[ac] = asset_classes.get(ac, 0) + 1
        
        print(f"\n📊 Asset Class Distribution:")
        for ac, count in sorted(asset_classes.items()):
            print(f"  {ac}: {count}")
    
    elif args.mode == "ticks":
        ticker = args.ticker or "ASML.AS"
        print(f"Generating {args.hours} hours of tick data for {ticker}...")
        
        ticks = generate_tick_data(ticker, args.hours)
        
        output_file = args.output or f"ticks_{ticker.replace('.', '_')}_{args.hours}h.{args.format}"
        
        if args.format == "json":
            with open(output_file, 'w') as f:
                json.dump(ticks, f, indent=2, default=str)
        else:
            with open(output_file, 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=ticks[0].keys())
                writer.writeheader()
                writer.writerows(ticks)
        
        print(f"✅ Generated {len(ticks)} ticks in {output_file}")
        
        # Summary
        if ticks:
            prices = [t['price'] for t in ticks]
            print(f"\n📊 Tick Summary:")
            print(f"  Price Range: {min(prices):.2f} - {max(prices):.2f}")
            print(f"  Avg Price: {sum(prices)/len(prices):.2f}")
            print(f"  Total Volume: {sum(t['volume'] for t in ticks):,}")
    
    elif args.mode == "general":
        # Import and use the original generate_trades function
        print(f"Generating {args.count} general trade records...")
        print("(Using legacy generator from generate_trades.py)")
        
        # Fallback to simple generation
        from generate_trades import generate_trades
        trades = generate_trades(args.count)
        
        output_file = args.output or f"trades_{args.count}.{args.format}"
        
        if args.format == "json":
            with open(output_file, 'w') as f:
                json.dump(trades, f, indent=2, default=str)
        else:
            with open(output_file, 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=trades[0].keys())
                writer.writeheader()
                writer.writerows(trades)
        
        print(f"✅ Generated {len(trades)} trades in {output_file}")

if __name__ == "__main__":
    main()