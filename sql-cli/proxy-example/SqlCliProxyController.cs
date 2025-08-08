using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using System.Text.Json;
using System.Net.Http;

namespace SqlCliProxy
{
    /// <summary>
    /// REST API Controller that acts as a proxy between sql-cli and kdb+
    /// This would be deployed as an ASP.NET Core Web API
    /// </summary>
    [ApiController]
    [Route("api/[controller]")]
    public class QueryController : ControllerBase
    {
        private readonly IKdbConnection _kdbConnection;
        private readonly ILogger<QueryController> _logger;

        public QueryController(IKdbConnection kdbConnection, ILogger<QueryController> logger)
        {
            _kdbConnection = kdbConnection;
            _logger = logger;
        }

        /// <summary>
        /// Main endpoint that sql-cli would call
        /// POST /api/query/execute
        /// </summary>
        [HttpPost("execute")]
        public async Task<IActionResult> ExecuteQuery([FromBody] QueryRequest request)
        {
            try
            {
                _logger.LogInformation($"Received SQL query: {request.SqlQuery}");
                
                // Step 1: Translate SQL to q
                var qQuery = TranslateQuery(request);
                _logger.LogInformation($"Translated to q: {qQuery}");
                
                // Step 2: Execute against kdb+
                var kdbResult = await _kdbConnection.ExecuteQueryAsync(qQuery);
                
                // Step 3: Transform result back to sql-cli format
                var response = TransformToSqlCliFormat(kdbResult, request);
                
                return Ok(response);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error executing query");
                return StatusCode(500, new { error = ex.Message });
            }
        }

        /// <summary>
        /// Schema endpoint for sql-cli autocomplete
        /// GET /api/query/schema/{tableName}
        /// </summary>
        [HttpGet("schema/{tableName}")]
        public async Task<IActionResult> GetSchema(string tableName)
        {
            try
            {
                // Get table metadata from kdb+
                var qQuery = $"meta {tableName}";
                var kdbResult = await _kdbConnection.ExecuteQueryAsync(qQuery);
                
                // Transform to sql-cli schema format
                var schema = new SchemaResponse
                {
                    TableName = tableName,
                    Columns = ExtractColumns(kdbResult)
                };
                
                return Ok(schema);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, $"Error getting schema for {tableName}");
                return StatusCode(500, new { error = ex.Message });
            }
        }

        private string TranslateQuery(QueryRequest request)
        {
            // Use our translator
            return SqlToQTranslator.TranslateToQ(
                request.SqlQuery,
                request.AstTree,
                request.Tokens
            );
        }

        private QueryResponse TransformToSqlCliFormat(KdbResult kdbResult, QueryRequest request)
        {
            // Transform kdb+ result to JSON format expected by sql-cli
            var data = new List<Dictionary<string, object>>();
            
            foreach (var row in kdbResult.Rows)
            {
                var jsonRow = new Dictionary<string, object>();
                for (int i = 0; i < kdbResult.Columns.Count; i++)
                {
                    var colName = kdbResult.Columns[i];
                    var value = row[i];
                    
                    // Handle kdb+ types to JSON conversion
                    jsonRow[colName] = ConvertKdbValue(value);
                }
                data.Add(jsonRow);
            }

            return new QueryResponse
            {
                Data = data,
                Count = data.Count,
                Query = new QueryInfo
                {
                    Select = ExtractSelectColumns(request.AstTree),
                    WhereClause = ExtractWhereClause(request.AstTree),
                    OrderBy = ExtractOrderBy(request.AstTree)
                },
                Source = "kdb",
                Table = ExtractTableName(request.AstTree),
                Cached = false
            };
        }

        private object ConvertKdbValue(object kdbValue)
        {
            // Convert kdb+ types to JSON-compatible types
            if (kdbValue == null) return null;
            
            switch (kdbValue)
            {
                case DateTime dt:
                    return dt.ToString("yyyy-MM-dd HH:mm:ss");
                case decimal d:
                    return (double)d;
                case byte[] bytes:
                    return Convert.ToBase64String(bytes);
                default:
                    return kdbValue.ToString();
            }
        }

        private List<string> ExtractSelectColumns(JsonElement ast)
        {
            if (ast.TryGetProperty("columns", out var columns))
            {
                return columns.EnumerateArray()
                    .Select(c => c.GetString())
                    .ToList();
            }
            return new List<string> { "*" };
        }

        private string ExtractWhereClause(JsonElement ast)
        {
            if (ast.TryGetProperty("where_clause", out var where))
            {
                return where.GetRawText();
            }
            return null;
        }

        private string ExtractOrderBy(JsonElement ast)
        {
            if (ast.TryGetProperty("order_by", out var orderBy))
            {
                return orderBy.GetRawText();
            }
            return null;
        }

        private string ExtractTableName(JsonElement ast)
        {
            if (ast.TryGetProperty("from_table", out var table))
            {
                return table.GetString();
            }
            return null;
        }

        private List<ColumnInfo> ExtractColumns(KdbResult metaResult)
        {
            var columns = new List<ColumnInfo>();
            
            foreach (var row in metaResult.Rows)
            {
                columns.Add(new ColumnInfo
                {
                    Name = row[0].ToString(),
                    Type = MapKdbTypeToSql(row[1].ToString()),
                    IsNullable = true // kdb+ doesn't have nullable concept like SQL
                });
            }
            
            return columns;
        }

        private string MapKdbTypeToSql(string kdbType)
        {
            // Map kdb+ types to SQL types
            return kdbType switch
            {
                "s" => "symbol",
                "j" => "long",
                "i" => "int",
                "f" => "float",
                "e" => "real",
                "d" => "date",
                "z" => "datetime",
                "c" => "char",
                "C" => "string",
                _ => "varchar"
            };
        }
    }

    // Request/Response models matching sql-cli format
    public class QueryRequest
    {
        public string SqlQuery { get; set; }
        public JsonElement AstTree { get; set; }
        public List<string> Tokens { get; set; }
        public bool CaseInsensitive { get; set; }
    }

    public class QueryResponse
    {
        public List<Dictionary<string, object>> Data { get; set; }
        public int Count { get; set; }
        public QueryInfo Query { get; set; }
        public string Source { get; set; }
        public string Table { get; set; }
        public bool Cached { get; set; }
    }

    public class QueryInfo
    {
        public List<string> Select { get; set; }
        public string WhereClause { get; set; }
        public string OrderBy { get; set; }
    }

    public class SchemaResponse
    {
        public string TableName { get; set; }
        public List<ColumnInfo> Columns { get; set; }
    }

    public class ColumnInfo
    {
        public string Name { get; set; }
        public string Type { get; set; }
        public bool IsNullable { get; set; }
    }

    // Interfaces for kdb+ connection (would be implemented separately)
    public interface IKdbConnection
    {
        Task<KdbResult> ExecuteQueryAsync(string qQuery);
    }

    public class KdbResult
    {
        public List<string> Columns { get; set; }
        public List<object[]> Rows { get; set; }
    }
}