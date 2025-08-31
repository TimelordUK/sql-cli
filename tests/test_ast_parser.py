#!/usr/bin/env python3
"""
Test suite for SQL parser AST validation, particularly for CASE expressions
"""

import subprocess
import pytest
import json
import re
from pathlib import Path
from typing import Dict, Any, Optional

class TestAstParser:
    """Test suite for validating AST output from the SQL parser"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.project_root = Path(__file__).parent.parent
        cls.sql_cli = str(cls.project_root / "target" / "release" / "sql-cli")
        
        # Build if needed
        if not Path(cls.sql_cli).exists():
            subprocess.run(["cargo", "build", "--release"], 
                          cwd=cls.project_root, check=True)
        
        # Create a dummy CSV for testing
        cls.test_csv = cls.project_root / "data" / "test_dummy.csv"
        if not cls.test_csv.exists():
            with open(cls.test_csv, 'w') as f:
                f.write("id,price,category,status\n")
                f.write("1,99.99,A,active\n")
                f.write("2,150.50,B,pending\n")
    
    def get_ast(self, query: str) -> Optional[str]:
        """Get the AST output for a query using --query-plan"""
        cmd = [
            self.sql_cli,
            str(self.test_csv),
            "-q", query,
            "--query-plan"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        
        # Even if the query fails to execute, we might still get AST output
        # The parser runs before the evaluator
        if result.returncode != 0 and "=== QUERY PLAN (AST) ===" not in result.stdout:
            print(f"Error: {result.stderr}")
            return None
            
        # Extract AST from output (between === markers)
        ast_match = re.search(r'=== QUERY PLAN \(AST\) ===\n(.*?)\n=== END QUERY PLAN ===', 
                             result.stdout, re.DOTALL)
        if ast_match:
            return ast_match.group(1)
        return result.stdout
    
    def assert_ast_contains(self, ast: str, expected: str):
        """Assert that the AST contains expected string"""
        assert expected in ast, f"Expected '{expected}' not found in AST:\n{ast}"
    
    def assert_ast_structure(self, ast: str, *expected_patterns):
        """Assert that AST contains expected structural patterns"""
        for pattern in expected_patterns:
            assert pattern in ast, f"Pattern '{pattern}' not found in AST:\n{ast}"
    
    # CASE EXPRESSION TESTS
    
    def test_simple_case_expression(self):
        """Test parsing of simple CASE expression"""
        query = """
        SELECT 
            id,
            CASE 
                WHEN price > 100 THEN 'Expensive'
                ELSE 'Cheap'
            END as price_category
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        # Check for CASE expression structure
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "when_branches")
        self.assert_ast_contains(ast, "else_branch")
        self.assert_ast_contains(ast, "WhenBranch")
        
        # Check for the condition and results
        self.assert_ast_structure(ast,
            "BinaryOp",
            "price",
            ">",
            "100",
            "Expensive",
            "Cheap"
        )
    
    def test_multiple_when_clauses(self):
        """Test CASE with multiple WHEN clauses"""
        query = """
        SELECT 
            CASE 
                WHEN price > 200 THEN 'Premium'
                WHEN price > 100 THEN 'Standard'
                WHEN price > 50 THEN 'Budget'
                ELSE 'Economy'
            END as tier
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        # Should have multiple WhenBranch entries
        when_count = ast.count("WhenBranch")
        assert when_count == 3, f"Expected 3 WhenBranch, found {when_count}"
        
        # Check all price thresholds are present
        self.assert_ast_structure(ast, "200", "100", "50")
        self.assert_ast_structure(ast, "Premium", "Standard", "Budget", "Economy")
    
    def test_case_without_else(self):
        """Test CASE expression without ELSE clause"""
        query = """
        SELECT 
            CASE 
                WHEN status = 'active' THEN 'Active'
                WHEN status = 'pending' THEN 'Pending'
            END as status_label
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        # Check that else_branch is None or not present
        # The exact representation depends on Debug implementation
        assert "else_branch: None" in ast or "else_branch: Some" not in ast
    
    def test_case_with_functions(self):
        """Test CASE with function calls in conditions"""
        query = """
        SELECT 
            CASE 
                WHEN MOD(id, 2) = 0 THEN 'Even'
                WHEN ROUND(price, 0) > 100 THEN 'High'
                ELSE 'Other'
            END as classification
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "FunctionCall")
        self.assert_ast_structure(ast, "MOD", "ROUND")
    
    def test_case_with_string_methods(self):
        """Test CASE with string method calls"""
        query = """
        SELECT 
            CASE 
                WHEN category.StartsWith('A') THEN 'A-Type'
                WHEN status.Contains('act') THEN 'Active-like'
                ELSE 'Other'
            END as type
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "MethodCall")
        self.assert_ast_structure(ast, "StartsWith", "Contains")
    
    def test_nested_case_expressions(self):
        """Test nested CASE expressions"""
        query = """
        SELECT 
            CASE 
                WHEN price > 100 THEN
                    CASE 
                        WHEN category = 'A' THEN 'Premium A'
                        ELSE 'Premium Other'
                    END
                ELSE 'Standard'
            END as classification
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        # Should have two CaseExpression nodes
        case_count = ast.count("CaseExpression")
        assert case_count == 2, f"Expected 2 CaseExpression nodes, found {case_count}"
    
    def test_case_in_where_clause(self):
        """Test CASE expression in WHERE clause"""
        query = """
        SELECT id, price
        FROM test_dummy
        WHERE CASE 
            WHEN category = 'A' THEN price > 50
            ELSE price > 100
        END
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "where_clause")
        self.assert_ast_contains(ast, "CaseExpression")
    
    def test_case_with_arithmetic(self):
        """Test CASE with arithmetic expressions"""
        query = """
        SELECT 
            CASE 
                WHEN price * 1.1 > 150 THEN 'High with tax'
                WHEN price + 10 > 100 THEN 'Medium with shipping'
                ELSE 'Low'
            END as price_tier
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "BinaryOp")
        self.assert_ast_structure(ast, "*", "+", "1.1", "10")
    
    def test_multiple_case_expressions(self):
        """Test multiple CASE expressions in same SELECT"""
        query = """
        SELECT 
            CASE WHEN price > 100 THEN 'High' ELSE 'Low' END as price_tier,
            CASE WHEN category = 'A' THEN 1 ELSE 0 END as is_category_a
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        # Should have two CaseExpression nodes
        case_count = ast.count("CaseExpression")
        assert case_count == 2, f"Expected 2 CaseExpression nodes, found {case_count}"
        
        # Check both aliases are present
        self.assert_ast_structure(ast, "price_tier", "is_category_a")
    
    def test_case_with_between(self):
        """Test CASE with BETWEEN operator"""
        query = """
        SELECT 
            CASE 
                WHEN price BETWEEN 50 AND 100 THEN 'Mid-range'
                WHEN price > 100 THEN 'High'
                ELSE 'Low'
            END as range
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "Between")
    
    def test_case_with_in_list(self):
        """Test CASE with IN operator"""
        query = """
        SELECT 
            CASE 
                WHEN category IN ('A', 'B') THEN 'Primary'
                WHEN status IN ('active', 'pending') THEN 'Current'
                ELSE 'Other'
            END as group_type
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "InList")
    
    def test_case_with_not(self):
        """Test CASE with NOT operator"""
        query = """
        SELECT 
            CASE 
                WHEN NOT status = 'inactive' THEN 'Active'
                ELSE 'Inactive'
            END as is_active
        FROM test_dummy
        """
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "CaseExpression")
        self.assert_ast_contains(ast, "Not")
    
    # NON-CASE TESTS (to ensure we didn't break anything)
    
    def test_basic_select_still_works(self):
        """Ensure basic SELECT still parses correctly"""
        query = "SELECT id, price FROM test_dummy WHERE price > 100"
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_structure(ast,
            "SelectStatement",
            "columns:",
            "where_clause:",
            "BinaryOp"
        )
        # Should NOT contain CASE
        assert "CaseExpression" not in ast
    
    def test_functions_still_work(self):
        """Ensure function calls still parse correctly"""
        query = "SELECT ROUND(price, 2) as rounded, MOD(id, 10) as bucket FROM test_dummy"
        ast = self.get_ast(query)
        assert ast is not None, "Failed to get AST"
        
        self.assert_ast_contains(ast, "FunctionCall")
        self.assert_ast_structure(ast, "ROUND", "MOD")
        # Should NOT contain CASE
        assert "CaseExpression" not in ast


if __name__ == "__main__":
    pytest.main([__file__, "-v"])