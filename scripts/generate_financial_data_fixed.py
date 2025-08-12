#!/usr/bin/env python3
"""
Fixed Financial Data Generator with proper order hierarchy
Client Order ID cascades through entire order chain
"""

import json
import csv
import random
import sys
import argparse
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum

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

@dataclass
class Order:
    """Order data structure with proper hierarchy"""
    order_id: str                    # This order's unique ID
    parent_order_id: Optional[str]   # Points to immediate parent
    client_order_id: str             # Original client ID - NEVER CHANGES
    root_order_id: str               # The top-level parent order ID
    
    ticker: str
    side: str
    quantity: int
    filled_quantity: int
    remaining_quantity: int
    price: Optional[float]
    order_type: str
    tif: str
    state: str
    venue: Optional[str]
    algo_type: Optional[str]
    timestamp: datetime
    update_timestamp: datetime
    
    # Additional fields
    average_price: float = 0.0
    commission: float = 0.0
    client_name: Optional[str] = None
    trader: Optional[str] = None
    desk: Optional[str] = None
    strategy: Optional[str] = None
    order_level: int = 0  # 0=client, 1=algo parent, 2=algo child, 3=SOR child
    
    def __post_init__(self):
        self.remaining_quantity = self.quantity - self.filled_quantity

# Market data
EU_LARGE_CAP_STOCKS = [
    {"ticker": "ASML.AS", "name": "ASML Holding", "price": 650.0, "daily_volume": 2500000},
    {"ticker": "NESN.VX", "name": "Nestle", "price": 105.0, "daily_volume": 3000000},
    {"ticker": "SAP.DE", "name": "SAP", "price": 140.0, "daily_volume": 2200000},
]

CLIENTS = [
    "Blackrock Asset Management", "Vanguard Group", "Fidelity Investments"
]

TRADERS = [f"TRD{i:03d}" for i in range(1, 21)]
DESKS = ["Equity Trading", "Program Trading", "Electronic Trading"]

def get_vwap_profile() -> List[float]:
    """VWAP volume distribution for a trading day"""
    return [
        0.12,  # 9:00-10:00 - Opening
        0.09,  # 10:00-11:00
        0.08,  # 11:00-12:00
        0.06,  # 12:00-13:00 - Lunch
        0.07,  # 13:00-14:00
        0.08,  # 14:00-15:00
        0.09,  # 15:00-16:00
        0.10,  # 16:00-17:00
        0.15,  # 17:00-17:30 - Close prep
        0.16,  # 17:30 - Closing auction
    ]

