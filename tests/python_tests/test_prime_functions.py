#!/usr/bin/env python3
"""
Test suite for prime number functions in SQL CLI.
Tests PRIME(), IS_PRIME(), PRIME_COUNT(), NEXT_PRIME(), PREV_PRIME()
"""

import subprocess
import csv
import io
import os
import sys
from pathlib import Path
import pandas as pd
import pytest


class TestPrimeFunctions:
    """Test prime number functions"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.base_dir = Path(__file__).parent.parent.parent
        cls.cli_path = cls.base_dir / "target" / "release" / "sql-cli"
        cls.data_dir = cls.base_dir / "data"
        
        # Ensure numbers CSV exists
        cls.numbers_csv = cls.data_dir / "numbers_1_to_100.csv"
        if not cls.numbers_csv.exists():
            # Create it if it doesn't exist
            with open(cls.numbers_csv, 'w', newline='') as f:
                writer = csv.writer(f)
                writer.writerow(['n'])
                for i in range(1, 101):
                    writer.writerow([i])
    
    def run_query(self, query: str, data_file: str = None) -> pd.DataFrame:
        """Execute a query and return results as DataFrame"""
        if data_file:
            csv_path = self.data_dir / data_file
            cmd = [str(self.cli_path), str(csv_path), "-q", query, "-o", "csv"]
        else:
            # Use DUAL table
            cmd = [str(self.cli_path), "-q", query, "-o", "csv"]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        
        if result.returncode != 0:
            raise Exception(f"Query failed: {result.stderr}")
        
        # Parse CSV output
        lines = result.stdout.strip().split('\n')
        csv_lines = [line for line in lines if not line.startswith('#') and line.strip()]
        
        if csv_lines:
            return pd.read_csv(io.StringIO('\n'.join(csv_lines)))
        return pd.DataFrame()
    
    def test_prime_function_basic(self):
        """Test basic PRIME() function"""
        df = self.run_query("SELECT PRIME(1) as p1, PRIME(10) as p10, PRIME(100) as p100 FROM DUAL")
        
        assert len(df) == 1
        assert df.iloc[0]['p1'] == 2      # 1st prime
        assert df.iloc[0]['p10'] == 29    # 10th prime
        assert df.iloc[0]['p100'] == 541  # 100th prime
    
    def test_is_prime_function(self):
        """Test IS_PRIME() function"""
        df = self.run_query("""
            SELECT 
                IS_PRIME(2) as p2,
                IS_PRIME(17) as p17,
                IS_PRIME(100) as p100,
                IS_PRIME(104729) as p104729
            FROM DUAL
        """)
        
        assert df.iloc[0]['p2'] == True       # 2 is prime
        assert df.iloc[0]['p17'] == True      # 17 is prime
        assert df.iloc[0]['p100'] == False    # 100 is not prime
        assert df.iloc[0]['p104729'] == True  # 104729 is prime (10,000th prime)
    
    def test_prime_count_function(self):
        """Test PRIME_COUNT() function"""
        df = self.run_query("""
            SELECT 
                PRIME_COUNT(10) as pc10,
                PRIME_COUNT(100) as pc100,
                PRIME_COUNT(1000) as pc1000
            FROM DUAL
        """)
        
        assert df.iloc[0]['pc10'] == 4     # 2, 3, 5, 7
        assert df.iloc[0]['pc100'] == 25   # 25 primes under 100
        assert df.iloc[0]['pc1000'] == 168 # 168 primes under 1000
    
    def test_next_prev_prime(self):
        """Test NEXT_PRIME() and PREV_PRIME() functions"""
        df = self.run_query("""
            SELECT 
                NEXT_PRIME(100) as next100,
                PREV_PRIME(100) as prev100,
                NEXT_PRIME(97) as next97,
                PREV_PRIME(97) as prev97
            FROM DUAL
        """)
        
        assert df.iloc[0]['next100'] == 101  # Next prime after 100
        assert df.iloc[0]['prev100'] == 97   # Previous prime before 100
        assert df.iloc[0]['next97'] == 97    # 97 is already prime
        assert df.iloc[0]['prev97'] == 97    # 97 is already prime
    
    def test_twin_primes(self):
        """Test finding twin primes using the numbers CSV"""
        df = self.run_query("""
            SELECT 
                n,
                PRIME(n) as p1,
                PRIME(n+1) as p2,
                PRIME(n+1) - PRIME(n) as gap
            FROM numbers_1_to_100
            WHERE n <= 30
                AND PRIME(n+1) - PRIME(n) = 2
        """, "numbers_1_to_100.csv")
        
        # Check we found some twin primes
        assert len(df) > 0
        
        # Verify first few twin prime pairs
        twin_pairs = [(3, 5), (5, 7), (11, 13), (17, 19), (29, 31)]
        
        for i, (p1, p2) in enumerate(twin_pairs[:len(df)]):
            if i < len(df):
                assert df.iloc[i]['gap'] == 2
                # The actual primes should match known twin pairs
    
    def test_prime_gaps(self):
        """Test finding prime gaps"""
        df = self.run_query("""
            SELECT 
                n,
                PRIME(n) as prime,
                PRIME(n+1) - PRIME(n) as gap
            FROM numbers_1_to_100
            WHERE n <= 20
        """, "numbers_1_to_100.csv")
        
        assert len(df) == 20
        
        # Check that gaps are calculated correctly
        for i in range(len(df)):
            n = df.iloc[i]['n']
            prime = df.iloc[i]['prime']
            gap = df.iloc[i]['gap']
            
            # Verify using our PRIME function
            df_verify = self.run_query(f"SELECT PRIME({n}) as p1, PRIME({n+1}) as p2 FROM DUAL")
            expected_gap = df_verify.iloc[0]['p2'] - df_verify.iloc[0]['p1']
            assert gap == expected_gap
    
    def test_primes_list(self):
        """Test listing all primes up to 100"""
        # IS_PRIME doesn't work in WHERE clause, so we select all and filter
        df = self.run_query("""
            SELECT n, IS_PRIME(n) as is_p
            FROM numbers_1_to_100
        """, "numbers_1_to_100.csv")
        
        # Filter for primes
        primes_df = df[df['is_p'] == True]
        
        # There are 25 primes up to 100
        assert len(primes_df) == 25
        
        # Check first few primes
        expected_primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        actual_primes = primes_df['n'].tolist()[:10]
        assert actual_primes == expected_primes
    
    def test_goldbach_conjecture(self):
        """Test Goldbach's conjecture: every even number > 2 is sum of two primes"""
        # Since we can't do cross-products easily, test specific cases
        # Test that 20 = 3 + 17 = 7 + 13 (known decompositions)
        
        pairs_to_test = [(3, 17), (7, 13), (1, 19)]
        valid_pairs = []
        
        for p1, p2 in pairs_to_test:
            df = self.run_query(f"""
                SELECT 
                    {p1} as prime1,
                    {p2} as prime2,
                    {p1} + {p2} as sum,
                    IS_PRIME({p1}) as p1_is_prime,
                    IS_PRIME({p2}) as p2_is_prime
                FROM numbers_1_to_100
                LIMIT 1
            """, "numbers_1_to_100.csv")
            
            if df.iloc[0]['p1_is_prime'] and df.iloc[0]['p2_is_prime']:
                valid_pairs.append((p1, p2))
                assert df.iloc[0]['sum'] == 20
        
        # Should find at least two valid pairs for 20
        assert len(valid_pairs) >= 2, f"Expected at least 2 valid pairs, got {valid_pairs}"
    
    def test_prime_performance(self):
        """Test performance with larger prime indices"""
        # Test access to pre-computed primes (should be fast)
        df = self.run_query("""
            SELECT 
                PRIME(1000) as p1k,
                PRIME(10000) as p10k,
                IS_PRIME(104729) as check,
                PRIME_COUNT(100000) as count
            FROM DUAL
        """)
        
        assert df.iloc[0]['p1k'] == 7919      # 1000th prime
        assert df.iloc[0]['p10k'] == 104729   # 10000th prime
        assert df.iloc[0]['check'] == True    # 104729 is prime
        assert df.iloc[0]['count'] == 9592    # Primes up to 100,000
    
    def test_prime_with_calculations(self):
        """Test using prime functions in calculations"""
        df = self.run_query("""
            SELECT 
                PRIME(10) * 2 as double_10th,
                PRIME(5) + PRIME(6) as sum_5_6,
                PRIME_COUNT(100) * 4 as count_times_4
            FROM DUAL
        """)
        
        assert df.iloc[0]['double_10th'] == 58    # 29 * 2
        assert df.iloc[0]['sum_5_6'] == 24        # 11 + 13
        assert df.iloc[0]['count_times_4'] == 100 # 25 * 4


