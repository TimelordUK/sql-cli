using Xunit;
using TradeApi.Services;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace TradeApi.Tests
{
    public class QueryProcessorTests
    {
        private readonly QueryProcessor _processor;

        public QueryProcessorTests()
        {
            _processor = new QueryProcessor(NullLogger<QueryProcessor>.Instance);
        }

        [Theory]
        [InlineData("platformOrderId.Contains(\"E\")", "PlatformOrderId.Contains(\"E\")")]
        [InlineData("price > 100", "Price > 100")]
        [InlineData("ticker = \"AAPL\"", "Ticker == \"AAPL\"")]
        [InlineData("dealid.Contains(\"123\")", "DealId.Contains(\"123\")")]
        [InlineData("platformOrderId.IndexOf(\"abc\") > 10", "PlatformOrderId.IndexOf(\"abc\") > 10")]
        [InlineData("ticker.StartsWith(\"AA\")", "Ticker.StartsWith(\"AA\")")]
        [InlineData("ticker.EndsWith(\"PL\")", "Ticker.EndsWith(\"PL\")")]
        public void ProcessWhereClause_FixesPropertyCasing(string input, string expected)
        {
            // Act
            var result = _processor.ProcessWhereClause(input);

            // Assert
            Assert.Equal(expected, result);
        }

        [Theory]
        [InlineData("ticker in (\"AAPL\", \"MSFT\")", "new[] {\"AAPL\", \"MSFT\"}.Contains(Ticker)")]
        [InlineData("status in (\"Executed\", \"Confirmed\")", "new[] {\"Executed\", \"Confirmed\"}.Contains(Status)")]
        public void ProcessWhereClause_HandlesInClauses(string input, string expected)
        {
            // Act
            var result = _processor.ProcessWhereClause(input);

            // Assert
            Assert.Equal(expected, result);
        }

        [Theory]
        [InlineData("price > 100 AND ticker = \"AAPL\"", "Price > 100 AND Ticker == \"AAPL\"")]
        [InlineData("quantity < 1000 OR notional > 50000", "Quantity < 1000 OR Notional > 50000")]
        public void ProcessWhereClause_HandlesComplexExpressions(string input, string expected)
        {
            // Act
            var result = _processor.ProcessWhereClause(input);

            // Assert
            Assert.Equal(expected, result);
        }

        [Fact]
        public void ProcessWhereClause_HandlesNullOrEmpty()
        {
            // Act & Assert
            Assert.Equal("", _processor.ProcessWhereClause(""));
            Assert.Equal(null, _processor.ProcessWhereClause(null));
        }

        [Theory]
        [InlineData("price > abc", false)] // Invalid field reference
        [InlineData("invalidfield.Contains(\"test\")", false)] // Non-existent property
        [InlineData("price > 100", true)] // Valid expression
        public void ValidateQuery_ReturnsCorrectValidation(string input, bool shouldBeValid)
        {
            // Act
            var errors = _processor.ValidateQuery(input);

            // Assert
            if (shouldBeValid)
            {
                Assert.Empty(errors);
            }
            else
            {
                Assert.NotEmpty(errors);
            }
        }
    }
}