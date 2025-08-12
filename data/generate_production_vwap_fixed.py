#!/usr/bin/env python3
"""
Production VWAP Generator with Proper Fill Propagation
Every slice fill cascades up: SOR → Slice → Algo Parent → Client
This is how real systems work - immediate propagation for position tracking
"""

import json
import csv
import random
import argparse
from datetime import datetime, timedelta
from typing import List, Dict, Tuple
from enum import Enum

class Urgency(Enum):
    PASSIVE = "PASSIVE"
    NORMAL = "NORMAL" 
    URGENT = "URGENT"
    CRITICAL = "CRITICAL"

class VenueResult(Enum):
    FILLED = "FILLED"
    PARTIAL = "PARTIAL"
    FADE = "FADE"
    REJECT = "REJECT"
    NO_CONN = "NO_CONNECTION"

class ProductionVWAPGenerator:
    def __init__(self, order_size: int, avg_slice_size: int, detail_level: str):
        self.order_size = order_size
        self.avg_slice_size = avg_slice_size
        self.detail_level = detail_level
        self.snapshots = []
        self.record_id = 0
        
        # Tracking cumulative fills
        self.total_filled = 0
        self.total_value = 0.0
        
        # Orders - keep state for updates
        self.client_order = None
        self.algo_parent = None
        
        # Market state
        self.market_price = 650.00
        
        # Stats
        self.stats = {
            'total_slices': 0,
            'total_routes': 0,
            'fade_count': 0,
            'partial_count': 0,
            'reject_count': 0
        }
        
    def add_snapshot(self, order: Dict, event_type: str, timestamp: datetime):
        """Add a snapshot to the tick database"""
        snapshot = order.copy()
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f'REC_{self.record_id:08d}'
        snapshot['market_price'] = self.market_price
        self.snapshots.append(snapshot)
        self.record_id += 1
        
    def propagate_fill_up_chain(self, slice_filled: int, slice_value: float, 
                                slice_order: Dict, timestamp: datetime, urgency: Urgency):
        """Propagate fill from slice → algo parent → client"""
        
        if slice_filled == 0:
            return
            
        # Update running totals
        self.total_filled += slice_filled
        self.total_value += slice_value
        
        # 1. Update SLICE order
        slice_order['filled_quantity'] += slice_filled
        slice_order['remaining_quantity'] = slice_order['quantity'] - slice_order['filled_quantity']
        if slice_order['filled_quantity'] > 0:
            slice_order['average_price'] = slice_value / slice_filled
        slice_order['state'] = 'FILLED' if slice_order['filled_quantity'] >= slice_order['quantity'] else 'PARTIAL'
        
        self.add_snapshot(slice_order, 'SLICE_UPDATE', timestamp)
        
        # 2. Propagate to ALGO PARENT
        timestamp += timedelta(milliseconds=5)
        self.algo_parent['filled_quantity'] = self.total_filled
        self.algo_parent['remaining_quantity'] = self.order_size - self.total_filled
        if self.total_filled > 0:
            self.algo_parent['average_price'] = self.total_value / self.total_filled
        self.algo_parent['state'] = 'FILLED' if self.total_filled >= self.order_size else 'WORKING'
        self.algo_parent['urgency'] = urgency.value
        
        # Calculate participation
        hour = (datetime.fromisoformat(timestamp.isoformat()).hour - 9)
        expected = (self.order_size // 7) * (hour + 1) if hour < 7 else self.order_size
        participation_pct = (self.total_filled / expected * 100) if expected > 0 else 100
        self.algo_parent['participation_pct'] = round(participation_pct, 1)
        
        self.add_snapshot(self.algo_parent, 'ALGO_UPDATE', timestamp)
        
        # 3. Propagate to CLIENT ORDER
        timestamp += timedelta(milliseconds=5)
        self.client_order['filled_quantity'] = self.total_filled
        self.client_order['remaining_quantity'] = self.order_size - self.total_filled
        self.client_order['average_price'] = self.algo_parent['average_price']
        self.client_order['state'] = self.algo_parent['state']
        
        self.add_snapshot(self.client_order, 'CLIENT_UPDATE', timestamp)
        
    def generate(self) -> List[Dict]:
        """Generate complete VWAP execution with proper propagation"""
        
        print(f"Generating VWAP for {self.order_size:,} shares")
        print(f"Expected slices: ~{self.order_size // self.avg_slice_size:,}")
        print(f"Detail level: {self.detail_level}")
        
        # Start time
        base_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
        current_time = base_time
        
        # 1. CLIENT ORDER
        self.client_order = {
            'order_id': 'CLIENT_001',
            'parent_order_id': None,
            'client_order_id': 'C20241216_PROD',
            'order_level': 0,
            'order_type': 'CLIENT',
            'ticker': 'ASML.AS',
            'side': 'Buy',
            'quantity': self.order_size,
            'filled_quantity': 0,
            'remaining_quantity': self.order_size,
            'average_price': 0.0,
            'state': 'PENDING',
            'client_name': 'Wellington Management'
        }
        self.add_snapshot(self.client_order, 'NEW', current_time)
        
        # 2. CLIENT ACCEPTED
        current_time += timedelta(seconds=1)
        self.client_order['state'] = 'ACCEPTED'
        self.add_snapshot(self.client_order, 'ACCEPTED', current_time)
        
        # 3. ALGO PARENT
        self.algo_parent = {
            'order_id': 'ALGO_001',
            'parent_order_id': 'CLIENT_001',
            'client_order_id': 'C20241216_PROD',
            'order_level': 1,
            'order_type': 'ALGO_PARENT',
            'ticker': 'ASML.AS',
            'side': 'Buy',
            'quantity': self.order_size,
            'filled_quantity': 0,
            'remaining_quantity': self.order_size,
            'average_price': 0.0,
            'state': 'WORKING',
            'algo_strategy': 'VWAP',
            'participation_pct': 0.0
        }
        self.add_snapshot(self.algo_parent, 'NEW', current_time)
        
        # 4. GENERATE SLICES
        slice_counter = 1
        sor_counter = 1
        
        # Process in hourly batches
        hours = 7
        num_slices = self.order_size // self.avg_slice_size
        slices_per_hour = num_slices // hours
        
        for hour in range(hours):
            if self.total_filled >= self.order_size:
                break
                
            # Determine urgency
            expected = (self.order_size // hours) * (hour + 1)
            participation_rate = (self.total_filled / expected * 100) if expected > 0 else 0
            
            if participation_rate < 70:
                urgency = Urgency.CRITICAL
            elif participation_rate < 85:
                urgency = Urgency.URGENT
            elif participation_rate < 95:
                urgency = Urgency.NORMAL
            else:
                urgency = Urgency.PASSIVE
            
            # Generate slices for this hour (cap for file size)
            hour_slices = min(slices_per_hour, 50)
            
            for slice_in_hour in range(hour_slices):
                if self.total_filled >= self.order_size:
                    break
                    
                # Time within hour
                current_time = base_time + timedelta(
                    hours=hour,
                    minutes=(60 // hour_slices) * slice_in_hour,
                    seconds=random.randint(0, 30)
                )
                
                # Market moves
                self.market_price += random.uniform(-0.1, 0.1)
                
                # Slice size based on urgency
                if urgency == Urgency.CRITICAL:
                    slice_size = random.randint(self.avg_slice_size * 2, self.avg_slice_size * 4)
                elif urgency == Urgency.URGENT:
                    slice_size = random.randint(self.avg_slice_size, self.avg_slice_size * 2)
                else:
                    slice_size = random.randint(self.avg_slice_size // 2, self.avg_slice_size)
                    
                slice_size = min(slice_size, self.order_size - self.total_filled)
                
                # Create slice
                slice_order = {
                    'order_id': f'SLICE_{slice_counter:05d}',
                    'parent_order_id': 'ALGO_001',
                    'client_order_id': 'C20241216_PROD',
                    'order_level': 2,
                    'order_type': 'ALGO_SLICE',
                    'ticker': 'ASML.AS',
                    'side': 'Buy',
                    'quantity': slice_size,
                    'filled_quantity': 0,
                    'remaining_quantity': slice_size,
                    'average_price': 0.0,
                    'state': 'PENDING',
                    'urgency': urgency.value
                }
                
                if self.detail_level in ['full', 'summary']:
                    self.add_snapshot(slice_order, 'NEW', current_time)
                
                self.stats['total_slices'] += 1
                slice_counter += 1
                
                # Route to venues
                slice_filled = 0
                slice_value = 0.0
                num_routes = 3 if urgency in [Urgency.CRITICAL, Urgency.URGENT] else 2
                
                for route_num in range(num_routes):
                    if slice_filled >= slice_size:
                        break
                        
                    venue = ['NYSE', 'NASDAQ', 'ARCA', 'BATS', 'DARK'][route_num % 5]
                    route_size = min((slice_size - slice_filled) // (num_routes - route_num), 
                                     slice_size - slice_filled)
                    
                    if route_size <= 0:
                        continue
                    
                    # Route timestamp
                    route_time = current_time + timedelta(milliseconds=10 + route_num * 50)
                    
                    # Determine outcome
                    outcome_rand = random.random()
                    
                    if outcome_rand < 0.05:  # 5% fade
                        result = VenueResult.FADE
                        filled_qty = 0
                        fill_price = 0
                        self.stats['fade_count'] += 1
                        
                    elif outcome_rand < 0.15:  # 10% partial
                        result = VenueResult.PARTIAL
                        filled_qty = route_size // 2
                        fill_price = self.market_price + random.uniform(-0.01, 0.02)
                        self.stats['partial_count'] += 1
                        
                    elif outcome_rand < 0.17:  # 2% reject
                        result = VenueResult.REJECT if random.random() < 0.5 else VenueResult.NO_CONN
                        filled_qty = 0
                        fill_price = 0
                        self.stats['reject_count'] += 1
                        
                    else:  # 83% filled
                        result = VenueResult.FILLED
                        filled_qty = route_size
                        # Price based on urgency
                        if urgency == Urgency.CRITICAL:
                            fill_price = self.market_price + random.uniform(0.02, 0.04)
                        elif urgency == Urgency.URGENT:
                            fill_price = self.market_price + random.uniform(0.01, 0.02)
                        else:
                            fill_price = self.market_price + random.uniform(-0.01, 0.01)
                    
                    # Create SOR route
                    if self.detail_level == 'full':
                        sor_order = {
                            'order_id': f'SOR_{sor_counter:06d}',
                            'parent_order_id': slice_order['order_id'],
                            'client_order_id': 'C20241216_PROD',
                            'order_level': 3,
                            'order_type': 'ROUTE',
                            'ticker': 'ASML.AS',
                            'side': 'Buy',
                            'quantity': route_size,
                            'filled_quantity': filled_qty,
                            'remaining_quantity': route_size - filled_qty,
                            'average_price': fill_price if filled_qty > 0 else 0,
                            'state': result.value,
                            'venue': venue,
                            'urgency': urgency.value
                        }
                        
                        if result == VenueResult.NO_CONN:
                            sor_order['reject_reason'] = f'No connection to {venue}-FIX-01'
                        elif result == VenueResult.FADE:
                            sor_order['fade_reason'] = 'Liquidity taken by competitor'
                            
                        self.add_snapshot(sor_order, f'VENUE_{result.value}', route_time)
                    
                    self.stats['total_routes'] += 1
                    sor_counter += 1
                    
                    if filled_qty > 0:
                        slice_filled += filled_qty
                        slice_value += filled_qty * fill_price
                        
                        # IMPORTANT: Propagate each fill up the chain immediately
                        if self.detail_level != 'none':
                            self.propagate_fill_up_chain(
                                filled_qty, 
                                filled_qty * fill_price,
                                slice_order,
                                route_time + timedelta(milliseconds=10),
                                urgency
                            )
        
        # Final update
        final_time = base_time + timedelta(hours=7)
        if self.total_filled >= self.order_size:
            self.client_order['state'] = 'FILLED'
            self.algo_parent['state'] = 'FILLED'
            self.add_snapshot(self.algo_parent, 'COMPLETED', final_time)
            self.add_snapshot(self.client_order, 'COMPLETED', final_time)
        
        # Print summary
        print(f"\n{'='*60}")
        print(f"EXECUTION COMPLETE:")
        print(f"  Filled: {self.total_filled:,}/{self.order_size:,} ({self.total_filled/self.order_size*100:.1f}%)")
        print(f"  VWAP: {self.total_value/self.total_filled if self.total_filled > 0 else 0:.2f}")
        print(f"  Total Slices: {self.stats['total_slices']:,}")
        print(f"  Total Routes: {self.stats['total_routes']:,}")
        print(f"  Fades: {self.stats['fade_count']}")
        print(f"  Partials: {self.stats['partial_count']}")
        print(f"  Rejects: {self.stats['reject_count']}")
        print(f"  Snapshots: {len(self.snapshots):,}")
        
        return self.snapshots

def main():
    parser = argparse.ArgumentParser(description='Generate production VWAP data with proper propagation')
    parser.add_argument('--size', type=int, default=2000000, help='Order size')
    parser.add_argument('--slice-size', type=int, default=2000, help='Average slice size')
    parser.add_argument('--detail', choices=['full', 'summary', 'client_only'], 
                       default='summary', help='Level of detail')
    parser.add_argument('--output', default='production_vwap_fixed', help='Output filename base')
    
    args = parser.parse_args()
    
    # Generate data
    generator = ProductionVWAPGenerator(
        order_size=args.size,
        avg_slice_size=args.slice_size,
        detail_level=args.detail
    )
    
    snapshots = generator.generate()
    
    # Export
    with open(f'{args.output}.json', 'w') as f:
        json.dump(snapshots, f, indent=2, default=str)
    
    if snapshots:
        keys = set()
        for s in snapshots:
            keys.update(s.keys())
        
        with open(f'{args.output}.csv', 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=sorted(keys))
            writer.writeheader()
            writer.writerows(snapshots)
    
    print(f"\n✅ Saved to {args.output}.csv and {args.output}.json")
    
    # File size check
    import os
    csv_size = os.path.getsize(f'{args.output}.csv') / (1024 * 1024)
    print(f"📁 CSV size: {csv_size:.2f} MB")

if __name__ == "__main__":
    main()