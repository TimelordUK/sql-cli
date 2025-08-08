using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;

namespace SqlCliProxy
{
    /// <summary>
    /// Advanced SQL to q translator that handles complex WHERE clauses from sql-cli AST
    /// </summary>
    public class AdvancedQTranslator
    {
        /// <summary>
        /// Translates the AST tree format shown in the debug output to q
        /// </summary>
        public static string TranslateAstTreeToQ(string astTreeText, string tableName, bool caseInsensitive = false)
        {
            var translator = new AdvancedQTranslator { CaseInsensitive = caseInsensitive };
            
            // Parse the tree structure
            var conditions = translator.ParseAstTree(astTreeText);
            
            // Build q query
            return translator.BuildQQuery(tableName, conditions);
        }

        private bool CaseInsensitive { get; set; }

        private List<string> ParseAstTree(string treeText)
        {
            var conditions = new List<string>();
            var lines = treeText.Split('\n', StringSplitOptions.RemoveEmptyEntries);
            
            foreach (var line in lines)
            {
                var trimmed = line.Trim();
                
                // Parse different AST node types
                if (trimmed.StartsWith("STARTS_WITH"))
                {
                    conditions.Add(ParseStartsWith(trimmed));
                }
                else if (trimmed.StartsWith("BETWEEN"))
                {
                    conditions.Add(ParseBetween(trimmed));
                }
                else if (trimmed.StartsWith("GREATER_THAN"))
                {
                    conditions.Add(ParseComparison(trimmed, ">"));
                }
                else if (trimmed.StartsWith("LESS_THAN"))
                {
                    conditions.Add(ParseComparison(trimmed, "<"));
                }
                else if (trimmed.StartsWith("EQUAL"))
                {
                    conditions.Add(ParseEqual(trimmed));
                }
                else if (trimmed.StartsWith("IN(") || trimmed.StartsWith("IN_IGNORE_CASE"))
                {
                    conditions.Add(ParseIn(trimmed));
                }
                else if (trimmed.StartsWith("CONTAINS"))
                {
                    conditions.Add(ParseContains(trimmed));
                }
            }
            
            return conditions.Where(c => !string.IsNullOrEmpty(c)).ToList();
        }

        private string ParseStartsWith(string node)
        {
            // STARTS_WITH_IGNORE_CASE(confirmationStatus, "pend")
            var match = Regex.Match(node, @"STARTS_WITH(?:_IGNORE_CASE)?\((\w+),\s*""(.+?)""\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var prefix = match.Groups[2].Value;
                
                if (CaseInsensitive || node.Contains("IGNORE_CASE"))
                {
                    // In q, we'd use lower for case-insensitive
                    return $"(lower[{column}] like lower[\"{prefix}*\"])";
                }
                return $"({column} like \"{prefix}*\")";
            }
            return "";
        }

        private string ParseBetween(string node)
        {
            // BETWEEN(commission, Number(30.0), Number(100.0))
            var match = Regex.Match(node, @"BETWEEN\((\w+),\s*Number\(([\d.]+)\),\s*Number\(([\d.]+)\)\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var lower = match.Groups[2].Value;
                var upper = match.Groups[3].Value;
                
                // In q: within operator or explicit range check
                return $"({column} within {lower} {upper})";
            }
            return "";
        }

        private string ParseComparison(string node, string op)
        {
            // GREATER_THAN(createdDate, String("2025-07-10 00:00:00"))
            var match = Regex.Match(node, @"(?:GREATER|LESS)_THAN(?:_OR_EQUAL)?\((\w+),\s*(?:String|Number)\(""?(.+?)""?\)\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var value = match.Groups[2].Value;
                
                // Handle datetime values
                if (value.Contains("-") && value.Length >= 10)
                {
                    // Convert datetime string to q date format
                    value = ConvertToQDate(value);
                }
                
                // Determine actual operator
                if (node.Contains("GREATER_THAN_OR_EQUAL"))
                    op = ">=";
                else if (node.Contains("GREATER_THAN"))
                    op = ">";
                else if (node.Contains("LESS_THAN_OR_EQUAL"))
                    op = "<=";
                else if (node.Contains("LESS_THAN"))
                    op = "<";
                
                return $"({column}{op}{value})";
            }
            return "";
        }

        private string ParseEqual(string node)
        {
            // EQUAL(confirmationStatus, String("pending"))
            var match = Regex.Match(node, @"EQUAL\((\w+),\s*(?:String|Number)\(""?(.+?)""?\)\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var value = match.Groups[2].Value;
                
                if (CaseInsensitive)
                {
                    // Case-insensitive comparison in q
                    return $"(lower[{column}]=lower[`{value}])";
                }
                
                // For symbols in q, use backtick
                if (!IsNumeric(value))
                {
                    return $"({column}=`{value})";
                }
                return $"({column}={value})";
            }
            return "";
        }

