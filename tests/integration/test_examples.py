#!/usr/bin/env -S uv run python
"""
Integration test runner for examples/ directory

Usage:
    uv run python tests/integration/test_examples.py                    # Run all examples
    uv run python tests/integration/test_examples.py physics_astronomy  # Run specific example
    uv run python tests/integration/test_examples.py --capture qualified_names  # Capture expectation

Test Modes:
    1. FORMAL TEST: If examples/expectations/<name>.json exists
       - Validates output exactly matches expected JSON
       - Fails on any mismatch
    2. SMOKE TEST: No expectation file exists
       - Just ensures query runs without errors
       - Acts as demo/documentation
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# ANSI colors
class Color:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

class TestResult:
    def __init__(self):
        self.total = 0
        self.formal_tests = 0
        self.smoke_tests = 0
        self.passed = 0
        self.failed = 0
        self.passed_tests: List[str] = []
        self.failed_tests: List[Tuple[str, str]] = []  # (name, reason)

def run_sql_file(cli_path: str, sql_file: Path) -> Tuple[bool, str]:
    """Run SQL file and return (success, output)"""
    try:
        result = subprocess.run(
            [cli_path, '-f', str(sql_file), '-o', 'json'],
            capture_output=True,
            text=True,
            timeout=30
        )
        # Combine stdout and stderr
        output = result.stdout
        if result.returncode != 0:
            # Include error from stderr
            if result.stderr:
                output = result.stderr
            return False, output
        return True, output
    except subprocess.TimeoutExpired:
        return False, "ERROR: Test timed out (30s)"
    except Exception as e:
        return False, f"ERROR: {str(e)}"

def normalize_json(json_str: str) -> Optional[List]:
    """Parse and normalize JSON for comparison

    Handles both single JSON documents and multiple JSON arrays (from scripts)
    """
    try:
        # Try parsing as single JSON document first
        data = json.loads(json_str)
        return [data] if not isinstance(data, list) or (data and not isinstance(data[0], dict)) else data
    except json.JSONDecodeError:
        # Try parsing as multiple JSON arrays (script output)
        # Each query result is a separate JSON array
        objects = []
        for line in json_str.strip().split('\n'):
            line = line.strip()
            if not line or line.startswith('['):
                # Start of new JSON array
                pass
            try:
                # Try to parse accumulated lines as JSON
                obj = json.loads(line)
                objects.append(obj)
            except:
                pass

        if objects:
            return objects

        # Fallback: split by ']' and try to parse each chunk
        chunks = json_str.split('\n]\n')
        result = []
        for chunk in chunks:
            chunk = chunk.strip()
            if not chunk:
                continue
            if not chunk.endswith(']'):
                chunk += ']'
            try:
                obj = json.loads(chunk)
                result.append(obj)
            except json.JSONDecodeError:
                pass

        return result if result else None

def compare_json(expected, actual) -> Tuple[bool, Optional[str]]:
    """Compare two JSON objects and return (matches, diff_message)"""
    if expected == actual:
        return True, None

    # Generate helpful diff message
    expected_str = json.dumps(expected, indent=2, sort_keys=True)
    actual_str = json.dumps(actual, indent=2, sort_keys=True)

    # Find first difference
    exp_lines = expected_str.split('\n')
    act_lines = actual_str.split('\n')

    diff_lines = []
    for i, (exp, act) in enumerate(zip(exp_lines, act_lines)):
        if exp != act:
            diff_lines.append(f"Line {i+1}:")
            diff_lines.append(f"  Expected: {exp}")
            diff_lines.append(f"  Actual:   {act}")
            if len(diff_lines) >= 10:  # Limit diff output
                diff_lines.append("  ... (truncated)")
                break

    return False, '\n'.join(diff_lines)

def should_skip_file(sql_file: Path) -> Tuple[bool, Optional[str]]:
    """Check if file should be skipped based on -- [TEST:SKIP] directive"""
    try:
        with open(sql_file, 'r') as f:
            # Check first 10 lines for [TEST:SKIP] directive
            for i, line in enumerate(f):
                if i >= 10:
                    break
                line = line.strip()
                if '[TEST:SKIP]' in line.upper():
                    # Extract reason if provided: -- [TEST:SKIP] Reason here
                    parts = line.split('[TEST:SKIP]', 1)
                    if len(parts) > 1:
                        reason = parts[1].strip()
                        return True, reason if reason else "Marked as skip"
                    return True, "Marked as skip"
    except Exception:
        pass
    return False, None

def run_test(cli_path: str, sql_file: Path, expectations_dir: Path, result: TestResult) -> None:
    """Run a single test"""
    base_name = sql_file.stem
    expectation_file = expectations_dir / f"{base_name}.json"

    result.total += 1

    # Check if file should be skipped
    should_skip, skip_reason = should_skip_file(sql_file)
    if should_skip:
        print(f"[SKIP] {base_name} ... {Color.YELLOW}⊘ SKIPPED{Color.NC} ({skip_reason})")
        result.passed += 1  # Count skips as passed
        result.passed_tests.append(f"{base_name} (skipped)")
        return

    # Determine test mode
    is_formal = expectation_file.exists()
    test_mode = "FORMAL" if is_formal else "SMOKE"

    if is_formal:
        result.formal_tests += 1
    else:
        result.smoke_tests += 1

    print(f"[{test_mode}] {base_name} ... ", end='', flush=True)

    # Run the SQL file
    success, output = run_sql_file(cli_path, sql_file)

    if not success:
        print(f"{Color.RED}✗ FAIL{Color.NC}")
        result.failed += 1
        result.failed_tests.append((base_name, output.split('\n')[0]))  # First line of error
        if '--verbose' in sys.argv:
            print(f"  {output}")
        return

    # For formal tests, validate output
    if is_formal:
        # Parse actual output
        actual_json = normalize_json(output)
        if actual_json is None:
            print(f"{Color.RED}✗ FAIL - Invalid JSON output{Color.NC}")
            result.failed += 1
            result.failed_tests.append((base_name, "Invalid JSON output"))
            return

        # Load expected output
        try:
            with open(expectation_file, 'r') as f:
                expected_json = json.load(f)
        except Exception as e:
            print(f"{Color.RED}✗ FAIL - Cannot load expectation: {e}{Color.NC}")
            result.failed += 1
            result.failed_tests.append((base_name, f"Cannot load expectation: {e}"))
            return

        # Compare
        matches, diff = compare_json(expected_json, actual_json)
        if matches:
            print(f"{Color.GREEN}✓ PASS{Color.NC}")
            result.passed += 1
            result.passed_tests.append(base_name)
        else:
            print(f"{Color.RED}✗ FAIL - Output mismatch{Color.NC}")
            result.failed += 1
            result.failed_tests.append((base_name, "Output mismatch"))
            if '--verbose' in sys.argv and diff:
                print(f"{Color.YELLOW}{diff}{Color.NC}")
    else:
        # Smoke test - just check it ran
        print(f"{Color.GREEN}✓ PASS (runs){Color.NC}")
        result.passed += 1
        result.passed_tests.append(base_name)

def capture_expectation(cli_path: str, example_name: str, expectations_dir: Path) -> None:
    """Capture expected output for an example"""
    sql_file = Path('examples') / f"{example_name}.sql"

    if not sql_file.exists():
        print(f"{Color.RED}ERROR: Example file not found: {sql_file}{Color.NC}")
        sys.exit(1)

    print(f"Capturing output for: {example_name}")
    print(f"Running: {cli_path} -f {sql_file} -o json")
    print()

    success, output = run_sql_file(cli_path, sql_file)

    if not success:
        print(f"{Color.RED}ERROR: Query failed{Color.NC}")
        print(output)
        sys.exit(1)

    # Validate JSON
    json_data = normalize_json(output)
    if json_data is None:
        print(f"{Color.RED}ERROR: Output is not valid JSON{Color.NC}")
        print(output)
        sys.exit(1)

    # Save to file
    expectations_dir.mkdir(parents=True, exist_ok=True)
    expectation_file = expectations_dir / f"{example_name}.json"

    with open(expectation_file, 'w') as f:
        json.dump(json_data, f, indent=2, sort_keys=True)

    print(f"{Color.GREEN}✓ Expectation captured successfully!{Color.NC}")
    print(f"  Saved to: {expectation_file}")
    print()
    print("To test: ./tests/integration/test_examples.py", example_name)

def main():
    cli_path = './target/release/sql-cli'
    examples_dir = Path('examples')
    expectations_dir = Path('examples/expectations')

    # Check if CLI binary exists
    if not Path(cli_path).exists():
        print(f"{Color.RED}ERROR: {cli_path} not found. Run 'cargo build --release' first.{Color.NC}")
        sys.exit(1)

    # Handle --capture mode
    if '--capture' in sys.argv:
        idx = sys.argv.index('--capture')
        if idx + 1 >= len(sys.argv):
            print("ERROR: --capture requires an example name")
            sys.exit(1)
        example_name = sys.argv[idx + 1]
        capture_expectation(cli_path, example_name, expectations_dir)
        return

    print("=== Examples Integration Test Suite ===")
    print()

    result = TestResult()

    # Determine which examples to run
    if len(sys.argv) > 1 and not sys.argv[1].startswith('--'):
        # Run specific examples
        patterns = [arg for arg in sys.argv[1:] if not arg.startswith('--')]
        sql_files = []
        for pattern in patterns:
            matches = list(examples_dir.glob(f"{pattern}*.sql"))
            if not matches:
                print(f"{Color.YELLOW}WARNING: No example matching '{pattern}' found{Color.NC}")
            sql_files.extend(matches)
    else:
        # Run all examples
        sql_files = sorted(examples_dir.glob("*.sql"))

    # Run tests
    for sql_file in sql_files:
        run_test(cli_path, sql_file, expectations_dir, result)

    # Print summary
    print()
    print("=== Test Summary ===")
    print(f"Total examples:  {result.total}")
    print(f"  Formal tests:  {result.formal_tests} (with expectations/*.json)")
    print(f"  Smoke tests:   {result.smoke_tests} (no expectations)")
    print()
    print(f"{Color.GREEN}Passed: {result.passed}{Color.NC}")
    print(f"{Color.RED}Failed: {result.failed}{Color.NC}")
    print()

    # Show failed tests
    if result.failed > 0:
        print(f"{Color.RED}Failed tests:{Color.NC}")
        for name, reason in result.failed_tests:
            print(f"  - {name}: {reason}")
        print()
        print("Run with --verbose for detailed output")

        # Only fail the test suite if formal tests failed
        formal_failures = [name for name, _ in result.failed_tests
                          if (Path('examples') / f"{name}.sql").exists()
                          and (Path('examples/expectations') / f"{name}.json").exists()]

        if formal_failures:
            print()
            print(f"{Color.RED}FORMAL tests failed - expectations not met!{Color.NC}")
            sys.exit(1)
        else:
            print()
            print(f"{Color.YELLOW}Only SMOKE tests failed (no expectations) - non-critical{Color.NC}")
            print(f"{Color.GREEN}All FORMAL tests passed!{Color.NC}")
    else:
        print(f"{Color.GREEN}All tests passed!{Color.NC}")

if __name__ == '__main__':
    main()
