#!/usr/bin/env python3
"""
Final Production VWAP Data Generator
Optimized for realistic volumes while staying under 100k rows
Configurable parameters for different scenarios
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

def generate_final_vwap(
    order_size: int = 1000000,
    avg_slice_size: int = 500,
    detail_level: str = "summary",  # "full", "summary", "client_only"
    fade_rate: float = 0.05,
    partial_rate: float = 0.10
) -> List[Dict]:
    """
    Generate production VWAP data
    
    Args:
        order_size: Total order size
        avg_slice_size: Average slice size (smaller = more slices)
        detail_level: How much detail to capture
        fade_rate: Probability of fade events
        partial_rate: Probability of partial fills
    """
    
    snapshots = []
    record_id = 0
    
    # Calculate number of slices
    num_slices = order_size // avg_slice_size
    
    print(f"Generating VWAP for {order_size:,} shares")
    print(f"Expected slices: ~{num_slices:,}")
    print(f"Detail level: {detail_level}")
    
    # Start time
    base_time = datetime.now().replace(hour=9, minute=0, second=0, microsecond=0)
    current_time = base_time
    
    # Market state
    market_price = 650.00
    
    # 1. CLIENT ORDER (always captured)
    client_order = {
        'order_id': 'CLIENT_001',
        'parent_order_id': None,
        'client_order_id': 'C20241216_MEGA',
        'order_level': 0,
        'order_type': 'CLIENT',
        'ticker': 'ASML.AS',
        'side': 'Buy',
        'quantity': order_size,
        'filled_quantity': 0,
        'remaining_quantity': order_size,
        'average_price': 0.0,
        'state': 'PENDING',
        'snapshot_time': current_time.isoformat(),
        'event_type': 'NEW',
        'record_id': f'REC_{record_id:08d}'
    }
    snapshots.append(client_order.copy())
    record_id += 1
    
    # 2. ALGO PARENT
    current_time += timedelta(seconds=1)
    algo_parent = {
        'order_id': 'ALGO_001',
        'parent_order_id': 'CLIENT_001',
        'client_order_id': 'C20241216_MEGA',
        'order_level': 1,
        'order_type': 'ALGO_PARENT',
        'ticker': 'ASML.AS',
        'side': 'Buy',
        'quantity': order_size,
        'filled_quantity': 0,
        'remaining_quantity': order_size,
        'average_price': 0.0,
        'state': 'WORKING',
        'algo_strategy': 'VWAP',
        'snapshot_time': current_time.isoformat(),
        'event_type': 'NEW',
        'record_id': f'REC_{record_id:08d}'
    }
    
    if detail_level != "client_only":
        snapshots.append(algo_parent.copy())
        record_id += 1
    
    # 3. GENERATE SLICES
    total_filled = 0
    total_value = 0.0
    slice_counter = 1
    sor_counter = 1
    
    # Track statistics
    stats = {
        'total_slices': 0,
        'total_routes': 0,
        'fade_count': 0,
        'partial_count': 0,
        'reject_count': 0
    }
    
    # Process in hourly batches to simulate VWAP schedule
    hours = 7  # Trading day
    slices_per_hour = num_slices // hours
    
    for hour in range(hours):
        if total_filled >= order_size:
            break
            
        # Determine urgency based on participation
        expected = (order_size // hours) * (hour + 1)
        if total_filled < expected * 0.7:
            urgency = Urgency.CRITICAL
        elif total_filled < expected * 0.9:
            urgency = Urgency.URGENT
        else:
            urgency = Urgency.NORMAL
        
        # Generate slices for this hour
        hour_slices = slices_per_hour if hour < hours - 1 else (num_slices - slice_counter + 1)
        
        for _ in range(min(hour_slices, 50)):  # Cap at 50 slices per hour for file size
            if total_filled >= order_size:
                break
                
            current_time += timedelta(seconds=random.randint(30, 120))
            
            # Vary slice size based on urgency
            if urgency == Urgency.CRITICAL:
                slice_size = random.randint(avg_slice_size * 2, avg_slice_size * 4)
            elif urgency == Urgency.URGENT:
                slice_size = random.randint(avg_slice_size, avg_slice_size * 2)
            else:
                slice_size = random.randint(avg_slice_size // 2, avg_slice_size)
            
            slice_size = min(slice_size, order_size - total_filled)
            
            # Create slice
            slice_order = {
                'order_id': f'SLICE_{slice_counter:05d}',
                'parent_order_id': 'ALGO_001',
                'client_order_id': 'C20241216_MEGA',
                'order_level': 2,
                'order_type': 'ALGO_SLICE',
                'ticker': 'ASML.AS',
                'side': 'Buy',
                'quantity': slice_size,
                'filled_quantity': 0,
                'remaining_quantity': slice_size,
                'average_price': 0.0,
                'state': 'PENDING',
                'urgency': urgency.value,
                'snapshot_time': current_time.isoformat(),
                'event_type': 'NEW',
                'record_id': f'REC_{record_id:08d}'
            }
            
            if detail_level == "full":
                snapshots.append(slice_order.copy())
                record_id += 1
            
            stats['total_slices'] += 1
            slice_counter += 1
            
            # Route to venues (simplified)
            slice_filled = 0
            slice_value = 0.0
            
            # Number of SOR routes based on urgency
            num_routes = 3 if urgency in [Urgency.CRITICAL, Urgency.URGENT] else 2
            
            for route_num in range(num_routes):
                if slice_filled >= slice_size:
                    break
                    
                venue = ['NYSE', 'NASDAQ', 'ARCA', 'BATS', 'DARK'][route_num % 5]
                route_size = slice_size // num_routes
                
                # Determine outcome
                outcome_rand = random.random()
                
                if outcome_rand < fade_rate:
                    # FADE
                    result = VenueResult.FADE
                    filled_qty = 0
                    stats['fade_count'] += 1
                    
                elif outcome_rand < fade_rate + partial_rate:
                    # PARTIAL
                    result = VenueResult.PARTIAL
                    filled_qty = route_size // 2
                    stats['partial_count'] += 1
                    
                elif outcome_rand < fade_rate + partial_rate + 0.02:
                    # REJECT/NO_CONN
                    result = VenueResult.NO_CONN if random.random() < 0.5 else VenueResult.REJECT
                    filled_qty = 0
                    stats['reject_count'] += 1
                    
                else:
                    # FILLED
                    result = VenueResult.FILLED
                    filled_qty = route_size
                
                # Price with slippage based on urgency
                if filled_qty > 0:
                    if urgency == Urgency.CRITICAL:
                        fill_price = market_price + random.uniform(0.02, 0.05)
                    elif urgency == Urgency.URGENT:
                        fill_price = market_price + random.uniform(0.01, 0.03)
                    else:
                        fill_price = market_price + random.uniform(-0.01, 0.01)
                else:
                    fill_price = 0
                
                # Create SOR route (only in full detail mode)
                if detail_level == "full":
                    sor_order = {
                        'order_id': f'SOR_{sor_counter:06d}',
                        'parent_order_id': slice_order['order_id'],
                        'client_order_id': 'C20241216_MEGA',
                        'order_level': 3,
                        'order_type': 'ROUTE',
                        'ticker': 'ASML.AS',
                        'side': 'Buy',
                        'quantity': route_size,
                        'filled_quantity': filled_qty,
                        'remaining_quantity': route_size - filled_qty,
                        'average_price': fill_price,
                        'state': result.value,
                        'venue': venue,
                        'snapshot_time': (current_time + timedelta(milliseconds=route_num * 50)).isoformat(),
                        'event_type': f'VENUE_{result.value}',
                        'record_id': f'REC_{record_id:08d}'
                    }
                    
                    if result == VenueResult.NO_CONN:
                        sor_order['reject_reason'] = f'No connection to {venue}-FIX-01'
                    elif result == VenueResult.FADE:
                        sor_order['fade_reason'] = 'Liquidity exhausted'
                    
                    snapshots.append(sor_order)
                    record_id += 1
                
                stats['total_routes'] += 1
                sor_counter += 1
                
                slice_filled += filled_qty
                slice_value += filled_qty * fill_price
            
            # Update totals
            total_filled += slice_filled
            total_value += slice_value
            
            # Update slice completion (in summary mode)
            if detail_level == "summary" and slice_filled > 0:
                slice_order['filled_quantity'] = slice_filled
                slice_order['remaining_quantity'] = slice_size - slice_filled
                slice_order['average_price'] = slice_value / slice_filled if slice_filled > 0 else 0
                slice_order['state'] = 'FILLED' if slice_filled >= slice_size else 'PARTIAL'
                slice_order['event_type'] = 'SLICE_SUMMARY'
                snapshots.append(slice_order.copy())
                record_id += 1
        
        # Hourly update to client
        current_time += timedelta(seconds=1)
        client_order['filled_quantity'] = total_filled
        client_order['remaining_quantity'] = order_size - total_filled
        client_order['average_price'] = total_value / total_filled if total_filled > 0 else 0
        client_order['state'] = 'WORKING' if total_filled < order_size else 'FILLED'
        client_order['snapshot_time'] = current_time.isoformat()
        client_order['event_type'] = 'CLIENT_UPDATE'
        client_order['record_id'] = f'REC_{record_id:08d}'
        client_order['hour'] = hour + 1
        client_order['urgency'] = urgency.value
        snapshots.append(client_order.copy())
        record_id += 1
    
    # Final summary
    print(f"\n{'='*60}")
    print(f"EXECUTION COMPLETE:")
    print(f"  Filled: {total_filled:,}/{order_size:,} ({total_filled/order_size*100:.1f}%)")
    print(f"  VWAP: {total_value/total_filled if total_filled > 0 else 0:.2f}")
    print(f"  Total Slices: {stats['total_slices']:,}")
    print(f"  Total Routes: {stats['total_routes']:,}")
    print(f"  Fades: {stats['fade_count']}")
    print(f"  Partials: {stats['partial_count']}")
    print(f"  Rejects: {stats['reject_count']}")
    print(f"  Snapshots: {len(snapshots):,}")
    
    return snapshots

def main():
    parser = argparse.ArgumentParser(description='Generate production VWAP data')
    parser.add_argument('--size', type=int, default=1000000, help='Order size')
    parser.add_argument('--slice-size', type=int, default=500, help='Average slice size')
    parser.add_argument('--detail', choices=['full', 'summary', 'client_only'], 
                       default='summary', help='Level of detail')
    parser.add_argument('--output', default='final_vwap', help='Output filename base')
    
    args = parser.parse_args()
    
    # Generate data
    snapshots = generate_final_vwap(
        order_size=args.size,
        avg_slice_size=args.slice_size,
        detail_level=args.detail
    )
    
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
    
    if len(snapshots) > 100000:
        print("⚠️  Warning: Over 100k rows. Consider using --detail summary or --slice-size larger")

if __name__ == "__main__":
    main()