#!/usr/bin/env python3
"""
Tests for flexible date parsing functions: PARSE_DATETIME, PARSE_DATETIME_UTC, and DATETIME constructor
"""

import subprocess
import json


def run_query(query: str, data_file: str = None) -> dict:
    """Execute a SQL query and return the result as JSON."""
    cmd = ["./target/release/sql-cli"]
    if data_file:
        cmd.append(data_file)
    cmd.extend(["-q", query, "-o", "json"])

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)


def test_parse_datetime_with_european_format():
    """Test PARSE_DATETIME with European date format (DD/MM/YYYY)"""
    query = "SELECT PARSE_DATETIME('15/01/2024', '%d/%m/%Y') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 00:00:00.000"
    print("✓ PARSE_DATETIME with European format")


def test_parse_datetime_with_american_format():
    """Test PARSE_DATETIME with American date format (MM/DD/YYYY)"""
    query = "SELECT PARSE_DATETIME('01/15/2024', '%m/%d/%Y') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 00:00:00.000"
    print("✓ PARSE_DATETIME with American format")


def test_parse_datetime_with_time():
    """Test PARSE_DATETIME with date and time"""
    query = "SELECT PARSE_DATETIME('15/01/2024 14:30:45', '%d/%m/%Y %H:%M:%S') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 14:30:45.000"
    print("✓ PARSE_DATETIME with date and time")


def test_parse_datetime_with_text_month():
    """Test PARSE_DATETIME with text month names"""
    query = "SELECT PARSE_DATETIME('Jan 15, 2024', '%b %d, %Y') as short_month, PARSE_DATETIME('January 15, 2024', '%B %d, %Y') as long_month"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["short_month"] == "2024-01-15 00:00:00.000"
    assert result[0]["long_month"] == "2024-01-15 00:00:00.000"
    print("✓ PARSE_DATETIME with text month names")


def test_parse_datetime_with_fix_format():
    """Test PARSE_DATETIME with FIX Protocol format"""
    query = "SELECT PARSE_DATETIME('20240115-14:30:45.567', '%Y%m%d-%H:%M:%S%.3f') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 14:30:45.567"
    print("✓ PARSE_DATETIME with FIX format")


def test_parse_datetime_utc_auto_detect():
    """Test PARSE_DATETIME_UTC with auto-detection (1 argument)"""
    query = "SELECT PARSE_DATETIME_UTC('2024-01-15 14:30:00') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 14:30:00.000"
    print("✓ PARSE_DATETIME_UTC with auto-detection")


def test_parse_datetime_utc_with_fix_auto_detect():
    """Test PARSE_DATETIME_UTC auto-detecting FIX format"""
    query = "SELECT PARSE_DATETIME_UTC('20240115-14:30:45.567') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 14:30:45.567"
    print("✓ PARSE_DATETIME_UTC with FIX auto-detection")


def test_parse_datetime_utc_with_custom_format():
    """Test PARSE_DATETIME_UTC with custom format (2 arguments)"""
    query = "SELECT PARSE_DATETIME_UTC('15/01/2024 14:30', '%d/%m/%Y %H:%M') as parsed"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["parsed"] == "2024-01-15 14:30:00.000"
    print("✓ PARSE_DATETIME_UTC with custom format")


def test_parse_datetime_utc_with_fix_data():
    """Test PARSE_DATETIME_UTC with actual FIX timestamps CSV file"""
    query = "SELECT order_id, PARSE_DATETIME_UTC(transaction_time) as parsed FROM fix_timestamps LIMIT 2"
    result = run_query(query, "data/fix_timestamps.csv")

    assert len(result) == 2
    assert result[0]["order_id"] == "ORD001"
    assert result[0]["parsed"] == "2025-09-25 14:52:15.567"
    assert result[1]["order_id"] == "ORD002"
    assert result[1]["parsed"] == "2025-09-25 14:52:15.789"
    print("✓ PARSE_DATETIME_UTC with FIX timestamps data")


def test_datetime_constructor_date_only():
    """Test DATETIME constructor with date only (3 args)"""
    query = "SELECT DATETIME(2024, 1, 15) as date_only"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["date_only"] == "2024-01-15 00:00:00.000"
    print("✓ DATETIME constructor with date only")


def test_datetime_constructor_with_time():
    """Test DATETIME constructor with date and time (6 args)"""
    query = "SELECT DATETIME(2024, 1, 15, 14, 30, 45) as with_time"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["with_time"] == "2024-01-15 14:30:45.000"
    print("✓ DATETIME constructor with time")


