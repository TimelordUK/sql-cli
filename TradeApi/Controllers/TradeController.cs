using Microsoft.AspNetCore.Mvc;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Dynamic.Core;
using System.Reflection;
using TradeApi.Models;
using TradeApi.Services;

namespace TradeApi.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class TradeController : ControllerBase
    {
        private readonly TradeDataService _tradeService;
        private readonly QueryProcessor _queryProcessor;
        private readonly ILogger<TradeController> _logger;
        
        public TradeController(TradeDataService tradeService, QueryProcessor queryProcessor, ILogger<TradeController> logger)
        {
            _tradeService = tradeService;
            _queryProcessor = queryProcessor;
            _logger = logger;
        }
        
        [HttpPost("query")]
        public IActionResult Query([FromBody] QueryRequest request)
        {
            try
            {
                _logger.LogInformation("Processing query request: Select={Select}, Where={Where}, OrderBy={OrderBy}", 
                    string.Join(",", request.Select ?? new List<string>()), 
                    request.Where, 
                    request.OrderBy);
                
                var query = _tradeService.GetTrades();
                
                // Apply WHERE clause using Dynamic LINQ
                if (!string.IsNullOrWhiteSpace(request.Where))
                {
                    try
                    {
                        var processedWhereClause = _queryProcessor.ProcessWhereClause(request.Where);
                        query = query.Where(processedWhereClause);
                        _logger.LogDebug("Applied WHERE filter: {ProcessedWhere}", processedWhereClause);
                    }
                    catch (Exception ex)
                    {
                        _logger.LogError(ex, "WHERE clause processing failed for query: {OriginalWhere}", request.Where);
                        return BadRequest(new { error = $"Invalid WHERE clause: {ex.Message}", originalQuery = request.Where });
                    }
                }
                
                // Apply ORDER BY
                if (!string.IsNullOrWhiteSpace(request.OrderBy))
                {
                    _logger.LogDebug("Applying ORDER BY: {OrderBy}", request.OrderBy);
                    query = query.OrderBy(request.OrderBy);
                }
                
                // Apply pagination
                if (request.Skip.HasValue)
                {
                    query = query.Skip(request.Skip.Value);
                }
                
                var takeAmount = request.Take ?? 100;
                query = query.Take(takeAmount);
                
                // Execute query and select fields
                _logger.LogDebug("Executing query, taking {TakeAmount} records", takeAmount);
                var results = query.ToList();
                _logger.LogInformation("Query returned {ResultCount} records", results.Count);
                
                if (request.Select != null && request.Select.Any() && !request.Select.Contains("*"))
                {
                    // Dynamic projection
                    var selectedData = results.Select(trade =>
                    {
                        var dict = new Dictionary<string, object>();
                        var tradeType = typeof(TradeDeal);
                        
                        foreach (var field in request.Select)
                        {
                            var prop = tradeType.GetProperty(field, 
                                BindingFlags.IgnoreCase | BindingFlags.Public | BindingFlags.Instance);
                            
                            if (prop != null)
                            {
                                dict[field] = prop.GetValue(trade);
                            }
                        }
                        
                        return dict;
                    }).ToList();
                    
                    return Ok(new { 
                        data = selectedData,
                        count = selectedData.Count,
                        query = new {
                            select = request.Select,
                            where = request.Where,
                            orderBy = request.OrderBy
                        }
                    });
                }
                
                return Ok(new { 
                    data = results,
                    count = results.Count,
                    query = new {
                        select = request.Select ?? new List<string> { "*" },
                        where = request.Where,
                        orderBy = request.OrderBy
                    }
                });
            }
            catch (Exception ex)
            {
                return BadRequest(new { 
                    error = ex.Message,
                    details = ex.InnerException?.Message,
                    query = request
                });
            }
        }
        
        [HttpGet("schema/trade_deal")]
        public IActionResult GetTradeDealSchema()
        {
            var columns = typeof(TradeDeal).GetProperties()
                .Select(p => new ColumnInfo
                {
                    Name = p.Name,
                    Type = GetSimpleTypeName(p.PropertyType),
                    IsNullable = IsNullable(p.PropertyType)
                })
                .OrderBy(c => c.Name)
                .ToList();
                
            return Ok(new SchemaResponse
            {
                TableName = "trade_deal",
                Columns = columns
            });
        }
        
        [HttpGet("sample")]
        public IActionResult GetSampleData()
        {
            var samples = _tradeService.GetTrades()
                .Take(5)
                .ToList();
                
            return Ok(samples);
        }
        
        private string GetSimpleTypeName(Type type)
        {
            if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Nullable<>))
            {
                return GetSimpleTypeName(type.GetGenericArguments()[0]);
            }
            
            return type.Name switch
            {
                "String" => "string",
                "Int32" => "int",
                "Decimal" => "decimal",
                "DateTime" => "datetime",
                "Boolean" => "bool",
                _ => type.Name.ToLower()
            };
        }
        
        private bool IsNullable(Type type)
        {
            return type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Nullable<>);
        }
    }
}