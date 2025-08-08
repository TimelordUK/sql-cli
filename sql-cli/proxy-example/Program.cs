using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;

namespace SqlToQDemo
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine(@"
╔══════════════════════════════════════════════════════════════╗
║           SQL-CLI to KDB+/Q Query Translator Demo           ║
╚══════════════════════════════════════════════════════════════╝
");

            // Demonstrate the exact translation from your debug output
            DemoRealQuery();
            
            // Show additional examples
            DemoAdditionalQueries();
            
            Console.WriteLine("\nDemo complete!");
        }

        static void DemoRealQuery()
        {
            Console.WriteLine("ACTUAL QUERY FROM YOUR DEBUG OUTPUT");
            Console.WriteLine("====================================\n");
            
            var sqlQuery = @"SELECT * FROM trades 
WHERE confirmationStatus.StartsWith('pend') 
  AND commission BETWEEN 30 AND 100 
  AND createdDate > DateTime(2025,07,10)";

            Console.WriteLine("Original SQL Query:");
            Console.WriteLine(sqlQuery);
            Console.WriteLine();

            // The exact AST from your debug output
            var astTreeLines = new[]
            {
                "STARTS_WITH_IGNORE_CASE(confirmationStatus, \"pend\")",
                "BETWEEN(commission, Number(30.0), Number(100.0))",
                "GREATER_THAN(createdDate, String(\"2025-07-10 00:00:00\"))"
            };

            Console.WriteLine("AST Tree (from sql-cli parser):");
            Console.WriteLine("AND");
            Console.WriteLine("  AND");
            foreach (var line in astTreeLines)
            {
                Console.WriteLine($"    {line}");
            }
            Console.WriteLine();

            // Translate each component
            var translator = new SimpleQTranslator { CaseInsensitive = true };
            var conditions = new List<string>();

            foreach (var astLine in astTreeLines)
            {
                var qCondition = translator.TranslateAstNode(astLine);
                if (!string.IsNullOrEmpty(qCondition))
                {
                    conditions.Add(qCondition);
                    Console.WriteLine($"AST: {astLine}");
                    Console.WriteLine($"→ q: {qCondition}");
                    Console.WriteLine();
                }
            }

            // Build final q query
            var qQuery = $"select from trades where {string.Join(",", conditions)}";
            
            Console.WriteLine("Final q Query:");
            Console.WriteLine("─────────────");
            Console.WriteLine(qQuery);
            Console.WriteLine();

            Console.WriteLine("Formatted for readability:");
            Console.WriteLine("─────────────────────────");
            Console.WriteLine(@"select from trades where 
    (lower[confirmationStatus] like lower[""pend*""]),
    (commission within 30 100),
    (createdDate>2025.07.10)");
            Console.WriteLine();

            Console.WriteLine("To test in q:");
            Console.WriteLine("────────────");
            Console.WriteLine("1. Start q session: q");
            Console.WriteLine("2. Load the script: \\l create_sample_trades.q");
            Console.WriteLine("3. Run the query above");
            Console.WriteLine("4. Results will show matching trades");
        }

        static void DemoAdditionalQueries()
        {
            Console.WriteLine("\n\nADDITIONAL QUERY EXAMPLES");
            Console.WriteLine("==========================\n");

            var examples = new[]
            {
                new
                {
                    Description = "Case-insensitive equality for 'pending'",
                    SQL = "WHERE confirmationStatus = 'pending'",
                    AST = "EQUAL(confirmationStatus, String(\"pending\"))",
                    QResult = "(lower[confirmationStatus]=lower[`pending])",
                    CaseInsensitive = true
                },
                new
                {
                    Description = "IN clause with multiple values",
                    SQL = "WHERE status IN ('Active', 'Pending', 'Confirmed')",
                    AST = "IN_IGNORE_CASE(status, [\"Active\", \"Pending\", \"Confirmed\"])",
                    QResult = "(lower[status] in (lower[`Active];lower[`Pending];lower[`Confirmed]))",
                    CaseInsensitive = true
                },
                new
                {
                    Description = "NOT EQUAL comparison",
                    SQL = "WHERE trader != 'John Smith'",
                    AST = "NOT_EQUAL(trader, String(\"John Smith\"))",
                    QResult = "(not trader=`$\"John Smith\")",
                    CaseInsensitive = false
                },
                new
                {
                    Description = "Complex numeric conditions",
                    SQL = "WHERE quantity >= 1000 AND price < 150.50",
                    AST = "GREATER_THAN_OR_EQUAL(quantity, Number(1000))\nLESS_THAN(price, Number(150.50))",
                    QResult = "(quantity>=1000),(price<150.50)",
                    CaseInsensitive = false
                }
            };

            var translator = new SimpleQTranslator();
            
            foreach (var example in examples)
            {
                Console.WriteLine($"Example: {example.Description}");
                Console.WriteLine($"SQL: {example.SQL}");
                Console.WriteLine($"AST: {example.AST}");
                
                translator.CaseInsensitive = example.CaseInsensitive;
                var astNodes = example.AST.Split('\n');
                var conditions = new List<string>();
                
                foreach (var node in astNodes)
                {
                    var condition = translator.TranslateAstNode(node);
                    if (!string.IsNullOrEmpty(condition))
                        conditions.Add(condition);
                }
                
                var qQuery = $"select from trades where {string.Join(",", conditions)}";
                Console.WriteLine($"q Query: {qQuery}");
                Console.WriteLine($"Expected: select from trades where {example.QResult}");
                Console.WriteLine();
            }
        }
    }

    /// <summary>
    /// Simple translator for demo purposes
    /// </summary>
    public class SimpleQTranslator
    {
        public bool CaseInsensitive { get; set; }

        public string TranslateAstNode(string astNode)
        {
            // StartsWith
            if (astNode.Contains("STARTS_WITH"))
            {
                var match = Regex.Match(astNode, @"STARTS_WITH(?:_IGNORE_CASE)?\((\w+),\s*""(.+?)""\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var prefix = match.Groups[2].Value;
                    
                    if (CaseInsensitive || astNode.Contains("IGNORE_CASE"))
                    {
                        return $"(lower[{column}] like lower[\"{prefix}*\"])";
                    }
                    return $"({column} like \"{prefix}*\")";
                }
            }

            // Between
            if (astNode.Contains("BETWEEN"))
            {
                var match = Regex.Match(astNode, @"BETWEEN\((\w+),\s*Number\(([\d.]+)\),\s*Number\(([\d.]+)\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var lower = match.Groups[2].Value;
                    var upper = match.Groups[3].Value;
                    return $"({column} within {lower} {upper})";
                }
            }

            // Greater Than
            if (astNode.Contains("GREATER_THAN") && !astNode.Contains("OR_EQUAL"))
            {
                var match = Regex.Match(astNode, @"GREATER_THAN\((\w+),\s*(?:String|Number)\(""?([^)]+?)""?\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var value = match.Groups[2].Value;
                    
                    // Convert date format
                    if (value.Contains("-"))
                    {
                        value = value.Substring(0, 10).Replace("-", ".");
                    }
                    
                    return $"({column}>{value})";
                }
            }

            // Greater Than or Equal
            if (astNode.Contains("GREATER_THAN_OR_EQUAL"))
            {
                var match = Regex.Match(astNode, @"GREATER_THAN_OR_EQUAL\((\w+),\s*Number\(([\d.]+)\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var value = match.Groups[2].Value;
                    return $"({column}>={value})";
                }
            }

            // Less Than
            if (astNode.Contains("LESS_THAN") && !astNode.Contains("OR_EQUAL"))
            {
                var match = Regex.Match(astNode, @"LESS_THAN\((\w+),\s*Number\(([\d.]+)\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var value = match.Groups[2].Value;
                    return $"({column}<{value})";
                }
            }

            // Equal
            if (astNode.Contains("EQUAL") && !astNode.Contains("NOT_EQUAL"))
            {
                var match = Regex.Match(astNode, @"EQUAL\((\w+),\s*String\(""(.+?)""\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var value = match.Groups[2].Value;
                    
                    if (CaseInsensitive)
                    {
                        return $"(lower[{column}]=lower[`{value}])";
                    }
                    return $"({column}=`{value})";
                }
            }

            // IN clause
            if (astNode.Contains("IN") && !astNode.Contains("CONTAINS"))
            {
                var match = Regex.Match(astNode, @"IN(?:_IGNORE_CASE)?\((\w+),\s*\[(.+?)\]\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var valuesStr = match.Groups[2].Value;
                    
                    var values = Regex.Matches(valuesStr, @"""(.+?)""")
                        .Cast<Match>()
                        .Select(m => m.Groups[1].Value)
                        .ToList();
                    
                    if (CaseInsensitive || astNode.Contains("IGNORE_CASE"))
                    {
                        var qValues = string.Join(";", values.Select(v => $"lower[`{v}]"));
                        return $"(lower[{column}] in ({qValues}))";
                    }
                    else
                    {
                        var qValues = string.Join(";", values.Select(v => $"`{v}"));
                        return $"({column} in ({qValues}))";
                    }
                }
            }

            // NOT_EQUAL
            if (astNode.Contains("NOT_EQUAL"))
            {
                var match = Regex.Match(astNode, @"NOT_EQUAL\((\w+),\s*String\(""(.+?)""\)\)");
                if (match.Success)
                {
                    var column = match.Groups[1].Value;
                    var value = match.Groups[2].Value;
                    return $"(not {column}=`$\"{value}\")";
                }
            }

            return "";
        }
    }
}