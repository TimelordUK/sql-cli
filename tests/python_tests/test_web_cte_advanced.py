#!/usr/bin/env python3
"""Tests for advanced WEB CTE features: JSON_PATH, METHOD, BODY, and HEADERS."""

import subprocess
import json
import os

SQL_CLI = "./target/release/sql-cli"

def run_query(query):
    """Execute a SQL query and return the result."""
    result = subprocess.run(
        [SQL_CLI, "-q", query, "-o", "csv"],
        capture_output=True,
        text=True,
        timeout=10
    )
    return result.stdout, result.stderr, result.returncode

def test_method_get():
    """Test explicit GET method."""
    query = """
    WITH WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts'
        METHOD GET
        FORMAT JSON
    )
    SELECT COUNT(*) as total FROM posts
    """
    stdout, stderr, returncode = run_query(query)
    assert returncode == 0, f"Query failed: {stderr}"
    assert "total" in stdout
    assert "100" in stdout  # JSONPlaceholder has 100 posts
    print("✓ GET method test passed")

def test_method_parsing():
    """Test that all HTTP methods are parsed correctly."""
    methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]

    for method in methods:
        query = f"""
        WITH WEB test AS (
            URL 'http://localhost:5000/test'
            METHOD {method}
            FORMAT JSON
        )
        SELECT 1 as test
        """
        # Just check that it parses without error by using --query-plan
        # Don't actually execute the query
        result = subprocess.run(
            [SQL_CLI, "-q", query, "--query-plan"],
            capture_output=True,
            text=True,
            timeout=2  # Short timeout for localhost
        )
        # Check that parsing succeeded - should show the AST with method
        assert "WebCTESpec" in result.stdout or "method:" in result.stdout.lower(), \
            f"Failed to parse METHOD {method}: {result.stderr}"

    print("✓ All HTTP methods parsed correctly")

def test_json_path_extraction():
    """Test JSON_PATH extraction (will fail gracefully without real nested data)."""
    query = """
    WITH WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts'
        FORMAT JSON
        JSON_PATH 'nonexistent.path'
    )
    SELECT * FROM posts LIMIT 1
    """
    stdout, stderr, returncode = run_query(query)
    # Should fail with path not found
    assert returncode != 0, "Expected query to fail with invalid JSON path"
    assert "not found in JSON" in stderr or "Failed to extract JSON path" in stderr
    print("✓ JSON_PATH error handling works correctly")

def test_headers_with_env_vars():
    """Test HEADERS clause with environment variables."""
    # Set a test environment variable
    os.environ["TEST_TOKEN"] = "test123"

    query = """
    WITH WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts'
        FORMAT JSON
        HEADERS (
            'Authorization': 'Bearer ${TEST_TOKEN}',
            'User-Agent': 'SQL-CLI-Test'
        )
    )
    SELECT COUNT(*) as total FROM posts
    """
    stdout, stderr, returncode = run_query(query)
    assert returncode == 0, f"Query with headers failed: {stderr}"
    assert "100" in stdout
    print("✓ HEADERS with environment variables test passed")

def test_body_parsing():
    """Test BODY clause parsing."""
    query = """
    WITH WEB test AS (
        URL 'http://localhost:5000/api/test'
        METHOD POST
        BODY '{
            "test": "data",
            "number": 123
        }'
        FORMAT JSON
    )
    SELECT 1 as test
    """
    # Test that it parses correctly (using localhost to avoid external network)
    result = subprocess.run(
        [SQL_CLI, "-q", query, "--query-plan"],
        capture_output=True,
        text=True,
        timeout=3  # Shorter timeout for localhost
    )
    # Check that the query plan shows the parsed structure
    assert "WebCTESpec" in result.stdout or "method:" in result.stdout.lower() or "body:" in result.stdout.lower(), \
        f"Failed to parse BODY clause: {result.stderr}"
    print("✓ BODY clause parsing test passed")

def test_combined_features():
    """Test combining multiple new features."""
    query = """
    WITH WEB posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts'
        METHOD GET
        FORMAT JSON
        HEADERS (
            'Accept': 'application/json',
            'User-Agent': 'SQL-CLI/1.52'
        )
    )
    SELECT
        userId,
        COUNT(*) as post_count
    FROM posts
    GROUP BY userId
    HAVING post_count >= 10
    ORDER BY userId
    LIMIT 5
    """
    stdout, stderr, returncode = run_query(query)
    assert returncode == 0, f"Combined features query failed: {stderr}"
    assert "userId" in stdout
    assert "post_count" in stdout
    print("✓ Combined features test passed")

def test_cache_seconds_still_works():
    """Ensure CACHE clause still works with new features."""
    query = """
    WITH WEB cached_posts AS (
        URL 'https://jsonplaceholder.typicode.com/posts'
        METHOD GET
        FORMAT JSON
        CACHE 3600
    )
    SELECT COUNT(*) as total FROM cached_posts
    """
    stdout, stderr, returncode = run_query(query)
    assert returncode == 0, f"CACHE clause query failed: {stderr}"
    # Check that we got results
    assert "total" in stdout
    assert "100" in stdout  # JSONPlaceholder has 100 posts
    print("✓ CACHE clause compatibility test passed")

def test_multiple_headers():
    """Test multiple headers in HEADERS clause."""
    query = """
    WITH WEB data AS (
        URL 'https://jsonplaceholder.typicode.com/users'
        FORMAT JSON
        HEADERS (
            'Accept': 'application/json',
            'Cache-Control': 'no-cache',
            'X-Custom-Header': 'test-value',
            'User-Agent': 'SQL-CLI-Test-Suite'
        )
    )
    SELECT COUNT(*) as user_count FROM data
    """
    stdout, stderr, returncode = run_query(query)
    assert returncode == 0, f"Multiple headers query failed: {stderr}"
    assert "user_count" in stdout
    assert "10" in stdout  # JSONPlaceholder has 10 users
    print("✓ Multiple headers test passed")

def main():
    """Run all tests."""
    print("Testing advanced WEB CTE features...")

    test_method_get()
    test_method_parsing()
    test_json_path_extraction()
    test_headers_with_env_vars()
    test_body_parsing()
    test_combined_features()
    test_cache_seconds_still_works()
    test_multiple_headers()

    print("\n✅ All advanced WEB CTE tests passed!")

if __name__ == "__main__":
    main()