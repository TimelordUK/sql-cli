using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;

namespace JsonSelector.Services
{
    /// <summary>
    /// Parses selector expressions like "Parties[PartyRole=11].PartyID"
    /// </summary>
    public class SelectorParser
    {
        private static readonly Regex PathSegmentRegex = new Regex(
            @"^([a-zA-Z_][a-zA-Z0-9_]*)" +               // Property name
            @"(?:\[([^\]]+)\])?" +                       // Optional filter in brackets
            @"(?:\.(.+))?$",                              // Optional remaining path
            RegexOptions.Compiled
        );

        private static readonly Regex FilterRegex = new Regex(
            @"^([a-zA-Z_][a-zA-Z0-9_]*)\s*(=|!=|>|<|>=|<=)\s*(.+)$",
            RegexOptions.Compiled
        );

        public static SelectorExpression Parse(string selector)
        {
            if (string.IsNullOrWhiteSpace(selector))
                throw new ArgumentException("Selector cannot be empty");

            // Check for aggregation suffix
            string aggregateFunc = null;
            var colonIndex = selector.LastIndexOf(':');
            if (colonIndex > 0 && colonIndex < selector.Length - 1)
            {
                var potentialFunc = selector.Substring(colonIndex + 1);
                if (IsAggregateFunction(potentialFunc))
                {
                    aggregateFunc = potentialFunc;
                    selector = selector.Substring(0, colonIndex);
                }
            }

            var segments = ParsePath(selector);

            return new SelectorExpression
            {
                Segments = segments,
                AggregateFunction = aggregateFunc
            };
        }

        private static List<PathSegment> ParsePath(string path)
        {
            var segments = new List<PathSegment>();
            var remaining = path;

            while (!string.IsNullOrEmpty(remaining))
            {
                var segment = ParseNextSegment(ref remaining);
                if (segment != null)
                    segments.Add(segment);
            }

            return segments;
        }

        private static PathSegment ParseNextSegment(ref string remaining)
        {
            // Handle dot at the beginning (continuation from previous segment)
            if (remaining.StartsWith("."))
                remaining = remaining.Substring(1);

            // Find the next segment boundary
            var dotIndex = FindNextDotOutsideBrackets(remaining);
            var currentPart = dotIndex >= 0 ? remaining.Substring(0, dotIndex) : remaining;
            remaining = dotIndex >= 0 ? remaining.Substring(dotIndex) : "";

            // Parse property and optional filter
            var bracketStart = currentPart.IndexOf('[');
            if (bracketStart < 0)
            {
                // Simple property
                return new PathSegment { Property = currentPart };
            }

            var bracketEnd = currentPart.IndexOf(']', bracketStart);
            if (bracketEnd < 0)
                throw new ArgumentException($"Unclosed bracket in selector: {currentPart}");

            var property = currentPart.Substring(0, bracketStart);
            var filterStr = currentPart.Substring(bracketStart + 1, bracketEnd - bracketStart - 1);

            var segment = new PathSegment { Property = property };

            // Parse filter
            if (int.TryParse(filterStr, out var index))
            {
                segment.ArrayIndex = index;
            }
            else if (filterStr == "*")
            {
                segment.SelectAll = true;
            }
            else
            {
                segment.Filter = ParseFilter(filterStr);
            }

            return segment;
        }

        private static int FindNextDotOutsideBrackets(string str)
        {
            int bracketDepth = 0;
            for (int i = 0; i < str.Length; i++)
            {
                if (str[i] == '[') bracketDepth++;
                else if (str[i] == ']') bracketDepth--;
                else if (str[i] == '.' && bracketDepth == 0)
                    return i;
            }
            return -1;
        }

        private static FilterExpression ParseFilter(string filter)
        {
            var match = FilterRegex.Match(filter.Trim());
            if (!match.Success)
                throw new ArgumentException($"Invalid filter expression: {filter}");

            var field = match.Groups[1].Value;
            var op = match.Groups[2].Value;
            var valueStr = match.Groups[3].Value.Trim();

            // Parse value - remove quotes if present
            object value;
            if (valueStr.StartsWith("\"") && valueStr.EndsWith("\""))
            {
                value = valueStr.Substring(1, valueStr.Length - 2);
            }
            else if (int.TryParse(valueStr, out var intVal))
            {
                value = intVal;
            }
            else if (double.TryParse(valueStr, out var doubleVal))
            {
                value = doubleVal;
            }
            else if (bool.TryParse(valueStr, out var boolVal))
            {
                value = boolVal;
            }
            else
            {
                value = valueStr;
            }

            return new FilterExpression
            {
                Field = field,
                Operator = op,
                Value = value
            };
        }

        private static bool IsAggregateFunction(string func)
        {
            var lower = func.ToLowerInvariant();
            return lower == "sum" || lower == "count" || lower == "avg" ||
                   lower == "min" || lower == "max" || lower == "first" ||
                   lower == "last" || lower.StartsWith("sum(") ||
                   lower.StartsWith("count(") || lower.StartsWith("avg(");
        }
    }

    public class SelectorExpression
    {
        public List<PathSegment> Segments { get; set; } = new();
        public string AggregateFunction { get; set; }
    }

    public class PathSegment
    {
        public string Property { get; set; }
        public int? ArrayIndex { get; set; }
        public bool SelectAll { get; set; }
        public FilterExpression Filter { get; set; }
    }

    public class FilterExpression
    {
        public string Field { get; set; }
        public string Operator { get; set; }
        public object Value { get; set; }
    }
}