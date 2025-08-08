using System;
using System.Collections.Generic;
using System.Text.Json;

namespace SqlCliProxy
{
    /// <summary>
    /// Concrete test example showing exact sql-cli input and q output
    /// </summary>
    public class TestQTranslation
    {
        public static void Main(string[] args)
        {
            Console.WriteLine("===========================================");
            Console.WriteLine("SQL-CLI to KDB+ Query Translation Test");
            Console.WriteLine("===========================================\n");

            // Test Case 1: Simple equality with case-insensitive
            TestSimpleEquality();
            
            // Test Case 2: Complex query from your debug output
            TestComplexQuery();
            
            // Test Case 3: Various operators
            TestVariousOperators();
        }

        static void TestSimpleEquality()
        {
            Console.WriteLine("TEST 1: Simple Equality (Case-Insensitive)");
            Console.WriteLine("-------------------------------------------");
            
            var sqlQuery = "SELECT * FROM trades WHERE confirmationStatus = 'pending'";
            
            // This is what sql-cli would send
            var astTree = @"
EQUAL(confirmationStatus, String(""pending""))";

            Console.WriteLine($"SQL Input: {sqlQuery}");
            Console.WriteLine($"Case Insensitive: true");
            Console.WriteLine($"\nAST Tree:{astTree}");
            
            var qQuery = AdvancedQTranslator.TranslateAstTreeToQ(astTree, "trades", caseInsensitive: true);
            
            Console.WriteLine($"\nGenerated q Query:");
            Console.WriteLine($"{qQuery}");
            Console.WriteLine("\nExpected q execution:");
            Console.WriteLine("q) select from trades where (lower[confirmationStatus]=lower[`pending])");
            Console.WriteLine("\n");
        }

        static void TestComplexQuery()
        {
            Console.WriteLine("TEST 2: Complex Query with Multiple Conditions");
            Console.WriteLine("-----------------------------------------------");
            
            var sqlQuery = @"SELECT * FROM trades 
WHERE confirmationStatus.StartsWith('pend') 
  AND commission BETWEEN 30 AND 100 
  AND createdDate > DateTime(2025,07,10)";

            // Exact AST from your debug output
            var astTree = @"
AND
  AND
    STARTS_WITH_IGNORE_CASE(confirmationStatus, ""pend"")
    BETWEEN(commission, Number(30.0), Number(100.0))
  GREATER_THAN(createdDate, String(""2025-07-10 00:00:00""))";

            Console.WriteLine($"SQL Input:\n{sqlQuery}");
            Console.WriteLine($"\nCase Insensitive: true");
            Console.WriteLine($"\nAST Tree:{astTree}");
            
            var qQuery = AdvancedQTranslator.TranslateAstTreeToQ(astTree, "trades", caseInsensitive: true);
            
            Console.WriteLine($"\nGenerated q Query:");
            Console.WriteLine($"{qQuery}");
            
            Console.WriteLine("\nFormatted for readability:");
            Console.WriteLine(@"q) select from trades where 
     (lower[confirmationStatus] like lower[""pend*""]),
     (commission within 30 100),
     (createdDate>2025.07.10)");
            Console.WriteLine("\n");
        }

        static void TestVariousOperators()
        {
            Console.WriteLine("TEST 3: Various SQL Operators");
            Console.WriteLine("------------------------------");
            
            var testCases = new List<(string sql, string ast, string expectedQ)>
            {
                (
                    "WHERE status IN ('Pending', 'Confirmed', 'Active')",
                    "IN_IGNORE_CASE(status, [\"Pending\", \"Confirmed\", \"Active\"])",
                    "select from trades where (lower[status] in (lower[`Pending];lower[`Confirmed];lower[`Active]))"
                ),
                (
                    "WHERE trader != 'John Smith'",
                    "NOT_EQUAL(trader, String(\"John Smith\"))",
                    "select from trades where (not trader=`$\"John Smith\")"
                ),
                (
                    "WHERE quantity >= 1000 AND price < 150.50",
                    "AND\n  GREATER_THAN_OR_EQUAL(quantity, Number(1000))\n  LESS_THAN(price, Number(150.50))",
                    "select from trades where (quantity>=1000),(price<150.50)"
                ),
                (
                    "WHERE instrumentName.Contains('BOND')",
                    "CONTAINS_IGNORE_CASE(instrumentName, \"BOND\")",
                    "select from trades where (lower[\"BOND\"] in lower[instrumentName])"
                ),
                (
                    "WHERE settlementDate BETWEEN DateTime(2025,01,01) AND DateTime(2025,12,31)",
                    "BETWEEN(settlementDate, String(\"2025-01-01\"), String(\"2025-12-31\"))",
                    "select from trades where (settlementDate within 2025.01.01 2025.12.31)"
                )
            };

            foreach (var (sql, ast, expectedQ) in testCases)
            {
                Console.WriteLine($"\nSQL: {sql}");
                Console.WriteLine($"AST: {ast}");
                
                var qQuery = AdvancedQTranslator.TranslateAstTreeToQ(ast, "trades", caseInsensitive: true);
                Console.WriteLine($"Generated q: {qQuery}");
            }
            
            Console.WriteLine("\n");
        }
    }

    /// <summary>
    /// Enhanced translator with more operators
    /// </summary>
    public partial class AdvancedQTranslator
    {
        // Add NOT_EQUAL support
        public string ParseNotEqual(string node)
        {
            var match = System.Text.RegularExpressions.Regex.Match(
                node, @"NOT_EQUAL\((\w+),\s*(?:String|Number)\(""?(.+?)""?\)\)");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var value = match.Groups[2].Value;
                
                if (CaseInsensitive)
                {
                    return $"(not lower[{column}]=lower[`$\"{value}\"])";
                }
                
                if (!IsNumeric(value))
                {
                    return $"(not {column}=`$\"{value}\")";
                }
                return $"(not {column}={value})";
            }
            return "";
        }
    }
}