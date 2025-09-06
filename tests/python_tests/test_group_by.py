"""Test GROUP BY functionality with various aggregate functions."""
import pandas as pd
import subprocess
import json
import os
import tempfile
import pytest
from pathlib import Path

SQL_CLI_PATH = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"


def run_sql_query(csv_file, query):
    """Execute SQL query and return results as list of dictionaries."""
    result = subprocess.run(
        [str(SQL_CLI_PATH), csv_file, "-q", query, "-o", "json"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse JSON output
    output = result.stdout.strip()
    
    # Remove comment lines that start with #
    lines = output.split('\n')
    json_lines = []
    for line in lines:
        if line and not line.startswith('#'):
            json_lines.append(line)
    
    if not json_lines:
        return []
    
    # Join back to get the full JSON
    json_str = '\n'.join(json_lines)
    
    try:
        # Parse as a JSON array
        results = json.loads(json_str)
        if isinstance(results, list):
            return results
        else:
            return [results]
    except json.JSONDecodeError:
        # If not valid JSON, return empty list
        return []


class TestBasicGroupBy:
    """Test basic GROUP BY functionality."""
    
    @pytest.fixture
    def trade_data(self):
        """Create test trade data."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("id,trader,book,quantity,price\n")
            f.write("1,Alice,BookA,100,50.5\n")
            f.write("2,Bob,BookA,200,51.0\n")
            f.write("3,Alice,BookB,150,49.5\n")
            f.write("4,Bob,BookB,250,52.0\n")
            f.write("5,Alice,BookA,300,50.0\n")
            f.write("6,Charlie,BookA,175,51.5\n")
            f.write("7,Charlie,BookB,225,49.0\n")
            csv_file = f.name
        
        yield csv_file
        os.unlink(csv_file)
    
    def test_group_by_count(self, trade_data):
        """Test GROUP BY with COUNT(*)."""
        query = "SELECT trader, COUNT(*) as trade_count FROM test GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        # Expected results
        expected = [
            {"trader": "Alice", "trade_count": 3},
            {"trader": "Bob", "trade_count": 2},
            {"trader": "Charlie", "trade_count": 2},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert actual["trade_count"] == exp["trade_count"]
    
    def test_group_by_sum(self, trade_data):
        """Test GROUP BY with SUM."""
        query = "SELECT trader, SUM(quantity) as total_qty FROM test GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        expected = [
            {"trader": "Alice", "total_qty": 550},
            {"trader": "Bob", "total_qty": 450},
            {"trader": "Charlie", "total_qty": 400},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert actual["total_qty"] == exp["total_qty"]
    
    def test_group_by_avg(self, trade_data):
        """Test GROUP BY with AVG."""
        query = "SELECT trader, AVG(price) as avg_price FROM test GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        expected = [
            {"trader": "Alice", "avg_price": 50.0},
            {"trader": "Bob", "avg_price": 51.5},
            {"trader": "Charlie", "avg_price": 50.25},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert abs(actual["avg_price"] - exp["avg_price"]) < 0.01
    
    def test_group_by_min_max(self, trade_data):
        """Test GROUP BY with MIN and MAX."""
        query = "SELECT book, MIN(price) as min_price, MAX(price) as max_price FROM test GROUP BY book ORDER BY book"
        results = run_sql_query(trade_data, query)
        
        expected = [
            {"book": "BookA", "min_price": 50.0, "max_price": 51.5},
            {"book": "BookB", "min_price": 49.0, "max_price": 52.0},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["book"] == exp["book"]
            assert abs(actual["min_price"] - exp["min_price"]) < 0.01
            assert abs(actual["max_price"] - exp["max_price"]) < 0.01


class TestMultiColumnGroupBy:
    """Test GROUP BY with multiple columns."""
    
    @pytest.fixture
    def trade_data(self):
        """Create test trade data."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("id,trader,book,quantity,price\n")
            f.write("1,Alice,BookA,100,50.5\n")
            f.write("2,Bob,BookA,200,51.0\n")
            f.write("3,Alice,BookB,150,49.5\n")
            f.write("4,Bob,BookB,250,52.0\n")
            f.write("5,Alice,BookA,300,50.0\n")
            csv_file = f.name
        
        yield csv_file
        os.unlink(csv_file)
    
    def test_multi_column_group_by(self, trade_data):
        """Test GROUP BY with multiple columns."""
        query = "SELECT trader, book, COUNT(*) as count, SUM(quantity) as total FROM test GROUP BY trader, book ORDER BY trader, book"
        results = run_sql_query(trade_data, query)
        
        expected = [
            {"trader": "Alice", "book": "BookA", "count": 2, "total": 400},
            {"trader": "Alice", "book": "BookB", "count": 1, "total": 150},
            {"trader": "Bob", "book": "BookA", "count": 1, "total": 200},
            {"trader": "Bob", "book": "BookB", "count": 1, "total": 250},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert actual["book"] == exp["book"]
            assert actual["count"] == exp["count"]
            assert actual["total"] == exp["total"]
    
    def test_group_by_with_expressions(self, trade_data):
        """Test GROUP BY with expressions in aggregates."""
        query = "SELECT trader, SUM(quantity * price) as total_value FROM test GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        # Calculate expected values
        # Alice: 100*50.5 + 150*49.5 + 300*50.0 = 5050 + 7425 + 15000 = 27475
        # Bob: 200*51.0 + 250*52.0 = 10200 + 13000 = 23200
        expected = [
            {"trader": "Alice", "total_value": 27475.0},
            {"trader": "Bob", "total_value": 23200.0},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert abs(actual["total_value"] - exp["total_value"]) < 0.01


class TestGroupByWithWhere:
    """Test GROUP BY combined with WHERE clauses."""
    
    @pytest.fixture
    def trade_data(self):
        """Create test trade data."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("id,trader,book,quantity,price\n")
            f.write("1,Alice,BookA,100,50.5\n")
            f.write("2,Bob,BookA,200,51.0\n")
            f.write("3,Alice,BookB,150,49.5\n")
            f.write("4,Bob,BookB,250,52.0\n")
            f.write("5,Alice,BookA,300,50.0\n")
            f.write("6,Charlie,BookA,175,51.5\n")
            csv_file = f.name
        
        yield csv_file
        os.unlink(csv_file)
    
    def test_group_by_with_where(self, trade_data):
        """Test GROUP BY with WHERE clause."""
        query = "SELECT trader, SUM(quantity) as total FROM test WHERE price > 50 GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        # Only trades with price > 50: Alice(100), Bob(200,250), Charlie(175)
        expected = [
            {"trader": "Alice", "total": 100},
            {"trader": "Bob", "total": 450},
            {"trader": "Charlie", "total": 175},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert actual["total"] == exp["total"]
    
    def test_group_by_specific_book(self, trade_data):
        """Test GROUP BY filtering for specific book."""
        query = "SELECT trader, COUNT(*) as count FROM test WHERE book = 'BookA' GROUP BY trader ORDER BY trader"
        results = run_sql_query(trade_data, query)
        
        expected = [
            {"trader": "Alice", "count": 2},
            {"trader": "Bob", "count": 1},
            {"trader": "Charlie", "count": 1},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["trader"] == exp["trader"]
            assert actual["count"] == exp["count"]


class TestGroupByEdgeCases:
    """Test edge cases for GROUP BY."""
    
    @pytest.fixture
    def simple_data(self):
        """Create simple test data."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("category,value\n")
            f.write("A,10\n")
            f.write("A,20\n")
            f.write("B,30\n")
            csv_file = f.name
        
        yield csv_file
        os.unlink(csv_file)
    
    def test_group_by_single_group(self, simple_data):
        """Test GROUP BY where all rows belong to one group."""
        # Create data where everything groups into one
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("category,value\n")
            f.write("A,10\n")
            f.write("A,20\n")
            f.write("A,30\n")
            csv_file = f.name
        
        try:
            query = "SELECT category, COUNT(*) as count, SUM(value) as total FROM test GROUP BY category"
            results = run_sql_query(csv_file, query)
            
            assert len(results) == 1
            assert results[0]["category"] == "A"
            assert results[0]["count"] == 3
            assert results[0]["total"] == 60
        finally:
            os.unlink(csv_file)
    
    def test_group_by_with_nulls(self, simple_data):
        """Test GROUP BY with NULL-like values."""
        # Create data with empty strings (treated as nulls in some contexts)
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("category,value\n")
            f.write("A,10\n")
            f.write(",20\n")  # Empty category
            f.write("A,30\n")
            f.write(",40\n")  # Empty category
            csv_file = f.name
        
        try:
            query = "SELECT category, COUNT(*) as count FROM test GROUP BY category ORDER BY category"
            results = run_sql_query(csv_file, query)
            
            # Should have two groups: empty string and "A"
            assert len(results) == 2
            
            # Find the groups - they may be in either order depending on sorting
            empty_group = None
            a_group = None
            for r in results:
                if r["category"] == "" or r["category"] is None:
                    empty_group = r
                elif r["category"] == "A":
                    a_group = r
            
            assert empty_group is not None
            assert empty_group["count"] == 2
            assert a_group is not None
            assert a_group["count"] == 2
        finally:
            os.unlink(csv_file)


class TestComplexGroupBy:
    """Test complex GROUP BY scenarios."""
    
    @pytest.fixture
    def sales_data(self):
        """Create sales data for testing."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
            f.write("date,region,product,quantity,revenue\n")
            f.write("2024-01-01,North,Widget,10,100.0\n")
            f.write("2024-01-01,North,Gadget,5,75.0\n")
            f.write("2024-01-01,South,Widget,15,150.0\n")
            f.write("2024-01-02,North,Widget,20,200.0\n")
            f.write("2024-01-02,South,Gadget,8,120.0\n")
            f.write("2024-01-02,South,Widget,12,120.0\n")
            csv_file = f.name
        
        yield csv_file
        os.unlink(csv_file)
    
    def test_three_column_group_by(self, sales_data):
        """Test GROUP BY with three columns."""
        query = "SELECT date, region, product, SUM(quantity) as total_qty FROM test GROUP BY date, region, product ORDER BY date, region, product"
        results = run_sql_query(sales_data, query)
        
        expected = [
            {"date": "2024-01-01", "region": "North", "product": "Gadget", "total_qty": 5},
            {"date": "2024-01-01", "region": "North", "product": "Widget", "total_qty": 10},
            {"date": "2024-01-01", "region": "South", "product": "Widget", "total_qty": 15},
            {"date": "2024-01-02", "region": "North", "product": "Widget", "total_qty": 20},
            {"date": "2024-01-02", "region": "South", "product": "Gadget", "total_qty": 8},
            {"date": "2024-01-02", "region": "South", "product": "Widget", "total_qty": 12},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["date"] == exp["date"]
            assert actual["region"] == exp["region"]
            assert actual["product"] == exp["product"]
            assert actual["total_qty"] == exp["total_qty"]
    
    def test_group_by_with_multiple_aggregates(self, sales_data):
        """Test GROUP BY with multiple aggregate functions."""
        query = """SELECT region, 
                         COUNT(*) as transactions, 
                         SUM(quantity) as total_qty,
                         SUM(revenue) as total_revenue,
                         AVG(revenue) as avg_revenue
                  FROM test 
                  GROUP BY region 
                  ORDER BY region"""
        results = run_sql_query(sales_data, query)
        
        # North: 3 transactions, qty=35, revenue=375, avg=125
        # South: 3 transactions, qty=35, revenue=390, avg=130
        expected = [
            {"region": "North", "transactions": 3, "total_qty": 35, "total_revenue": 375.0, "avg_revenue": 125.0},
            {"region": "South", "transactions": 3, "total_qty": 35, "total_revenue": 390.0, "avg_revenue": 130.0},
        ]
        
        assert len(results) == len(expected)
        for actual, exp in zip(results, expected):
            assert actual["region"] == exp["region"]
            assert actual["transactions"] == exp["transactions"]
            assert actual["total_qty"] == exp["total_qty"]
            assert abs(actual["total_revenue"] - exp["total_revenue"]) < 0.01
            assert abs(actual["avg_revenue"] - exp["avg_revenue"]) < 0.01