#!/usr/bin/env python3
"""
Generate expected results for SQL queries using pandas
This ensures our SQL engine produces the same results as pandas
"""

import pandas as pd
import numpy as np
import json
from pathlib import Path
from typing import Dict, Any, List
import math

class SqlTestExpectationGenerator:
    """Generate expected results for SQL queries using pandas"""
    
    def __init__(self, data_file: str):
        """Initialize with test data"""
        self.project_root = Path(__file__).parent.parent
        self.data_path = self.project_root / data_file
        self.df = pd.read_csv(self.data_path)
        
    def generate_arithmetic_expectations(self) -> Dict[str, Any]:
        """Generate expected results for arithmetic function tests"""
        expectations = {}
        
        # Basic multiplication
        result = self.df[self.df['id'] <= 5][['id']].copy()
        result['total'] = self.df[self.df['id'] <= 5]['price'] * self.df[self.df['id'] <= 5]['quantity']
        expectations['basic_multiplication'] = result.to_csv(index=False)
        
        # Complex arithmetic with tax
        result = self.df[self.df['id'] <= 5][['id']].copy()
        result['total_with_tax'] = np.round(
            self.df[self.df['id'] <= 5]['price'] * 
            self.df[self.df['id'] <= 5]['quantity'] * 
            (1 + self.df[self.df['id'] <= 5]['tax_rate']), 2
        )
        expectations['complex_arithmetic'] = result.to_csv(index=False)
        
        # ROUND function
        result = self.df[self.df['id'] <= 3][['id']].copy()
        result['price_int'] = np.round(self.df[self.df['id'] <= 3]['price'], 0)
        result['price_2dp'] = np.round(self.df[self.df['id'] <= 3]['price'], 2)
        expectations['round_function'] = result.to_csv(index=False)
        
        # ABS function
        result = self.df[self.df['id'] <= 5][['id']].copy()
        result['profit'] = self.df[self.df['id'] <= 5]['price'] - self.df[self.df['id'] <= 5]['cost']
        result['abs_profit'] = np.abs(result['profit'])
        expectations['abs_function'] = result.to_csv(index=False)
        
        # POWER and SQRT
        result = self.df[self.df['id'] <= 5][['id']].copy()
        result['qty_squared'] = np.power(self.df[self.df['id'] <= 5]['quantity'], 2)
        result['price_sqrt'] = np.round(np.sqrt(self.df[self.df['id'] <= 5]['price']), 2)
        expectations['power_sqrt'] = result.to_csv(index=False)
        
        # MOD function
        result = self.df[self.df['id'] <= 10][['id']].copy()
        result['id_mod_3'] = self.df[self.df['id'] <= 10]['id'] % 3
        result['qty_mod_5'] = self.df[self.df['id'] <= 10]['quantity'] % 5
        expectations['mod_function'] = result.to_csv(index=False)
        
        # PI constant
        result = self.df[self.df['id'] == 1][['id']].copy()
        result['pi_value'] = round(math.pi, 4)
        expectations['pi_constant'] = result.to_csv(index=False)
        
        # Nested math (hypotenuse)
        result = self.df[self.df['id'] <= 5][['id']].copy()
        result['hypotenuse'] = np.round(
            np.sqrt(
                np.power(self.df[self.df['id'] <= 5]['price'], 2) + 
                np.power(self.df[self.df['id'] <= 5]['cost'], 2)
            ), 2
        )
        expectations['nested_math'] = result.to_csv(index=False)
        
        # Aggregation with math
        subset = self.df[self.df['id'] <= 20]
        result = pd.DataFrame({
            'total_count': [len(subset)],
            'avg_price': [round(subset['price'].mean(), 2)],
            'total_revenue': [round((subset['price'] * subset['quantity']).sum(), 2)]
        })
        expectations['aggregation_math'] = result.to_csv(index=False)
        
        # Division and QUOTIENT
        subset = self.df[(self.df['id'] <= 5) & (self.df['quantity'] > 0)].copy()
        result = subset[['id']].copy()
        result['unit_price'] = np.round(subset['price'] / subset['quantity'], 2)
        result['price_quotient'] = np.floor(subset['price'] / subset['quantity'])
        expectations['division_quotient'] = result.to_csv(index=False)
        
        # FLOOR and CEILING
        result = self.df[self.df['id'] <= 5][['id', 'price']].copy()
        result['price_floor'] = np.floor(self.df[self.df['id'] <= 5]['price'])
        result['price_ceiling'] = np.ceil(self.df[self.df['id'] <= 5]['price'])
        expectations['floor_ceiling'] = result.to_csv(index=False)
        
        return expectations
    
    def generate_string_expectations(self) -> Dict[str, Any]:
        """Generate expected results for string function tests"""
        # String tests will be done with a different dataset that has string columns
        # For now, return empty dict since test_arithmetic.csv is numeric-focused
        return {}
    
    def save_expectations(self, output_file: str = "test_expectations.json"):
        """Save all expectations to a JSON file"""
        all_expectations = {
            'arithmetic': self.generate_arithmetic_expectations(),
            'string': self.generate_string_expectations()
        }
        
        output_path = self.project_root / "tests" / output_file
        with open(output_path, 'w') as f:
            json.dump(all_expectations, f, indent=2)
        
        print(f"Saved expectations to {output_path}")
        return all_expectations

def main():
    """Generate test expectations"""
    generator = SqlTestExpectationGenerator("data/test_arithmetic.csv")
    expectations = generator.save_expectations()
    
    # Print sample
    print("\nSample expectations generated:")
    for category, tests in expectations.items():
        print(f"\n{category.upper()} tests: {len(tests)} test cases")
        for test_name in list(tests.keys())[:3]:
            print(f"  - {test_name}")

if __name__ == "__main__":
    main()