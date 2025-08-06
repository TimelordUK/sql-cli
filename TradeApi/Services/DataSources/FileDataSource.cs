using CsvHelper;
using System.Dynamic;
using System.Globalization;
using Newtonsoft.Json;

namespace TradeApi.Services.DataSources
{
    public class FileDataSource : IDataSource
    {
        private readonly string _dataDirectory;
        private readonly ILogger<FileDataSource> _logger;
        private readonly Dictionary<string, string> _fileMapping;

        public string SourceName => "Files";
        public string[] SupportedTables { get; }

        public FileDataSource(IConfiguration configuration, ILogger<FileDataSource> logger)
        {
            _logger = logger;
            _dataDirectory = configuration["DataSources:FileDirectory"] ?? Path.Combine(Directory.GetCurrentDirectory(), "..", "data");
            
            // Map table names to file paths
            _fileMapping = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
            {
                ["customers"] = "customers.csv",
                ["business_crime"] = "BusinessCrimeBoroughLevel.csv",
                ["sample_trades"] = "sample_trades.json"
            };
            
            SupportedTables = _fileMapping.Keys.ToArray();
        }

        public bool SupportsTable(string tableName)
        {
            return _fileMapping.ContainsKey(tableName);
        }

        public async Task<IQueryable<dynamic>> GetDataAsync(string tableName)
        {
            if (!_fileMapping.TryGetValue(tableName, out var fileName))
            {
                throw new ArgumentException($"Table {tableName} not supported by File source");
            }

            var filePath = Path.Combine(_dataDirectory, fileName);
            
            if (!File.Exists(filePath))
            {
                _logger.LogWarning("File not found: {FilePath}", filePath);
                return new List<dynamic>().AsQueryable();
            }

            var extension = Path.GetExtension(fileName).ToLower();
            var data = new List<dynamic>();

            if (extension == ".csv")
            {
                using var reader = new StreamReader(filePath);
                using var csv = new CsvReader(reader, CultureInfo.InvariantCulture);
                
                csv.Read();
                csv.ReadHeader();
                var headers = csv.HeaderRecord;

                while (csv.Read())
                {
                    dynamic expando = new ExpandoObject();
                    var expandoDict = (IDictionary<string, object>)expando;
                    
                    foreach (var header in headers)
                    {
                        var value = csv.GetField(header);
                        
                        // Try to parse as number
                        if (decimal.TryParse(value, out var decimalValue))
                        {
                            expandoDict[header] = decimalValue;
                        }
                        else if (bool.TryParse(value, out var boolValue))
                        {
                            expandoDict[header] = boolValue;
                        }
                        else
                        {
                            expandoDict[header] = value;
                        }
                    }
                    
                    data.Add(expando);
                }
            }
            else if (extension == ".json")
            {
                var json = await File.ReadAllTextAsync(filePath);
                var items = JsonConvert.DeserializeObject<List<dynamic>>(json);
                data.AddRange(items);
            }
            
            _logger.LogInformation("Loaded {Count} records from file {FileName}", data.Count, fileName);
            return data.AsQueryable();
        }

        public async Task<object> GetSchemaAsync(string tableName)
        {
            if (!_fileMapping.TryGetValue(tableName, out var fileName))
            {
                throw new ArgumentException($"Table {tableName} not supported by File source");
            }

            var filePath = Path.Combine(_dataDirectory, fileName);
            var columns = new List<TableColumnInfo>();

            if (!File.Exists(filePath))
            {
                return new TableSchema
                {
                    TableName = tableName,
                    Columns = columns
                };
            }

            var extension = Path.GetExtension(fileName).ToLower();

            if (extension == ".csv")
            {
                using var reader = new StreamReader(filePath);
                using var csv = new CsvReader(reader, CultureInfo.InvariantCulture);
                
                csv.Read();
                csv.ReadHeader();
                
                foreach (var header in csv.HeaderRecord)
                {
                    columns.Add(new TableColumnInfo
                    {
                        Name = header,
                        Type = "string", // Default type, could be improved with sampling
                        IsNullable = true
                    });
                }
            }
            else if (extension == ".json")
            {
                var json = await File.ReadAllTextAsync(filePath);
                var items = JsonConvert.DeserializeObject<List<dynamic>>(json);
                
                if (items?.Count > 0)
                {
                    var firstItem = items[0];
                    var expandoDict = (IDictionary<string, object>)firstItem;
                    
                    foreach (var kvp in expandoDict)
                    {
                        columns.Add(new TableColumnInfo
                        {
                            Name = kvp.Key,
                            Type = kvp.Value?.GetType().Name ?? "string",
                            IsNullable = true
                        });
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