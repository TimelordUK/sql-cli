using Microsoft.AspNetCore.Mvc.Testing;
using System.Text;
using System.Text.Json;
using Xunit;
using TradeApi.Models;

namespace TradeApi.Tests
{
    public class TradeControllerIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
    {
        private readonly WebApplicationFactory<Program> _factory;
        private readonly HttpClient _client;

        public TradeControllerIntegrationTests(WebApplicationFactory<Program> factory)
        {
            _factory = factory;
            _client = _factory.CreateClient();
        }

        [Fact]
        public async Task QueryTrades_BasicSelect_ReturnsData()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "DealId", "PlatformOrderId", "Price" },
                Take = 5
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            Assert.True(data.GetArrayLength() > 0);
            Assert.True(data.GetArrayLength() <= 5);
        }

        [Fact]
        public async Task QueryTrades_ContainsFilter_ReturnsFilteredResults()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "*" },
                Where = "PlatformOrderId.Contains(\"200000\")",
                Take = 10
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            
            // Verify all results contain "200000" in PlatformOrderId
            foreach (var item in data.EnumerateArray())
            {
                if (item.TryGetProperty("PlatformOrderId", out var platformOrderId))
                {
                    var value = platformOrderId.GetString();
                    Assert.Contains("200000", value);
                }
            }
        }

        [Fact]
        public async Task QueryTrades_PriceFilter_ReturnsCorrectResults()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "DealId", "Price" },
                Where = "Price > 200",
                Take = 10
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            
            // Verify all results have price > 200
            foreach (var item in data.EnumerateArray())
            {
                if (item.TryGetProperty("Price", out var priceElement))
                {
                    var price = priceElement.GetDecimal();
                    Assert.True(price > 200);
                }
            }
        }

        [Fact]
        public async Task QueryTrades_OrderBy_ReturnsOrderedResults()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "DealId", "Price" },
                OrderBy = "Price DESC",
                Take = 5
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            
            // Verify results are ordered by price descending
            decimal? lastPrice = null;
            foreach (var item in data.EnumerateArray())
            {
                if (item.TryGetProperty("Price", out var priceElement))
                {
                    var currentPrice = priceElement.GetDecimal();
                    if (lastPrice.HasValue)
                    {
                        Assert.True(currentPrice <= lastPrice.Value);
                    }
                    lastPrice = currentPrice;
                }
            }
        }

        [Fact]
        public async Task QueryTrades_ComplexQuery_ReturnsCorrectResults()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "DealId", "Ticker", "Price", "Quantity" },
                Where = "Ticker == \"AAPL\" AND Price > 100",
                OrderBy = "Price DESC",
                Take = 10
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            
            // Verify all results match criteria
            foreach (var item in data.EnumerateArray())
            {
                if (item.TryGetProperty("Ticker", out var tickerElement) &&
                    item.TryGetProperty("Price", out var priceElement))
                {
                    var ticker = tickerElement.GetString();
                    var price = priceElement.GetDecimal();
                    
                    Assert.Equal("AAPL", ticker);
                    Assert.True(price > 100);
                }
            }
        }

        [Fact]
        public async Task QueryTrades_StringMethods_ReturnsCorrectResults()
        {
            // Arrange - Test IndexOf, StartsWith, EndsWith
            var request = new QueryRequest
            {
                Select = new List<string> { "DealId", "PlatformOrderId", "Ticker" },
                Where = "PlatformOrderId.IndexOf(\"2000\") >= 0",
                Take = 5
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(result.TryGetProperty("data", out var data));
            
            // Verify all results contain "2000" in PlatformOrderId
            foreach (var item in data.EnumerateArray())
            {
                if (item.TryGetProperty("PlatformOrderId", out var platformOrderId))
                {
                    var value = platformOrderId.GetString();
                    Assert.Contains("2000", value);
                }
            }
        }

        [Fact]
        public async Task QueryTrades_InvalidQuery_ReturnsBadRequest()
        {
            // Arrange
            var request = new QueryRequest
            {
                Select = new List<string> { "*" },
                Where = "InvalidField.Contains(\"test\")", // Invalid field
                Take = 5
            };

            // Act
            var response = await PostQueryRequest(request);

            // Assert
            Assert.Equal(System.Net.HttpStatusCode.BadRequest, response.StatusCode);
        }

        [Fact]
        public async Task GetSchema_ReturnsValidSchema()
        {
            // Act
            var response = await _client.GetAsync("/api/trade/schema/trade_deal");

            // Assert
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();
            var schema = JsonSerializer.Deserialize<JsonElement>(content);
            
            Assert.True(schema.TryGetProperty("tableName", out var tableName));
            Assert.Equal("trade_deal", tableName.GetString());
            
            Assert.True(schema.TryGetProperty("columns", out var columns));
            Assert.True(columns.GetArrayLength() > 0);
        }

        private async Task<HttpResponseMessage> PostQueryRequest(QueryRequest request)
        {
            var json = JsonSerializer.Serialize(request, new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase
            });
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            
            return await _client.PostAsync("/api/trade/query", content);
        }
    }
}