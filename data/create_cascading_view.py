#!/usr/bin/env python3
"""
Create cascading view showing how fills update the client order
This simulates how the client sees their order updating in real-time
"""

import json
from datetime import datetime

def create_cascading_view():
    # Load the properly structured data
    with open('vwap_proper_orders.json', 'r') as f:
        orders = json.load(f)
    
    with open('vwap_proper_fills.json', 'r') as f:
        fills = json.load(f)
    
    # Find the client order (Level 0)
    client_order = [o for o in orders if o['order_level'] == 0][0]
    client_order_id = client_order['client_order_id']
    root_order_id = client_order['order_id']
    
    print(f"Client Order ID: {client_order_id}")
    print(f"Root Order ID: {root_order_id}")
    print(f"Total Quantity: {client_order['quantity']:,}")
    
    # Get all fills and sort by timestamp
    fills.sort(key=lambda x: x['timestamp'])
    
    # Create cascading updates - each fill updates the client order
    cascading_updates = []
    
    # Initial state - order accepted
    cascading_updates.append({
        'update_time': client_order['timestamp'],
        'update_type': 'Order Accepted',
        'order_id': client_order['order_id'],
        'client_order_id': client_order_id,
        'ticker': client_order['ticker'],
        'side': client_order['side'],
        'ordered_quantity': client_order['quantity'],
        'filled_quantity': 0,
        'remaining_quantity': client_order['quantity'],
        'fill_percentage': 0.0,
        'average_price': 0.0,
        'state': 'Accepted',
        'last_fill_venue': None,
        'last_fill_quantity': 0,
        'last_fill_price': 0.0,
        'total_commission': 0.0,
        'total_fees': 0.0
    })
    
    # Process each fill and show how it updates the client order
    cumulative_qty = 0
    cumulative_value = 0
    cumulative_commission = 0
    cumulative_fees = 0
    
    for fill in fills:
        cumulative_qty += fill['quantity']
        cumulative_value += fill['quantity'] * fill['price']
        cumulative_commission += fill.get('commission', 0)
        cumulative_fees += fill.get('fees', 0)
        
        avg_price = cumulative_value / cumulative_qty if cumulative_qty > 0 else 0
        
        # This is what the client sees after each fill
        cascading_updates.append({
            'update_time': fill['timestamp'],
            'update_type': 'Fill',
            'order_id': client_order['order_id'],
            'client_order_id': client_order_id,
            'ticker': client_order['ticker'],
            'side': client_order['side'],
            'ordered_quantity': client_order['quantity'],
            'filled_quantity': cumulative_qty,
            'remaining_quantity': client_order['quantity'] - cumulative_qty,
            'fill_percentage': round((cumulative_qty / client_order['quantity']) * 100, 2),
            'average_price': round(avg_price, 4),
            'state': 'Working' if cumulative_qty < client_order['quantity'] else 'Filled',
            'last_fill_venue': fill['venue'],
            'last_fill_quantity': fill['quantity'],
            'last_fill_price': fill['price'],
            'total_commission': round(cumulative_commission, 2),
            'total_fees': round(cumulative_fees, 2)
        })
    
    # Save cascading view
    with open('vwap_cascading_view.json', 'w') as f:
        json.dump(cascading_updates, f, indent=2)
    
    # Also create CSV
    import csv
    with open('vwap_cascading_view.csv', 'w', newline='') as f:
        if cascading_updates:
            writer = csv.DictWriter(f, fieldnames=cascading_updates[0].keys())
            writer.writeheader()
            writer.writerows(cascading_updates)
    
    print(f"\n✅ Created vwap_cascading_view with {len(cascading_updates)} updates")
    print(f"   (1 acceptance + {len(fills)} fills)")
    
    # Show sample progression
    print("\n📊 Client View Progression:")
    print("=" * 80)
    
    # Show key moments
    moments = [0, min(10, len(cascading_updates)-1), 
               len(cascading_updates)//2, len(cascading_updates)-1]
    
    for idx in moments:
        if idx < len(cascading_updates):
            update = cascading_updates[idx]
            print(f"{update['update_time']}: "
                  f"{update['filled_quantity']:,}/{update['ordered_quantity']:,} "
                  f"({update['fill_percentage']:.1f}%) "
                  f"@ {update['average_price']:.2f} "
                  f"[{update['state']}]")
    
    return cascading_updates

if __name__ == "__main__":
    create_cascading_view()