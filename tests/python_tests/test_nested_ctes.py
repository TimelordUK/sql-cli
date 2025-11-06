#!/usr/bin/env python3
"""Test nested CTEs and automatic CTE hoisting."""

import subprocess
import sys
import tempfile
import os

def run_query(query, data_file=None):
    """Run a query and return output."""
    cmd = ["./target/release/sql-cli"]
    if data_file:
        cmd.append(data_file)
    cmd.extend(["-q", query, "-o", "csv"])

    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr

def test_simple_nested_cte():
    """Test a simple nested CTE without data file."""
    print("Testing simple nested CTE...")
    query = """
    SELECT * FROM (
        WITH inner AS (SELECT 1 as value)
        SELECT * FROM inner
    ) t
    """
    returncode, stdout, stderr = run_query(query)

    if returncode != 0:
        print(f"✗ Simple nested CTE failed: {stderr}")
        return False

    if "value" in stdout and "1" in stdout:
        print("✓ Simple nested CTE works")
        return True
    else:
        print(f"✗ Simple nested CTE returned unexpected output: {stdout}")
        return False

def test_nested_cte_with_data():
    """Test nested CTE with actual data file."""
    print("\nTesting nested CTE with data file...")

    # Create test data
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,name,value\n")
        f.write("1,Alice,100\n")
        f.write("2,Bob,200\n")
        f.write("3,Charlie,300\n")
        data_file = f.name

    try:
        # Test nested CTE with data reference
        query = """
        SELECT * FROM (
            WITH summary AS (
                SELECT id, name, value
                FROM test
            )
            SELECT * FROM summary
        ) t
        WHERE value > 150
        """

        returncode, stdout, stderr = run_query(query, data_file)

        if returncode != 0:
            print(f"✗ Nested CTE with data failed: {stderr}")
            return False

        if "Bob" in stdout and "Charlie" in stdout and "Alice" not in stdout:
            print("✓ Nested CTE with data works")
            return True
        else:
            print(f"✗ Unexpected output: {stdout}")
            return False
    finally:
        os.unlink(data_file)

def test_deeply_nested_ctes():
    """Test deeply nested CTEs (3 levels)."""
    print("\nTesting deeply nested CTEs...")

    query = """
    SELECT * FROM (
        WITH level3 AS (
            SELECT * FROM (
                WITH level2 AS (
                    SELECT * FROM (
                        WITH level1 AS (
                            SELECT 1 as a, 2 as b
                        )
                        SELECT a + b as result FROM level1
                    ) t1
                )
                SELECT result * 2 as doubled FROM level2
            ) t2
        )
        SELECT doubled + 10 as final FROM level3
    ) t3
    """

    returncode, stdout, stderr = run_query(query)

    if returncode != 0:
        print(f"✗ Deeply nested CTEs failed: {stderr}")
        return False

    # (1 + 2) * 2 + 10 = 16
    if "final" in stdout and "16" in stdout:
        print("✓ Deeply nested CTEs work")
        return True
    else:
        print(f"✗ Unexpected output: {stdout}")
        return False

def test_trade_reconciliation_pattern():
    """Test the trade reconciliation pattern with nested CTEs."""
    print("\nTesting trade reconciliation pattern...")

    # Create trade data
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("DealType,PlatformOrderId,DealId,Environment\n")
        f.write("Swap,ORD-001,ORD-001,PROD\n")
        f.write("Swap,TT001|ORD-001,TT001,UAT\n")
        f.write("Forward,ORD-002,ORD-002,PROD\n")
        f.write("Forward,TT002|ORD-002,TT002,UAT\n")
        data_file = f.name

    try:
        # Simplified trade reconciliation query
        query = """
        SELECT deal_type, order_id, prod_deal_id
        FROM (
            WITH extracted AS (
                SELECT
                    DealType as deal_type,
                    CASE
                        WHEN CONTAINS(PlatformOrderId, '|') THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
                        ELSE PlatformOrderId
                    END as order_id,
                    DealId as prod_deal_id,
                    Environment
                FROM test
            )
            SELECT * FROM extracted
        ) t
        WHERE Environment = 'PROD'
        ORDER BY deal_type
        """

        returncode, stdout, stderr = run_query(query, data_file)

        if returncode != 0:
            print(f"✗ Trade reconciliation pattern failed: {stderr}")
            print(f"Query was: {query}")
            return False

        if "ORD-001" in stdout and "ORD-002" in stdout:
            print("✓ Trade reconciliation pattern works")
            return True
        else:
            print(f"✗ Unexpected output: {stdout}")
            return False
    finally:
        os.unlink(data_file)

def main():
    """Run all tests."""
    print("=" * 60)
    print("Testing Nested CTEs and Automatic CTE Hoisting")
    print("=" * 60)

    # Build the project first
    print("\nBuilding project...")
    result = subprocess.run(["cargo", "build", "--release"], capture_output=True)
    if result.returncode != 0:
        print("Failed to build project")
        sys.exit(1)

    tests = [
        test_simple_nested_cte,
        test_nested_cte_with_data,
        test_deeply_nested_ctes,
        test_trade_reconciliation_pattern
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"✗ Test {test.__name__} raised exception: {e}")
            failed += 1

    print("\n" + "=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    print("=" * 60)

    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()