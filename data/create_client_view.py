#!/usr/bin/env python3
"""
Create simplified client view of VWAP execution
Shows only what the client sees: accumulated fills over time
"""

import json
from datetime import datetime

def create_client_view():
    # Load the data
    with open('vwap_example_orders.json', 'r') as f:
        orders = json.load(f)
    
    with open('vwap_example_fills.json', 'r') as f:
        fills = json.load(f)
    
    # Find parent order
    parent = [o for o in orders if not o.get('parent_order_id')][0]
    parent_id = parent['order_id']
    
    # Get all fills for this parent
    parent_fills = [f for f in fills if f.get('root_order_id') == parent_id]
    
    # Sort by timestamp
    parent_fills.sort(key=lambda x: x['timestamp'])
    
    # Create client view with cumulative updates
    client_updates = []
    cumulative_qty = 0
    cumulative_value = 0
    
    for fill in parent_fills:
        cumulative_qty += fill['quantity']
        cumulative_value += fill['quantity'] * fill['price']
        avg_price = cumulative_value / cumulative_qty if cumulative_qty > 0 else 0
        
        client_update = {
            'timestamp': fill['timestamp'],
            'order_id': parent_id,
            'client_name': parent['client_name'],
            'ticker': parent['ticker'],
            'side': parent['side'],
            'ordered_quantity': parent['quantity'],
            'filled_quantity': cumulative_qty,
            'remaining_quantity': parent['quantity'] - cumulative_qty,
            'fill_percentage': round((cumulative_qty / parent['quantity']) * 100, 2),
            'average_price': round(avg_price, 4),
            'last_fill_price': fill['price'],
            'last_fill_qty': fill['quantity'],
            'last_fill_venue': fill['venue'],
            'total_venues_used': len(set(f['venue'] for f in parent_fills[:parent_fills.index(fill)+1])),
            'total_fills': parent_fills.index(fill) + 1,
            'status': 'Working' if cumulative_qty < parent['quantity'] else 'Filled'
        }
        client_updates.append(client_update)
    
    # Save client view
    with open('vwap_client_view.json', 'w') as f:
        json.dump(client_updates, f, indent=2)
    
    print(f"Created vwap_client_view.json with {len(client_updates)} updates")
    print(f"Final fill: {cumulative_qty:,} / {parent['quantity']:,} ({cumulative_qty/parent['quantity']*100:.1f}%)")
    print(f"Average price: {cumulative_value/cumulative_qty if cumulative_qty > 0 else 0:.2f}")
    
    # Also create a summary record for the parent
    summary = {
        'order_id': parent_id,
        'client_name': parent['client_name'],
        'ticker': parent['ticker'],
        'side': parent['side'],
        'order_date': parent['timestamp'],
        'ordered_quantity': parent['quantity'],
        'filled_quantity': cumulative_qty,
        'fill_percentage': round((cumulative_qty / parent['quantity']) * 100, 2),
        'average_price': round(cumulative_value / cumulative_qty if cumulative_qty > 0 else 0, 4),
        'total_commission': sum(f.get('commission', 0) for f in parent_fills),
        'total_fees': sum(f.get('fees', 0) for f in parent_fills),
        'venues_used': list(set(f['venue'] for f in parent_fills)),
        'first_fill_time': parent_fills[0]['timestamp'] if parent_fills else None,
        'last_fill_time': parent_fills[-1]['timestamp'] if parent_fills else None,
        'total_fills': len(parent_fills),
        'status': parent['state']
    }
    
    with open('vwap_summary.json', 'w') as f:
        json.dump([summary], f, indent=2)
    
    print(f"Created vwap_summary.json with parent order summary")
    
    return client_updates, summary

if __name__ == "__main__":
    create_client_view()