class VWAPAlgoSimulator:
    """Simulates VWAP with proper order hierarchy"""
    
    def __init__(self, client_order_id: str, ticker: str, quantity: int, 
                 side: str, client_name: str, start_time: datetime):
        self.client_order_id = client_order_id  # This NEVER changes
        self.ticker = ticker
        self.quantity = quantity
        self.side = side
        self.client_name = client_name
        self.start_time = start_time
        self.vwap_profile = get_vwap_profile()
        
        self.orders = []
        self.fills = []
        self.order_counter = 0
        
    def generate_execution(self) -> Tuple[List[Dict], List[Dict]]:
        """Generate complete execution with proper hierarchy"""
        
        current_time = self.start_time
        
        # 1. CLIENT ORDER (Level 0) - Original from buy-side client
        client_order = Order(
            order_id=f"CLIENT_{self.client_order_id}",
            parent_order_id=None,  # No parent - this is the root
            client_order_id=self.client_order_id,  # Client's original ID
            root_order_id=f"CLIENT_{self.client_order_id}",  # Points to itself
            ticker=self.ticker,
            side=self.side,
            quantity=self.quantity,
            filled_quantity=0,
            remaining_quantity=self.quantity,
            price=None,
            order_type="Market",
            tif="Day",
            state=OrderState.PENDING.value,
            venue=None,
            algo_type=None,
            timestamp=current_time,
            update_timestamp=current_time,
            client_name=self.client_name,
            trader=random.choice(TRADERS),
            desk=random.choice(DESKS),
            strategy="VWAP",
            order_level=0
        )
        self.orders.append(asdict(client_order))
        
        # 2. ALGO PARENT ORDER (Level 1) - Sell-side creates algo order
        current_time += timedelta(seconds=1)
        algo_parent_id = f"ALGO_{current_time.timestamp():.0f}"
        algo_parent = Order(
            order_id=algo_parent_id,
            parent_order_id=client_order.order_id,  # Points to client order
            client_order_id=self.client_order_id,  # PRESERVES client ID
            root_order_id=client_order.order_id,    # Root is client order
            ticker=self.ticker,
            side=self.side,
            quantity=self.quantity,
            filled_quantity=0,
            remaining_quantity=self.quantity,
            price=None,
            order_type="Market",
            tif="Day",
            state=OrderState.ACCEPTED.value,
            venue=None,
            algo_type="VWAP",
            timestamp=current_time,
            update_timestamp=current_time,
            client_name=self.client_name,
            trader=client_order.trader,
            desk=client_order.desk,
            strategy="VWAP",
            order_level=1
        )
        self.orders.append(asdict(algo_parent))
        
        # Update client order state
        self.orders[0]['state'] = OrderState.ACCEPTED.value
        self.orders[0]['update_timestamp'] = current_time.isoformat()
        
        # 3. Generate ALGO CHILD ORDERS (Level 2) throughout the day
        total_filled = 0
        
        for hour_idx, participation_rate in enumerate(self.vwap_profile):
            hour_quantity = int(self.quantity * participation_rate)
            
            if hour_quantity == 0 or total_filled >= self.quantity:
                continue
            
            # Create multiple slices per hour
            num_slices = random.randint(2, 5)
            slice_size = hour_quantity // num_slices
            
            for slice_idx in range(num_slices):
                if total_filled >= self.quantity:
                    break
                
                # Calculate timing
                minutes_offset = (60 // num_slices) * slice_idx + random.randint(0, 5)
                order_time = self.start_time + timedelta(hours=hour_idx, minutes=minutes_offset)
                
                # Determine slice quantity
                remaining = self.quantity - total_filled
                slice_qty = min(slice_size + random.randint(-slice_size//4, slice_size//4), remaining)
                
                if slice_qty <= 0:
                    continue
                
                # Create ALGO CHILD ORDER (Level 2)
                algo_child_id = f"ALGOCHILD_{order_time.timestamp():.0f}_{self.order_counter}"
                self.order_counter += 1
                
                algo_child = Order(
                    order_id=algo_child_id,
                    parent_order_id=algo_parent_id,  # Points to algo parent
                    client_order_id=self.client_order_id,  # PRESERVES client ID
                    root_order_id=client_order.order_id,   # Root is still client order
                    ticker=self.ticker,
                    side=self.side,
                    quantity=slice_qty,
                    filled_quantity=0,
                    remaining_quantity=slice_qty,
                    price=None,
                    order_type="Market",
                    tif="IOC",
                    state=OrderState.NEW.value,
                    venue=None,
                    algo_type="VWAP",
                    timestamp=order_time,
                    update_timestamp=order_time,
                    client_name=self.client_name,
                    trader=client_order.trader,
                    desk=client_order.desk,
                    strategy="VWAP Slice",
                    order_level=2
                )
                self.orders.append(asdict(algo_child))
                
                # 4. Route to SOR - create SOR CHILD ORDERS (Level 3)
                sor_orders, sor_fills = self._route_to_sor(algo_child, order_time)
                self.orders.extend(sor_orders)
                self.fills.extend(sor_fills)
                
                # Update algo child with fills
                child_filled = sum(f['quantity'] for f in sor_fills)
                algo_child_dict = self.orders[-len(sor_orders)-1]  # Get the algo child we just added
                algo_child_dict['filled_quantity'] = child_filled
                algo_child_dict['remaining_quantity'] = slice_qty - child_filled
                algo_child_dict['state'] = OrderState.FILLED.value if child_filled >= slice_qty else OrderState.PARTIALLY_FILLED.value
                algo_child_dict['update_timestamp'] = (order_time + timedelta(seconds=1)).isoformat()
                
                total_filled += child_filled
        
        # 5. Update parent orders with final fills
        # Update algo parent
        for order in self.orders:
            if order['order_id'] == algo_parent_id:
                order['filled_quantity'] = total_filled
                order['remaining_quantity'] = self.quantity - total_filled
                order['state'] = OrderState.FILLED.value if total_filled >= self.quantity else OrderState.PARTIALLY_FILLED.value
                order['update_timestamp'] = (self.start_time + timedelta(hours=8, minutes=30)).isoformat()
                break
        
        # Update client order
        self.orders[0]['filled_quantity'] = total_filled
        self.orders[0]['remaining_quantity'] = self.quantity - total_filled
        self.orders[0]['state'] = OrderState.FILLED.value if total_filled >= self.quantity else OrderState.PARTIALLY_FILLED.value
        self.orders[0]['update_timestamp'] = (self.start_time + timedelta(hours=8, minutes=30)).isoformat()
        
        return self.orders, self.fills
    
    def _route_to_sor(self, algo_child: Order, timestamp: datetime) -> Tuple[List[Dict], List[Dict]]:
        """Smart Order Router - splits to venues"""
        sor_orders = []
        sor_fills = []
        
        venue_distribution = {
            Venue.NYSE: 0.35,
            Venue.NASDAQ: 0.25,
            Venue.ARCA: 0.15,
            Venue.BATS: 0.10,
            Venue.DARK_POOL: 0.10,
            Venue.IEX: 0.05,
        }
        
        remaining_qty = algo_child.quantity
        venues = list(venue_distribution.keys())
        random.shuffle(venues)
        
        for venue in venues[:random.randint(1, 3)]:  # Use 1-3 venues
            if remaining_qty <= 0:
                break
            
            venue_qty = int(remaining_qty * venue_distribution[venue] * random.uniform(0.8, 1.2))
            venue_qty = min(venue_qty, remaining_qty)
            
            if venue_qty <= 0:
                continue
            
            # Create SOR CHILD ORDER (Level 3)
            sor_order_id = f"SOR_{timestamp.timestamp():.0f}_{venue.value}_{self.order_counter}"
            self.order_counter += 1
            
            sor_order = Order(
                order_id=sor_order_id,
                parent_order_id=algo_child.order_id,  # Points to algo child
                client_order_id=self.client_order_id,  # PRESERVES client ID
                root_order_id=algo_child.root_order_id,  # Root is still client order
                ticker=algo_child.ticker,
                side=algo_child.side,
                quantity=venue_qty,
                filled_quantity=venue_qty,  # Assume immediate fill
                remaining_quantity=0,
                price=self._calculate_execution_price(timestamp),
                order_type="Market",
                tif="IOC",
                state=OrderState.FILLED.value,
                venue=venue.value,
                algo_type=None,
                timestamp=timestamp + timedelta(milliseconds=random.randint(1, 100)),
                update_timestamp=timestamp + timedelta(milliseconds=random.randint(100, 500)),
                client_name=algo_child.client_name,
                trader=algo_child.trader,
                desk=algo_child.desk,
                strategy="SOR",
                order_level=3
            )
            
            sor_orders.append(asdict(sor_order))
            
            # Generate fills
            fill = {
                "fill_id": f"FILL_{sor_order_id}",
                "order_id": sor_order.order_id,
                "parent_order_id": algo_child.order_id,
                "client_order_id": self.client_order_id,  # PRESERVES client ID
                "root_order_id": algo_child.root_order_id,
                "ticker": sor_order.ticker,
                "side": sor_order.side,
                "quantity": venue_qty,
                "price": sor_order.price,
                "venue": venue.value,
                "timestamp": (timestamp + timedelta(milliseconds=random.randint(100, 1000))).isoformat(),
                "counterparty": random.choice(["MM01", "MM02", "HFT01"]),
                "commission": round(venue_qty * sor_order.price * 0.0002, 2),
                "fees": round(venue_qty * sor_order.price * 0.00003, 2),
            }
            sor_fills.append(fill)
            
            remaining_qty -= venue_qty
        
        return sor_orders, sor_fills
    
    def _calculate_execution_price(self, timestamp: datetime) -> float:
        """Calculate realistic execution price"""
        base_price = 650.0  # Default for ASML
        price_move = random.gauss(0, 0.002)
        spread = base_price * 0.0005
        
        if self.side == "Buy":
            price = base_price * (1 + price_move) + spread/2
        else:
            price = base_price * (1 + price_move) - spread/2
        
        return round(price, 2)

def generate_vwap_execution_fixed(
    client_order_id: str = None,
    ticker: str = "ASML.AS",
    quantity: int = 100000,
    side: str = "Buy",
    client: str = "Blackrock Asset Management",
    start_time: datetime = None
) -> Tuple[List[Dict], List[Dict]]:
    """Generate VWAP execution with proper order hierarchy"""
    
    client_order_id = client_order_id or f"C{random.randint(100000, 999999)}"
    start_time = start_time or datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    
    simulator = VWAPAlgoSimulator(
        client_order_id=client_order_id,
        ticker=ticker,
        quantity=quantity,
        side=side,
        client_name=client,
        start_time=start_time
    )
    
    return simulator.generate_execution()

def main():
    parser = argparse.ArgumentParser(description="Generate VWAP execution with proper hierarchy")
    parser.add_argument("--client-order-id", default="C123456", help="Client order ID")
    parser.add_argument("--ticker", default="ASML.AS", help="Ticker symbol")
    parser.add_argument("--quantity", type=int, default=100000, help="Order quantity")
    parser.add_argument("--side", choices=["Buy", "Sell"], default="Buy", help="Order side")
    parser.add_argument("--client", default="Blackrock Asset Management", help="Client name")
    parser.add_argument("--output", default="vwap_fixed", help="Output filename base")
    
    args = parser.parse_args()
    
    print(f"Generating VWAP execution with proper hierarchy...")
    print(f"Client Order ID: {args.client_order_id} (will cascade to all children)")
    
    orders, fills = generate_vwap_execution_fixed(
        client_order_id=args.client_order_id,
        ticker=args.ticker,
        quantity=args.quantity,
        side=args.side,
        client=args.client
    )
    
    # Save orders
    orders_file = f"{args.output}_orders.json"
    with open(orders_file, 'w') as f:
        json.dump(orders, f, indent=2, default=str)
    
    # Save fills
    fills_file = f"{args.output}_fills.json"
    with open(fills_file, 'w') as f:
        json.dump(fills, f, indent=2, default=str)
    
    print(f"\n✅ Generated {len(orders)} orders in {orders_file}")
    print(f"✅ Generated {len(fills)} fills in {fills_file}")
    
    # Show hierarchy
    print(f"\n📊 Order Hierarchy:")
    level_counts = {}
    for order in orders:
        level = order.get('order_level', 0)
        level_counts[level] = level_counts.get(level, 0) + 1
    
    print(f"  Level 0 (Client):     {level_counts.get(0, 0)} orders")
    print(f"  Level 1 (Algo Parent): {level_counts.get(1, 0)} orders")
    print(f"  Level 2 (Algo Child):  {level_counts.get(2, 0)} orders")
    print(f"  Level 3 (SOR Child):   {level_counts.get(3, 0)} orders")
    
    # Verify client_order_id cascades
    client_ids = set(o['client_order_id'] for o in orders)
    print(f"\n✅ Client Order ID preserved: {len(client_ids)} unique = {'YES' if len(client_ids) == 1 else 'NO'}")
    if len(client_ids) == 1:
        print(f"  All orders have client_order_id: {list(client_ids)[0]}")

if __name__ == "__main__":
    main()