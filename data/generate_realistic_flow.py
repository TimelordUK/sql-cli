#!/usr/bin/env python3
"""
Generate realistic trading flow with fades, partials, and retries
Creates tick database showing what support engineers actually see
"""

import json
import csv
from datetime import datetime, timedelta
import random

def generate_realistic_vwap_flow():
    """Generate a realistic VWAP execution with market microstructure issues"""
    
    snapshots = []
    record_id = 0
    
    # Start time
    current_time = datetime.now().replace(hour=9, minute=30, second=0, microsecond=0)
    
    # Market prices
    market_price = 650.00
    
    def add_snapshot(order, event_type, timestamp, market_price=None):
        nonlocal record_id
        snapshot = order.copy()
        snapshot['snapshot_time'] = timestamp.isoformat()
        snapshot['event_type'] = event_type
        snapshot['record_id'] = f"REC_{record_id:08d}"
        if market_price:
            snapshot['market_price'] = market_price
        snapshots.append(snapshot)
        record_id += 1
        return snapshot
    
    # 1. CLIENT ORDER
    client_order = {
        'order_id': 'CLIENT_C20241216_100001',
        'parent_order_id': None,
        'client_order_id': 'C20241216_100001',
        'order_level': 0,
        'order_type': 'CLIENT',
        'ticker': 'ASML.AS',
        'side': 'Buy',
        'quantity': 50000,
        'filled_quantity': 0,
        'remaining_quantity': 50000,
        'average_price': 0.0,
        'state': 'PENDING',
        'trader': 'TRD001',
        'client_name': 'Blackrock'
    }
    
    add_snapshot(client_order, 'NEW', current_time, market_price)
    
    # 2. ALGO ACCEPTS
    current_time += timedelta(seconds=1)
    client_order['state'] = 'ACCEPTED'
    add_snapshot(client_order, 'ACCEPTED', current_time, market_price)
    
    # 3. ALGO PARENT
    algo_parent = {
        'order_id': 'ALGO_50001',
        'parent_order_id': 'CLIENT_C20241216_100001',
        'client_order_id': 'C20241216_100001',
        'order_level': 1,
        'order_type': 'ALGO_PARENT',
        'ticker': 'ASML.AS',
        'side': 'Buy',
        'quantity': 50000,
        'filled_quantity': 0,
        'remaining_quantity': 50000,
        'average_price': 0.0,
        'state': 'WORKING',
        'algo_strategy': 'VWAP'
    }
    add_snapshot(algo_parent, 'NEW', current_time, market_price)
    
    # Track cumulative fills
    total_filled = 0
    total_value = 0.0
    
    # 4. GENERATE SLICES WITH REALISTIC ISSUES
    slices = [
        {'qty': 15000, 'scenario': 'normal'},
        {'qty': 20000, 'scenario': 'fade_retry'},
        {'qty': 15000, 'scenario': 'partial_fills'}
    ]
    
    for slice_num, slice_config in enumerate(slices, 1):
        current_time += timedelta(minutes=30)
        market_price += random.uniform(-0.5, 0.5)
        
        # Create algo slice
        slice_order = {
            'order_id': f'SLICE_{60000 + slice_num}',
            'parent_order_id': 'ALGO_50001',
            'client_order_id': 'C20241216_100001',
            'order_level': 2,
            'order_type': 'ALGO_SLICE',
            'ticker': 'ASML.AS',
            'side': 'Buy',
            'quantity': slice_config['qty'],
            'filled_quantity': 0,
            'remaining_quantity': slice_config['qty'],
            'average_price': 0.0,
            'state': 'PENDING'
        }
        add_snapshot(slice_order, 'NEW', current_time, market_price)
        
        current_time += timedelta(milliseconds=10)
        slice_order['state'] = 'ACCEPTED'
        add_snapshot(slice_order, 'ACCEPTED', current_time, market_price)
        
        slice_filled = 0
        slice_value = 0
        
        if slice_config['scenario'] == 'normal':
            # Normal execution - split across 3 venues
            venues = [('NYSE', 5000), ('NASDAQ', 5000), ('ARCA', 5000)]
            
            for venue, qty in venues:
                current_time += timedelta(milliseconds=50)
                
                # SOR child
                sor_order = {
                    'order_id': f'SOR_{70000 + slice_num * 10 + venues.index((venue, qty))}',
                    'parent_order_id': slice_order['order_id'],
                    'client_order_id': 'C20241216_100001',
                    'order_level': 3,
                    'order_type': 'ROUTE',
                    'ticker': 'ASML.AS',
                    'side': 'Buy',
                    'quantity': qty,
                    'filled_quantity': 0,
                    'remaining_quantity': qty,
                    'average_price': 0.0,
                    'state': 'PENDING',
                    'venue': venue,
                    'instruction': 'IOC'
                }
                add_snapshot(sor_order, 'NEW', current_time, market_price)
                
                # Venue accepts
                current_time += timedelta(milliseconds=20)
                sor_order['state'] = 'ACCEPTED'
                add_snapshot(sor_order, 'VENUE_ACCEPTED', current_time, market_price)
                
                # Venue fills
                current_time += timedelta(milliseconds=random.randint(50, 150))
                fill_price = market_price + random.uniform(-0.02, 0.05)
                sor_order['filled_quantity'] = qty
                sor_order['remaining_quantity'] = 0
                sor_order['average_price'] = fill_price
                sor_order['state'] = 'FILLED'
                add_snapshot(sor_order, 'VENUE_FILLED', current_time, market_price)
                
                slice_filled += qty
                slice_value += qty * fill_price
        
        elif slice_config['scenario'] == 'fade_retry':
            # First attempt - experience fade
            venues_try1 = [('NYSE', 8000), ('NASDAQ', 8000), ('BATS', 4000)]
            
            for venue, qty in venues_try1:
                current_time += timedelta(milliseconds=50)
                
                sor_order = {
                    'order_id': f'SOR_{71000 + venues_try1.index((venue, qty))}',
                    'parent_order_id': slice_order['order_id'],
                    'client_order_id': 'C20241216_100001',
                    'order_level': 3,
                    'order_type': 'ROUTE',
                    'ticker': 'ASML.AS',
                    'side': 'Buy',
                    'quantity': qty,
                    'filled_quantity': 0,
                    'remaining_quantity': qty,
                    'average_price': 0.0,
                    'state': 'PENDING',
                    'venue': venue,
                    'instruction': 'EOE',
                    'attempt': 1
                }
                add_snapshot(sor_order, 'NEW', current_time, market_price)
                
                if venue == 'NASDAQ':
                    # FADE - someone else got liquidity
                    current_time += timedelta(milliseconds=30)
                    sor_order['state'] = 'FADE'
                    sor_order['fade_reason'] = 'Liquidity taken by competitor'
                    add_snapshot(sor_order, 'VENUE_FADE', current_time, market_price)
                    print(f"    ❌ FADE at {venue}: Lost {qty:,} shares")
                
                elif venue == 'BATS':
                    # PARTIAL FILL
                    current_time += timedelta(milliseconds=40)
                    partial_qty = 1500
                    fill_price = market_price + 0.02
                    sor_order['filled_quantity'] = partial_qty
                    sor_order['remaining_quantity'] = qty - partial_qty
                    sor_order['average_price'] = fill_price
                    sor_order['state'] = 'PARTIAL'
                    add_snapshot(sor_order, 'VENUE_PARTIAL', current_time, market_price)
                    slice_filled += partial_qty
                    slice_value += partial_qty * fill_price
                    print(f"    ⚠️  PARTIAL at {venue}: {partial_qty:,}/{qty:,}")
                
                else:  # NYSE
                    # FILLED
                    current_time += timedelta(milliseconds=60)
                    fill_price = market_price + 0.01
                    sor_order['filled_quantity'] = qty
                    sor_order['remaining_quantity'] = 0
                    sor_order['average_price'] = fill_price
                    sor_order['state'] = 'FILLED'
                    add_snapshot(sor_order, 'VENUE_FILLED', current_time, market_price)
                    slice_filled += qty
                    slice_value += qty * fill_price
            
            # RETRY for unfilled quantity
            remaining = slice_config['qty'] - slice_filled
            if remaining > 0:
                current_time += timedelta(milliseconds=500)
                print(f"    🔄 RETRY: {remaining:,} shares remaining")
                
                # Retry orders
                retry_venues = [('DARK', remaining)]
                for venue, qty in retry_venues:
                    sor_order = {
                        'order_id': f'SOR_{71100}',
                        'parent_order_id': slice_order['order_id'],
                        'client_order_id': 'C20241216_100001',
                        'order_level': 3,
                        'order_type': 'ROUTE',
                        'ticker': 'ASML.AS',
                        'side': 'Buy',
                        'quantity': qty,
                        'filled_quantity': 0,
                        'remaining_quantity': qty,
                        'average_price': 0.0,
                        'state': 'PENDING',
                        'venue': venue,
                        'instruction': 'IOC',
                        'attempt': 2
                    }
                    add_snapshot(sor_order, 'NEW_RETRY', current_time, market_price)
                    
                    # Dark pool fills
                    current_time += timedelta(milliseconds=100)
                    fill_price = market_price - 0.01  # Better price in dark
                    sor_order['filled_quantity'] = qty
                    sor_order['remaining_quantity'] = 0
                    sor_order['average_price'] = fill_price
                    sor_order['state'] = 'FILLED'
                    add_snapshot(sor_order, 'VENUE_FILLED', current_time, market_price)
                    slice_filled += qty
                    slice_value += qty * fill_price
        
        # Update slice order
        current_time += timedelta(milliseconds=50)
        slice_order['filled_quantity'] = slice_filled
        slice_order['remaining_quantity'] = slice_config['qty'] - slice_filled
        slice_order['average_price'] = slice_value / slice_filled if slice_filled > 0 else 0
        slice_order['state'] = 'FILLED' if slice_filled >= slice_config['qty'] else 'PARTIAL'
        add_snapshot(slice_order, 'SLICE_COMPLETE', current_time, market_price)
        
        # Update totals
        total_filled += slice_filled
        total_value += slice_value
        
        # CASCADE TO PARENT
        current_time += timedelta(milliseconds=10)
        algo_parent['filled_quantity'] = total_filled
        algo_parent['remaining_quantity'] = 50000 - total_filled
        algo_parent['average_price'] = total_value / total_filled if total_filled > 0 else 0
        algo_parent['state'] = 'FILLED' if total_filled >= 50000 else 'WORKING'
        add_snapshot(algo_parent, 'ALGO_UPDATE', current_time, market_price)
        
        # CASCADE TO CLIENT
        current_time += timedelta(milliseconds=10)
        client_order['filled_quantity'] = total_filled
        client_order['remaining_quantity'] = 50000 - total_filled
        client_order['average_price'] = algo_parent['average_price']
        client_order['state'] = 'FILLED' if total_filled >= 50000 else 'WORKING'
        add_snapshot(client_order, 'CLIENT_UPDATE', current_time, market_price)
    
    # Export
    with open('realistic_flow.json', 'w') as f:
        json.dump(snapshots, f, indent=2, default=str)
    
    # Export CSV
    if snapshots:
        keys = set()
        for s in snapshots:
            keys.update(s.keys())
        
        with open('realistic_flow.csv', 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=sorted(keys))
            writer.writeheader()
            writer.writerows(snapshots)
    
    print(f"\n✅ Generated {len(snapshots)} snapshots")
    print(f"📊 Final: {total_filled:,}/50,000 @ {total_value/total_filled:.2f}")
    
    # Analysis
    fade_events = [s for s in snapshots if s.get('event_type') == 'VENUE_FADE']
    partial_events = [s for s in snapshots if s.get('event_type') == 'VENUE_PARTIAL']
    retry_events = [s for s in snapshots if s.get('event_type') == 'NEW_RETRY']
    
    print(f"\n📈 Events:")
    print(f"  Fades: {len(fade_events)}")
    print(f"  Partials: {len(partial_events)}")  
    print(f"  Retries: {len(retry_events)}")
    
    return snapshots

if __name__ == "__main__":
    print("GENERATING REALISTIC VWAP FLOW WITH MICROSTRUCTURE ISSUES")
    print("=" * 60)
    generate_realistic_vwap_flow()