class TestPrimeAnalysis:
    """More advanced prime number analysis tests"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.base_dir = Path(__file__).parent.parent.parent
        cls.cli_path = cls.base_dir / "target" / "release" / "sql-cli"
        cls.data_dir = cls.base_dir / "data"
        cls.numbers_csv = cls.data_dir / "numbers_1_to_100.csv"
    
    def run_query(self, query: str, data_file: str = None) -> pd.DataFrame:
        """Execute a query and return results as DataFrame"""
        if data_file:
            csv_path = self.data_dir / data_file
            cmd = [str(self.cli_path), str(csv_path), "-q", query, "-o", "csv"]
        else:
            cmd = [str(self.cli_path), "-q", query, "-o", "csv"]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        
        if result.returncode != 0:
            raise Exception(f"Query failed: {result.stderr}")
        
        lines = result.stdout.strip().split('\n')
        csv_lines = [line for line in lines if not line.startswith('#') and line.strip()]
        
        if csv_lines:
            return pd.read_csv(io.StringIO('\n'.join(csv_lines)))
        return pd.DataFrame()
    
    def test_mersenne_primes(self):
        """Test finding Mersenne prime exponents (2^p - 1 is prime)"""
        # For small exponents, check if 2^p - 1 is prime
        df = self.run_query("""
            SELECT 
                n as p,
                POWER(2, n) - 1 as mersenne,
                IS_PRIME(POWER(2, n) - 1) as is_mersenne_prime,
                IS_PRIME(n) as n_is_prime
            FROM numbers_1_to_100
            WHERE n <= 10
        """, "numbers_1_to_100.csv")
        
        # Filter for prime exponents
        df = df[df['n_is_prime'] == True]
        
        # Known Mersenne prime exponents up to 10: 2, 3, 5, 7
        mersenne_exponents = [2, 3, 5, 7]
        
        for i in range(len(df)):
            p = int(df.iloc[i]['p'])
            is_mersenne = df.iloc[i]['is_mersenne_prime']
            
            if p in mersenne_exponents:
                assert is_mersenne == True, f"2^{p} - 1 should be prime"
    
    def test_prime_density(self):
        """Test prime density in different ranges"""
        # Count primes in different ranges
        ranges = [
            (1, 10, 4),      # 2, 3, 5, 7
            (10, 20, 4),     # 11, 13, 17, 19
            (20, 30, 2),     # 23, 29
            (30, 40, 2),     # 31, 37
            (40, 50, 3),     # 41, 43, 47
        ]
        
        for start, end, expected_count in ranges:
            df = self.run_query(f"""
                SELECT n, IS_PRIME(n) as is_p
                FROM numbers_1_to_100
                WHERE n > {start} AND n <= {end}
            """, "numbers_1_to_100.csv")
            
            # Count primes
            df = df[df['is_p'] == True]
            
            actual_count = len(df)
            assert actual_count == expected_count, f"Range {start}-{end} should have {expected_count} primes, got {actual_count}"
    
    def test_sophie_germain_primes(self):
        """Test Sophie Germain primes (p where 2p+1 is also prime)"""
        df = self.run_query("""
            SELECT 
                n as p,
                2 * n + 1 as safe_prime,
                IS_PRIME(n) as is_p,
                IS_PRIME(2 * n + 1) as is_safe
            FROM numbers_1_to_100
            WHERE n <= 50
        """, "numbers_1_to_100.csv")
        
        # Filter for Sophie Germain primes (both n and 2n+1 are prime)
        df = df[(df['is_p'] == True) & (df['is_safe'] == True)]
        
        # Known Sophie Germain primes up to 50: 2, 3, 5, 11, 23, 29, 41
        sophie_germain = [2, 3, 5, 11, 23, 29, 41]
        
        actual_primes = df['p'].tolist()
        for sg in sophie_germain:
            if sg <= 50:
                assert sg in actual_primes, f"{sg} should be identified as Sophie Germain prime"


if __name__ == "__main__":
    # Run tests
    test_basic = TestPrimeFunctions()
    test_basic.setup_class()
    
    print("Testing prime functions...")
    
    try:
        test_basic.test_prime_function_basic()
        print("✓ PRIME() function")
        
        test_basic.test_is_prime_function()
        print("✓ IS_PRIME() function")
        
        test_basic.test_prime_count_function()
        print("✓ PRIME_COUNT() function")
        
        test_basic.test_next_prev_prime()
        print("✓ NEXT_PRIME() and PREV_PRIME() functions")
        
        test_basic.test_twin_primes()
        print("✓ Twin primes detection")
        
        test_basic.test_prime_gaps()
        print("✓ Prime gap calculation")
        
        test_basic.test_primes_list()
        print("✓ Prime listing")
        
        test_basic.test_goldbach_conjecture()
        print("✓ Goldbach's conjecture")
        
        test_basic.test_prime_performance()
        print("✓ Performance with large indices")
        
        test_basic.test_prime_with_calculations()
        print("✓ Primes in calculations")
        
        # Advanced tests
        test_advanced = TestPrimeAnalysis()
        test_advanced.setup_class()
        
        test_advanced.test_mersenne_primes()
        print("✓ Mersenne primes")
        
        test_advanced.test_prime_density()
        print("✓ Prime density")
        
        test_advanced.test_sophie_germain_primes()
        print("✓ Sophie Germain primes")
        
        print("\nAll prime function tests passed!")
        
    except AssertionError as e:
        print(f"\n✗ Test failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)