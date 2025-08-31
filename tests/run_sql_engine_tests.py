#!/usr/bin/env python3
"""
Automated SQL engine test runner
Compares sql-cli output against pandas-generated expectations
"""

import subprocess
import json
import sys
from pathlib import Path
from io import StringIO
import pandas as pd
import numpy as np
from typing import Tuple, List, Dict, Any

class SqlEngineTestRunner:
    """Run SQL engine tests and validate against expectations"""
    
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.sql_cli = str(self.project_root / "target" / "release" / "sql-cli")
        self.expectations_file = self.project_root / "tests" / "test_expectations.json"
        
        # Build if needed
        if not Path(self.sql_cli).exists():
            print("Building sql-cli...")
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=self.project_root, check=True)
        
        # Load or generate expectations
        if not self.expectations_file.exists():
            print("Generating test expectations...")
            subprocess.run([sys.executable, str(self.project_root / "tests" / "generate_sql_test_expectations.py")],
                          cwd=self.project_root, check=True)
        
        with open(self.expectations_file) as f:
            self.expectations = json.load(f)
    
    def run_sql_query(self, query: str, input_file: str = "data/test_arithmetic.csv") -> Tuple[bool, str, str]:
        """Run a SQL query and return output"""
        cmd = [
            self.sql_cli,
            str(self.project_root / input_file),
            "-q", query,
            "-o", "csv"
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return result.returncode == 0, result.stdout, result.stderr
    
    def compare_csv_output(self, actual: str, expected: str, tolerance: float = 1e-6) -> Tuple[bool, str]:
        """Compare actual CSV output with expected"""
        try:
            actual_df = pd.read_csv(StringIO(actual))
            expected_df = pd.read_csv(StringIO(expected))
            
            # Check shape
            if actual_df.shape != expected_df.shape:
                return False, f"Shape mismatch: {actual_df.shape} != {expected_df.shape}"
            
            # Check columns
            if list(actual_df.columns) != list(expected_df.columns):
                return False, f"Columns mismatch: {list(actual_df.columns)} != {list(expected_df.columns)}"
            
            # Compare values
            for col in actual_df.columns:
                if pd.api.types.is_numeric_dtype(actual_df[col]):
                    if not np.allclose(actual_df[col].fillna(0), expected_df[col].fillna(0), 
                                      rtol=tolerance, atol=tolerance):
                        return False, f"Numeric mismatch in column '{col}'"
                else:
                    if not actual_df[col].fillna("").astype(str).equals(
                           expected_df[col].fillna("").astype(str)):
                        return False, f"String mismatch in column '{col}'"
            
            return True, ""
        except Exception as e:
            return False, f"Error comparing output: {e}"
    
    def run_test_suite(self) -> Tuple[int, int]:
        """Run all test suites"""
        print("\n" + "="*60)
        print("SQL ENGINE TEST SUITE")
        print("="*60)
        
        test_definitions = {
            'arithmetic': [
                ("Basic multiplication", 
                 "SELECT id, price * quantity as total FROM test_arithmetic WHERE id <= 5",
                 'basic_multiplication'),
                
                ("Complex arithmetic with tax",
                 "SELECT id, ROUND(price * quantity * (1 + tax_rate), 2) as total_with_tax FROM test_arithmetic WHERE id <= 5",
                 'complex_arithmetic'),
                
                ("ROUND function",
                 "SELECT id, ROUND(price, 0) as price_int, ROUND(price, 2) as price_2dp FROM test_arithmetic WHERE id <= 3",
                 'round_function'),
                
                ("ABS function",
                 "SELECT id, price - cost as profit, ABS(price - cost) as abs_profit FROM test_arithmetic WHERE id <= 5",
                 'abs_function'),
                
                ("POWER and SQRT",
                 "SELECT id, POWER(quantity, 2) as qty_squared, ROUND(SQRT(price), 2) as price_sqrt FROM test_arithmetic WHERE id <= 5",
                 'power_sqrt'),
                
                ("MOD function",
                 "SELECT id, MOD(id, 3) as id_mod_3, MOD(quantity, 5) as qty_mod_5 FROM test_arithmetic WHERE id <= 10",
                 'mod_function'),
                
                ("PI constant",
                 "SELECT id, ROUND(PI(), 4) as pi_value FROM test_arithmetic WHERE id = 1",
                 'pi_constant'),
                
                ("Nested math functions",
                 "SELECT id, ROUND(SQRT(POWER(price, 2) + POWER(cost, 2)), 2) as hypotenuse FROM test_arithmetic WHERE id <= 5",
                 'nested_math'),
                
                ("Aggregation with math",
                 "SELECT COUNT(*) as total_count, ROUND(AVG(price), 2) as avg_price, ROUND(SUM(price * quantity), 2) as total_revenue FROM test_arithmetic WHERE id <= 20",
                 'aggregation_math'),
                
                ("Division and QUOTIENT",
                 "SELECT id, ROUND(price / quantity, 2) as unit_price, QUOTIENT(price, quantity) as price_quotient FROM test_arithmetic WHERE id <= 5 AND quantity > 0",
                 'division_quotient'),
                
                ("FLOOR and CEILING",
                 "SELECT id, price, FLOOR(price) as price_floor, CEILING(price) as price_ceiling FROM test_arithmetic WHERE id <= 5",
                 'floor_ceiling'),
            ]
        }
        
        total_passed = 0
        total_failed = 0
        failures = []
        
        for category, tests in test_definitions.items():
            print(f"\n{category.upper()} TESTS")
            print("-" * 40)
            
            for test_name, query, expectation_key in tests:
                print(f"  {test_name}...", end=" ")
                
                # Get expected result
                if expectation_key not in self.expectations.get(category, {}):
                    print("⚠️  (no expectation)")
                    continue
                
                expected = self.expectations[category][expectation_key]
                
                # Run query
                success, stdout, stderr = self.run_sql_query(query)
                
                if not success:
                    print(f"❌ (query failed: {stderr[:50]})")
                    total_failed += 1
                    failures.append((test_name, f"Query failed: {stderr}"))
                    continue
                
                # Compare results
                match, error = self.compare_csv_output(stdout, expected)
                
                if match:
                    print("✅")
                    total_passed += 1
                else:
                    print(f"❌ ({error})")
                    total_failed += 1
                    failures.append((test_name, error))
        
        # Print summary
        print("\n" + "="*60)
        print(f"RESULTS: {total_passed} passed, {total_failed} failed")
        
        if failures:
            print("\nFAILURES:")
            for name, error in failures[:5]:  # Show first 5 failures
                print(f"  • {name}: {error}")
            if len(failures) > 5:
                print(f"  ... and {len(failures) - 5} more")
        
        return total_passed, total_failed

def main():
    """Main test runner"""
    runner = SqlEngineTestRunner()
    passed, failed = runner.run_test_suite()
    
    # Exit with appropriate code
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()