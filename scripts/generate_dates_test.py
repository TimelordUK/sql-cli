#!/usr/bin/env python3
"""
Generate a test CSV with date columns at known offsets for testing DATEDIFF.
Creates patterns with 1, 5, 10, 20 day differences both forward and backward.
"""

import csv
from datetime import datetime, timedelta
import random

def generate_dates_csv(filename='test_dates.csv', rows=100):
    """Generate CSV with predictable date patterns."""
    
    # Base date for calculations
    base_date = datetime(2024, 1, 15)
    
    # Define patterns for date differences
    patterns = [
        {'days': 1, 'count': 20},    # 20 rows with 1 day diff
        {'days': 5, 'count': 20},    # 20 rows with 5 day diff
        {'days': 10, 'count': 20},   # 20 rows with 10 day diff
        {'days': 20, 'count': 20},   # 20 rows with 20 day diff
        {'days': -1, 'count': 5},    # 5 rows with -1 day diff (past)
        {'days': -5, 'count': 5},    # 5 rows with -5 day diff
        {'days': -10, 'count': 5},   # 5 rows with -10 day diff
        {'days': -20, 'count': 5},   # 5 rows with -20 day diff
    ]
    
    rows_data = []
    row_id = 1
    
    # Generate rows for each pattern
    for pattern in patterns:
        for i in range(pattern['count']):
            # Create order_date starting from base_date with some variation
            order_date = base_date + timedelta(days=row_id//2)
            
            # ship_date is order_date + pattern days
            ship_date = order_date + timedelta(days=pattern['days'])
            
            # due_date is order_date + 30 days (for DATEADD testing)
            due_date = order_date + timedelta(days=30)
            
            # birth_date for age calculations (random 20-60 years ago)
            years_ago = random.randint(20, 60)
            birth_date = order_date - timedelta(days=years_ago * 365)
            
            # last_login for activity tracking (random 0-90 days ago from order_date)
            last_login = order_date - timedelta(days=random.randint(0, 90))
            
            # Status based on pattern
            if pattern['days'] < 0:
                status = 'early'
            elif pattern['days'] <= 5:
                status = 'on_time'
            else:
                status = 'delayed'
            
            # Customer and amount for realistic data
            customer = f"CUST{row_id:04d}"
            amount = round(random.uniform(100, 5000), 2)
            
            rows_data.append({
                'id': row_id,
                'customer': customer,
                'amount': amount,
                'order_date': order_date.strftime('%Y-%m-%d'),
                'ship_date': ship_date.strftime('%Y-%m-%d'),
                'due_date': due_date.strftime('%Y-%m-%d'),
                'birth_date': birth_date.strftime('%Y-%m-%d'),
                'last_login': last_login.strftime('%Y-%m-%d %H:%M:%S'),
                'status': status,
                'expected_diff': pattern['days'],  # For validation
                'notes': f"Pattern: {pattern['days']} days difference"
            })
            
            row_id += 1
    
    # Write to CSV
    with open(filename, 'w', newline='') as csvfile:
        fieldnames = ['id', 'customer', 'amount', 'order_date', 'ship_date', 
                     'due_date', 'birth_date', 'last_login', 'status', 
                     'expected_diff', 'notes']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        writer.writerows(rows_data)
    
    print(f"Generated {filename} with {len(rows_data)} rows")
    print("\nDate patterns in the file:")
    print("- Rows 1-20: ship_date = order_date + 1 day")
    print("- Rows 21-40: ship_date = order_date + 5 days")
    print("- Rows 41-60: ship_date = order_date + 10 days")
    print("- Rows 61-80: ship_date = order_date + 20 days")
    print("- Rows 81-85: ship_date = order_date - 1 day (early)")
    print("- Rows 86-90: ship_date = order_date - 5 days (early)")
    print("- Rows 91-95: ship_date = order_date - 10 days (early)")
    print("- Rows 96-100: ship_date = order_date - 20 days (early)")
    print("\nAdditional columns:")
    print("- due_date: Always order_date + 30 days")
    print("- birth_date: Random 20-60 years before order_date")
    print("- last_login: Random 0-90 days before order_date")
    print("\nTest queries to try:")
    print("SELECT id, DATEDIFF('day', order_date, ship_date) as delivery_days, expected_diff FROM test_dates")
    print("SELECT id, DATEDIFF('day', birth_date, order_date) / 365 as age_years FROM test_dates")
    print("SELECT id, DATEDIFF('day', last_login, NOW()) as days_inactive FROM test_dates")
    print("SELECT id, DATEADD('day', 30, order_date) as calculated_due, due_date FROM test_dates")

if __name__ == '__main__':
    generate_dates_csv()