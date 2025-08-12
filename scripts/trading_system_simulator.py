#!/usr/bin/env python3
"""
Trading System Simulator
Simulates client, algo engine, SOR, and venues as separate components
All updates are captured in a tick database
"""

import json
import csv
import random
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
from enum import Enum
import copy

# ============== TICK DATABASE ==============
class TickDatabase:
    """Central tick database that captures all order updates"""
    
    def __init__(self):
        self.snapshots = []
        self.record_counter = 0
        
    def capture_snapshot(self, order: Dict, event_type: str, timestamp: datetime):
        """Capture a snapshot of an order at a point in time"""
        snapshot = copy.deepcopy(order)
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f"REC_{self.record_counter:08d}"
        self.record_counter += 1
        self.snapshots.append(snapshot)
        print(f"  📸 [{timestamp.strftime('%H:%M:%S.%f')[:-3]}] {event_type}: {order['order_id']} "
              f"({order['filled_quantity']}/{order['quantity']})")
        
    def export(self, filename: str):
        """Export tick database to CSV/JSON"""
        # JSON export
        with open(f"{filename}.json", 'w') as f:
            json.dump(self.snapshots, f, indent=2, default=str)
        
        # CSV export
        if self.snapshots:
            with open(f"{filename}.csv", 'w', newline='') as f:
                writer = csv.DictWriter(f, fieldnames=self.snapshots[0].keys())
                writer.writeheader()
                writer.writerows(self.snapshots)
        
        return len(self.snapshots)

# ============== TRADING COMPONENTS ==============

class Client:
    """Buy-side client sending orders"""
    
    def __init__(self, name: str, tick_db: TickDatabase):
        self.name = name
        self.tick_db = tick_db
        self.orders = {}
        
    def send_order(self, client_order_id: str, ticker: str, side: str, 
                   quantity: int, timestamp: datetime) -> Dict:
        """Client sends an order"""
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
            'state': 'PENDING',
            'algo_strategy': None,
            'venue': None,
            'trader': 'TRD001',
            'desk': 'Equity Trading',
            'client_name': self.name,
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        
        self.orders[order['order_id']] = order
        self.tick_db.capture_snapshot(order, 'NEW', timestamp)
        
        print(f"\n💼 CLIENT: Sending order {client_order_id} for {quantity:,} {ticker}")
        return order
    
    def receive_fill_update(self, order_id: str, filled_qty: int, avg_price: float, 
                           state: str, timestamp: datetime):
        """Client receives fill update from broker"""
        if order_id in self.orders:
            order = self.orders[order_id]
            order['filled_quantity'] = filled_qty
            order['remaining_quantity'] = order['quantity'] - filled_qty
            order['average_price'] = avg_price
            order['state'] = state
            order['update_time'] = timestamp.isoformat()
            
            self.tick_db.capture_snapshot(order, 'FILL_UPDATE', timestamp)
            
            print(f"  ✅ CLIENT: Received update - {filled_qty:,}/{order['quantity']:,} "
                  f"@ {avg_price:.2f} [{state}]")

