using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using Microsoft.Extensions.Logging;
using TradeApi.Models;

namespace TradeApi.Services
{
    public class QueryProcessor
    {
        private readonly Dictionary<string, string> _propertyMappings;
        private readonly ILogger<QueryProcessor> _logger;
        
        public QueryProcessor(ILogger<QueryProcessor> logger)
        {
            _logger = logger;
            // Map common variations to actual property names
            _propertyMappings = GetPropertyMappings();
        }
        
        public string ProcessWhereClause(string whereClause)
        {
            if (string.IsNullOrWhiteSpace(whereClause))
                return whereClause;
                
            _logger.LogDebug("Processing WHERE clause: {WhereClause}", whereClause);
            
            var processed = whereClause;
            
            // Handle .Contains("value") patterns
            processed = ProcessContainsPatterns(processed);
            
            // Handle property name casing
            processed = FixPropertyNameCasing(processed);
            
            // Handle IN clauses: field in (val1, val2)
            processed = ProcessInClauses(processed);
            
            // Handle comparison operators
            processed = ProcessComparisonOperators(processed);
            
            _logger.LogDebug("Processed WHERE clause result: {ProcessedClause}", processed);
            return processed;
        }
        
        private string ProcessContainsPatterns(string input)
        {
            // Pattern: propertyName.Contains("value")
            // This should work as-is in Dynamic LINQ, just need correct casing
            return input;
        }
        
        private string FixPropertyNameCasing(string input)
        {
            var result = input;
            
            foreach (var mapping in _propertyMappings)
            {
                // Replace case-insensitive property names with correct casing
                var pattern = $@"\b{Regex.Escape(mapping.Key)}\b";
                result = Regex.Replace(result, pattern, mapping.Value, RegexOptions.IgnoreCase);
            }
            
            return result;
        }
        
        private string ProcessInClauses(string input)
        {
            // Convert: field in (val1, val2, val3)
            // To: new[] {val1, val2, val3}.Contains(field)
            
            var inPattern = @"(\w+)\s+in\s*\(([^)]+)\)";
            var matches = Regex.Matches(input, inPattern, RegexOptions.IgnoreCase);
            
            var result = input;
            foreach (Match match in matches)
            {
                var fieldName = match.Groups[1].Value;
                var values = match.Groups[2].Value;
                
                // Fix property casing
                if (_propertyMappings.TryGetValue(fieldName.ToLower(), out var correctFieldName))
                {
                    fieldName = correctFieldName;
                }
                
                var replacement = $"new[] {{{values}}}.Contains({fieldName})";
                result = result.Replace(match.Value, replacement);
            }
            
            return result;
        }
        
        private string ProcessComparisonOperators(string input)
        {
            // Handle various comparison operators
            var result = input;
            
            // Handle string equality with quotes
            result = Regex.Replace(result, @"(\w+)\s*=\s*(['""][^'""]*['""])", "$1 == $2");
            
            return result;
        }
        
        private Dictionary<string, string> GetPropertyMappings()
        {
            var properties = typeof(TradeDeal).GetProperties();
            var mappings = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            
            foreach (var prop in properties)
            {
                // Add various casing variations
                mappings[prop.Name.ToLower()] = prop.Name;
                mappings[prop.Name] = prop.Name;
                
                // Add camelCase variation
                if (prop.Name.Length > 1)
                {
                    var camelCase = char.ToLower(prop.Name[0]) + prop.Name.Substring(1);
                    mappings[camelCase] = prop.Name;
                }
                
                // Add snake_case variation
                var snakeCase = Regex.Replace(prop.Name, @"([A-Z])", "_$1").ToLower().TrimStart('_');
                mappings[snakeCase] = prop.Name;
            }
            
            return mappings;
        }
        
        public List<string> ValidateQuery(string whereClause)
        {
            var errors = new List<string>();
            
            if (string.IsNullOrWhiteSpace(whereClause))
                return errors;
                
            try
            {
                var processed = ProcessWhereClause(whereClause);
                // Could add more validation here
            }
            catch (Exception ex)
            {
                errors.Add($"Query processing error: {ex.Message}");
            }
            
            return errors;
        }
    }
}