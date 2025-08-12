#!/usr/bin/env python3
"""
Trading System Simulator V2 - Realistic State Transitions
Models proper order lifecycle: PENDING → ACCEPTED → FILLED
Includes market price tracking and proper event propagation
"""

import json
import csv
import random
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass
from enum import Enum
import copy

# ============== ORDER STATES ==============
class OrderState(Enum):
    """Order lifecycle states"""
    PENDING = "PENDING"
    ACCEPTED = "ACCEPTED" 
    REJECTED = "REJECTED"
    WORKING = "WORKING"
    PARTIALLY_FILLED = "PARTIALLY_FILLED"
    FILLED = "FILLED"
    CANCELLED = "CANCELLED"
    PENDING_CANCEL = "PENDING_CANCEL"

# ============== MARKET SIMULATOR ==============
class MarketSimulator:
    """Simulates market prices"""
    
    def __init__(self, base_price: float = 650.0):
        self.base_price = base_price
        self.current_price = base_price
        self.bid = base_price - 0.01
        self.ask = base_price + 0.01
        
    def tick(self) -> Tuple[float, float, float]:
        """Generate new market tick"""
        # Random walk with mean reversion
        move = random.gauss(0, 0.1) - (self.current_price - self.base_price) * 0.01
        self.current_price += move
        
        # Update bid/ask
        spread = random.uniform(0.01, 0.05)
        self.bid = self.current_price - spread/2
        self.ask = self.current_price + spread/2
        
        return round(self.current_price, 2), round(self.bid, 2), round(self.ask, 2)

# ============== TICK DATABASE ==============
class TickDatabase:
    """Central tick database that captures all order updates"""
    
    def __init__(self):
        self.snapshots = []
        self.record_counter = 0
        
    def capture_snapshot(self, order: Dict, event_type: str, timestamp: datetime, 
                         market_price: float = None):
        """Capture a snapshot of an order at a point in time"""
        snapshot = copy.deepcopy(order)
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f"REC_{self.record_counter:08d}"
        if market_price:
            snapshot['market_price'] = market_price
        self.record_counter += 1
        self.snapshots.append(snapshot)
        
        # Log output
        level_names = {0: 'CLIENT', 1: 'ALGO', 2: 'SLICE', 3: 'ROUTE'}
        level_name = level_names.get(order.get('order_level', -1), 'UNKNOWN')
        
        print(f"  📸 [{timestamp.strftime('%H:%M:%S.%f')[:-3]}] {event_type:12} "
              f"L{order.get('order_level', '?')}:{level_name:6} {order['order_id']:20} "
              f"({order['filled_quantity']:,}/{order['quantity']:,}) [{order['state']}]")
        
    def export(self, filename: str):
        """Export tick database to CSV/JSON"""
        with open(f"{filename}.json", 'w') as f:
            json.dump(self.snapshots, f, indent=2, default=str)
        
        if self.snapshots:
            with open(f"{filename}.csv", 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=self.snapshots[0].keys())
                writer.writeheader()
                writer.writerows(self.snapshots)
        
        return len(self.snapshots)

# ============== TRADING COMPONENTS ==============

class Venue:
    """Exchange venue that executes orders"""
    
    def __init__(self, name: str, tick_db: TickDatabase, market: MarketSimulator):
        self.name = name
        self.tick_db = tick_db
        self.market = market
        self.orders = {}
        
    def receive_order(self, order: Dict, timestamp: datetime) -> Dict:
        """Venue receives order - goes PENDING → ACCEPTED → FILLED"""
        market_price, bid, ask = self.market.tick()
        
        # Store order
        self.orders[order['order_id']] = order
        
        # Step 1: Order arrives at venue - PENDING
        order['venue_state'] = 'PENDING'
        self.tick_db.capture_snapshot(order, 'VENUE_PENDING', timestamp, market_price)
        
        # Step 2: Venue accepts order - ACCEPTED
        timestamp += timedelta(milliseconds=random.randint(5, 20))
        order['state'] = OrderState.ACCEPTED.value
        order['venue_state'] = 'ACCEPTED'
        order['update_time'] = timestamp.isoformat()
        self.tick_db.capture_snapshot(order, 'VENUE_ACCEPTED', timestamp, market_price)
        
        # Step 3: Execute order - FILLED
        timestamp += timedelta(milliseconds=random.randint(20, 100))
        market_price, bid, ask = self.market.tick()
        
        # Determine fill price based on side
        if order['side'] == 'Buy':
            fill_price = ask + random.uniform(0, 0.02)  # Pay the spread
        else:
            fill_price = bid - random.uniform(0, 0.02)
            
        order['filled_quantity'] = order['quantity']
        order['remaining_quantity'] = 0
        order['average_price'] = round(fill_price, 4)
        order['state'] = OrderState.FILLED.value
        order['venue_state'] = 'FILLED'
        order['fill_time'] = timestamp.isoformat()
        order['update_time'] = timestamp.isoformat()
        
        self.tick_db.capture_snapshot(order, 'VENUE_FILLED', timestamp, market_price)
        
        print(f"    💹 {self.name}: Executed {order['quantity']:,} @ {fill_price:.2f} (market: {market_price:.2f})")
        
        return {
            'venue': self.name,
            'order_id': order['order_id'],
            'quantity': order['quantity'],
            'price': fill_price,
            'market_price': market_price,
            'timestamp': timestamp
        }

