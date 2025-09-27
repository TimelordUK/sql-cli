using System;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json.Linq;

namespace JsonSelector.Services
{
    /// <summary>
    /// Evaluates selector expressions against JSON objects
    /// </summary>
    public class SelectorEvaluator
    {
        public static object Evaluate(JToken root, string selectorString)
        {
            if (root == null) return null;

            var selector = SelectorParser.Parse(selectorString);
            var result = EvaluateSegments(root, selector.Segments);

            // Apply aggregate function if specified
            if (!string.IsNullOrEmpty(selector.AggregateFunction))
            {
                var aggregateResult = ApplyAggregate(result, selector.AggregateFunction);
                // If aggregate returns a simple type, convert it to JToken for consistency
                if (aggregateResult is JToken)
                    return ConvertToSimpleType((JToken)aggregateResult);
                else
                    return aggregateResult;
            }

            return ConvertToSimpleType(result);
        }

        private static JToken EvaluateSegments(JToken current, List<PathSegment> segments)
        {
            foreach (var segment in segments)
            {
                if (current == null || current.Type == JTokenType.Null)
                    return null;

                current = EvaluateSegment(current, segment);
            }

            return current;
        }

        private static JToken EvaluateSegment(JToken current, PathSegment segment)
        {
            // Navigate to property
            if (!string.IsNullOrEmpty(segment.Property))
            {
                if (current is JObject obj)
                {
                    current = obj[segment.Property];
                    if (current == null) return null;
                }
                else if (current is JArray)
                {
                    // If current is array and we're accessing a property,
                    // map over all elements
                    var array = (JArray)current;
                    var results = new JArray();
                    foreach (var item in array)
                    {
                        if (item is JObject itemObj)
                        {
                            var value = itemObj[segment.Property];
                            if (value != null)
                                results.Add(value);
                        }
                    }
                    current = results.Count > 0 ? results : null;
                }
                else
                {
                    return null;
                }
            }

            // Apply array operations
            if (segment.ArrayIndex.HasValue)
            {
                if (current is JArray array)
                {
                    var index = segment.ArrayIndex.Value;
                    if (index < 0) index = array.Count + index; // Support negative indexing
                    if (index >= 0 && index < array.Count)
                        current = array[index];
                    else
                        return null;
                }
            }
            else if (segment.SelectAll)
            {
                // Keep as array for further processing
            }
            else if (segment.Filter != null)
            {
                if (current is JArray array)
                {
                    current = FilterArray(array, segment.Filter);
                }
            }

            return current;
        }

        private static JToken FilterArray(JArray array, FilterExpression filter)
        {
            var results = new JArray();

            foreach (var item in array)
            {
                if (item is JObject obj)
                {
                    var fieldValue = obj[filter.Field];
                    if (fieldValue != null && EvaluateCondition(fieldValue, filter.Operator, filter.Value))
                    {
                        results.Add(item);
                    }
                }
            }

            // Return single item if only one match, otherwise return array
            if (results.Count == 1)
                return results[0];
            else if (results.Count > 1)
                return results;
            else
                return null;
        }

        private static bool EvaluateCondition(JToken value, string op, object compareValue)
        {
            try
            {
                var valueObj = value.ToObject<object>();

                switch (op)
                {
                    case "=":
                    case "==":
                        return CompareValues(valueObj, compareValue) == 0;
                    case "!=":
                        return CompareValues(valueObj, compareValue) != 0;
                    case ">":
                        return CompareValues(valueObj, compareValue) > 0;
                    case "<":
                        return CompareValues(valueObj, compareValue) < 0;
                    case ">=":
                        return CompareValues(valueObj, compareValue) >= 0;
                    case "<=":
                        return CompareValues(valueObj, compareValue) <= 0;
                    default:
                        return false;
                }
            }
            catch
            {
                return false;
            }
        }

        private static int CompareValues(object v1, object v2)
        {
            // Convert to comparable types
            if (v1 == null && v2 == null) return 0;
            if (v1 == null) return -1;
            if (v2 == null) return 1;

            // Try numeric comparison
            if (IsNumeric(v1) && IsNumeric(v2))
            {
                var d1 = Convert.ToDouble(v1);
                var d2 = Convert.ToDouble(v2);
                return d1.CompareTo(d2);
            }

            // String comparison
            return string.Compare(v1.ToString(), v2.ToString(), StringComparison.OrdinalIgnoreCase);
        }

        private static bool IsNumeric(object value)
        {
            return value is int || value is long || value is double || value is float || value is decimal;
        }

        private static object ApplyAggregate(JToken result, string function)
        {
            var funcLower = function.ToLowerInvariant();

            // Extract property name if function has parentheses
            string propertyName = null;
            if (funcLower.Contains("(") && funcLower.Contains(")"))
            {
                var start = funcLower.IndexOf('(') + 1;
                var end = funcLower.IndexOf(')');
                propertyName = function.Substring(start, end - start).Trim();
                funcLower = funcLower.Substring(0, funcLower.IndexOf('('));
            }

            if (result is JArray array)
            {
                switch (funcLower)
                {
                    case "count":
                        return array.Count;

                    case "sum":
                        return array.Sum(item => GetNumericValue(item, propertyName));

                    case "avg":
                        if (array.Count == 0) return null;
                        return array.Average(item => GetNumericValue(item, propertyName));

                    case "min":
                        if (array.Count == 0) return null;
                        return array.Min(item => GetNumericValue(item, propertyName));

                    case "max":
                        if (array.Count == 0) return null;
                        return array.Max(item => GetNumericValue(item, propertyName));

                    case "first":
                        return array.FirstOrDefault();

                    case "last":
                        return array.LastOrDefault();

                    default:
                        return result;
                }
            }

            return result;
        }

        private static double GetNumericValue(JToken token, string propertyName = null)
        {
            if (!string.IsNullOrEmpty(propertyName) && token is JObject obj)
            {
                token = obj[propertyName];
            }

            if (token == null || token.Type == JTokenType.Null)
                return 0;

            return token.Value<double>();
        }

        private static object ConvertToSimpleType(JToken token)
        {
            if (token == null || token.Type == JTokenType.Null)
                return null;

            switch (token.Type)
            {
                case JTokenType.String:
                    return token.Value<string>();
                case JTokenType.Integer:
                    return token.Value<long>();
                case JTokenType.Float:
                    return token.Value<double>();
                case JTokenType.Boolean:
                    return token.Value<bool>();
                case JTokenType.Date:
                    return token.Value<DateTime>();
                case JTokenType.Array:
                    // For arrays, return first value or concatenated string
                    var array = (JArray)token;
                    if (array.Count == 0) return null;
                    if (array.Count == 1) return ConvertToSimpleType(array[0]);
                    // Return as comma-separated string for multiple values
                    return string.Join(",", array.Select(t => ConvertToSimpleType(t)?.ToString() ?? ""));
                case JTokenType.Object:
                    // Return as JSON string
                    return token.ToString(Newtonsoft.Json.Formatting.None);
                default:
                    return token.ToString();
            }
        }
    }
}