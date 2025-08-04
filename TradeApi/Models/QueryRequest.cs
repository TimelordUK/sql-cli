using System.Collections.Generic;

namespace TradeApi.Models
{
    public class QueryRequest
    {
        public List<string>? Select { get; set; }
        public string? Where { get; set; }
        public string? OrderBy { get; set; }
        public int? Skip { get; set; }
        public int? Take { get; set; }
    }
    
    public class SchemaResponse
    {
        public string TableName { get; set; } = string.Empty;
        public List<ColumnInfo> Columns { get; set; } = new();
    }
    
    public class ColumnInfo
    {
        public string Name { get; set; } = string.Empty;
        public string Type { get; set; } = string.Empty;
        public bool IsNullable { get; set; }
    }
}