class AlgoEngine:
    """Algo engine that executes VWAP strategy"""
    
    def __init__(self, tick_db: TickDatabase):
        self.tick_db = tick_db
        self.parent_orders = {}
        self.child_orders = {}
        self.order_counter = 1000
        
    def receive_order(self, client_order: Dict, timestamp: datetime) -> Dict:
        """Receive order from client and create algo parent"""
        # Accept client order
        client_order['state'] = 'ACCEPTED'
        self.tick_db.capture_snapshot(client_order, 'ACCEPTED', timestamp)
        
        # Create algo parent order
        algo_parent = {
            'order_id': f"ALGO_{self.order_counter}",
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
            'state': 'WORKING',
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
            'fills': []
        }
        
        self.tick_db.capture_snapshot(algo_parent, 'NEW', timestamp)
        
        print(f"\n🤖 ALGO: Created VWAP parent {algo_parent['order_id']}")
        return algo_parent
    
    def generate_child_slice(self, parent_id: str, slice_qty: int, 
                           timestamp: datetime) -> Dict:
        """Generate child order slice"""
        parent_data = self.parent_orders[parent_id]
        parent = parent_data['order']
        
        child = {
            'order_id': f"ALGOCHILD_{self.order_counter}",
            'parent_order_id': parent_id,
            'client_order_id': parent['client_order_id'],
            'order_level': 2,
            'order_type': 'ALGO_CHILD',
            'ticker': parent['ticker'],
            'side': parent['side'],
            'quantity': slice_qty,
            'filled_quantity': 0,
            'remaining_quantity': slice_qty,
            'average_price': 0.0,
            'state': 'PENDING',
            'algo_strategy': 'VWAP_SLICE',
            'venue': None,
            'trader': 'ALGO',
            'desk': 'Algo Trading',
            'client_name': parent['client_name'],
            'create_time': timestamp.isoformat(),
            'update_time': timestamp.isoformat()
        }
        
        self.order_counter += 1
        self.child_orders[child['order_id']] = child
        self.tick_db.capture_snapshot(child, 'NEW', timestamp)
        
        print(f"  🔄 ALGO: Created child slice {child['order_id']} for {slice_qty:,} shares")
        return child
    
    def receive_fill(self, child_id: str, fill_qty: int, fill_price: float, 
                    venue: str, timestamp: datetime):
        """Receive fill from SOR and cascade up"""
        if child_id not in self.child_orders:
            return
        
        child = self.child_orders[child_id]
        
        # Update child order
        child['filled_quantity'] += fill_qty
        child['remaining_quantity'] = child['quantity'] - child['filled_quantity']
        if child['filled_quantity'] > 0:
            # Update average price
            prev_value = child['average_price'] * (child['filled_quantity'] - fill_qty)
            new_value = fill_price * fill_qty
            child['average_price'] = (prev_value + new_value) / child['filled_quantity']
        
        child['state'] = 'FILLED' if child['remaining_quantity'] == 0 else 'PARTIAL'
        child['update_time'] = timestamp.isoformat()
        
        self.tick_db.capture_snapshot(child, 'FILL', timestamp)
        
        # CASCADE TO PARENT
        parent_id = child['parent_order_id']
        if parent_id in self.parent_orders:
            parent_data = self.parent_orders[parent_id]
            parent = parent_data['order']
            parent_data['fills'].append({'qty': fill_qty, 'price': fill_price})
            
            # Update parent
            parent['filled_quantity'] += fill_qty
            parent['remaining_quantity'] = parent['quantity'] - parent['filled_quantity']
            
            # Calculate VWAP
            total_value = sum(f['qty'] * f['price'] for f in parent_data['fills'])
            parent['average_price'] = total_value / parent['filled_quantity'] if parent['filled_quantity'] > 0 else 0
            
            parent['state'] = 'FILLED' if parent['remaining_quantity'] == 0 else 'WORKING'
            parent['update_time'] = timestamp.isoformat()
            
            self.tick_db.capture_snapshot(parent, 'FILL', timestamp)
            
            # CASCADE TO CLIENT ORDER
            client_order = parent_data['client_order']
            client_order['filled_quantity'] = parent['filled_quantity']
            client_order['remaining_quantity'] = parent['remaining_quantity']
            client_order['average_price'] = parent['average_price']
            client_order['state'] = parent['state']
            client_order['update_time'] = timestamp.isoformat()
            
            self.tick_db.capture_snapshot(client_order, 'FILL', timestamp)
            
            return client_order