class SmartOrderRouter:
    """SOR that routes orders to venues"""
    
    def __init__(self, tick_db: TickDatabase, venues: Dict[str, Venue], market: MarketSimulator):
        self.tick_db = tick_db
        self.venues = venues
        self.market = market
        self.order_counter = 5000
        self.orders = {}
        
    def route_order(self, algo_child: Dict, timestamp: datetime) -> List[Dict]:
        """Route order to multiple venues - each child goes PENDING → ACCEPTED → FILLED"""
        
        # Get current market price
        market_price, _, _ = self.market.tick()
        
        # SOR receives the algo child order
        algo_child['state'] = OrderState.WORKING.value
        self.tick_db.capture_snapshot(algo_child, 'SOR_RECEIVED', timestamp, market_price)
        
        # Decide venue routing (split equally for simplicity)
        selected_venues = random.sample(list(self.venues.keys()), min(3, len(self.venues)))
        qty_per_venue = algo_child['quantity'] // len(selected_venues)
        remainder = algo_child['quantity'] % len(selected_venues)
        
        print(f"\n  🔀 SOR: Routing {algo_child['order_id']} ({algo_child['quantity']:,} shares) to {selected_venues}")
        
        sor_orders = []
        fills = []
        total_filled = 0
        total_value = 0
        
        for idx, venue_name in enumerate(selected_venues):
            timestamp += timedelta(milliseconds=10)
            market_price, _, _ = self.market.tick()
            
            # Add remainder to last venue
            venue_qty = qty_per_venue + (remainder if idx == len(selected_venues) - 1 else 0)
            
            # Create SOR child order
            sor_order = {
                'order_id': f"SOR_{self.order_counter:05d}",
                'parent_order_id': algo_child['order_id'],
                'client_order_id': algo_child['client_order_id'],
                'order_level': 3,  # SOR level
                'order_type': 'ROUTE',
                'ticker': algo_child['ticker'],
                'side': algo_child['side'],
                'quantity': venue_qty,
                'filled_quantity': 0,
                'remaining_quantity': venue_qty,
                'average_price': 0.0,
                'state': OrderState.PENDING.value,
                'algo_strategy': None,
                'venue': venue_name,
                'trader': 'SOR',
                'desk': 'Smart Router',
                'client_name': algo_child['client_name'],
                'create_time': timestamp.isoformat(),
                'update_time': timestamp.isoformat()
            }
            
            self.order_counter += 1
            self.orders[sor_order['order_id']] = sor_order
            
            # Capture NEW order
            self.tick_db.capture_snapshot(sor_order, 'NEW', timestamp, market_price)
            
            # Send to venue and get fill
            venue = self.venues[venue_name]
            fill = venue.receive_order(sor_order, timestamp + timedelta(milliseconds=5))
            
            # Update our copy with venue's fill
            sor_order['filled_quantity'] = fill['quantity']
            sor_order['remaining_quantity'] = 0
            sor_order['average_price'] = fill['price']
            sor_order['state'] = OrderState.FILLED.value
            sor_order['update_time'] = fill['timestamp'].isoformat()
            
            # Capture SOR's view of the fill
            self.tick_db.capture_snapshot(sor_order, 'SOR_FILLED', fill['timestamp'], fill['market_price'])
            
            sor_orders.append(sor_order)
            fills.append(fill)
            
            total_filled += fill['quantity']
            total_value += fill['quantity'] * fill['price']
            
            timestamp = fill['timestamp']
        
        # Update algo child order with aggregated fills
        timestamp += timedelta(milliseconds=10)
        algo_child['filled_quantity'] = total_filled
        algo_child['remaining_quantity'] = algo_child['quantity'] - total_filled
        algo_child['average_price'] = round(total_value / total_filled, 4) if total_filled > 0 else 0
        algo_child['state'] = OrderState.FILLED.value if total_filled >= algo_child['quantity'] else OrderState.PARTIALLY_FILLED.value
        algo_child['update_time'] = timestamp.isoformat()
        
        # SOR reports back to algo
        market_price, _, _ = self.market.tick()
        self.tick_db.capture_snapshot(algo_child, 'SOR_COMPLETE', timestamp, market_price)
        
        return sor_orders, fills, timestamp

