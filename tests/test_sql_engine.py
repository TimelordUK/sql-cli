#!/usr/bin/env python3
"""
Simple SQL engine tests with clear expected results
"""

import subprocess
import sys
from pathlib import Path
from io import StringIO
import pandas as pd

class SimpleSqlTester:
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.sql_cli = str(self.project_root / "target" / "release" / "sql-cli")
        self.passed = 0
        self.failed = 0
        self.failures = []
        
        # Build if needed
        if not Path(self.sql_cli).exists():
            print("Building sql-cli...")
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=self.project_root, check=True)
    
    def run_query(self, csv_file: str, query: str):
        """Run a query and return the output"""
        cmd = [self.sql_cli, str(self.project_root / "data" / csv_file), "-q", query, "-o", "csv"]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0, result.stdout.strip(), result.stderr.strip()
    
    def test_case(self, name: str, csv_file: str, query: str, expected_check):
        """Run a single test case"""
        print(f"  {name}...", end=" ")
        success, output, error = self.run_query(csv_file, query)
        
        if not success:
            print(f"❌ Query failed: {error[:50]}")
            self.failed += 1
            self.failures.append((name, f"Query failed: {error}"))
            return
        
        # Parse output
        try:
            if output:
                df = pd.read_csv(StringIO(output))
            else:
                df = pd.DataFrame()
            
            # Run the check
            if expected_check(df, output):
                print("✅")
                self.passed += 1
            else:
                print(f"❌ Output mismatch")
                self.failed += 1
                self.failures.append((name, f"Output: {output[:100]}"))
        except Exception as e:
            print(f"❌ {e}")
            self.failed += 1
            self.failures.append((name, str(e)))
    
    def run_tests(self):
        """Run all test cases"""
        print("\n" + "="*60)
        print("SQL ENGINE TESTS")
        print("="*60)
        
        # ARITHMETIC TESTS
        print("\nARITHMETIC OPERATIONS:")
        print("-"*40)
        
        self.test_case(
            "Addition",
            "test_simple_math.csv",
            "SELECT id, a + b as result FROM test_simple_math WHERE id = 1",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 11
        )
        
        self.test_case(
            "Multiplication",
            "test_simple_math.csv",
            "SELECT id, a * b as result FROM test_simple_math WHERE id = 2",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 40
        )
        
        self.test_case(
            "ROUND function",
            "test_simple_math.csv",
            "SELECT id, ROUND(c, 0) as result FROM test_simple_math WHERE id = 3",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 2
        )
        
        self.test_case(
            "ABS function",
            "test_simple_math.csv",
            "SELECT id, ABS(a - d) as result FROM test_simple_math WHERE id = 10",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 80
        )
        
        self.test_case(
            "POWER function",
            "test_simple_math.csv",
            "SELECT id, POWER(a, 2) as result FROM test_simple_math WHERE id = 5",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 25
        )
        
        self.test_case(
            "SQRT function",
            "test_simple_math.csv",
            "SELECT id, SQRT(e) as result FROM test_simple_math WHERE id = 4",
            lambda df, _: len(df) == 1 and abs(df.iloc[0]['result'] - 4) < 0.001
        )
        
        self.test_case(
            "MOD function",
            "test_simple_math.csv",
            "SELECT id, MOD(b, 7) as result FROM test_simple_math WHERE id = 3",
            lambda df, _: len(df) == 1 and df.iloc[0]['result'] == 2
        )
        
        self.test_case(
            "Complex expression",
            "test_simple_math.csv",
            "SELECT id, ROUND((a + b) * c / 2, 1) as result FROM test_simple_math WHERE id = 2",
            lambda df, _: len(df) == 1 and abs(df.iloc[0]['result'] - 11.0) < 0.1
        )
        
        # STRING TESTS
        print("\nSTRING OPERATIONS:")
        print("-"*40)
        
        self.test_case(
            "Contains method",
            "test_simple_strings.csv",
            "SELECT id, name FROM test_simple_strings WHERE name.Contains('li')",
            lambda df, _: len(df) == 2 and set(df['id'].tolist()) == {1, 3}
        )
        
        self.test_case(
            "EndsWith method",
            "test_simple_strings.csv",
            "SELECT id FROM test_simple_strings WHERE email.EndsWith('.com')",
            lambda df, _: len(df) == 6 and set(df['id'].tolist()) == {1, 3, 4, 8, 9, 10}
        )
        
        self.test_case(
            "StartsWith method",
            "test_simple_strings.csv",
            "SELECT id FROM test_simple_strings WHERE status.StartsWith('A')",
            lambda df, _: len(df) == 5 and set(df['id'].tolist()) == {1, 3, 5, 6, 7}
        )
        
        self.test_case(
            "Trim method",
            "test_simple_strings.csv",
            "SELECT id, name.Trim() as trimmed FROM test_simple_strings WHERE id = 4",
            lambda df, _: len(df) == 1 and df.iloc[0]['trimmed'] == 'David'
        )
        
        self.test_case(
            "Length method",
            "test_simple_strings.csv",
            "SELECT id, name.Length() as len FROM test_simple_strings WHERE id = 1",
            lambda df, _: len(df) == 1 and df.iloc[0]['len'] == 5
        )
        
        self.test_case(
            "IndexOf method",
            "test_simple_strings.csv",
            "SELECT id, code.IndexOf('2') as pos FROM test_simple_strings WHERE id = 1",
            lambda df, _: len(df) == 1 and df.iloc[0]['pos'] == 4
        )
        
        # AGGREGATION TESTS
        print("\nAGGREGATION:")
        print("-"*40)
        
        self.test_case(
            "COUNT function",
            "test_simple_math.csv",
            "SELECT COUNT(*) as total FROM test_simple_math WHERE a <= 5",
            lambda df, _: len(df) == 1 and df.iloc[0]['total'] == 5
        )
        
        self.test_case(
            "SUM function",
            "test_simple_math.csv",
            "SELECT SUM(a) as total FROM test_simple_math WHERE a <= 5",
            lambda df, _: len(df) == 1 and df.iloc[0]['total'] == 15
        )
        
        self.test_case(
            "AVG function",
            "test_simple_math.csv",
            "SELECT ROUND(AVG(a), 1) as average FROM test_simple_math WHERE a <= 4",
            lambda df, _: len(df) == 1 and abs(df.iloc[0]['average'] - 2.5) < 0.1
        )
        
        # COMPLEX QUERIES
        print("\nCOMPLEX QUERIES:")
        print("-"*40)
        
        self.test_case(
            "Math in WHERE clause",
            "test_simple_math.csv",
            "SELECT id FROM test_simple_math WHERE a * b > 100",
            lambda df, _: len(df) == 14  # IDs 6-20 have a*b > 100
        )
        
        self.test_case(
            "Multiple conditions",
            "test_simple_strings.csv",
            "SELECT id FROM test_simple_strings WHERE status = 'Active' AND email.EndsWith('.com')",
            lambda df, _: len(df) == 3 and set(df['id'].tolist()) == {1, 3, 9}
        )
        
        # Print summary
        print("\n" + "="*60)
        print(f"RESULTS: {self.passed} passed, {self.failed} failed")
        
        if self.failures:
            print("\nFAILURES:")
            for name, error in self.failures:
                print(f"  • {name}: {error[:100]}")
        
        return self.failed == 0

def main():
    tester = SimpleSqlTester()
    success = tester.run_tests()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()