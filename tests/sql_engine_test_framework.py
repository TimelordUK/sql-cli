#!/usr/bin/env python3
"""
SQL Engine Test Framework for sql-cli
Validates SQL query results against expected outputs generated with pandas
"""

import json
import csv
import subprocess
import pandas as pd
import numpy as np
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
import sys
import tempfile
import os

@dataclass
class TestCase:
    """Represents a single SQL test case"""
    name: str
    description: str
    sql: str
    expected_csv: Optional[str] = None
    expected_json: Optional[str] = None
    expected_rows: Optional[int] = None
    input_file: str = "data/test_arithmetic.csv"
    output_format: str = "csv"
    should_fail: bool = False
    error_contains: Optional[str] = None

class SqlEngineTestFramework:
    """Framework for testing the SQL engine in non-interactive mode"""
    
    def __init__(self, sql_cli_path: str = None):
        """Initialize the test framework"""
        self.project_root = Path(__file__).parent.parent
        self.sql_cli = sql_cli_path or str(self.project_root / "target" / "release" / "sql-cli")
        self.test_data_dir = self.project_root / "data"
        self.test_results_dir = self.project_root / "test_results"
        self.test_results_dir.mkdir(exist_ok=True)
        
        # Check if sql-cli exists
        if not Path(self.sql_cli).exists():
            print(f"Warning: {self.sql_cli} not found. Building...")
            self._build_sql_cli()
    
    def _build_sql_cli(self):
        """Build sql-cli if it doesn't exist"""
        subprocess.run(["cargo", "build", "--release"], 
                      cwd=self.project_root, check=True)
    
    def run_query(self, input_file: str, query: str, 
                  output_format: str = "csv") -> Tuple[bool, str, str]:
        """
        Run a SQL query using sql-cli non-interactive mode
        Returns: (success, stdout, stderr)
        """
        cmd = [
            self.sql_cli,
            input_file,
            "-q", query,
            "-o", output_format
        ]
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return False, "", "Query timed out after 10 seconds"
        except Exception as e:
            return False, "", str(e)
    
    def parse_csv_output(self, output: str) -> pd.DataFrame:
        """Parse CSV output from sql-cli into a pandas DataFrame"""
        if not output.strip():
            return pd.DataFrame()
        
        # Use StringIO to parse CSV
        from io import StringIO
        return pd.read_csv(StringIO(output))
    
    def parse_json_output(self, output: str) -> List[Dict]:
        """Parse JSON output from sql-cli"""
        if not output.strip():
            return []
        return json.loads(output)
    
    def compare_dataframes(self, actual: pd.DataFrame, 
                          expected: pd.DataFrame, 
                          tolerance: float = 1e-6) -> Tuple[bool, str]:
        """
        Compare two dataframes for equality
        Returns: (match, error_message)
        """
        # Check shape
        if actual.shape != expected.shape:
            return False, f"Shape mismatch: actual={actual.shape}, expected={expected.shape}"
        
        # Check column names
        if list(actual.columns) != list(expected.columns):
            return False, f"Column mismatch: actual={list(actual.columns)}, expected={list(expected.columns)}"
        
        # Check data
        for col in actual.columns:
            actual_col = actual[col]
            expected_col = expected[col]
            
            # Handle numeric columns with tolerance
            if pd.api.types.is_numeric_dtype(actual_col) and pd.api.types.is_numeric_dtype(expected_col):
                if not np.allclose(actual_col.fillna(0), expected_col.fillna(0), 
                                  rtol=tolerance, atol=tolerance, equal_nan=True):
                    diff_mask = ~np.isclose(actual_col.fillna(0), expected_col.fillna(0), 
                                           rtol=tolerance, atol=tolerance)
                    diff_indices = actual.index[diff_mask].tolist()
                    return False, f"Numeric mismatch in column '{col}' at indices {diff_indices[:5]}"
            else:
                # String/object comparison
                actual_str = actual_col.fillna("").astype(str)
                expected_str = expected_col.fillna("").astype(str)
                if not actual_str.equals(expected_str):
                    diff_mask = actual_str != expected_str
                    diff_indices = actual.index[diff_mask].tolist()
                    return False, f"String mismatch in column '{col}' at indices {diff_indices[:5]}"
        
        return True, ""
    
    def run_test_case(self, test: TestCase) -> Tuple[bool, str]:
        """
        Run a single test case
        Returns: (passed, error_message)
        """
        print(f"  Running: {test.name}...", end=" ")
        
        # Get full path to input file
        input_path = str(self.project_root / test.input_file)
        
        # Run the query
        success, stdout, stderr = self.run_query(
            input_path, test.sql, test.output_format
        )
        
        # Check if it should fail
        if test.should_fail:
            if success:
                print("❌")
                return False, "Expected query to fail but it succeeded"
            if test.error_contains and test.error_contains not in stderr:
                print("❌")
                return False, f"Expected error to contain '{test.error_contains}' but got: {stderr}"
            print("✅")
            return True, ""
        
        # Check if query failed unexpectedly
        if not success:
            print("❌")
            return False, f"Query failed: {stderr}"
        
        # Validate output based on format
        if test.output_format == "csv":
            actual_df = self.parse_csv_output(stdout)
            
            # Check expected CSV
            if test.expected_csv:
                expected_df = pd.read_csv(StringIO(test.expected_csv))
                match, error = self.compare_dataframes(actual_df, expected_df)
                if not match:
                    print("❌")
                    return False, error
            
            # Check expected row count
            if test.expected_rows is not None:
                if len(actual_df) != test.expected_rows:
                    print("❌")
                    return False, f"Row count mismatch: actual={len(actual_df)}, expected={test.expected_rows}"
        
        elif test.output_format == "json":
            actual_json = self.parse_json_output(stdout)
            
            # Check expected JSON
            if test.expected_json:
                expected_json = json.loads(test.expected_json)
                if actual_json != expected_json:
                    print("❌")
                    return False, f"JSON mismatch"
            
            # Check expected row count
            if test.expected_rows is not None:
                if len(actual_json) != test.expected_rows:
                    print("❌")
                    return False, f"Row count mismatch: actual={len(actual_json)}, expected={test.expected_rows}"
        
        print("✅")
        return True, ""
    
    def run_test_suite(self, tests: List[TestCase]) -> Tuple[int, int]:
        """
        Run a suite of test cases
        Returns: (passed_count, failed_count)
        """
        passed = 0
        failed = 0
        failures = []
        
        for test in tests:
            success, error = self.run_test_case(test)
            if success:
                passed += 1
            else:
                failed += 1
                failures.append((test.name, error))
        
        # Print summary
        print(f"\nResults: {passed} passed, {failed} failed")
        
        if failures:
            print("\nFailures:")
            for name, error in failures:
                print(f"  - {name}: {error}")
        
        return passed, failed
    
    def generate_expected_results_with_pandas(self, 
                                             input_file: str, 
                                             queries: Dict[str, str]) -> Dict[str, pd.DataFrame]:
        """
        Generate expected results using pandas for validation
        This simulates what our SQL engine should produce
        """
        # Load the test data
        df = pd.read_csv(self.project_root / input_file)
        results = {}
        
        for name, query in queries.items():
            # Parse and execute simplified SQL with pandas
            # This is a simplified version - expand as needed
            if "SELECT * FROM" in query:
                results[name] = df.copy()
            # Add more SQL parsing logic as needed
            
        return results

