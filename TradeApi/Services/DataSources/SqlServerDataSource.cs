using Microsoft.Data.SqlClient;
using System.Data;
using System.Dynamic;
using System.Linq.Dynamic.Core;
using Newtonsoft.Json;

namespace TradeApi.Services.DataSources
{
    public class SqlServerDataSource : IDataSource
    {
        private readonly string _connectionString;
        private readonly Dictionary<string, string> _tableMapping;
        private readonly ILogger<SqlServerDataSource> _logger;

        public string SourceName => "SqlServer";
        public string[] SupportedTables { get; }

        public SqlServerDataSource(IConfiguration configuration, ILogger<SqlServerDataSource> logger)
        {
            _logger = logger;
            _connectionString = configuration.GetConnectionString("SqlServer");
            
            // Map friendly names to actual database tables
            _tableMapping = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
            {
                ["client_mappings"] = "dbo.ClientMappings",
                ["accounts"] = "dbo.Accounts",
                ["reference_data"] = "dbo.ReferenceData",
                ["positions"] = "dbo.Positions",
                ["orders"] = "dbo.Orders"
            };
            
            SupportedTables = _tableMapping.Keys.ToArray();
        }

        public bool SupportsTable(string tableName)
        {
            return _tableMapping.ContainsKey(tableName);
        }

        public async Task<IQueryable<dynamic>> GetDataAsync(string tableName)
        {
            if (!_tableMapping.TryGetValue(tableName, out var actualTable))
            {
                throw new ArgumentException($"Table {tableName} not supported by SQL Server source");
            }

            var data = new List<dynamic>();
            
            using (var connection = new SqlConnection(_connectionString))
            {
                await connection.OpenAsync();
                
                // Use parameterized query to prevent SQL injection
                var query = $"SELECT * FROM {actualTable}";
                using (var command = new SqlCommand(query, connection))
                {
                    using (var reader = await command.ExecuteReaderAsync())
                    {
                        var schemaTable = reader.GetSchemaTable();
                        
                        while (await reader.ReadAsync())
                        {
                            dynamic expando = new ExpandoObject();
                            var expandoDict = (IDictionary<string, object>)expando;
                            
                            for (int i = 0; i < reader.FieldCount; i++)
                            {
                                var name = reader.GetName(i);
                                var value = reader.IsDBNull(i) ? null : reader.GetValue(i);
                                expandoDict[name] = value;
                            }
                            
                            data.Add(expando);
                        }
                    }
                }
            }
            
            _logger.LogInformation("Loaded {Count} records from {Table}", data.Count, actualTable);
            return data.AsQueryable();
        }

        public async Task<object> GetSchemaAsync(string tableName)
        {
            if (!_tableMapping.TryGetValue(tableName, out var actualTable))
            {
                throw new ArgumentException($"Table {tableName} not supported by SQL Server source");
            }

            var columns = new List<TableColumnInfo>();
            
            using (var connection = new SqlConnection(_connectionString))
            {
                await connection.OpenAsync();
                
                var query = @"
                    SELECT 
                        COLUMN_NAME as Name,
                        DATA_TYPE as Type,
                        IS_NULLABLE as IsNullable
                    FROM INFORMATION_SCHEMA.COLUMNS
                    WHERE TABLE_SCHEMA + '.' + TABLE_NAME = @tableName
                    ORDER BY ORDINAL_POSITION";
                
                using (var command = new SqlCommand(query, connection))
                {
                    command.Parameters.AddWithValue("@tableName", actualTable);
                    
                    using (var reader = await command.ExecuteReaderAsync())
                    {
                        while (await reader.ReadAsync())
                        {
                            columns.Add(new TableColumnInfo
                            {
                                Name = reader.GetString(0),
                                Type = reader.GetString(1),
                                IsNullable = reader.GetString(2) == "YES"
                            });
                        }
                    }
                }
            }
            
            return new TableSchema
            {
                TableName = tableName,
                Columns = columns
            };
        }
    }
}