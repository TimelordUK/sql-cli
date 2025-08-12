#!/usr/bin/env python3
"""
Generate realistic tick database export - exactly what a broker would provide
Each fill/update creates a new snapshot row with updated values
All orders (parent, algo, SOR) integrated in one dataset
"""

import json
import csv
import random
from datetime import datetime, timedelta
from typing import List, Dict, Any

class TickDatabaseGenerator:
    """Simulates broker tick database with order snapshots"""
    
    def __init__(self, client_order_id: str, quantity: int, ticker: str = "ASML.AS"):
        self.client_order_id = client_order_id
        self.quantity = quantity
        self.ticker = ticker
        self.base_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
        self.current_time = self.base_time
        self.snapshots = []
        self.order_counter = 1000
        
    def add_snapshot(self, order: Dict, event_type: str = "UPDATE"):
        """Add a snapshot to the tick database"""
        snapshot = order.copy()
        snapshot['snapshot_time'] = self.current_time.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f"REC_{len(self.snapshots):08d}"
        self.snapshots.append(snapshot)
        
    def generate_complete_flow(self) -> List[Dict]:
        """Generate complete VWAP execution with all snapshots"""
        
        # 1. CLIENT ORDER ARRIVES - First snapshot
        client_order = {
            'order_id': f'ORD_{self.order_counter}',
            'parent_order_id': None,
            'client_order_id': self.client_order_id,
            'order_level': 0,  # Client level
            'order_type': 'CLIENT',
            'ticker': self.ticker,
            'side': 'Buy',
            'quantity': self.quantity,
            'filled_quantity': 0,
            'remaining_quantity': self.quantity,
            'average_price': 0.0,
            'state': 'PENDING',
            'algo_strategy': None,
            'venue': None,
            'trader': 'TRD001',
            'desk': 'Equity Trading',
            'client_name': 'Blackrock Asset Management',
        }
        self.add_snapshot(client_order, 'NEW')
        client_order_id_internal = client_order['order_id']
        self.order_counter += 1
        
        # 2. CLIENT ORDER ACCEPTED - Update snapshot
        self.current_time += timedelta(seconds=1)
        client_order['state'] = 'ACCEPTED'
        self.add_snapshot(client_order, 'ACCEPTED')
        
        # 3. ROUTE TO ALGO - Create algo parent order
        self.current_time += timedelta(seconds=1)
        algo_parent = {
            'order_id': f'ALGO_{self.order_counter}',
            'parent_order_id': client_order_id_internal,
            'client_order_id': self.client_order_id,
            'order_level': 1,  # Algo parent level
            'order_type': 'ALGO_PARENT',
            'ticker': self.ticker,
            'side': 'Buy',
            'quantity': self.quantity,
            'filled_quantity': 0,
            'remaining_quantity': self.quantity,
            'average_price': 0.0,
            'state': 'WORKING',
            'algo_strategy': 'VWAP',
            'venue': None,
            'trader': 'ALGO',
            'desk': 'Algo Trading',
            'client_name': 'Blackrock Asset Management',
        }
        self.add_snapshot(algo_parent, 'NEW')
        algo_parent_id = algo_parent['order_id']
        self.order_counter += 1
        
        # 4. GENERATE ALGO SLICES throughout the day
        vwap_profile = [0.12, 0.09, 0.08, 0.06, 0.07, 0.08, 0.09, 0.10, 0.15, 0.16]
        total_filled = 0
        fills_data = []
        
        for hour_idx, participation in enumerate(vwap_profile):
            if total_filled >= self.quantity:
                break
                
            hour_quantity = int(self.quantity * participation)
            slices_per_hour = random.randint(2, 4)
            
            for slice_idx in range(slices_per_hour):
                if total_filled >= self.quantity:
                    break
                
                # Time for this slice
                self.current_time = self.base_time + timedelta(
                    hours=hour_idx, 
                    minutes=(60 // slices_per_hour) * slice_idx + random.randint(0, 10)
                )
                
                # Create algo child order
                slice_qty = min(
                    hour_quantity // slices_per_hour + random.randint(-500, 500),
                    self.quantity - total_filled
                )
                
                if slice_qty <= 0:
                    continue
                
                algo_child = {
                    'order_id': f'ALGOCHILD_{self.order_counter}',
                    'parent_order_id': algo_parent_id,
                    'client_order_id': self.client_order_id,
                    'order_level': 2,  # Algo child level
                    'order_type': 'ALGO_CHILD',
                    'ticker': self.ticker,
                    'side': 'Buy',
                    'quantity': slice_qty,
                    'filled_quantity': 0,
                    'remaining_quantity': slice_qty,
                    'average_price': 0.0,
                    'state': 'PENDING',
                    'algo_strategy': 'VWAP_SLICE',
                    'venue': None,
                    'trader': 'ALGO',
                    'desk': 'Algo Trading',
                    'client_name': 'Blackrock Asset Management',
                }
                self.add_snapshot(algo_child, 'NEW')
                algo_child_id = algo_child['order_id']
                self.order_counter += 1
                
                # Route to SOR - split across venues
                venues = ['NYSE', 'NASDAQ', 'ARCA', 'BATS', 'DARK']
                num_venues = random.randint(1, 3)
                selected_venues = random.sample(venues, num_venues)
                
                slice_filled = 0
                slice_value = 0
                
                for venue in selected_venues:
                    if slice_filled >= slice_qty:
                        break
                    
                    # Create SOR child
                    self.current_time += timedelta(milliseconds=random.randint(10, 100))
                    venue_qty = min(
                        slice_qty // num_venues + random.randint(-100, 100),
                        slice_qty - slice_filled
                    )
                    
                    if venue_qty <= 0:
                        continue
                    
                    sor_child = {
                        'order_id': f'SOR_{self.order_counter}',
                        'parent_order_id': algo_child_id,
                        'client_order_id': self.client_order_id,
                        'order_level': 3,  # SOR level
                        'order_type': 'SOR_CHILD',
                        'ticker': self.ticker,
                        'side': 'Buy',
                        'quantity': venue_qty,
                        'filled_quantity': 0,
                        'remaining_quantity': venue_qty,
                        'average_price': 0.0,
                        'state': 'SENT',
                        'algo_strategy': None,
                        'venue': venue,
                        'trader': 'SOR',
                        'desk': 'Smart Router',
                        'client_name': 'Blackrock Asset Management',
                    }
                    self.add_snapshot(sor_child, 'NEW')
                    sor_child_id = sor_child['order_id']
                    self.order_counter += 1
                    
                    # Generate fills for this SOR order
                    self.current_time += timedelta(milliseconds=random.randint(50, 500))
                    fill_price = 650.0 + random.uniform(-1, 1)
                    
                    # SOR order filled
                    sor_child['filled_quantity'] = venue_qty
                    sor_child['remaining_quantity'] = 0
                    sor_child['average_price'] = fill_price
                    sor_child['state'] = 'FILLED'
                    self.add_snapshot(sor_child, 'FILL')
                    
                    slice_filled += venue_qty
                    slice_value += venue_qty * fill_price
                    
                    # Store fill data for propagation
                    fills_data.append({
                        'qty': venue_qty,
                        'price': fill_price,
                        'venue': venue,
                        'time': self.current_time
                    })
                
                # Update algo child with fills
                if slice_filled > 0:
                    self.current_time += timedelta(milliseconds=100)
                    algo_child['filled_quantity'] = slice_filled
                    algo_child['remaining_quantity'] = slice_qty - slice_filled
                    algo_child['average_price'] = slice_value / slice_filled if slice_filled > 0 else 0
                    algo_child['state'] = 'FILLED' if slice_filled >= slice_qty else 'PARTIAL'
                    self.add_snapshot(algo_child, 'FILL')
                    
                    total_filled += slice_filled
                    
                    # CASCADE FILL TO ALGO PARENT
                    self.current_time += timedelta(milliseconds=50)
                    algo_parent['filled_quantity'] = total_filled
                    algo_parent['remaining_quantity'] = self.quantity - total_filled
                    # Calculate VWAP
                    total_value = sum(f['qty'] * f['price'] for f in fills_data)
                    algo_parent['average_price'] = total_value / total_filled if total_filled > 0 else 0
                    algo_parent['state'] = 'WORKING' if total_filled < self.quantity else 'FILLED'
                    self.add_snapshot(algo_parent, 'FILL')
                    
                    # CASCADE FILL TO CLIENT ORDER
                    self.current_time += timedelta(milliseconds=50)
                    client_order['filled_quantity'] = total_filled
                    client_order['remaining_quantity'] = self.quantity - total_filled
                    client_order['average_price'] = total_value / total_filled if total_filled > 0 else 0
                    client_order['state'] = 'WORKING' if total_filled < self.quantity else 'FILLED'
                    self.add_snapshot(client_order, 'FILL')
        
        # Final status updates
        if total_filled >= self.quantity:
            self.current_time += timedelta(seconds=1)
            for order in [client_order, algo_parent]:
                order['state'] = 'FILLED'
                self.add_snapshot(order, 'DONE')
        
        return self.snapshots

def main():
    """Generate tick database export"""
    
    # Generate realistic tick database
    generator = TickDatabaseGenerator(
        client_order_id='C20241215_98765',
        quantity=100000,
        ticker='ASML.AS'
    )
    
    snapshots = generator.generate_complete_flow()
    
    # Save as JSON
    with open('tick_database.json', 'w') as f:
        json.dump(snapshots, f, indent=2, default=str)
    
    # Save as CSV (what support would typically provide)
    with open('tick_database.csv', 'w', newline='') as f:
        if snapshots:
            writer = csv.DictWriter(f, fieldnames=snapshots[0].keys())
            writer.writeheader()
            writer.writerows(snapshots)
    
    print(f"✅ Generated tick_database with {len(snapshots)} snapshots")
    
    # Show statistics
    order_types = {}
    event_types = {}
    
    for snap in snapshots:
        ot = snap.get('order_type', 'UNKNOWN')
        order_types[ot] = order_types.get(ot, 0) + 1
        
        et = snap.get('event_type', 'UNKNOWN')
        event_types[et] = event_types.get(et, 0) + 1
    
    print("\n📊 Snapshot Breakdown:")
    print("Order Types:")
    for ot, count in sorted(order_types.items()):
        print(f"  {ot}: {count}")
    
    print("\nEvent Types:")
    for et, count in sorted(event_types.items()):
        print(f"  {et}: {count}")
    
    # Show how to query
    print("\n🔍 Query Examples:")
    print("-- See client order progression only")
    print("SELECT * FROM tick_database WHERE order_level = 0 ORDER BY snapshot_time")
    print("\n-- See all orders for this client trade")
    print("SELECT * FROM tick_database WHERE client_order_id = 'C20241215_98765'")
    print("\n-- See fills only at client level")
    print("SELECT * FROM tick_database WHERE order_level = 0 AND event_type = 'FILL'")
    
    # Count unique orders
    unique_orders = len(set(s['order_id'] for s in snapshots))
    print(f"\n✅ Total unique orders: {unique_orders}")
    print(f"✅ Total snapshots: {len(snapshots)}")
    print(f"✅ Client order ID preserved: {all(s['client_order_id'] == 'C20241215_98765' for s in snapshots)}")

if __name__ == "__main__":
    main()