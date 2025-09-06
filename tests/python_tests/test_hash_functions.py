#!/usr/bin/env python3
"""
Test suite for hash functions in sql-cli.
Tests MD5, SHA1, SHA256, SHA512 implementations.
"""

import subprocess
import csv
import io
import hashlib
from pathlib import Path
import tempfile

# Path to the sql-cli executable
SQL_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "sql-cli"

def run_query(csv_file, query):
    """Run a SQL query and return the results as a list of dictionaries."""
    result = subprocess.run(
        [str(SQL_CLI), str(csv_file), "-q", query, "-o", "csv"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        raise Exception(f"Query failed: {result.stderr}")
    
    # Parse CSV output
    output_lines = result.stdout.strip().split('\n')
    csv_lines = [line for line in output_lines if not line.startswith('#')]
    
    if not csv_lines:
        return []
    
    reader = csv.DictReader(io.StringIO('\n'.join(csv_lines)))
    return list(reader)

def test_md5():
    """Test MD5 hash function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("text\n")
        f.write("hello\n")
        f.write("world\n")
        f.write("test123\n")
        f.write("\n")  # Empty string
        temp_file = f.name
    
    try:
        query = "SELECT text, MD5(text) as hash FROM test"
        results = run_query(temp_file, query)
        
        # Verify against Python's hashlib
        for row in results:
            text = row['text']
            expected = hashlib.md5(text.encode()).hexdigest()
            assert row['hash'] == expected, f"MD5 mismatch for '{text}': {row['hash']} != {expected}"
        
        print("✓ test_md5 passed")
    finally:
        Path(temp_file).unlink()

def test_sha1():
    """Test SHA1 hash function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("text\n")
        f.write("hello\n")
        f.write("test\n")
        f.write("sql-cli\n")
        temp_file = f.name
    
    try:
        query = "SELECT text, SHA1(text) as hash FROM test"
        results = run_query(temp_file, query)
        
        # Verify against Python's hashlib
        for row in results:
            text = row['text']
            expected = hashlib.sha1(text.encode()).hexdigest()
            assert row['hash'] == expected, f"SHA1 mismatch for '{text}': {row['hash']} != {expected}"
        
        print("✓ test_sha1 passed")
    finally:
        Path(temp_file).unlink()

def test_sha256():
    """Test SHA256 hash function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("text\n")
        f.write("hello\n")
        f.write("SHA256 test\n")
        f.write("1234567890\n")
        temp_file = f.name
    
    try:
        query = "SELECT text, SHA256(text) as hash FROM test"
        results = run_query(temp_file, query)
        
        # Verify against Python's hashlib
        for row in results:
            text = row['text']
            expected = hashlib.sha256(text.encode()).hexdigest()
            assert row['hash'] == expected, f"SHA256 mismatch for '{text}': {row['hash']} != {expected}"
        
        print("✓ test_sha256 passed")
    finally:
        Path(temp_file).unlink()

def test_sha512():
    """Test SHA512 hash function."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("text\n")
        f.write("hello\n")
        f.write("SHA512 test\n")
        f.write("\n")  # Empty string
        temp_file = f.name
    
    try:
        query = "SELECT text, SHA512(text) as hash FROM test"
        results = run_query(temp_file, query)
        
        # Verify against Python's hashlib
        for row in results:
            text = row['text']
            expected = hashlib.sha512(text.encode()).hexdigest()
            assert row['hash'] == expected, f"SHA512 mismatch for '{text}': {row['hash']} != {expected}"
        
        print("✓ test_sha512 passed")
    finally:
        Path(temp_file).unlink()

def test_null_handling():
    """Test NULL handling in hash functions."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("id,text\n")
        f.write("1,hello\n")
        f.write("2,\n")  # NULL in CSV
        temp_file = f.name
    
    try:
        # Test all hash functions with NULL - should return NULL (empty in CSV)
        for func in ['MD5', 'SHA1', 'SHA256', 'SHA512']:
            query = f"SELECT id, {func}(text) as hash FROM test WHERE id = 2"
            results = run_query(temp_file, query)
            
            # NULL should return NULL (empty in CSV output)
            assert results[0]['hash'] == '', f"{func} should return NULL for NULL input"
        
        # Test hashing numbers (auto-converted to strings)
        query = "SELECT MD5(123) as hash"
        results = run_query(temp_file, query)
        expected = hashlib.md5('123'.encode()).hexdigest()
        assert results[0]['hash'] == expected, f"MD5 of number should work"
        
        print("✓ test_null_handling passed")
    finally:
        Path(temp_file).unlink()

def test_case_sensitivity():
    """Test that hash functions produce different results for different cases."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False) as f:
        f.write("text\n")
        f.write("Hello\n")
        f.write("hello\n")
        f.write("HELLO\n")
        temp_file = f.name
    
    try:
        query = "SELECT text, MD5(text) as hash FROM test"
        results = run_query(temp_file, query)
        
        # All three should have different hashes
        hashes = [r['hash'] for r in results]
        assert len(hashes) == len(set(hashes)), "Different cases should produce different hashes"
        
        print("✓ test_case_sensitivity passed")
    finally:
        Path(temp_file).unlink()

def main():
    """Run all tests."""
    print("Running hash function tests...")
    
    # Check if sql-cli is built
    if not SQL_CLI.exists():
        print(f"Error: sql-cli not found at {SQL_CLI}")
        print("Please run: cargo build --release")
        return 1
    
    tests = [
        test_md5,
        test_sha1,
        test_sha256,
        test_sha512,
        test_null_handling,
        test_case_sensitivity,
    ]
    
    failed = 0
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
    
    if failed == 0:
        print(f"\n✅ All {len(tests)} hash function tests passed!")
    else:
        print(f"\n❌ {failed}/{len(tests)} tests failed")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())