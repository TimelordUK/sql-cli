using System.Collections.Generic;
using System.Threading.Tasks;

namespace TradeApi.Services.DataSources
{
    public interface IDataSource
    {
        string SourceName { get; }
        string[] SupportedTables { get; }
        Task<IQueryable<dynamic>> GetDataAsync(string tableName);
        Task<object> GetSchemaAsync(string tableName);
        bool SupportsTable(string tableName);
    }

    public class TableColumnInfo
    {
        public string Name { get; set; }
        public string Type { get; set; }
        public bool IsNullable { get; set; }
    }

    public class TableSchema
    {
        public string TableName { get; set; }
        public List<TableColumnInfo> Columns { get; set; }
    }
}