def create_arithmetic_test_cases() -> List[TestCase]:
    """Create test cases for arithmetic functions"""
    tests = []
    
    # Basic arithmetic operations
    tests.append(TestCase(
        name="basic_multiplication",
        description="Test basic multiplication in SELECT",
        sql="SELECT id, price * quantity as total FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    tests.append(TestCase(
        name="complex_arithmetic",
        description="Test complex arithmetic expression",
        sql="SELECT id, ROUND(price * quantity * (1 + tax_rate), 2) as total_with_tax FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    # ROUND function
    tests.append(TestCase(
        name="round_function",
        description="Test ROUND function with different precisions",
        sql="SELECT id, ROUND(price, 0) as price_int, ROUND(price, 2) as price_2dp FROM test_arithmetic WHERE id <= 3",
        expected_rows=3
    ))
    
    # ABS function
    tests.append(TestCase(
        name="abs_function",
        description="Test ABS function",
        sql="SELECT id, price - cost as profit, ABS(price - cost) as abs_profit FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    # POWER and SQRT
    tests.append(TestCase(
        name="power_sqrt",
        description="Test POWER and SQRT functions",
        sql="SELECT id, POWER(quantity, 2) as qty_squared, ROUND(SQRT(price), 2) as price_sqrt FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    # MOD function
    tests.append(TestCase(
        name="mod_function",
        description="Test MOD function",
        sql="SELECT id, MOD(id, 3) as id_mod_3, MOD(quantity, 5) as qty_mod_5 FROM test_arithmetic WHERE id <= 10",
        expected_rows=10
    ))
    
    # Mathematical constants
    tests.append(TestCase(
        name="pi_constant",
        description="Test PI() constant",
        sql="SELECT id, ROUND(PI(), 4) as pi_value FROM test_arithmetic WHERE id = 1",
        expected_rows=1
    ))
    
    # Complex nested functions
    tests.append(TestCase(
        name="nested_math",
        description="Test nested mathematical functions",
        sql="SELECT id, ROUND(SQRT(POWER(price, 2) + POWER(cost, 2)), 2) as hypotenuse FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    # Aggregation with math
    tests.append(TestCase(
        name="aggregation_math",
        description="Test aggregation with mathematical functions",
        sql="SELECT COUNT(*) as total_count, ROUND(AVG(price), 2) as avg_price, ROUND(SUM(price * quantity), 2) as total_revenue FROM test_arithmetic WHERE id <= 20",
        expected_rows=1
    ))
    
    # Division and QUOTIENT
    tests.append(TestCase(
        name="division_quotient",
        description="Test division and QUOTIENT function",
        sql="SELECT id, ROUND(price / quantity, 2) as unit_price, QUOTIENT(price, quantity) as price_quotient FROM test_arithmetic WHERE id <= 5 AND quantity > 0",
        expected_rows=5
    ))
    
    # FLOOR and CEILING
    tests.append(TestCase(
        name="floor_ceiling",
        description="Test FLOOR and CEILING functions",
        sql="SELECT id, price, FLOOR(price) as price_floor, CEILING(price) as price_ceiling FROM test_arithmetic WHERE id <= 5",
        expected_rows=5
    ))
    
    # Error cases
    tests.append(TestCase(
        name="division_by_zero",
        description="Test division by zero handling",
        sql="SELECT id, price / 0 as undefined FROM test_arithmetic WHERE id = 1",
        should_fail=True,
        error_contains="division"
    ))
    
    return tests

def main():
    """Main test runner"""
    print("SQL Engine Test Framework")
    print("=" * 50)
    
    # Initialize framework
    framework = SqlEngineTestFramework()
    
    # Run arithmetic tests
    print("\nArithmetic Function Tests:")
    print("-" * 30)
    arithmetic_tests = create_arithmetic_test_cases()
    passed, failed = framework.run_test_suite(arithmetic_tests)
    
    # Exit with appropriate code
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()