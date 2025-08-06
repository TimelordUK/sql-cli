using System.Dynamic;
using System.Text.Json;
using Newtonsoft.Json;

namespace TradeApi.Services.DataSources
{
    public class PublicApiDataSource : IDataSource
    {
        private readonly HttpClient _httpClient;
        private readonly ILogger<PublicApiDataSource> _logger;
        private readonly Dictionary<string, ApiEndpoint> _endpoints;

        public string SourceName => "PublicAPI";
        public string[] SupportedTables { get; }

        public PublicApiDataSource(IHttpClientFactory httpClientFactory, ILogger<PublicApiDataSource> logger)
        {
            _logger = logger;
            _httpClient = httpClientFactory.CreateClient();
            
            // Configure public API endpoints that return JSON arrays
            _endpoints = new Dictionary<string, ApiEndpoint>(StringComparer.OrdinalIgnoreCase)
            {
                ["countries"] = new ApiEndpoint 
                { 
                    Url = "https://restcountries.com/v3.1/all",
                    ResponsePath = null // Direct array response
                },
                ["users"] = new ApiEndpoint
                {
                    Url = "https://jsonplaceholder.typicode.com/users",
                    ResponsePath = null
                },
                ["posts"] = new ApiEndpoint
                {
                    Url = "https://jsonplaceholder.typicode.com/posts",
                    ResponsePath = null
                },
                ["crypto_prices"] = new ApiEndpoint
                {
                    Url = "https://api.coinbase.com/v2/exchange-rates?currency=USD",
                    ResponsePath = "data.rates" // Nested object, needs transformation
                }
            };
            
            SupportedTables = _endpoints.Keys.ToArray();
        }

        public bool SupportsTable(string tableName)
        {
            return _endpoints.ContainsKey(tableName);
        }

        public async Task<IQueryable<dynamic>> GetDataAsync(string tableName)
        {
            if (!_endpoints.TryGetValue(tableName, out var endpoint))
            {
                throw new ArgumentException($"Table {tableName} not supported by Public API source");
            }

            try
            {
                _logger.LogInformation("Fetching data from {Url} for table {Table}", endpoint.Url, tableName);
                
                var response = await _httpClient.GetStringAsync(endpoint.Url);
                var data = new List<dynamic>();

                // Special handling for different API response formats
                if (tableName == "crypto_prices")
                {
                    // Transform key-value pairs into records
                    var json = JsonDocument.Parse(response);
                    if (json.RootElement.TryGetProperty("data", out var dataElement) &&
                        dataElement.TryGetProperty("rates", out var ratesElement))
                    {
                        foreach (var rate in ratesElement.EnumerateObject())
                        {
                            dynamic expando = new ExpandoObject();
                            var expandoDict = (IDictionary<string, object>)expando;
                            expandoDict["Currency"] = rate.Name;
                            expandoDict["Rate"] = decimal.Parse(rate.Value.GetString() ?? "0");
                            expandoDict["BaseCurrency"] = "USD";
                            expandoDict["Timestamp"] = DateTime.UtcNow;
                            data.Add(expando);
                        }
                    }
                }
                else if (tableName == "countries")
                {
                    // Flatten complex country data
                    var countries = JsonConvert.DeserializeObject<List<dynamic>>(response);
                    foreach (var country in countries)
                    {
                        dynamic expando = new ExpandoObject();
                        var expandoDict = (IDictionary<string, object>)expando;
                        
                        // Extract key fields from complex structure
                        expandoDict["Name"] = country.name?.common?.ToString() ?? "";
                        expandoDict["OfficialName"] = country.name?.official?.ToString() ?? "";
                        expandoDict["Region"] = country.region?.ToString() ?? "";
                        expandoDict["SubRegion"] = country.subregion?.ToString() ?? "";
                        expandoDict["Population"] = country.population ?? 0;
                        expandoDict["Area"] = country.area ?? 0;
                        expandoDict["Capital"] = country.capital?[0]?.ToString() ?? "";
                        expandoDict["CCA2"] = country.cca2?.ToString() ?? "";
                        expandoDict["CCA3"] = country.cca3?.ToString() ?? "";
                        
                        data.Add(expando);
                    }
                }
                else
                {
                    // Direct JSON array parsing
                    var items = JsonConvert.DeserializeObject<List<dynamic>>(response);
                    data.AddRange(items);
                }
                
                _logger.LogInformation("Loaded {Count} records from {Table}", data.Count, tableName);
                return data.AsQueryable();
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Failed to fetch data from {Table}", tableName);
                throw;
            }
        }

        public async Task<object> GetSchemaAsync(string tableName)
        {
            // For public APIs, we can infer schema from a sample
            var data = await GetDataAsync(tableName);
            var firstRecord = data.FirstOrDefault();
            
            if (firstRecord == null)
            {
                return new TableSchema
                {
                    TableName = tableName,
                    Columns = new List<TableColumnInfo>()
                };
            }

            var columns = new List<TableColumnInfo>();
            var expandoDict = (IDictionary<string, object>)firstRecord;
            
            foreach (var kvp in expandoDict)
            {
                columns.Add(new TableColumnInfo
                {
                    Name = kvp.Key,
                    Type = kvp.Value?.GetType().Name ?? "string",
                    IsNullable = true
                });
            }
            
            return new TableSchema
            {
                TableName = tableName,
                Columns = columns
            };
        }

        private class ApiEndpoint
        {
            public string Url { get; set; }
            public string ResponsePath { get; set; }
        }
    }
}