using System;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json.Linq;

namespace JsonSelector.Services
{
    /// <summary>
    /// Enhanced selector evaluator that supports both JSON and XML field selection
    /// </summary>
    public class EnhancedSelectorEvaluator
    {
        private readonly XmlFieldParser _xmlParser;
        private readonly Dictionary<string, object> _xmlFieldCache;

        public EnhancedSelectorEvaluator()
        {
            _xmlParser = new XmlFieldParser();
            _xmlFieldCache = new Dictionary<string, object>();
        }

        /// <summary>
        /// Evaluate a selector that may contain XML field references
        /// </summary>
        public object Evaluate(JToken root, string selectorString)
        {
            if (root == null) return null;

            // Check if this is an XML selector
            if (_xmlParser.IsXmlSelector(selectorString))
            {
                return EvaluateXmlSelector(root, selectorString);
            }

            // Use regular selector evaluation
            return SelectorEvaluator.Evaluate(root, selectorString);
        }

        /// <summary>
        /// Evaluate multiple selectors and merge results
        /// </summary>
        public Dictionary<string, object> EvaluateMultiple(JToken root, List<string> selectors)
        {
            var results = new Dictionary<string, object>();

            foreach (var selector in selectors)
            {
                var columnName = GetColumnName(selector);
                var value = Evaluate(root, selector);
                results[columnName] = value;
            }

            return results;
        }

        /// <summary>
        /// Evaluate an XML selector
        /// </summary>
        private object EvaluateXmlSelector(JToken root, string selectorString)
        {
            var (jsonPath, xPath) = _xmlParser.ParseXmlSelector(selectorString);

            // First, get to the JSON field containing XML
            var xmlField = SelectorEvaluator.Evaluate(root, jsonPath);

            if (xmlField == null)
                return null;

            // Convert to string (the XML content)
            string xmlContent = null;
            if (xmlField is string)
            {
                xmlContent = (string)xmlField;
            }
            else if (xmlField is JValue jValue)
            {
                xmlContent = jValue.Value?.ToString();
            }
            else
            {
                xmlContent = xmlField.ToString();
            }

            if (string.IsNullOrEmpty(xmlContent))
                return null;

            // Check if this looks like base64 encoded (common for FIX FPML fields)
            if (IsBase64String(xmlContent))
            {
                try
                {
                    var bytes = Convert.FromBase64String(xmlContent);
                    xmlContent = System.Text.Encoding.UTF8.GetString(bytes);
                }
                catch
                {
                    // Not base64 or decoding failed, use as-is
                }
            }

            // Extract value using XPath
            return _xmlParser.ExtractXmlValue(xmlContent, xPath);
        }

        /// <summary>
        /// Get column name from selector
        /// </summary>
        private string GetColumnName(string selector)
        {
            // For XML selectors, create a meaningful column name
            if (_xmlParser.IsXmlSelector(selector))
            {
                var (jsonPath, xPath) = _xmlParser.ParseXmlSelector(selector);

                // Clean up the XPath for column naming
                var cleanXPath = xPath
                    .Replace("//", "")
                    .Replace("/", "_")
                    .Replace(":", "_")
                    .Replace("[", "_")
                    .Replace("]", "")
                    .Replace("@", "");

                // Get last part of JSON path
                var jsonParts = jsonPath.Split('.');
                var jsonField = jsonParts.LastOrDefault() ?? "xml";

                return $"{jsonField}_{cleanXPath}";
            }

            // For regular selectors, use the last segment
            var parts = selector.Split('.');
            var lastPart = parts[parts.Length - 1];

            // Clean up array notation and aggregates
            if (lastPart.Contains(':'))
            {
                var aggParts = lastPart.Split(':');
                return $"{aggParts[0]}_{aggParts[1]}";
            }

            return lastPart
                .Replace("[*]", "_all")
                .Replace("[", "_")
                .Replace("]", "")
                .Replace("=", "_eq_");
        }

        /// <summary>
        /// Check if a string is base64 encoded
        /// </summary>
        private bool IsBase64String(string s)
        {
            if (string.IsNullOrEmpty(s))
                return false;

            // Quick check for common XML start
            if (s.TrimStart().StartsWith("<"))
                return false;

            try
            {
                // Check if it's valid base64
                var buffer = Convert.FromBase64String(s);

                // Additional check: see if decoded content looks like XML
                var decoded = System.Text.Encoding.UTF8.GetString(buffer);
                return decoded.TrimStart().StartsWith("<");
            }
            catch
            {
                return false;
            }
        }

        /// <summary>
        /// Process a collection of JSON objects with selectors
        /// </summary>
        public List<Dictionary<string, object>> ProcessCollection(
            JArray items,
            List<string> selectors,
            Dictionary<string, object> globalContext = null)
        {
            var results = new List<Dictionary<string, object>>();

            foreach (var item in items)
            {
                var row = EvaluateMultiple(item, selectors);

                // Add any global context fields
                if (globalContext != null)
                {
                    foreach (var kvp in globalContext)
                    {
                        row[kvp.Key] = kvp.Value;
                    }
                }

                results.Add(row);
            }

            return results;
        }
    }
}