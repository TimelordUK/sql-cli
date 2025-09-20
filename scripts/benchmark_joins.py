#!/usr/bin/env python3
"""
Simple JOIN performance benchmark for SQL CLI
Tests both hash joins (equality) and nested loop joins (inequality)
"""

import subprocess
import time
import sys
import statistics

# Colors for terminal output
class Colors:
    BLUE = '\033[0;34m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    CYAN = '\033[0;36m'
    BOLD = '\033[1m'
    NC = '\033[0m'  # No Color

def run_query(query: str, runs: int = 3) -> float:
    """Run a query multiple times and return average time in milliseconds"""
    times = []

    for _ in range(runs):
        try:
            start = time.perf_counter()
            result = subprocess.run(
                ['./target/release/sql-cli', '-q', query, '-o', 'csv'],
                capture_output=True,
                text=True,
                timeout=30
            )
            end = time.perf_counter()

            if result.returncode == 0:
                elapsed_ms = (end - start) * 1000
                times.append(elapsed_ms)
            else:
                print(f"Error: {result.stderr}")
                return None

        except subprocess.TimeoutExpired:
            print("Timeout!")
            return None

    return statistics.mean(times) if times else None

def format_time(ms: float) -> str:
    """Format time for display"""
    if ms is None:
        return "ERROR"
    elif ms < 100:
        return f"{ms:.2f}ms"
    elif ms < 1000:
        return f"{ms:.1f}ms"
    else:
        return f"{ms/1000:.2f}s"

def main():
    print(f"{Colors.BOLD}{Colors.BLUE}=== SQL CLI JOIN Performance Benchmarks ==={Colors.NC}")
    print("=" * 60)

    # Build release version
    print(f"{Colors.YELLOW}Building release version...{Colors.NC}")
    try:
        subprocess.run(['cargo', 'build', '--release', '--quiet'], check=True)
    except subprocess.CalledProcessError:
        print(f"{Colors.RED}Build failed!{Colors.NC}")
        sys.exit(1)

    # Test different JOIN sizes
    test_configs = [
        (10, "Tiny"),
        (50, "Small"),
        (100, "Medium"),
        (200, "Large"),
        (500, "Extra Large"),
    ]

    print(f"\n{Colors.CYAN}Testing Hash JOIN (equality) - O(n+m) expected{Colors.NC}")
    print("-" * 60)
    print(f"{'Size':<15} {'Rows Joined':<15} {'Result Count':<15} {'Time':<10}")

    for size, label in test_configs:
        query = f"""
        WITH t1 AS (SELECT value as n FROM RANGE(1, {size})),
             t2 AS (SELECT value as m FROM RANGE(1, {size}))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n = m
        """
        time_ms = run_query(query, runs=3)
        print(f"{label:<15} {size}x{size:<14} {size:<15} {format_time(time_ms):<10}")

    print(f"\n{Colors.CYAN}Testing Nested Loop JOIN (inequality <=) - O(n*m) expected{Colors.NC}")
    print("-" * 60)
    print(f"{'Size':<15} {'Rows Joined':<15} {'Result Count':<15} {'Time':<10}")

    for size, label in test_configs:
        query = f"""
        WITH t1 AS (SELECT value as n FROM RANGE(1, {size})),
             t2 AS (SELECT value as m FROM RANGE(1, {size}))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n <= m
        """
        time_ms = run_query(query, runs=3)
        result_count = size * (size + 1) // 2  # Triangular number formula
        print(f"{label:<15} {size}x{size:<14} {result_count:<15} {format_time(time_ms):<10}")

    # Test cumulative sum pattern
    print(f"\n{Colors.CYAN}Testing Cumulative Sum Pattern (self-join with GROUP BY){Colors.NC}")
    print("-" * 60)
    print(f"{'Size':<15} {'Operation':<25} {'Time':<10}")

    for size, label in [(10, "Tiny"), (20, "Small"), (30, "Medium"), (50, "Large")]:
        query = f"""
        WITH nums AS (SELECT value as n FROM RANGE(1, {size})),
             nums2 AS (SELECT value as m FROM RANGE(1, {size}))
        SELECT n, SUM(m) FROM nums INNER JOIN nums2 ON n >= m GROUP BY n
        """
        time_ms = run_query(query, runs=3)
        print(f"{label:<15} {'Cumulative sum to ' + str(size):<25} {format_time(time_ms):<10}")

    # Test LEFT JOIN with inequality
    print(f"\n{Colors.CYAN}Testing LEFT JOIN with inequality{Colors.NC}")
    print("-" * 60)
    print(f"{'Size':<15} {'Operation':<25} {'Time':<10}")

    for size, label in test_configs[:4]:  # Skip the largest
        query = f"""
        WITH small AS (SELECT value as s FROM RANGE(1, {size})),
             large AS (SELECT value * 2 as l FROM RANGE(1, {size}))
        SELECT COUNT(*) FROM small LEFT JOIN large ON s < l
        """
        time_ms = run_query(query, runs=3)
        print(f"{label:<15} {'LEFT JOIN s < l*2':<25} {format_time(time_ms):<10}")

    # Performance comparison
    print(f"\n{Colors.GREEN}=== Performance Analysis ==={Colors.NC}")
    print("-" * 60)

    # Compare small vs large for both types
    small_eq = run_query("""
        WITH t1 AS (SELECT value as n FROM RANGE(1, 50)),
             t2 AS (SELECT value as m FROM RANGE(1, 50))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n = m
    """)

    large_eq = run_query("""
        WITH t1 AS (SELECT value as n FROM RANGE(1, 200)),
             t2 AS (SELECT value as m FROM RANGE(1, 200))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n = m
    """)

    small_ineq = run_query("""
        WITH t1 AS (SELECT value as n FROM RANGE(1, 50)),
             t2 AS (SELECT value as m FROM RANGE(1, 50))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n <= m
    """)

    large_ineq = run_query("""
        WITH t1 AS (SELECT value as n FROM RANGE(1, 200)),
             t2 AS (SELECT value as m FROM RANGE(1, 200))
        SELECT COUNT(*) FROM t1 INNER JOIN t2 ON n <= m
    """)

    if all([small_eq, large_eq, small_ineq, large_ineq]):
        eq_scaling = large_eq / small_eq
        ineq_scaling = large_ineq / small_ineq
        size_scaling = (200 * 200) / (50 * 50)  # 16x more comparisons

        print(f"Size increase: 50x50 → 200x200 ({size_scaling:.1f}x more comparisons)")
        print(f"Hash JOIN time increase: {eq_scaling:.1f}x (near-linear scaling)")
        print(f"Nested loop JOIN time increase: {ineq_scaling:.1f}x (quadratic scaling)")

        if eq_scaling < 10:
            print(f"{Colors.GREEN}✓ Hash JOIN shows good O(n) scaling{Colors.NC}")
        else:
            print(f"{Colors.YELLOW}⚠ Hash JOIN scaling higher than expected{Colors.NC}")

        if ineq_scaling > 10:
            print(f"{Colors.YELLOW}⚠ Nested loop JOIN shows expected O(n²) scaling{Colors.NC}")
        else:
            print(f"{Colors.GREEN}✓ Nested loop JOIN performing better than O(n²){Colors.NC}")

if __name__ == '__main__':
    main()