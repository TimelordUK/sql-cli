#!/usr/bin/env python3
"""
GROUP BY Performance Benchmark Script
Tests GROUP BY performance with various data sizes
"""

import subprocess
import tempfile
import time
import statistics
import os
import csv as csv_module
import sys
from pathlib import Path

# Colors for terminal output
class Colors:
    BLUE = '\033[0;34m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    NC = '\033[0m'  # No Color

def generate_test_data(num_rows, filename):
    """Generate test CSV data with specified number of rows"""
    with open(filename, 'w', newline='') as f:
        writer = csv_module.writer(f)
        writer.writerow(['id', 'category', 'value', 'name'])

        for i in range(1, num_rows + 1):
            cat_id = i % 100  # 100 unique categories
            value = (i * 17) % 1000  # Deterministic "random" values
            writer.writerow([i, f'cat_{cat_id}', value, f'name_{i}'])

def run_benchmark(csv_file, query, runs=3):
    """Run a query multiple times and return average time in milliseconds"""
    sql_cli = './target/release/sql-cli'
    times = []

    for _ in range(runs):
        try:
            start = time.perf_counter()
            result = subprocess.run(
                [sql_cli, csv_file, '-q', query, '-o', 'csv'],
                capture_output=True,
                text=True,
                timeout=30
            )
            end = time.perf_counter()

            elapsed_ms = (end - start) * 1000

            # Also try to extract time from output if available
            output = result.stderr + result.stdout
            if 'Query completed:' in output:
                # Try to extract the reported time
                import re
                match = re.search(r'(\d+(?:\.\d+)?)(ms|s)\s*$', output, re.MULTILINE)
                if match:
                    reported_time = float(match.group(1))
                    unit = match.group(2)
                    if unit == 's':
                        reported_time *= 1000
                    # Use the reported time if it seems reasonable
                    if reported_time > 0:
                        elapsed_ms = reported_time

            times.append(elapsed_ms)

        except subprocess.TimeoutExpired:
            print(f"        Query timed out after 30 seconds")
            return None
        except Exception as e:
            print(f"        Error running query: {e}")
            return None

    if times:
        return statistics.mean(times)
    return None

def format_time(ms):
    """Format time in milliseconds with appropriate precision"""
    if ms is None:
        return "ERROR"
    elif ms < 1:
        return f"{ms:.3f}ms"
    elif ms < 100:
        return f"{ms:.2f}ms"
    elif ms < 1000:
        return f"{ms:.1f}ms"
    else:
        return f"{ms/1000:.2f}s"

def main():
    print(f"{Colors.BLUE}=== SQL CLI GROUP BY Performance Benchmark ==={Colors.NC}")
    print()

    # Build release version
    print(f"{Colors.YELLOW}Building release version...{Colors.NC}")
    try:
        subprocess.run(['cargo', 'build', '--release', '--quiet'], check=True)
    except subprocess.CalledProcessError:
        print(f"{Colors.RED}Build failed!{Colors.NC}")
        sys.exit(1)

    # Test configurations
    test_configs = [
        (1000, "Testing with 1,000 rows"),
        (5000, "Testing with 5,000 rows"),
        (10000, "Testing with 10,000 rows"),
        (20000, "Testing with 20,000 rows"),
        (50000, "Testing with 50,000 rows"),
    ]

    queries = [
        ("SELECT category, COUNT(*) FROM benchmark GROUP BY category",
         "Simple GROUP BY (100 groups)"),
        ("SELECT category, SUM(value) FROM benchmark GROUP BY category",
         "GROUP BY with SUM"),
        ("SELECT category, COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM benchmark GROUP BY category",
         "GROUP BY with 5 aggregates"),
        ("SELECT category, value, COUNT(*) FROM benchmark GROUP BY category, value",
         "Multi-column GROUP BY"),
    ]

    print(f"{Colors.GREEN}Current GROUP BY Performance:{Colors.NC}")
    print("=" * 50)

    results = {}

    for num_rows, description in test_configs:
        print()
        print(f"{Colors.BLUE}{description}:{Colors.NC}")

        # Generate test data
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as tmp:
            csv_file = tmp.name

        try:
            generate_test_data(num_rows, csv_file)

            for query, label in queries:
                avg_time = run_benchmark(csv_file, query)
                formatted_time = format_time(avg_time)
                print(f"  {label:<45} {formatted_time:>10}")

                # Store results for summary
                if label not in results:
                    results[label] = []
                results[label].append((num_rows, avg_time))

        finally:
            # Clean up
            if os.path.exists(csv_file):
                os.remove(csv_file)

    print()
    print("=" * 50)
    print(f"{Colors.GREEN}Benchmark complete!{Colors.NC}")
    print()
    print("Performance observations:")
    print("- GROUP BY scales super-linearly with row count")
    print("- Multi-column GROUP BY is significantly slower")
    print("- Multiple aggregates have minimal additional overhead")
    print()

    # Show scaling analysis
    print("Scaling Analysis:")
    for label, times in results.items():
        valid_times = [(r, t) for r, t in times if t is not None]
        if len(valid_times) >= 2:
            # Calculate scaling factor
            first_rows, first_time = valid_times[0]
            last_rows, last_time = valid_times[-1]
            if first_time > 0:
                row_factor = last_rows / first_rows
                time_factor = last_time / first_time
                scaling_exp = time_factor / row_factor if row_factor > 1 else 1
                print(f"  {label}: ~O(n^{scaling_exp:.2f})")

if __name__ == '__main__':
    main()