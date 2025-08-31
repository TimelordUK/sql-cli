# Python Test Suite

This directory contains the Python test suite for the SQL CLI project. All tests are automatically discovered and run by the test runner.

## Structure

```
tests/python_tests/
├── __init__.py                               # Package marker
├── README.md                                 # This file
├── test_sql_engine_pytest.py                # Core SQL engine functionality tests
├── test_string_methods_comprehensive.py     # String method tests
├── test_math_date_functions.py              # Mathematical and date function tests  
├── test_advanced_sql_queries.py             # Advanced SQL query tests
├── test_ast_parser.py                        # AST parser validation tests
└── test_case_when_evaluation.py             # CASE WHEN expression tests
```

## Running Tests

### Run All Python Tests
```bash
./run_python_tests.sh
```

### Run Individual Test Files
```bash
uv run pytest tests/python_tests/test_case_when_evaluation.py -v
```

### Run Specific Test Methods
```bash
uv run pytest tests/python_tests/test_case_when_evaluation.py::TestCaseWhenEvaluation::test_basic_case_when_numeric -v
```

## Test Categories

### Core Functionality
- **test_sql_engine_pytest.py**: Basic SQL operations (SELECT, WHERE, math functions, string methods)
- **test_advanced_sql_queries.py**: Complex queries with nested functions and multiple operations

### Feature-Specific Tests
- **test_string_methods_comprehensive.py**: String methods (.Trim(), .Length(), .Contains(), etc.)
- **test_math_date_functions.py**: Mathematical functions (ROUND, FLOOR, POWER) and date functions
- **test_case_when_evaluation.py**: CASE WHEN conditional expressions
- **test_ast_parser.py**: Parser AST validation (ensures parsing works correctly)

## Adding New Tests

### Automatic Discovery
Simply create a new file matching the pattern `test_*.py` in this directory. It will be automatically discovered and run by the test runner.

### Test File Template
```python
#!/usr/bin/env python3
"""
Description of what this test suite covers
"""

import subprocess
import pytest
import pandas as pd
from pathlib import Path
from io import StringIO


class TestYourFeature:
    """Test suite for your feature"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
    
    def run_query(self, csv_file: str, query: str):
        """Helper to run a SQL query and return DataFrame and error"""
        cmd = [
            self.sql_cli, 
            str(self.project_root / "data" / csv_file), 
            "-q", query, 
            "-o", "csv"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        
        if result.returncode != 0:
            return None, result.stderr.strip()
            
        if result.stdout.strip():
            return pd.read_csv(StringIO(result.stdout.strip())), None
        return pd.DataFrame(), None

    def test_your_feature(self):
        """Test your feature functionality"""
        query = "SELECT * FROM test_simple_math WHERE id = 1"
        df, err = self.run_query("test_simple_math.csv", query)
        
        assert df is not None, f"Query failed: {err}"
        assert len(df) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

### Key Points for New Tests
1. **Path Resolution**: Use `Path(__file__).parent.parent.parent` to get the project root
2. **Test Data**: Use existing test CSV files in `data/` directory (`test_simple_math.csv`, `test_simple_strings.csv`)
3. **Error Handling**: Always check for query errors and provide meaningful assertions
4. **Test Isolation**: Each test should be independent and not rely on other tests

## Test Data

The tests use CSV files in the `data/` directory:
- `test_simple_math.csv`: Numeric data for mathematical operations
- `test_simple_strings.csv`: String data for text operations

## Integration with CI/CD

The `run_python_tests.sh` script automatically:
1. Builds the SQL CLI if needed
2. Generates test data if missing
3. Discovers and runs all tests in this directory
4. Reports results with clear success/failure status

## Performance Expectations

- Full test suite: ~1 second for 111+ tests
- Individual test files: 100-300ms typically
- Tests run in non-interactive mode for reliability