using System.Collections.Generic;

namespace JsonSelector.Models
{
    /// <summary>
    /// Request model for JSON querying
    /// </summary>
    public class QueryRequest
    {
        /// <summary>
        /// Field selectors - key is output column name, value is selector expression
        /// </summary>
        public Dictionary<string, string> Select { get; set; } = new();

        /// <summary>
        /// Optional WHERE conditions for filtering
        /// </summary>
        public Dictionary<string, object> Where { get; set; } = new();

        /// <summary>
        /// Output format: "json" or "csv"
        /// </summary>
        public string OutputFormat { get; set; } = "json";

        /// <summary>
        /// Optional limit on number of rows
        /// </summary>
        public int? Limit { get; set; }
    }

    /// <summary>
    /// Response model
    /// </summary>
    public class QueryResponse
    {
        public List<string> Columns { get; set; } = new();
        public List<List<object>> Rows { get; set; } = new();
        public int RowCount { get; set; }
        public string Format { get; set; } = "json";
    }
}