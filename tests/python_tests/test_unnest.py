#!/usr/bin/env python3
"""
Test suite for UNNEST row expansion functionality
"""

import subprocess
import json
import csv
import os
import sys
import tempfile
import pytest
from pathlib import Path

# Find the sql-cli binary. The .exe suffix is required on Windows: without it
# the release `exists()` check fails, this file quietly falls through to the
# debug path, and Windows then resolves `sql-cli` -> `sql-cli.exe` anyway — so
# the tests pass while exercising the DEBUG binary. That is worse than the
# outright failure the same omission caused in test_quoted_columns.py.
PROJECT_ROOT = Path(__file__).parent.parent.parent
_SUFFIX = ".exe" if sys.platform == "win32" else ""
SQL_CLI = PROJECT_ROOT / "target" / "release" / f"sql-cli{_SUFFIX}"

if not SQL_CLI.exists():
    SQL_CLI = PROJECT_ROOT / "target" / "debug" / f"sql-cli{_SUFFIX}"


def run_query(csv_file, query):
    """Run a SQL query and return results as a list of dictionaries."""
    cmd = [str(SQL_CLI), csv_file, "-q", query, "-o", "json"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")

    # Parse JSON output
    lines = result.stdout.strip().split('\n')
    # Filter out comment lines
    json_lines = [l for l in lines if not l.startswith('#')]
    if not json_lines:
        return []

    return json.loads(''.join(json_lines))


class TestUnnest:
    """Test UNNEST row expansion functionality."""

    @classmethod
    def setup_class(cls):
        """Create test data files."""
        cls.temp_dir = tempfile.mkdtemp()

        # Use the existing FIX allocations test data
        cls.fix_file = str(PROJECT_ROOT / "data" / "fix_allocations.csv")

        # Create additional test data for edge cases
        cls.edge_case_file = os.path.join(cls.temp_dir, "unnest_edge_cases.csv")
        with open(cls.edge_case_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['id', 'tags', 'values'])
            writer.writerow(['1', 'A|B|C', '1,2,3'])
            writer.writerow(['2', 'X|Y', '10,20,30'])  # Mismatched lengths
            writer.writerow(['3', 'Z', '100'])  # Single item
            writer.writerow(['4', '', ''])  # Empty strings

    @classmethod
    def teardown_class(cls):
        """Clean up test data files."""
        import shutil
        if os.path.exists(cls.temp_dir):
            shutil.rmtree(cls.temp_dir)

    def test_single_unnest(self):
        """Test UNNEST on a single column."""
        query = """
            SELECT order_id, UNNEST(accounts, '|') AS account
            FROM fix_allocations
            WHERE order_id = 'ORD001'
        """
        results = run_query(self.fix_file, query)

        assert len(results) == 3
        assert results[0]['order_id'] == 'ORD001'
        assert results[0]['account'] == 'ACC_1'
        assert results[1]['account'] == 'ACC_2'
        assert results[2]['account'] == 'ACC_3'

    def test_multiple_unnest_matching_lengths(self):
        """Test UNNEST on multiple columns with matching lengths."""
        query = """
            SELECT
                order_id,
                UNNEST(accounts, '|') AS account,
                UNNEST(amounts, ',') AS amount
            FROM fix_allocations
            WHERE order_id = 'ORD001'
        """
        results = run_query(self.fix_file, query)

        assert len(results) == 3
        assert results[0]['order_id'] == 'ORD001'
        assert results[0]['account'] == 'ACC_1'
        assert results[0]['amount'] == '200'
        assert results[1]['account'] == 'ACC_2'
        assert results[1]['amount'] == '200'
        assert results[2]['account'] == 'ACC_3'
        assert results[2]['amount'] == '200'

    def test_multiple_unnest_mismatched_lengths(self):
        """Test UNNEST with mismatched array lengths (NULL padding)."""
        query = """
            SELECT
                id,
                UNNEST(tags, '|') AS tag,
                UNNEST(values, ',') AS value
            FROM unnest_edge_cases
            WHERE id = '2'
        """
        results = run_query(self.edge_case_file, query)

        # tags has 2 items, values has 3 items -> 3 output rows
        assert len(results) == 3
        assert results[0]['tag'] == 'X'
        assert results[0]['value'] == '10'
        assert results[1]['tag'] == 'Y'
        assert results[1]['value'] == '20'
        # Third row should have NULL for tag (exhausted)
        assert results[2]['tag'] is None
        assert results[2]['value'] == '30'

    def test_unnest_all_rows(self):
        """Test UNNEST on all rows in table."""
        query = """
            SELECT
                order_id,
                symbol,
                UNNEST(accounts, '|') AS account,
                UNNEST(amounts, ',') AS amount
            FROM fix_allocations
        """
        results = run_query(self.fix_file, query)

        # ORD001: 3 rows, ORD002: 2 rows, ORD003: 1 row = 6 total
        assert len(results) == 6

        # Check some specific values
        ord001_rows = [r for r in results if r['order_id'] == 'ORD001']
        assert len(ord001_rows) == 3
        assert all(r['symbol'] == 'ZX5Y' for r in ord001_rows)

        ord002_rows = [r for r in results if r['order_id'] == 'ORD002']
        assert len(ord002_rows) == 2
        assert all(r['symbol'] == 'ABCD' for r in ord002_rows)

    def test_unnest_with_order_by(self):
        """Test UNNEST combined with ORDER BY."""
        query = """
            SELECT
                UNNEST(accounts, '|') AS account,
                order_id
            FROM fix_allocations
            ORDER BY account
        """
        results = run_query(self.fix_file, query)

        assert len(results) == 6
        # Check ordering
        accounts = [r['account'] for r in results]
        assert accounts == ['ACC_1', 'ACC_1', 'ACC_2', 'ACC_3', 'ACC_4', 'ACC_5']

    def test_unnest_with_where_filter(self):
        """Test UNNEST with WHERE clause filtering before expansion."""
        query = """
            SELECT
                order_id,
                UNNEST(accounts, '|') AS account
            FROM fix_allocations
            WHERE msg_type = 'AS'
        """
        results = run_query(self.fix_file, query)

        # Only ORD001 and ORD002 have msg_type='AS'
        # ORD001: 3 accounts, ORD002: 2 accounts = 5 total
        assert len(results) == 5
        assert all(r['order_id'] in ['ORD001', 'ORD002'] for r in results)

    def test_unnest_single_item(self):
        """Test UNNEST on a column with single item (no delimiter)."""
        query = """
            SELECT
                UNNEST(accounts, '|') AS account,
                UNNEST(amounts, ',') AS amount
            FROM fix_allocations
            WHERE order_id = 'ORD003'
        """
        results = run_query(self.fix_file, query)

        # Single account, single amount
        assert len(results) == 1
        assert results[0]['account'] == 'ACC_1'
        assert results[0]['amount'] == '1000'

    def test_unnest_preserves_regular_columns(self):
        """Test that regular columns are replicated correctly in expanded rows."""
        query = """
            SELECT
                msg_type,
                order_id,
                symbol,
                UNNEST(accounts, '|') AS account
            FROM fix_allocations
            WHERE order_id = 'ORD001'
        """
        results = run_query(self.fix_file, query)

        assert len(results) == 3
        # All rows should have the same msg_type, order_id, symbol
        assert all(r['msg_type'] == 'AS' for r in results)
        assert all(r['order_id'] == 'ORD001' for r in results)
        assert all(r['symbol'] == 'ZX5Y' for r in results)
        # But different accounts
        assert results[0]['account'] == 'ACC_1'
        assert results[1]['account'] == 'ACC_2'
        assert results[2]['account'] == 'ACC_3'

    def test_unnest_different_delimiters(self):
        """Test UNNEST with different delimiter characters."""
        query = """
            SELECT
                UNNEST(tags, '|') AS tag,
                UNNEST(values, ',') AS value
            FROM unnest_edge_cases
            WHERE id = '1'
        """
        results = run_query(self.edge_case_file, query)

        assert len(results) == 3
        # tags split by |, values split by ,
        assert results[0]['tag'] == 'A'
        assert results[0]['value'] == '1'
        assert results[1]['tag'] == 'B'
        assert results[1]['value'] == '2'
        assert results[2]['tag'] == 'C'
        assert results[2]['value'] == '3'


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
