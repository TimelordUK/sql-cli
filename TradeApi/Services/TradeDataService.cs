using System;
using System.Collections.Generic;
using System.Linq;
using TradeApi.Models;

namespace TradeApi.Services
{
    public class TradeDataService
    {
        private readonly List<TradeDeal> _trades;
        private readonly Random _random = new Random(42); // Seeded for consistency
        
        public TradeDataService()
        {
            _trades = GenerateMockTrades(5000);
        }
        
        public IQueryable<TradeDeal> GetTrades()
        {
            return _trades.AsQueryable();
        }
        
        private List<TradeDeal> GenerateMockTrades(int count)
        {
            var trades = new List<TradeDeal>();
            var instruments = new[] { "AAPL", "MSFT", "GOOGL", "AMZN", "TSLA", "JPM", "GS", "BAC", "WMT", "JNJ" };
            var counterparties = new[] { "Goldman Sachs", "JP Morgan", "Morgan Stanley", "Citi", "Bank of America", "Barclays", "UBS", "Deutsche Bank" };
            var traders = new[] { "John Smith", "Jane Doe", "Bob Johnson", "Alice Williams", "Charlie Brown", "David Lee" };
            var books = new[] { "Equity Trading", "Fixed Income", "FX Trading", "Derivatives", "Prime Services" };
            var strategies = new[] { "Market Making", "Arbitrage", "Directional", "Hedging", "Flow Trading" };
            var statuses = new[] { "Executed", "Confirmed", "Settled", "Pending", "Cancelled" };
            var currencies = new[] { "USD", "EUR", "GBP", "JPY", "CHF" };
            var sides = new[] { "Buy", "Sell" };
            
            var baseDate = DateTime.Now.AddYears(-2);
            
            for (int i = 0; i < count; i++)
            {
                var tradeDate = baseDate.AddDays(_random.Next(0, 730));
                var instrument = instruments[_random.Next(instruments.Length)];
                
                trades.Add(new TradeDeal
                {
                    DealId = $"D{1000000 + i}",
                    PlatformOrderId = $"PO{2000000 + i}",
                    ExternalOrderId = $"EXT{3000000 + i}",
                    ParentOrderId = _random.Next(10) > 7 ? $"P{4000000 + _random.Next(1000)}" : null,
                    
                    TradeDate = tradeDate,
                    SettlementDate = tradeDate.AddDays(_random.Next(1, 3)),
                    ValueDate = tradeDate.AddDays(2),
                    MaturityDate = _random.Next(10) > 5 ? tradeDate.AddMonths(_random.Next(1, 24)) : (DateTime?)null,
                    CreatedDate = tradeDate.AddMinutes(-_random.Next(1, 60)),
                    LastModifiedDate = tradeDate.AddMinutes(_random.Next(1, 120)),
                    
                    InstrumentId = $"INS_{instrument}",
                    InstrumentName = GetInstrumentName(instrument),
                    InstrumentType = GetInstrumentType(instrument),
                    ISIN = $"US{_random.Next(100000000, 999999999)}",
                    CUSIP = $"{_random.Next(100000000, 999999999)}",
                    Ticker = instrument,
                    Exchange = GetExchange(instrument),
                    
                    Quantity = _random.Next(100, 10000),
                    Price = Math.Round((decimal)(_random.NextDouble() * 400 + 10), 2),
                    Notional = 0, // Will calculate below
                    Commission = Math.Round((decimal)(_random.NextDouble() * 100), 2),
                    Fees = Math.Round((decimal)(_random.NextDouble() * 50), 2),
                    AccruedInterest = _random.Next(10) > 7 ? Math.Round((decimal)(_random.NextDouble() * 1000), 2) : null,
                    
                    Counterparty = counterparties[_random.Next(counterparties.Length)],
                    CounterpartyId = $"CP{5000000 + _random.Next(1000)}",
                    CounterpartyType = _random.Next(10) > 5 ? "Institution" : "Broker",
                    CounterpartyCountry = GetCountry(),
                    Trader = traders[_random.Next(traders.Length)],
                    TraderId = $"T{6000 + _random.Next(100)}",
                    Book = books[_random.Next(books.Length)],
                    Portfolio = $"Portfolio_{_random.Next(1, 20)}",
                    Strategy = strategies[_random.Next(strategies.Length)],
                    Desk = $"Desk_{_random.Next(1, 10)}",
                    
                    Status = statuses[_random.Next(statuses.Length)],
                    ConfirmationStatus = _random.Next(10) > 2 ? "Confirmed" : "Pending",
                    SettlementStatus = _random.Next(10) > 3 ? "Settled" : "Unsettled",
                    AllocationStatus = _random.Next(10) > 5 ? "Allocated" : "Unallocated",
                    
                    PV01 = Math.Round((decimal)(_random.NextDouble() * 1000), 2),
                    DV01 = Math.Round((decimal)(_random.NextDouble() * 100), 2),
                    Delta = Math.Round((decimal)(_random.NextDouble() * 2 - 1), 4),
                    Gamma = Math.Round((decimal)(_random.NextDouble() * 0.1), 4),
                    Vega = Math.Round((decimal)(_random.NextDouble() * 50), 2),
                    Duration = Math.Round((decimal)(_random.NextDouble() * 10), 2),
                    Yield = Math.Round((decimal)(_random.NextDouble() * 5), 3),
                    
                    Currency = currencies[_random.Next(currencies.Length)],
                    Side = sides[_random.Next(sides.Length)],
                    ProductType = GetProductType(),
                    Venue = GetVenue(),
                    ClearingHouse = _random.Next(10) > 3 ? "DTCC" : "LCH",
                    Prime = _random.Next(10) > 5 ? counterparties[_random.Next(3)] : null,
                    IsElectronic = _random.Next(10) > 3,
                    Comments = _random.Next(10) > 7 ? "Special handling required" : null
                });
                
                // Calculate notional
                trades[i].Notional = trades[i].Quantity * trades[i].Price;
                trades[i].SettlementAmount = trades[i].Notional + trades[i].Commission + trades[i].Fees;
            }
            
            return trades;
        }
        
