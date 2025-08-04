using System;

namespace TradeApi.Models
{
    public class TradeDeal
    {
        // Core identifiers
        public string DealId { get; set; }
        public string PlatformOrderId { get; set; }
        public string ExternalOrderId { get; set; }
        public string ParentOrderId { get; set; }
        
        // Dates
        public DateTime TradeDate { get; set; }
        public DateTime SettlementDate { get; set; }
        public DateTime ValueDate { get; set; }
        public DateTime? MaturityDate { get; set; }
        public DateTime CreatedDate { get; set; }
        public DateTime LastModifiedDate { get; set; }
        
        // Instrument details
        public string InstrumentId { get; set; }
        public string InstrumentName { get; set; }
        public string InstrumentType { get; set; }
        public string ISIN { get; set; }
        public string CUSIP { get; set; }
        public string Ticker { get; set; }
        public string Exchange { get; set; }
        
        // Trade details
        public decimal Quantity { get; set; }
        public decimal Price { get; set; }
        public decimal Notional { get; set; }
        public decimal SettlementAmount { get; set; }
        public decimal? AccruedInterest { get; set; }
        public decimal Commission { get; set; }
        public decimal Fees { get; set; }
        
        // Parties
        public string Counterparty { get; set; }
        public string CounterpartyId { get; set; }
        public string CounterpartyType { get; set; }
        public string CounterpartyCountry { get; set; }
        public string Trader { get; set; }
        public string TraderId { get; set; }
        public string Book { get; set; }
        public string Portfolio { get; set; }
        public string Strategy { get; set; }
        public string Desk { get; set; }
        
        // Status
        public string Status { get; set; }
        public string ConfirmationStatus { get; set; }
        public string SettlementStatus { get; set; }
        public string AllocationStatus { get; set; }
        
        // Risk metrics
        public decimal? PV01 { get; set; }
        public decimal? DV01 { get; set; }
        public decimal? Delta { get; set; }
        public decimal? Gamma { get; set; }
        public decimal? Vega { get; set; }
        public decimal? Duration { get; set; }
        public decimal? Yield { get; set; }
        
        // Additional fields
        public string Currency { get; set; }
        public string Side { get; set; } // Buy/Sell
        public string ProductType { get; set; }
        public string Venue { get; set; }
        public string ClearingHouse { get; set; }
        public string Prime { get; set; }
        public bool IsElectronic { get; set; }
        public string Comments { get; set; }
    }
}