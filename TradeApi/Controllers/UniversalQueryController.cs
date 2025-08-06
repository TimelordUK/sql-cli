using Microsoft.AspNetCore.Mvc;
using System.Linq.Dynamic.Core;
using TradeApi.Models;
using TradeApi.Services;

namespace TradeApi.Controllers
{
    [ApiController]
    [Route("api/query")]
    public class UniversalQueryController : ControllerBase
    {
        private readonly DataSourceRouter _router;
        private readonly QueryProcessor _queryProcessor;
        private readonly TradeDataService _tradeService; // For legacy trade_deal
        private readonly ILogger<UniversalQueryController> _logger;

        public UniversalQueryController(
            DataSourceRouter router, 
            QueryProcessor queryProcessor,
            TradeDataService tradeService,
            ILogger<UniversalQueryController> logger)
        {
            _router = router;
            _queryProcessor = queryProcessor;
            _tradeService = tradeService;
            _logger = logger;
        }

        [HttpPost("sql")]
        public async Task<IActionResult> ExecuteSql([FromBody] SqlQueryRequest request)
        {
            try
            {
                _logger.LogInformation("Executing SQL: {Sql}", request.Sql);
                
                // Parse SQL to extract table name
                var tableName = ExtractTableName(request.Sql);
                if (string.IsNullOrEmpty(tableName))
                {
                    return BadRequest(new { error = "Could not extract table name from SQL" });
                }

                // Get data source
                var sourceData = await _router.GetDataForTableAsync(tableName);
                IQueryable<dynamic> query;
                
                if (sourceData == null)
                {
                    // Use legacy trade service
                    query = _tradeService.GetTrades().Cast<dynamic>();
                }
                else
                {
                    query = sourceData;
                }

                // Extract WHERE clause
                var whereClause = ExtractWhereClause(request.Sql);
                if (!string.IsNullOrEmpty(whereClause))
                {
                    var processedWhere = _queryProcessor.ProcessWhereClause(whereClause);
                    query = query.Where(processedWhere);
                }

                // Extract ORDER BY
                var orderBy = ExtractOrderBy(request.Sql);
                if (!string.IsNullOrEmpty(orderBy))
                {
                    query = query.OrderBy(orderBy);
                }

                // Apply limits
                var limit = ExtractLimit(request.Sql) ?? 1000;
                query = query.Take(limit);

                var results = query.ToList();
                
                return Ok(new
                {
                    data = results,
                    count = results.Count,
                    source = _router.GetSourceForTable(tableName),
                    table = tableName,
                    cached = request.UseCache ?? false
                });
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Query execution failed");
                return BadRequest(new { error = ex.Message });
            }
        }

        [HttpGet("tables")]
        public IActionResult GetAvailableTables()
        {
            var tables = _router.GetAllAvailableTables();
            return Ok(tables);
        }

        [HttpGet("schema/{tableName}")]
        public async Task<IActionResult> GetTableSchema(string tableName)
        {
            try
            {
                var schema = await _router.GetSchemaForTableAsync(tableName);
                return Ok(schema);
            }
            catch (ArgumentException ex)
            {
                return NotFound(new { error = ex.Message });
            }
        }

        [HttpPost("cache/save")]
        public async Task<IActionResult> SaveToCache([FromBody] CacheRequest request)
        {
            try
            {
                // Get data for caching
                var sourceData = await _router.GetDataForTableAsync(request.TableName);
                IQueryable<dynamic> query;
                
                if (sourceData == null)
                {
                    query = _tradeService.GetTrades().Cast<dynamic>();
                }
                else
                {
                    query = sourceData;
                }

                // Apply any filters
                if (!string.IsNullOrEmpty(request.WhereClause))
                {
                    var processedWhere = _queryProcessor.ProcessWhereClause(request.WhereClause);
                    query = query.Where(processedWhere);
                }

                var data = query.ToList();
                
                // Generate cache ID
                var cacheId = string.IsNullOrEmpty(request.CacheId) 
                    ? $"{request.TableName}_{DateTime.UtcNow:yyyyMMdd_HHmmss}"
                    : request.CacheId;
                
                // Store in cache (simplified - in real app, use proper caching)
                HttpContext.Session.SetString($"cache_{cacheId}", 
                    System.Text.Json.JsonSerializer.Serialize(data));
                
                return Ok(new
                {
                    cacheId = cacheId,
                    recordCount = data.Count,
                    source = _router.GetSourceForTable(request.TableName),
                    timestamp = DateTime.UtcNow
                });
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Cache save failed");
                return BadRequest(new { error = ex.Message });
            }
        }

        private string ExtractTableName(string sql)
        {
            var parts = sql.Split(new[] { "FROM", "from" }, StringSplitOptions.TrimEntries);
            if (parts.Length < 2) return null;
            
            var afterFrom = parts[1].Split(' ', StringSplitOptions.RemoveEmptyEntries);
            return afterFrom.FirstOrDefault()?.Trim('`', '"', '[', ']');
        }

        private string ExtractWhereClause(string sql)
        {
            var lowerSql = sql.ToLower();
            var whereIndex = lowerSql.IndexOf(" where ");
            if (whereIndex < 0) return null;
            
            var afterWhere = sql.Substring(whereIndex + 7);
            var orderIndex = afterWhere.ToLower().IndexOf(" order by ");
            var limitIndex = afterWhere.ToLower().IndexOf(" limit ");
            
            var endIndex = new[] { orderIndex, limitIndex }
                .Where(i => i >= 0)
                .DefaultIfEmpty(afterWhere.Length)
                .Min();
            
            return afterWhere.Substring(0, endIndex).Trim();
        }

        private string ExtractOrderBy(string sql)
        {
            var lowerSql = sql.ToLower();
            var orderIndex = lowerSql.IndexOf(" order by ");
            if (orderIndex < 0) return null;
            
            var afterOrder = sql.Substring(orderIndex + 10);
            var limitIndex = afterOrder.ToLower().IndexOf(" limit ");
            
            var endIndex = limitIndex >= 0 ? limitIndex : afterOrder.Length;
            return afterOrder.Substring(0, endIndex).Trim();
        }

        private int? ExtractLimit(string sql)
        {
            var lowerSql = sql.ToLower();
            var limitIndex = lowerSql.IndexOf(" limit ");
            if (limitIndex < 0) return null;
            
            var afterLimit = sql.Substring(limitIndex + 7).Trim();
            var parts = afterLimit.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            
            if (parts.Length > 0 && int.TryParse(parts[0], out var limit))
            {
                return limit;
            }
            
            return null;
        }
    }

    public class SqlQueryRequest
    {
        public string Sql { get; set; }
        public bool? UseCache { get; set; }
    }

    public class CacheRequest
    {
        public string TableName { get; set; }
        public string CacheId { get; set; }
        public string WhereClause { get; set; }
    }
}