using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;

namespace SqlCliProxy
{
    /// <summary>
    /// Translates SQL AST from sql-cli to kdb+/q queries
    /// </summary>
    public class SqlToQTranslator
    {
        /// <summary>
        /// Main translation function that takes the AST and produces a q query
        /// </summary>
        public static string TranslateToQ(string sqlQuery, JsonElement astTree, List<string> tokens)
        {
            var translator = new SqlToQTranslator();
            
            // Parse the main components
            var selectColumns = translator.ExtractSelectColumns(astTree);
            var tableName = translator.ExtractTableName(astTree);
            var whereClause = translator.ExtractWhereClause(astTree);
            
            // Build the q query
            return translator.BuildQQuery(tableName, selectColumns, whereClause);
        }

        private List<string> ExtractSelectColumns(JsonElement ast)
        {
            // For SELECT *, we'll handle it specially in q
            if (ast.TryGetProperty("columns", out var columns))
            {
                var cols = new List<string>();
                foreach (var col in columns.EnumerateArray())
                {
                    cols.Add(col.GetString());
                }
                return cols;
            }
            return new List<string> { "*" };
        }

        private string ExtractTableName(JsonElement ast)
        {
            if (ast.TryGetProperty("from_table", out var table))
            {
                return table.GetString();
            }
            return "trades"; // default
        }

        private string ExtractWhereClause(JsonElement ast)
        {
            if (!ast.TryGetProperty("where_clause", out var whereClause))
                return "";

            return TranslateWhereExpression(whereClause);
        }

        private string TranslateWhereExpression(JsonElement expr)
        {
            // This would parse the WHERE clause AST and convert to q
            var qConditions = new List<string>();
            
            if (expr.TryGetProperty("conditions", out var conditions))
            {
                foreach (var condition in conditions.EnumerateArray())
                {
                    var qCond = TranslateCondition(condition);
                    if (!string.IsNullOrEmpty(qCond))
                        qConditions.Add(qCond);
                }
            }

            return string.Join(",", qConditions);
        }

        private string TranslateCondition(JsonElement condition)
        {
            if (!condition.TryGetProperty("expr", out var expr))
                return "";

            var exprStr = expr.GetString();
            
            // Parse different expression types
            if (exprStr.Contains("StartsWith"))
            {
                return TranslateStartsWith(exprStr);
            }
            else if (exprStr.Contains("BETWEEN"))
            {
                return TranslateBetween(exprStr);
            }
            else if (exprStr.Contains(">"))
            {
                return TranslateComparison(exprStr, ">");
            }
            else if (exprStr.Contains("="))
            {
                return TranslateEquality(exprStr);
            }
            
            return "";
        }

        private string TranslateStartsWith(string expr)
        {
            // confirmationStatus.StartsWith('pend') -> confirmationStatus like "pend*"
            var match = System.Text.RegularExpressions.Regex.Match(
                expr, @"(\w+)\.StartsWith\(.*?['""](.+?)['""]\)");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var prefix = match.Groups[2].Value;
                return $"({column} like \"{prefix}*\")";
            }
            return "";
        }

        private string TranslateBetween(string expr)
        {
            // commission BETWEEN 30 AND 100 -> (commission>=30)&(commission<=100)
            var match = System.Text.RegularExpressions.Regex.Match(
                expr, @"(\w+) BETWEEN .*?(\d+\.?\d*) AND .*?(\d+\.?\d*)");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var lower = match.Groups[2].Value;
                var upper = match.Groups[3].Value;
                return $"(({column}>={lower})&({column}<={upper}))";
            }
            return "";
        }

        private string TranslateComparison(string expr, string op)
        {
            // createdDate > DateTime(2025-07-10) -> createdDate>2025.07.10
            var match = System.Text.RegularExpressions.Regex.Match(
                expr, @"(\w+)\s*" + System.Text.RegularExpressions.Regex.Escape(op) + @"\s*DateTime\((\d{4})-(\d{2})-(\d{2})");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var year = match.Groups[2].Value;
                var month = match.Groups[3].Value;
                var day = match.Groups[4].Value;
                return $"({column}>{year}.{month}.{day})";
            }
            
            // Simple numeric comparison
            match = System.Text.RegularExpressions.Regex.Match(
                expr, @"(\w+)\s*" + System.Text.RegularExpressions.Regex.Escape(op) + @"\s*(\d+\.?\d*)");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var value = match.Groups[2].Value;
                return $"({column}{op}{value})";
            }
            
            return "";
        }

        private string TranslateEquality(string expr)
        {
            // confirmationStatus = 'pending' -> confirmationStatus=`pending
            var match = System.Text.RegularExpressions.Regex.Match(
                expr, @"(\w+)\s*=\s*['""](.+?)['""]");
            
            if (match.Success)
            {
                var column = match.Groups[1].Value;
                var value = match.Groups[2].Value;
                return $"({column}=`{value})";
            }
            return "";
        }

        private string BuildQQuery(string table, List<string> columns, string whereClause)
        {
            var qQuery = new StringBuilder();
            
            // In q, we typically use select statement like:
            // select from trades where (confirmationStatus like "pend*"),(commission>=30)&(commission<=100),createdDate>2025.07.10
            
            qQuery.Append("select ");
            
            // Handle column selection
            if (columns.Count == 1 && columns[0] == "*")
            {
                // Select all columns - in q this is just "select"
            }
            else
            {
                // Specific columns: select col1,col2,col3
                qQuery.Append(string.Join(",", columns));
                qQuery.Append(" ");
            }
            
            qQuery.Append($"from {table}");
            
            if (!string.IsNullOrEmpty(whereClause))
            {
                qQuery.Append($" where {whereClause}");
            }
            
            return qQuery.ToString();
        }
    }

    /// <summary>
    /// Example demonstrating the translation
    /// </summary>
    public class TranslationExample
    {
        public static void Main()
        {
            // Example input from sql-cli
            var sqlQuery = "SELECT * FROM trades where confirmationStatus.StartsWith('pend') and commission between 30 and 100 and createdDate > DateTime(2025,07,10)";
            
            // Simulated AST from the debug output
            var astJson = @"{
                ""columns"": [""*""],
                ""from_table"": ""trades"",
                ""where_clause"": {
                    ""conditions"": [
                        {
                            ""expr"": ""confirmationStatus.StartsWith('pend')"",
                            ""connector"": ""AND""
                        },
                        {
                            ""expr"": ""commission BETWEEN 30 AND 100"",
                            ""connector"": ""AND""
                        },
                        {
                            ""expr"": ""createdDate > DateTime(2025-07-10)""
                        }
                    ]
                }
            }";

            var ast = JsonDocument.Parse(astJson).RootElement;
            var tokens = new List<string>(); // Would come from tokenizer
            
            var qQuery = SqlToQTranslator.TranslateToQ(sqlQuery, ast, tokens);
            
            Console.WriteLine("Original SQL:");
            Console.WriteLine(sqlQuery);
            Console.WriteLine("\nTranslated q query:");
            Console.WriteLine(qQuery);
            Console.WriteLine("\nExpected q output:");
            Console.WriteLine("select from trades where (confirmationStatus like \"pend*\"),((commission>=30)&(commission<=100)),(createdDate>2025.07.10)");
        }
    }
}