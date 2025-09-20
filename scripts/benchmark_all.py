#!/usr/bin/env python3
"""
Comprehensive SQL CLI Benchmark Suite
Tests all major operations and tracks performance over time
"""

import subprocess
import tempfile
import time
import statistics
import os
import csv as csv_module
import sys
import json
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

# Colors for terminal output
class Colors:
    BLUE = '\033[0;34m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    CYAN = '\033[0;36m'
    MAGENTA = '\033[0;35m'
    BOLD = '\033[1m'
    NC = '\033[0m'  # No Color

class BenchmarkSuite:
    def __init__(self, sql_cli_path='./target/release/sql-cli'):
        self.sql_cli = sql_cli_path
        self.results = {}
        self.test_sizes = [1000, 5000, 10000, 20000, 50000]

    def generate_test_data(self, num_rows: int, filename: str):
        """Generate test CSV data with specified number of rows"""
        with open(filename, 'w', newline='') as f:
            writer = csv_module.writer(f)
            writer.writerow(['id', 'name', 'email', 'category', 'value', 'timestamp', 'description'])

            for i in range(1, num_rows + 1):
                name = f"user_{i % 1000}"
                email = f"user{i}@example{i % 100}.com"
                cat = f"category_{i % 50}"
                value = (i * 17) % 10000
                day = (i % 30) + 1
                timestamp = f"2024-01-{day:02d} 12:00:00"
                desc = f"Description for item {i} with some text that might match patterns"
                writer.writerow([i, name, email, cat, value, timestamp, desc])

    def run_query(self, csv_file: str, query: str, runs: int = 3) -> Optional[float]:
        """Run a query multiple times and return average time in milliseconds"""
        times = []

        for _ in range(runs):
            try:
                start = time.perf_counter()
                result = subprocess.run(
                    [self.sql_cli, csv_file, '-q', query, '-o', 'csv'],
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                end = time.perf_counter()

                elapsed_ms = (end - start) * 1000

                # Try to extract reported time from output
                output = result.stderr + result.stdout
                if 'Query completed:' in output:
                    import re
                    match = re.search(r'(\d+(?:\.\d+)?)(ms|s)\s*$', output, re.MULTILINE)
                    if match:
                        reported_time = float(match.group(1))
                        if match.group(2) == 's':
                            reported_time *= 1000
                        if reported_time > 0:
                            elapsed_ms = reported_time

                times.append(elapsed_ms)

            except subprocess.TimeoutExpired:
                return None
            except Exception:
                return None

        return statistics.mean(times) if times else None

    def format_time(self, ms: Optional[float]) -> str:
        """Format time for display"""
        if ms is None:
            return "ERROR"
        elif ms < 1:
            return f"{ms:.3f}ms"
        elif ms < 100:
            return f"{ms:.2f}ms"
        elif ms < 1000:
            return f"{ms:.1f}ms"
        elif ms < 10000:
            return f"{ms/1000:.2f}s"
        else:
            return f"{ms/1000:.1f}s"

    def benchmark_group_by(self, csv_file: str, num_rows: int) -> Dict[str, float]:
        """Benchmark GROUP BY operations"""
        queries = [
            ("Simple GROUP BY",
             "SELECT category, COUNT(*) FROM benchmark GROUP BY category"),
            ("GROUP BY with SUM",
             "SELECT category, SUM(value) FROM benchmark GROUP BY category"),
            ("GROUP BY with multiple aggregates",
             "SELECT category, COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM benchmark GROUP BY category"),
            ("Multi-column GROUP BY",
             "SELECT category, value % 10 as bucket, COUNT(*) FROM benchmark GROUP BY category, value % 10"),
        ]

        results = {}
        for name, query in queries:
            time_ms = self.run_query(csv_file, query)
            results[name] = time_ms
            print(f"    {name:<45} {self.format_time(time_ms):>10}")

        return results

    def benchmark_like(self, csv_file: str, num_rows: int) -> Dict[str, float]:
        """Benchmark LIKE pattern matching"""
        queries = [
            ("LIKE with suffix wildcard",
             "SELECT COUNT(*) FROM benchmark WHERE email LIKE '%@example10.com'"),
            ("LIKE with prefix wildcard",
             "SELECT COUNT(*) FROM benchmark WHERE name LIKE '%_100'"),
            ("LIKE with both wildcards",
             "SELECT COUNT(*) FROM benchmark WHERE description LIKE '%item%text%'"),
            ("Multiple LIKE conditions",
             "SELECT COUNT(*) FROM benchmark WHERE email LIKE '%@example%.com' AND name LIKE 'user_%'"),
            ("LIKE with result set",
             "SELECT * FROM benchmark WHERE email LIKE '%@example1.com' LIMIT 100"),
        ]

        results = {}
        for name, query in queries:
            time_ms = self.run_query(csv_file, query)
            results[name] = time_ms
            print(f"    {name:<45} {self.format_time(time_ms):>10}")

        return results

    def benchmark_window_functions(self, csv_file: str, num_rows: int) -> Dict[str, float]:
        """Benchmark window functions (LAG/LEAD/ROW_NUMBER)"""
        # Limit rows for window functions to keep reasonable
        limit = min(1000, num_rows // 2)

        queries = [
            ("LAG function",
             f"SELECT id, value, LAG(value, 1) OVER (ORDER BY id) as prev FROM benchmark LIMIT {limit}"),
            ("LEAD function",
             f"SELECT id, value, LEAD(value, 1) OVER (ORDER BY id) as next FROM benchmark LIMIT {limit}"),
            ("LAG + LEAD combined",
             f"SELECT id, value, LAG(value, 1) OVER (ORDER BY id) as prev, LEAD(value, 1) OVER (ORDER BY id) as next FROM benchmark LIMIT {limit}"),
            ("LAG with PARTITION BY",
             "SELECT id, category, value, LAG(value, 1) OVER (PARTITION BY category ORDER BY id) FROM benchmark WHERE category IN ('category_1', 'category_2', 'category_3')"),
            ("ROW_NUMBER",
             f"SELECT id, value, ROW_NUMBER() OVER (ORDER BY value DESC) as rank FROM benchmark LIMIT {limit}"),
        ]

        results = {}
        for name, query in queries:
            time_ms = self.run_query(csv_file, query)
            results[name] = time_ms
            print(f"    {name:<45} {self.format_time(time_ms):>10}")

        return results

    def benchmark_basic_operations(self, csv_file: str, num_rows: int) -> Dict[str, float]:
        """Benchmark basic SQL operations"""
        queries = [
            ("Simple SELECT *",
             "SELECT * FROM benchmark LIMIT 100"),
            ("WHERE numeric comparison",
             "SELECT * FROM benchmark WHERE value > 5000"),
            ("WHERE string equality",
             "SELECT * FROM benchmark WHERE category = 'category_10'"),
            ("Complex WHERE (3 conditions)",
             "SELECT * FROM benchmark WHERE value > 5000 AND category = 'category_10' AND name LIKE 'user_%'"),
            ("ORDER BY single column",
             "SELECT * FROM benchmark ORDER BY value DESC LIMIT 100"),
            ("ORDER BY multiple columns",
             "SELECT * FROM benchmark ORDER BY category, value DESC LIMIT 100"),
            ("COUNT DISTINCT",
             "SELECT COUNT(DISTINCT category) FROM benchmark"),
        ]

        results = {}
        for name, query in queries:
            time_ms = self.run_query(csv_file, query)
            results[name] = time_ms
            print(f"    {name:<45} {self.format_time(time_ms):>10}")

        return results

    def run_all_benchmarks(self, sizes: Optional[List[int]] = None):
        """Run all benchmarks for specified data sizes"""
        if sizes is None:
            sizes = self.test_sizes

        print(f"{Colors.BOLD}{Colors.BLUE}=== SQL CLI Comprehensive Benchmark Suite ==={Colors.NC}")
        print(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("=" * 70)

        all_results = {}

        for num_rows in sizes:
            print(f"\n{Colors.BOLD}{Colors.GREEN}Testing with {num_rows:,} rows:{Colors.NC}")
            print("-" * 70)

            # Generate test data
            with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as tmp:
                csv_file = tmp.name

            try:
                self.generate_test_data(num_rows, csv_file)

                row_results = {}

                # Run each category of benchmarks
                print(f"\n{Colors.CYAN}Basic Operations:{Colors.NC}")
                row_results['basic'] = self.benchmark_basic_operations(csv_file, num_rows)

                print(f"\n{Colors.CYAN}LIKE Pattern Matching:{Colors.NC}")
                row_results['like'] = self.benchmark_like(csv_file, num_rows)

                print(f"\n{Colors.CYAN}GROUP BY Operations:{Colors.NC}")
                row_results['group_by'] = self.benchmark_group_by(csv_file, num_rows)

                print(f"\n{Colors.CYAN}Window Functions:{Colors.NC}")
                row_results['window'] = self.benchmark_window_functions(csv_file, num_rows)

                all_results[num_rows] = row_results

            finally:
                if os.path.exists(csv_file):
                    os.remove(csv_file)

        self.results = all_results
        return all_results

    def print_summary(self):
        """Print summary and scaling analysis"""
        print(f"\n{Colors.BOLD}{Colors.GREEN}=== Performance Summary ==={Colors.NC}")
        print("=" * 70)

        # Collect all operations for scaling analysis
        operations = {}
        for num_rows, categories in self.results.items():
            for category, queries in categories.items():
                for query_name, time_ms in queries.items():
                    if query_name not in operations:
                        operations[query_name] = []
                    operations[query_name].append((num_rows, time_ms))

        # Calculate scaling for each operation
        print(f"\n{Colors.YELLOW}Scaling Analysis:{Colors.NC}")
        print("-" * 70)

        for op_name, times in sorted(operations.items()):
            valid_times = [(r, t) for r, t in times if t is not None]
            if len(valid_times) >= 2:
                first_rows, first_time = valid_times[0]
                last_rows, last_time = valid_times[-1]

                if first_time > 0 and last_time > 0:
                    row_factor = last_rows / first_rows
                    time_factor = last_time / first_time

                    if row_factor > 1:
                        # Calculate approximate complexity
                        import math
                        scaling_exp = math.log(time_factor) / math.log(row_factor)

                        # Determine complexity class
                        if scaling_exp < 0.3:
                            complexity = "O(1) - constant"
                            color = Colors.GREEN
                        elif scaling_exp < 0.7:
                            complexity = "O(log n) - logarithmic"
                            color = Colors.GREEN
                        elif scaling_exp < 1.3:
                            complexity = "O(n) - linear"
                            color = Colors.CYAN
                        elif scaling_exp < 1.7:
                            complexity = "O(n log n)"
                            color = Colors.YELLOW
                        elif scaling_exp < 2.3:
                            complexity = "O(n²) - quadratic"
                            color = Colors.RED
                        else:
                            complexity = f"O(n^{scaling_exp:.1f})"
                            color = Colors.RED

                        print(f"  {op_name:<40} {color}{complexity}{Colors.NC}")

        # Find best and worst performers
        print(f"\n{Colors.YELLOW}Performance Highlights:{Colors.NC}")
        print("-" * 70)

        # Find fastest operations at 50k rows (or largest size tested)
        largest_size = max(self.results.keys())
        all_times = []
        for category, queries in self.results[largest_size].items():
            for query_name, time_ms in queries.items():
                if time_ms is not None:
                    all_times.append((query_name, time_ms))

        all_times.sort(key=lambda x: x[1])

        print(f"\n{Colors.GREEN}Fastest operations at {largest_size:,} rows:{Colors.NC}")
        for name, time_ms in all_times[:5]:
            print(f"  {name:<40} {self.format_time(time_ms):>10}")

        print(f"\n{Colors.RED}Slowest operations at {largest_size:,} rows:{Colors.NC}")
        for name, time_ms in all_times[-5:]:
            print(f"  {name:<40} {self.format_time(time_ms):>10}")

    def save_results(self, filename: Optional[str] = None):
        """Save results to JSON file for comparison"""
        if filename is None:
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            filename = f"benchmark_results_{timestamp}.json"

        output = {
            'timestamp': datetime.now().isoformat(),
            'results': self.results
        }

        with open(filename, 'w') as f:
            json.dump(output, f, indent=2)

        print(f"\n{Colors.GREEN}Results saved to {filename}{Colors.NC}")
        return filename

    def compare_results(self, baseline_file: str, current_file: Optional[str] = None):
        """Compare current results with baseline"""
        with open(baseline_file, 'r') as f:
            baseline = json.load(f)

        if current_file:
            with open(current_file, 'r') as f:
                current = json.load(f)
        else:
            current = {'results': self.results}

        print(f"\n{Colors.BOLD}{Colors.MAGENTA}=== Performance Comparison ==={Colors.NC}")
        print(f"Baseline: {baseline.get('timestamp', 'Unknown')}")
        print(f"Current:  {current.get('timestamp', datetime.now().isoformat())}")
        print("=" * 70)

        # Compare each operation
        improvements = []
        regressions = []

        for num_rows in sorted(current['results'].keys()):
            if str(num_rows) not in baseline['results']:
                continue

            print(f"\n{Colors.CYAN}Results for {num_rows:,} rows:{Colors.NC}")

            for category in current['results'][num_rows]:
                for query in current['results'][num_rows][category]:
                    current_time = current['results'][num_rows][category].get(query)
                    baseline_time = baseline['results'][str(num_rows)].get(category, {}).get(query)

                    if current_time and baseline_time:
                        improvement = (baseline_time - current_time) / baseline_time * 100

                        if improvement > 5:
                            color = Colors.GREEN
                            symbol = "↑"
                            improvements.append((query, improvement))
                        elif improvement < -5:
                            color = Colors.RED
                            symbol = "↓"
                            regressions.append((query, improvement))
                        else:
                            color = Colors.YELLOW
                            symbol = "→"

                        print(f"  {query:<40} {color}{symbol} {improvement:+.1f}%{Colors.NC}")

        # Summary
        if improvements:
            print(f"\n{Colors.GREEN}Top Improvements:{Colors.NC}")
            for query, improvement in sorted(improvements, key=lambda x: x[1], reverse=True)[:5]:
                print(f"  {query:<40} {improvement:+.1f}%")

        if regressions:
            print(f"\n{Colors.RED}Regressions:{Colors.NC}")
            for query, improvement in sorted(regressions, key=lambda x: x[1])[:5]:
                print(f"  {query:<40} {improvement:+.1f}%")

def main():
    parser = argparse.ArgumentParser(description='SQL CLI Comprehensive Benchmark Suite')
    parser.add_argument('--sizes', nargs='+', type=int,
                       help='Data sizes to test (default: 1000 5000 10000 20000 50000)')
    parser.add_argument('--save', action='store_true',
                       help='Save results to JSON file')
    parser.add_argument('--compare', type=str, metavar='BASELINE_FILE',
                       help='Compare with baseline results file')
    parser.add_argument('--category', choices=['basic', 'like', 'group_by', 'window', 'all'],
                       default='all', help='Benchmark category to run')

    args = parser.parse_args()

    # Build release version
    print(f"{Colors.YELLOW}Building release version...{Colors.NC}")
    try:
        subprocess.run(['cargo', 'build', '--release', '--quiet'], check=True)
    except subprocess.CalledProcessError:
        print(f"{Colors.RED}Build failed!{Colors.NC}")
        sys.exit(1)

    # Initialize benchmark suite
    suite = BenchmarkSuite()

    # Run benchmarks
    sizes = args.sizes if args.sizes else [1000, 5000, 10000, 20000, 50000]
    suite.run_all_benchmarks(sizes)
    suite.print_summary()

    # Save results if requested
    if args.save:
        filename = suite.save_results()

        # If this is the first save, also create a baseline
        baseline_path = Path('benchmark_baseline.json')
        if not baseline_path.exists():
            import shutil
            shutil.copy(filename, baseline_path)
            print(f"{Colors.GREEN}Baseline created at {baseline_path}{Colors.NC}")

    # Compare with baseline if requested
    if args.compare:
        suite.compare_results(args.compare)

if __name__ == '__main__':
    main()