        private string GetInstrumentName(string ticker)
        {
            var names = new Dictionary<string, string>
            {
                ["AAPL"] = "Apple Inc.",
                ["MSFT"] = "Microsoft Corporation",
                ["GOOGL"] = "Alphabet Inc.",
                ["AMZN"] = "Amazon.com Inc.",
                ["TSLA"] = "Tesla Inc.",
                ["JPM"] = "JPMorgan Chase & Co.",
                ["GS"] = "Goldman Sachs Group Inc.",
                ["BAC"] = "Bank of America Corp.",
                ["WMT"] = "Walmart Inc.",
                ["JNJ"] = "Johnson & Johnson"
            };
            return names.TryGetValue(ticker, out var name) ? name : ticker;
        }
        
        private string GetInstrumentType(string ticker)
        {
            return new[] { "JPM", "GS", "BAC" }.Contains(ticker) ? "Financial" : "Equity";
        }
        
        private string GetExchange(string ticker)
        {
            return new[] { "NYSE", "NASDAQ", "AMEX" }[_random.Next(3)];
        }
        
        private string GetCountry()
        {
            var countries = new[] { "US", "UK", "JP", "DE", "FR", "CH", "CA", "AU" };
            return countries[_random.Next(countries.Length)];
        }
        
        private string GetProductType()
        {
            var types = new[] { "Cash Equity", "ETF", "Option", "Future", "Bond" };
            return types[_random.Next(types.Length)];
        }
        
        private string GetVenue()
        {
            var venues = new[] { "NYSE", "NASDAQ", "BATS", "IEX", "ARCA", "Direct" };
            return venues[_random.Next(venues.Length)];
        }
    }
}