class SmartOrderRouter:
    """SOR that routes orders to venues"""
    
    def __init__(self, tick_db: TickDatabase):
        self.tick_db = tick_db
        self.order_counter = 5000
        self.orders = {}
        
    def route_order(self, algo_child: Dict, timestamp: datetime) -> List[Dict]:
        """Route order to multiple venues"""
        venues = ['NYSE', 'NASDAQ', 'ARCA', 'BATS', 'DARK']
        num_venues = random.randint(1, 3)
        selected_venues = random.sample(venues, num_venues)
        
        sor_orders = []
        remaining = algo_child['quantity']
        
        print(f"  🔀 SOR: Routing {algo_child['order_id']} to {selected_venues}")
        
        for venue in selected_venues:
            if remaining <= 0:
                break
            
            venue_qty = remaining // len(selected_venues) + random.randint(-100, 100)
            venue_qty = min(max(venue_qty, 100), remaining)
            
            sor_order = {
                'order_id': f"SOR_{self.order_counter}",
                'parent_order_id': algo_child['order_id'],
                'client_order_id': algo_child['client_order_id'],
                'order_level': 3,
                'order_type': 'SOR_CHILD',
                'ticker': algo_child['ticker'],
                'side': algo_child['side'],
                'quantity': venue_qty,
                'filled_quantity': 0,
                'remaining_quantity': venue_qty,
                'average_price': 0.0,
                'state': 'SENT',
                'algo_strategy': None,
                'venue': venue,
                'trader': 'SOR',
                'desk': 'Smart Router',
                'client_name': algo_child['client_name'],
                'create_time': timestamp.isoformat(),
                'update_time': timestamp.isoformat()
            }
            
            self.order_counter += 1
            self.orders[sor_order['order_id']] = sor_order
            self.tick_db.capture_snapshot(sor_order, 'NEW', timestamp)
            
            sor_orders.append(sor_order)
            remaining -= venue_qty
        
        return sor_orders
    
    def receive_venue_fill(self, order_id: str, fill_price: float, timestamp: datetime) -> Dict:
        """Receive fill from venue"""
        if order_id not in self.orders:
            return None
        
        order = self.orders[order_id]
        order['filled_quantity'] = order['quantity']
        order['remaining_quantity'] = 0
        order['average_price'] = fill_price
        order['state'] = 'FILLED'
        order['update_time'] = timestamp.isoformat()
        
        self.tick_db.capture_snapshot(order, 'FILL', timestamp)
        
        return order

class Venue:
    """Exchange venue that executes orders"""
    
    def __init__(self, name: str, tick_db: TickDatabase):
        self.name = name
        self.tick_db = tick_db
        self.base_price = 650.0
        
    def execute_order(self, sor_order: Dict, timestamp: datetime) -> Dict:
        """Execute order at venue"""
        # Simulate execution with some price variance
        fill_price = self.base_price + random.uniform(-1, 1)
        
        fill = {
            'venue': self.name,
            'order_id': sor_order['order_id'],
            'quantity': sor_order['quantity'],
            'price': fill_price,
            'timestamp': timestamp.isoformat()
        }
        
        print(f"    💹 {self.name}: Filled {sor_order['quantity']:,} @ {fill_price:.2f}")
        
        return fill

# ============== MAIN SIMULATION ==============