        private string ParseIn(string node)
        {
            // IN_IGNORE_CASE(status, ["pending", "confirmed"])
            var match = Regex.Match(node, @"IN(?:_IGNORE_CASE)?\((\w+),\s*\[(.+?)\]\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var valuesStr = match.Groups[2].Value;
                
                // Parse values list
                var values = Regex.Matches(valuesStr, @"""(.+?)""")
                    .Select(m => m.Groups[1].Value)
                    .ToList();
                
                if (values.Any())
                {
                    if (CaseInsensitive || node.Contains("IGNORE_CASE"))
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
            return "";
        }

        private string ParseContains(string node)
        {
            // CONTAINS(description, "text")
            var match = Regex.Match(node, @"CONTAINS(?:_IGNORE_CASE)?\((\w+),\s*""(.+?)""\)");
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var search = match.Groups[2].Value;
                
                if (CaseInsensitive || node.Contains("IGNORE_CASE"))
                {
                    // q string search with ss (string search)
                    return $"(lower[\"{search}\"] in lower[{column}])";
                }
                return $"(\"{search}\" in {column})";
            }
            return "";
        }

        private string ConvertToQDate(string dateStr)
        {
            // Convert "2025-07-10 00:00:00" to q date format 2025.07.10
            var match = Regex.Match(dateStr, @"(\d{4})-(\d{2})-(\d{2})");
            if (match.Success)
            {
                return $"{match.Groups[1].Value}.{match.Groups[2].Value}.{match.Groups[3].Value}";
            }
            return dateStr;
        }

        private bool IsNumeric(string value)
        {
            return double.TryParse(value, out _);
        }

        private string BuildQQuery(string table, List<string> conditions)
        {
            var query = new StringBuilder();
            query.Append($"select from {table}");
            
            if (conditions.Any())
            {
                query.Append(" where ");
                query.Append(string.Join(",", conditions));
            }
            
            return query.ToString();
        }

        /// <summary>
        /// Example showing the translation of your complex query
        /// </summary>
        public static void DemoComplexTranslation()
        {
            var astTree = @"
AND
  AND
    STARTS_WITH_IGNORE_CASE(confirmationStatus, ""pend"")
    BETWEEN(commission, Number(30.0), Number(100.0))
  GREATER_THAN(createdDate, String(""2025-07-10 00:00:00""))";

            var qQuery = TranslateAstTreeToQ(astTree, "trades", caseInsensitive: true);
            
            Console.WriteLine("AST Tree Input:");
            Console.WriteLine(astTree);
            Console.WriteLine("\nTranslated q Query:");
            Console.WriteLine(qQuery);
            Console.WriteLine("\nExpected Output:");
            Console.WriteLine("select from trades where (lower[confirmationStatus] like lower[\"pend*\"]),(commission within 30 100),(createdDate>2025.07.10)");
            
            // Additional examples
            Console.WriteLine("\n--- More Examples ---");
            
            // Example with IN clause
            var inExample = "IN_IGNORE_CASE(status, [\"pending\", \"confirmed\", \"completed\"])";
            var conditions = new AdvancedQTranslator { CaseInsensitive = true }.ParseAstTree(inExample);
            Console.WriteLine($"\nIN clause: {inExample}");
            Console.WriteLine($"q translation: {conditions[0]}");
            
            // Example with equality
            var equalExample = "EQUAL(trader, String(\"John Smith\"))";
            conditions = new AdvancedQTranslator { CaseInsensitive = true }.ParseAstTree(equalExample);
            Console.WriteLine($"\nEquality: {equalExample}");
            Console.WriteLine($"q translation: {conditions[0]}");
        }
    }

    /// <summary>
    /// Integration point for sql-cli
    /// </summary>
    public class SqlCliKdbProxy
    {
        /// <summary>
        /// This method would be called by sql-cli when in proxy mode
        /// </summary>
        public static async Task<string> ProcessQuery(string sqlQuery, string astDebugOutput, bool caseInsensitive)
        {
            // Extract AST tree section from debug output
            var astTreeSection = ExtractAstTreeSection(astDebugOutput);
            
            // Extract table name
            var tableName = ExtractTableName(astDebugOutput);
            
            // Translate to q
            var qQuery = AdvancedQTranslator.TranslateAstTreeToQ(astTreeSection, tableName, caseInsensitive);
            
            // Here you would send to kdb+ and get results
            // For demo, just return the translated query
            return qQuery;
        }

        private static string ExtractAstTreeSection(string debugOutput)
        {
            var startMarker = "AST Tree:";
            var endMarker = "Note: Parentheses";
            
            var startIdx = debugOutput.IndexOf(startMarker);
            var endIdx = debugOutput.IndexOf(endMarker);
            
            if (startIdx >= 0 && endIdx > startIdx)
            {
                var treeSection = debugOutput.Substring(startIdx + startMarker.Length, endIdx - startIdx - startMarker.Length);
                return treeSection.Trim();
            }
            
            return "";
        }

        private static string ExtractTableName(string debugOutput)
        {
            var match = Regex.Match(debugOutput, @"Table Name:\s*(\w+)");
            return match.Success ? match.Groups[1].Value : "trades";
        }
    }
}