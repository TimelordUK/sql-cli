#!/usr/bin/env python3
"""
Generate test CSV with numeric columns for arithmetic operation testing.
Creates realistic business data with prices, costs, taxes, etc.
"""

import csv
import random
from decimal import Decimal, ROUND_HALF_UP

def generate_arithmetic_csv(filename='test_arithmetic.csv', rows=100):
    """Generate CSV with numeric columns for arithmetic testing."""
    
    random.seed(42)  # For reproducible test data
    
    rows_data = []
    
    for i in range(1, rows + 1):
        # Generate realistic business data
        quantity = random.randint(1, 100)
        base_price = round(random.uniform(10, 500), 2)
        
        # Cost is typically 60-85% of price
        cost = round(base_price * random.uniform(0.60, 0.85), 2)
        
        # Discount is 0-20% of base price
        discount = round(base_price * random.uniform(0, 0.20), 2)
        
        # Tax rate between 5-15%
        tax_rate = round(random.uniform(0.05, 0.15), 2)
        
        # Weight in kg
        weight = round(random.uniform(0.1, 25.0), 2)
        
        # Rating out of 5
        rating = round(random.uniform(1.0, 5.0), 1)
        
        # Commission based on total sale value
        total_sale = quantity * (base_price - discount)
        commission = round(total_sale * random.uniform(0.01, 0.05), 2)
        
        # Profit margin calculation
        profit_margin = round((base_price - cost) / base_price, 2)
        
        rows_data.append({
            'id': i,
            'quantity': quantity,
            'price': base_price,
            'cost': cost,
            'discount': discount,
            'tax_rate': tax_rate,
            'weight': weight,
            'rating': rating,
            'commission': commission,
            'profit_margin': profit_margin
        })
    
    # Write to CSV
    with open(filename, 'w', newline='') as csvfile:
        fieldnames = ['id', 'quantity', 'price', 'cost', 'discount', 
                     'tax_rate', 'weight', 'rating', 'commission', 'profit_margin']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        writer.writerows(rows_data)
    
    print(f"Generated {filename} with {len(rows_data)} rows")
    print("\nColumn descriptions:")
    print("- id: Unique identifier (1-100)")
    print("- quantity: Number of items (1-100)")
    print("- price: Unit price in dollars")
    print("- cost: Unit cost in dollars")
    print("- discount: Discount amount per unit")
    print("- tax_rate: Tax rate as decimal (0.05-0.15)")
    print("- weight: Weight in kg")
    print("- rating: Customer rating (1.0-5.0)")
    print("- commission: Sales commission amount")
    print("- profit_margin: Profit margin as decimal")
    
    print("\nUseful SQL queries to test:")
    print("-- Calculate total revenue with tax")
    print("SELECT id, quantity * (price - discount) * (1 + tax_rate) as total_with_tax FROM test_arithmetic")
    print("\n-- Calculate profit per item")
    print("SELECT id, (price - cost - discount) as profit_per_item FROM test_arithmetic")
    print("\n-- Find high-margin products")
    print("SELECT * FROM test_arithmetic WHERE profit_margin > 0.25")
    print("\n-- Calculate weighted average rating")
    print("SELECT SUM(rating * quantity) / SUM(quantity) as weighted_avg_rating FROM test_arithmetic")
    print("\n-- Complex calculation: ROI")
    print("SELECT id, ((price - cost) * quantity - commission) / (cost * quantity) as roi FROM test_arithmetic")
    print("\n-- Find products where commission exceeds profit")
    print("SELECT * FROM test_arithmetic WHERE commission > (price - cost) * quantity")
    print("\n-- Calculate shipping cost based on weight")
    print("SELECT id, weight * 2.5 + 5 as shipping_cost FROM test_arithmetic")
    print("\n-- Products with good value (high rating, low price)")
    print("SELECT * FROM test_arithmetic WHERE rating > 4 AND price < 100")

if __name__ == '__main__':
    import sys
    
    # Allow custom filename and row count
    filename = sys.argv[1] if len(sys.argv) > 1 else 'test_arithmetic.csv'
    rows = int(sys.argv[2]) if len(sys.argv) > 2 else 100
    
    generate_arithmetic_csv(filename, rows)