def run_vwap_simulation():
    """Run complete VWAP simulation with all components"""
    
    print("=" * 80)
    print("TRADING SYSTEM SIMULATION - VWAP EXECUTION")
    print("=" * 80)
    
    # Initialize components
    tick_db = TickDatabase()
    client = Client("Blackrock Asset Management", tick_db)
    algo_engine = AlgoEngine(tick_db)
    sor = SmartOrderRouter(tick_db)
    venues = {
        'NYSE': Venue('NYSE', tick_db),
        'NASDAQ': Venue('NASDAQ', tick_db),
        'ARCA': Venue('ARCA', tick_db),
        'BATS': Venue('BATS', tick_db),
        'DARK': Venue('DARK', tick_db)
    }
    
    # Simulation parameters
    client_order_id = 'C20241215_123456'
    ticker = 'ASML.AS'
    side = 'Buy'
    total_quantity = 100000
    
    # Start time
    current_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    
    # 1. CLIENT SENDS ORDER
    client_order = client.send_order(client_order_id, ticker, side, total_quantity, current_time)
    
    # 2. ALGO ENGINE RECEIVES AND ACCEPTS
    current_time += timedelta(seconds=1)
    algo_parent = algo_engine.receive_order(client_order, current_time)
    
    # 3. EXECUTE VWAP THROUGHOUT THE DAY
    vwap_profile = [0.12, 0.09, 0.08, 0.06, 0.07, 0.08, 0.09, 0.10, 0.15, 0.16]
    total_filled = 0
    
    for hour_idx, participation in enumerate(vwap_profile[:5]):  # Simplified - first 5 hours
        if total_filled >= total_quantity:
            break
        
        hour_quantity = int(total_quantity * participation)
        slices_per_hour = 2  # Simplified
        
        print(f"\n⏰ Hour {hour_idx + 1} - Target: {hour_quantity:,} shares")
        
        for slice_idx in range(slices_per_hour):
            if total_filled >= total_quantity:
                break
            
            # Time for this slice
            current_time = datetime.now().replace(
                hour=9 + hour_idx,
                minute=(60 // slices_per_hour) * slice_idx,
                second=0,
                microsecond=0
            )
            
            slice_qty = min(
                hour_quantity // slices_per_hour,
                total_quantity - total_filled
            )
            
            if slice_qty <= 0:
                continue
            
            # Generate algo child slice
            algo_child = algo_engine.generate_child_slice(
                algo_parent['order_id'], 
                slice_qty, 
                current_time
            )
            
            # Route to SOR
            current_time += timedelta(milliseconds=100)
            sor_orders = sor.route_order(algo_child, current_time)
            
            # Execute at venues and cascade fills back
            for sor_order in sor_orders:
                current_time += timedelta(milliseconds=random.randint(50, 200))
                
                # Venue executes
                venue = venues[sor_order['venue']]
                fill = venue.execute_order(sor_order, current_time)
                
                # SOR receives fill
                current_time += timedelta(milliseconds=50)
                filled_sor = sor.receive_venue_fill(sor_order['order_id'], fill['price'], current_time)
                
                # Algo engine receives fill and cascades up
                current_time += timedelta(milliseconds=50)
                updated_client_order = algo_engine.receive_fill(
                    algo_child['order_id'],
                    filled_sor['quantity'],
                    filled_sor['average_price'],
                    filled_sor['venue'],
                    current_time
                )
                
                # Client receives update
                if updated_client_order:
                    current_time += timedelta(milliseconds=50)
                    client.receive_fill_update(
                        updated_client_order['order_id'],
                        updated_client_order['filled_quantity'],
                        updated_client_order['average_price'],
                        updated_client_order['state'],
                        current_time
                    )
                
                total_filled = updated_client_order['filled_quantity'] if updated_client_order else total_filled
    
    # Export tick database
    print("\n" + "=" * 80)
    num_snapshots = tick_db.export('tick_database')
    
    print(f"\n✅ Simulation complete!")
    print(f"📊 Generated {num_snapshots} snapshots in tick_database.csv/.json")
    print(f"💼 Final fill: {total_filled:,}/{total_quantity:,} shares")
    
    # Show sample queries
    print("\n🔍 Sample Queries:")
    print("-- Client view only (what client sees)")
    print("SELECT * FROM tick_database WHERE order_level = 0 ORDER BY snapshot_time")
    print("\n-- See progression of a specific order")
    print("SELECT * FROM tick_database WHERE order_id = 'CLIENT_C20241215_123456'")
    print("\n-- All fills at any level")
    print("SELECT * FROM tick_database WHERE event_type = 'FILL'")

if __name__ == "__main__":
    run_vwap_simulation()