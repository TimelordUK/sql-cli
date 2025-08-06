using System.Linq.Dynamic.Core;
using TradeApi.Services.DataSources;

namespace TradeApi.Services
{
    public class DataSourceRouter
    {
        private readonly List<IDataSource> _dataSources;
        private readonly ILogger<DataSourceRouter> _logger;
        private readonly Dictionary<string, IDataSource> _tableToSourceMapping;

        public DataSourceRouter(IEnumerable<IDataSource> dataSources, ILogger<DataSourceRouter> logger)
        {
            _dataSources = dataSources.ToList();
            _logger = logger;
            _tableToSourceMapping = new Dictionary<string, IDataSource>(StringComparer.OrdinalIgnoreCase);
            
            // Build table to source mapping
            foreach (var source in _dataSources)
            {
                foreach (var table in source.SupportedTables)
                {
                    if (!_tableToSourceMapping.ContainsKey(table))
                    {
                        _tableToSourceMapping[table] = source;
                        _logger.LogInformation("Mapped table {Table} to source {Source}", table, source.SourceName);
                    }
                }
            }
        }

        public async Task<IQueryable<dynamic>> GetDataForTableAsync(string tableName)
        {
            if (_tableToSourceMapping.TryGetValue(tableName, out var dataSource))
            {
                _logger.LogInformation("Routing table {Table} to {Source}", tableName, dataSource.SourceName);
                return await dataSource.GetDataAsync(tableName);
            }

            // Check if it's the special trade_deal table (original functionality)
            if (tableName.Equals("trade_deal", StringComparison.OrdinalIgnoreCase))
            {
                _logger.LogInformation("Using legacy trade_deal service");
                return null; // Indicate to use legacy service
            }

            throw new ArgumentException($"Table '{tableName}' not found in any data source");
        }

        public async Task<object> GetSchemaForTableAsync(string tableName)
        {
            if (_tableToSourceMapping.TryGetValue(tableName, out var dataSource))
            {
                return await dataSource.GetSchemaAsync(tableName);
            }

            throw new ArgumentException($"Table '{tableName}' not found in any data source");
        }

        public Dictionary<string, List<string>> GetAllAvailableTables()
        {
            var result = new Dictionary<string, List<string>>();
            
            foreach (var source in _dataSources)
            {
                result[source.SourceName] = source.SupportedTables.ToList();
            }
            
            // Add legacy trade_deal
            if (!result.ContainsKey("TradeApi"))
            {
                result["TradeApi"] = new List<string>();
            }
            result["TradeApi"].Add("trade_deal");
            
            return result;
        }

        public string GetSourceForTable(string tableName)
        {
            if (_tableToSourceMapping.TryGetValue(tableName, out var dataSource))
            {
                return dataSource.SourceName;
            }
            
            if (tableName.Equals("trade_deal", StringComparison.OrdinalIgnoreCase))
            {
                return "TradeApi";
            }
            
            return null;
        }
    }
}