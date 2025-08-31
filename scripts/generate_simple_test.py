#!/usr/bin/env python3
"""
Generate simple, predictable test CSV files for easy verification
"""

import csv
from pathlib import Path

def generate_simple_arithmetic_csv():
    """Generate a simple arithmetic test CSV with predictable values"""
    
    data = []
    for i in range(1, 21):  # 20 rows
        data.append({
            'id': i,
            'a': i,          # Simple sequence: 1, 2, 3...
            'b': i * 10,     # 10, 20, 30...
            'c': i * 0.5,    # 0.5, 1.0, 1.5...
            'd': 100 - i,    # 99, 98, 97...
            'e': i ** 2,     # 1, 4, 9, 16...
        })
    
    project_root = Path(__file__).parent.parent
    output_path = project_root / 'data' / 'test_simple_math.csv'
    
    with open(output_path, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)
    
    print(f"Generated test_simple_math.csv with {len(data)} rows")
    print("\nExpected test results:")
    print("  SELECT a + b WHERE id = 1  -> 11")
    print("  SELECT a * b WHERE id = 2  -> 40")
    print("  SELECT ROUND(c, 0) WHERE id = 3  -> 2")
    print("  SELECT ABS(a - d) WHERE id = 10  -> 80")
    print("  SELECT POWER(a, 2) WHERE id = 5  -> 25")
    print("  SELECT SQRT(e) WHERE id = 4  -> 4")
    print("  SELECT MOD(b, 7) WHERE id = 3  -> 2")
    
    return output_path

def generate_simple_string_csv():
    """Generate a simple string test CSV with predictable values"""
    
    data = [
        {'id': 1, 'name': 'Alice', 'email': 'alice@example.com', 'status': 'Active', 'code': 'ABC123'},
        {'id': 2, 'name': 'Bob', 'email': 'bob@test.org', 'status': 'Inactive', 'code': 'DEF456'},
        {'id': 3, 'name': 'Charlie', 'email': 'charlie@company.com', 'status': 'Active', 'code': 'GHI789'},
        {'id': 4, 'name': '  David  ', 'email': 'david@example.com', 'status': 'Pending', 'code': 'JKL012'},
        {'id': 5, 'name': 'Eve', 'email': 'eve@gmail.com', 'status': 'Active', 'code': 'MNO345'},
        {'id': 6, 'name': 'Frank', 'email': 'frank@yahoo.com', 'status': 'Archived', 'code': 'PQR678'},
        {'id': 7, 'name': '  Grace', 'email': 'grace@example.org', 'status': 'Active', 'code': 'STU901'},
        {'id': 8, 'name': 'Henry  ', 'email': 'henry@test.com', 'status': 'Inactive', 'code': 'VWX234'},
        {'id': 9, 'name': 'Iris', 'email': 'iris@company.com', 'status': 'Active', 'code': 'YZA567'},
        {'id': 10, 'name': 'Jack', 'email': 'jack@example.com', 'status': 'New', 'code': 'BCD890'},
    ]
    
    project_root = Path(__file__).parent.parent
    output_path = project_root / 'data' / 'test_simple_strings.csv'
    
    with open(output_path, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)
    
    print(f"\nGenerated test_simple_strings.csv with {len(data)} rows")
    print("\nExpected test results:")
    print("  SELECT * WHERE name.Contains('li')  -> Alice(1), Charlie(3)")
    print("  SELECT * WHERE email.EndsWith('.com')  -> 1, 3, 4, 8, 9, 10")
    print("  SELECT * WHERE status.StartsWith('A')  -> 1, 3, 5, 7, 9, 6")
    print("  SELECT name.Trim() WHERE id = 4  -> 'David'")
    print("  SELECT name.Length() WHERE id = 1  -> 5")
    print("  SELECT code.IndexOf('2') WHERE id = 1  -> 4")
    
    return output_path

def main():
    """Generate all test files"""
    print("Generating simple test CSV files...\n")
    print("=" * 50)
    
    generate_simple_arithmetic_csv()
    print("\n" + "=" * 50)
    generate_simple_string_csv()
    
    print("\n" + "=" * 50)
    print("\nTest files generated successfully!")

if __name__ == "__main__":
    main()