def test_datetime_constructor_validates_dates():
    """Test DATETIME constructor with edge cases"""
    # Test month boundaries
    query = "SELECT DATETIME(2024, 12, 31, 23, 59, 59) as year_end"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["year_end"] == "2024-12-31 23:59:59.000"
    print("✓ DATETIME constructor validates dates")


def test_datetime_constructor_partial_time():
    """Test DATETIME constructor with partial time components"""
    query = "SELECT DATETIME(2024, 12, 25, 18) as hour_only, DATETIME(2024, 12, 25, 18, 45) as hour_minute"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["hour_only"] == "2024-12-25 18:00:00.000"
    assert result[0]["hour_minute"] == "2024-12-25 18:45:00.000"
    print("✓ DATETIME constructor with partial time")


def test_combined_date_functions():
    """Test combining new functions with existing date functions"""
    query = """
        SELECT
            DATETIME(2024, 1, 1) as start_date,
            PARSE_DATETIME('31/12/2024', '%d/%m/%Y') as end_date,
            DATEDIFF('day', DATETIME(2024, 1, 1), PARSE_DATETIME('31/12/2024', '%d/%m/%Y')) as days_diff
    """
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["start_date"] == "2024-01-01 00:00:00.000"
    assert result[0]["end_date"] == "2024-12-31 00:00:00.000"
    assert result[0]["days_diff"] == 365
    print("✓ Combined date functions")


def test_parse_datetime_with_null():
    """Test PARSE_DATETIME with NULL values"""
    query = "SELECT PARSE_DATETIME(NULL, '%Y-%m-%d') as null_result"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["null_result"] is None
    print("✓ PARSE_DATETIME with NULL")


def test_datetime_constructor_with_leap_year():
    """Test DATETIME constructor with leap year date"""
    query = "SELECT DATETIME(2024, 2, 29) as leap_day"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["leap_day"] == "2024-02-29 00:00:00.000"
    print("✓ DATETIME constructor with leap year")


def test_parse_datetime_iso8601():
    """Test PARSE_DATETIME with ISO 8601 format"""
    query = "SELECT PARSE_DATETIME('2024-01-15T14:30:45', '%Y-%m-%dT%H:%M:%S') as iso_format"
    result = run_query(query)

    assert len(result) == 1
    assert result[0]["iso_format"] == "2024-01-15 14:30:45.000"
    print("✓ PARSE_DATETIME with ISO 8601 format")


def test_parse_datetime_with_milliseconds():
    """Test parsing timestamps with millisecond precision"""
    query = """
        SELECT
            order_id,
            transaction_time,
            PARSE_DATETIME(transaction_time, '%Y%m%d-%H:%M:%S%.3f') as parsed,
            DATEDIFF('millisecond',
                DATETIME(2025, 9, 25, 14, 52, 15),
                PARSE_DATETIME(transaction_time, '%Y%m%d-%H:%M:%S%.3f')
            ) as ms_offset
        FROM fix_timestamps
        LIMIT 3
    """
    result = run_query(query, "data/fix_timestamps.csv")

    assert len(result) == 3
    assert result[0]["ms_offset"] == 567
    assert result[1]["ms_offset"] == 789
    assert result[2]["ms_offset"] == 891
    print("✓ Parse timestamps with millisecond precision")


if __name__ == "__main__":
    print("\n=== Testing Flexible Date Parsing Functions ===\n")

    try:
        # PARSE_DATETIME tests
        test_parse_datetime_with_european_format()
        test_parse_datetime_with_american_format()
        test_parse_datetime_with_time()
        test_parse_datetime_with_text_month()
        test_parse_datetime_with_fix_format()
        test_parse_datetime_iso8601()
        test_parse_datetime_with_milliseconds()
        test_parse_datetime_with_null()

        # PARSE_DATETIME_UTC tests
        test_parse_datetime_utc_auto_detect()
        test_parse_datetime_utc_with_fix_auto_detect()
        test_parse_datetime_utc_with_custom_format()
        test_parse_datetime_utc_with_fix_data()

        # DATETIME constructor tests
        test_datetime_constructor_date_only()
        test_datetime_constructor_with_time()
        test_datetime_constructor_validates_dates()
        test_datetime_constructor_partial_time()
        test_datetime_constructor_with_leap_year()

        # Combined tests
        test_combined_date_functions()

        print("\n✅ All tests passed!")

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        exit(1)
    except subprocess.CalledProcessError as e:
        print(f"\n❌ Query execution failed:")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        exit(1)