class AlgoEngine:
    """Algo engine that executes VWAP strategy"""
    
    def __init__(self, tick_db: TickDatabase, sor: SmartOrderRouter, market: MarketSimulator):
        self.tick_db = tick_db
        self.sor = sor
        self.market = market
        self.parent_orders = {}
        self.child_orders = {}
        self.order_counter = 1000
        
    def receive_order(self, client_order: Dict, timestamp: datetime) -> Dict:
        """Receive order from client and create algo parent"""
        market_price, _, _ = self.market.tick()
        
        # Update client order state
        client_order['state'] = OrderState.ACCEPTED.value
        client_order['update_time'] = timestamp.isoformat()
        self.tick_db.capture_snapshot(client_order, 'ACCEPTED', timestamp, market_price)
        
        # Create algo parent order
        algo_parent = {
            'order_id': f"ALGO_{self.order_counter:05d}",
            'parent_order_id': client_order['order_id'],
            'client_order_id': client_order['client_order_id'],
            'order_level': 1,
            'order_type': 'ALGO_PARENT',
            'ticker': client_order['ticker'],
            'side': client_order['side'],
            'quantity': client_order['quantity'],
            'filled_quantity': 0,
            'remaining_quantity': client_order['quantity'],
            'average_price': 0.0,
            'state': OrderState.WORKING.value,
            'algo_strategy': 'VWAP',
            'venue': None,
            'trader': 'ALGO',
            'desk': 'Algo Trading',
            'client_name': client_order['client_name'],
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        
        self.order_counter += 1
        self.parent_orders[algo_parent['order_id']] = {
            'order': algo_parent,
            'client_order': client_order,
            'fills': [],
            'child_count': 0,
            'filled_children': 0
        }
        
        self.tick_db.capture_snapshot(algo_parent, 'NEW', timestamp, market_price)
        
        print(f"\n🤖 ALGO: Created VWAP parent {algo_parent['order_id']} for {algo_parent['quantity']:,} shares")
        return algo_parent
    
    def generate_child_slice(self, parent_id: str, slice_qty: int, timestamp: datetime) -> Tuple[Dict, datetime]:
        """Generate child order slice and route to SOR"""
        market_price, _, _ = self.market.tick()
        
        parent_data = self.parent_orders[parent_id]
        parent = parent_data['order']
        parent_data['child_count'] += 1
        
        # Create algo child order
        child = {
            'order_id': f"SLICE_{self.order_counter:05d}",
            'parent_order_id': parent_id,
            'client_order_id': parent['client_order_id'],
            'order_level': 2,
            'order_type': 'ALGO_SLICE',
            'ticker': parent['ticker'],
            'side': parent['side'],
            'quantity': slice_qty,
            'filled_quantity': 0,
            'remaining_quantity': slice_qty,
            'average_price': 0.0,
            'state': OrderState.PENDING.value,
            'algo_strategy': 'VWAP_SLICE',
            'venue': None,
            'trader': 'ALGO',
            'desk': 'Algo Trading',
            'client_name': parent['client_name'],
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        
        self.order_counter += 1
        self.child_orders[child['order_id']] = {
            'order': child,
            'parent_id': parent_id
        }
        
        self.tick_db.capture_snapshot(child, 'NEW', timestamp, market_price)
        
        print(f"\n  📤 ALGO: Sending slice {child['order_id']} for {slice_qty:,} shares to SOR")
        
        # Send to SOR
        timestamp += timedelta(milliseconds=10)
        child['state'] = OrderState.ACCEPTED.value
        child['update_time'] = timestamp.isoformat()
        self.tick_db.capture_snapshot(child, 'ACCEPTED', timestamp, market_price)
        
        # Route through SOR
        sor_orders, fills, timestamp = self.sor.route_order(child, timestamp)
        
        # Process fills and cascade up
        timestamp = self._process_fills(child, fills, timestamp)
        
        return child, timestamp
    
    def _process_fills(self, child: Dict, fills: List[Dict], timestamp: datetime) -> datetime:
        """Process fills and cascade updates up the chain"""
        if not fills:
            return timestamp
        
        # Calculate aggregate fill
        total_qty = sum(f['quantity'] for f in fills)
        total_value = sum(f['quantity'] * f['price'] for f in fills)
        avg_price = total_value / total_qty if total_qty > 0 else 0
        
        timestamp += timedelta(milliseconds=10)
        market_price, _, _ = self.market.tick()
        
        # Update child order
        child['filled_quantity'] = total_qty
        child['remaining_quantity'] = child['quantity'] - total_qty
        child['average_price'] = round(avg_price, 4)
        child['state'] = OrderState.FILLED.value if total_qty >= child['quantity'] else OrderState.PARTIALLY_FILLED.value
        child['update_time'] = timestamp.isoformat()
        
        self.tick_db.capture_snapshot(child, 'ALGO_SLICE_FILLED', timestamp, market_price)
        
        # CASCADE TO PARENT
        parent_id = child['parent_order_id']
        if parent_id in self.parent_orders:
            timestamp += timedelta(milliseconds=5)
            parent_data = self.parent_orders[parent_id]
            parent = parent_data['order']
            
            # Track this child as filled
            if child['state'] == OrderState.FILLED.value:
                parent_data['filled_children'] += 1
            
            # Add fills to parent tracking
            parent_data['fills'].extend(fills)
            
            # Update parent totals
            parent['filled_quantity'] += total_qty
            parent['remaining_quantity'] = parent['quantity'] - parent['filled_quantity']
            
            # Calculate parent VWAP
            all_fills = parent_data['fills']
            total_parent_value = sum(f['quantity'] * f['price'] for f in all_fills)
            parent['average_price'] = round(total_parent_value / parent['filled_quantity'], 4) if parent['filled_quantity'] > 0 else 0
            
            # Check if parent is fully filled
            if parent['filled_quantity'] >= parent['quantity']:
                parent['state'] = OrderState.FILLED.value
                print(f"\n  ✅ ALGO PARENT FILLED: {parent['order_id']} - {parent['filled_quantity']:,} @ {parent['average_price']:.2f}")
            else:
                parent['state'] = OrderState.WORKING.value
            
            parent['update_time'] = timestamp.isoformat()
            self.tick_db.capture_snapshot(parent, 'ALGO_PARENT_UPDATE', timestamp, market_price)
            
            # CASCADE TO CLIENT
            timestamp += timedelta(milliseconds=5)
            client_order = parent_data['client_order']
            client_order['filled_quantity'] = parent['filled_quantity']
            client_order['remaining_quantity'] = parent['remaining_quantity']
            client_order['average_price'] = parent['average_price']
            
            # Client order is FILLED only when parent is FILLED
            if parent['state'] == OrderState.FILLED.value:
                client_order['state'] = OrderState.FILLED.value
                print(f"  ✅ CLIENT ORDER FILLED: {client_order['order_id']} - {client_order['filled_quantity']:,} @ {client_order['average_price']:.2f}")
            else:
                client_order['state'] = OrderState.WORKING.value
            
            client_order['update_time'] = timestamp.isoformat()
            self.tick_db.capture_snapshot(client_order, 'CLIENT_UPDATE', timestamp, market_price)
        
        return timestamp

class Client:
    """Buy-side client sending orders"""
    
    def __init__(self, name: str, tick_db: TickDatabase, market: MarketSimulator):
        self.name = name
        self.tick_db = tick_db
        self.market = market
        self.orders = {}
        
    def send_order(self, client_order_id: str, ticker: str, side: str, 
                   quantity: int, timestamp: datetime) -> Dict:
        """Client sends an order"""
        market_price, _, _ = self.market.tick()
        
        order = {
            'order_id': f"CLIENT_{client_order_id}",
            'parent_order_id': None,
            'client_order_id': client_order_id,
            'order_level': 0,
            'order_type': 'CLIENT',
            'ticker': ticker,
            'side': side,
            'quantity': quantity,
            'filled_quantity': 0,
            'remaining_quantity': quantity,
            'average_price': 0.0,
            'state': OrderState.PENDING.value,
            'algo_strategy': None,
            'venue': None,
            'trader': 'TRD001',
            'desk': 'Equity Trading',
            'client_name': self.name,
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        
        self.orders[order['order_id']] = order
        self.tick_db.capture_snapshot(order, 'NEW', timestamp, market_price)
        
        print(f"\n💼 CLIENT: Sending order {client_order_id} for {quantity:,} {ticker} (market: {market_price:.2f})")
        return order

# ============== MAIN SIMULATION ==============

def run_vwap_simulation():
    """Run complete VWAP simulation with proper state transitions"""
    
    print("=" * 80)
    print("TRADING SYSTEM SIMULATION V2 - REALISTIC STATE TRANSITIONS")
    print("=" * 80)
    
    # Initialize components
    market = MarketSimulator(base_price=650.0)
    tick_db = TickDatabase()
    
    venues = {
        'NYSE': Venue('NYSE', tick_db, market),
        'NASDAQ': Venue('NASDAQ', tick_db, market),
        'ARCA': Venue('ARCA', tick_db, market),
        'BATS': Venue('BATS', tick_db, market),
        'DARK': Venue('DARK', tick_db, market)
    }
    
    sor = SmartOrderRouter(tick_db, venues, market)
    algo_engine = AlgoEngine(tick_db, sor, market)
    client = Client("Blackrock Asset Management", tick_db, market)
    
    # Simulation parameters
    client_order_id = 'C20241215_999888'
    ticker = 'ASML.AS'
    side = 'Buy'
    total_quantity = 30000  # Smaller for clearer example
    
    # Start time
    current_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    
    # 1. CLIENT SENDS ORDER
    client_order = client.send_order(client_order_id, ticker, side, total_quantity, current_time)
    
    # 2. ALGO ENGINE RECEIVES AND ACCEPTS
    current_time += timedelta(seconds=1)
    algo_parent = algo_engine.receive_order(client_order, current_time)
    
    # 3. EXECUTE VWAP WITH SIMPLIFIED SLICING
    # For demonstration, we'll do 3 slices of 10000 each
    slice_sizes = [10000, 10000, 10000]
    
    for slice_num, slice_size in enumerate(slice_sizes, 1):
        # Add some time between slices
        current_time += timedelta(minutes=30)
        
        print(f"\n{'='*60}")
        print(f"SLICE {slice_num}: {slice_size:,} shares")
        print(f"{'='*60}")
        
        # Generate and execute slice
        child, current_time = algo_engine.generate_child_slice(
            algo_parent['order_id'],
            slice_size,
            current_time
        )
        
        # Add small delay before next slice
        current_time += timedelta(seconds=5)
    
    # Export tick database
    print("\n" + "=" * 80)
    num_snapshots = tick_db.export('tick_database_v2')
    
    print(f"\n✅ Simulation complete!")
    print(f"📊 Generated {num_snapshots} snapshots in tick_database_v2.csv/.json")
    
    # Summary statistics
    final_client = [s for s in tick_db.snapshots if s['order_id'].startswith('CLIENT_')][-1]
    print(f"\n📈 Final Status:")
    print(f"  Client Order: {final_client['state']}")
    print(f"  Filled: {final_client['filled_quantity']:,} / {final_client['quantity']:,}")
    print(f"  VWAP: {final_client['average_price']:.2f}")
    
    # Show sample queries
    print("\n🔍 Sample Queries:")
    print("-- Client view only")
    print("SELECT * FROM tick_database_v2 WHERE order_level = 0 ORDER BY snapshot_time")
    print("\n-- See state transitions for a specific order")
    print("SELECT snapshot_time, order_id, state, event_type FROM tick_database_v2")
    print("WHERE order_id = 'CLIENT_C20241215_999888' ORDER BY snapshot_time")
    print("\n-- Track fill propagation")
    print("SELECT * FROM tick_database_v2 WHERE event_type LIKE '%FILLED%'")

if __name__ == "__main__":
    run_vwap_simulation()