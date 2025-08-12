#!/usr/bin/env python3
"""
Create fill updates that show each fill as a separate event
This creates a dataset where each row represents a fill event with running totals
"""

import json
from datetime import datetime

def create_fill_updates():
    # Load the data
    with open('vwap_example_orders.json', 'r') as f:
        orders = json.load(f)
    
    with open('vwap_example_fills.json', 'r') as f:
        fills = json.load(f)
    
    # Find parent order
    parent = [o for o in orders if not o.get('parent_order_id')][0]
    parent_id = parent['order_id']
    
    # Get all fills for this parent and sort by time
    parent_fills = [f for f in fills if f.get('root_order_id') == parent_id]
    parent_fills.sort(key=lambda x: x['timestamp'])
    
    # Create individual fill events with running totals
    fill_events = []
    running_total = 0
    running_value = 0
    
    for i, fill in enumerate(parent_fills, 1):
        running_total += fill['quantity']
        running_value += fill['quantity'] * fill['price']
        
        fill_event = {
            # Event info
            'event_id': f"FILL_{i:04d}",
            'event_type': 'Fill',
            'event_timestamp': fill['timestamp'],
            
            # Order info
            'order_id': parent_id,
            'client_order_id': parent['client_order_id'],
            'client_name': parent['client_name'],
            'ticker': parent['ticker'],
            'side': parent['side'],
            
            # This fill
            'fill_id': fill['fill_id'],
            'fill_quantity': fill['quantity'],
            'fill_price': fill['price'],
            'fill_venue': fill['venue'],
            
            # Running totals
            'total_ordered': parent['quantity'],
            'total_filled': running_total,
            'total_remaining': parent['quantity'] - running_total,
            'fill_percentage': round((running_total / parent['quantity']) * 100, 2),
            'average_price': round(running_value / running_total, 4),
            
            # Progress
            'fill_number': i,
            'total_fills': len(parent_fills),
            'is_complete': running_total >= parent['quantity']
        }
        fill_events.append(fill_event)
    
    # Save as fill events
    with open('vwap_fill_events.json', 'w') as f:
        json.dump(fill_events, f, indent=2)
    
    print(f"Created vwap_fill_events.json with {len(fill_events)} fill events")
    
    # Also create CSV version for better compatibility
    import csv
    with open('vwap_fill_events.csv', 'w', newline='') as f:
        if fill_events:
            writer = csv.DictWriter(f, fieldnames=fill_events[0].keys())
            writer.writeheader()
            writer.writerows(fill_events)
    
    print(f"Created vwap_fill_events.csv with {len(fill_events)} fill events")
    
    # Show sample
    print("\nSample fill events:")
    for event in fill_events[:5]:
        print(f"  {event['event_timestamp']}: Fill #{event['fill_number']} - "
              f"{event['fill_quantity']:,} @ {event['fill_price']:.2f} "
              f"(Total: {event['total_filled']:,}/{event['total_ordered']:,} = {event['fill_percentage']:.1f}%)")
    
    return fill_events

if __name__ == "__main__":
    create_fill_updates()