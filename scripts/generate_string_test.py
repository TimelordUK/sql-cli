#!/usr/bin/env python3
"""
Generate test CSV with string data for testing SQL string functions
"""

import csv
import random
from pathlib import Path

def generate_string_test_csv(filename='test_strings.csv', rows=50):
    """Generate a CSV file with string data for testing"""
    
    random.seed(42)  # For reproducible test data
    
    # Sample data pools
    first_names = ['Alice', 'Bob', 'Charlie', 'Diana', 'Eve', 'Frank', 'Grace', 'Henry', 'Iris', 'Jack']
    last_names = ['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez']
    departments = ['Engineering', 'Sales', 'Marketing', 'Support', 'Finance', 'HR', 'Operations', 'IT', 'Legal', 'R&D']
    products = ['Widget', 'Gadget', 'Tool', 'Device', 'Instrument', 'Apparatus', 'Machine', 'Equipment', 'System', 'Module']
    categories = ['Premium', 'Standard', 'Basic', 'Professional', 'Enterprise', 'Personal', 'Commercial', 'Industrial']
    domains = ['gmail.com', 'yahoo.com', 'outlook.com', 'company.com', 'example.org', 'test.net']
    
    data = []
    for i in range(1, rows + 1):
        first = random.choice(first_names)
        last = random.choice(last_names)
        
        # Sometimes add spaces for testing trim functions
        if i % 5 == 0:
            first = f"  {first}  "
        if i % 7 == 0:
            last = f" {last} "
        
        full_name = f"{first.strip()} {last.strip()}"
        email = f"{first.strip().lower()}.{last.strip().lower()}@{random.choice(domains)}"
        department = random.choice(departments)
        
        # Product with category
        product_name = f"{random.choice(categories)} {random.choice(products)}"
        
        # Description with varying content for testing Contains
        descriptions = [
            f"High-quality {product_name.lower()} for professional use",
            f"This {product_name.lower()} is perfect for everyday tasks",
            f"Premium solution featuring advanced {random.choice(products).lower()} technology",
            f"Budget-friendly option with essential features",
            f"Enterprise-grade {product_name.lower()} with extended warranty",
            f"Compact and efficient design",
            f"Latest model with improved performance",
            f"Classic design meets modern functionality"
        ]
        description = random.choice(descriptions)
        
        # SKU code
        sku = f"{random.choice(['PRD', 'ITM', 'SKU'])}-{i:04d}-{random.choice(['A', 'B', 'C', 'X', 'Y', 'Z'])}"
        
        # Phone number
        phone = f"+1-{random.randint(100, 999)}-{random.randint(100, 999)}-{random.randint(1000, 9999)}"
        
        # Status
        status = random.choice(['Active', 'Inactive', 'Pending', 'Archived', 'New', 'Updated'])
        
        # Notes with mixed case and content
        notes = random.choice([
            "URGENT: Needs review",
            "approved by manager",
            "Waiting for Customer Feedback",
            "ready for deployment",
            "CANCELLED - see ticket #1234",
            "In Progress",
            "completed successfully",
            ""  # Some empty notes
        ])
        
        data.append({
            'id': i,
            'first_name': first,
            'last_name': last,
            'full_name': full_name,
            'email': email,
            'department': department,
            'product_name': product_name,
            'description': description,
            'sku': sku,
            'phone': phone,
            'status': status,
            'notes': notes
        })
    
    # Write to CSV
    project_root = Path(__file__).parent.parent
    output_path = project_root / 'data' / filename
    
    with open(output_path, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)
    
    print(f"Generated {filename} with {rows} rows")
    print(f"Saved to: {output_path}")
    
    # Show sample
    print("\nSample data (first 3 rows):")
    for row in data[:3]:
        print(f"  ID {row['id']}: {row['full_name']} - {row['product_name']}")

if __name__ == "__main__":
    generate_string_test_csv()