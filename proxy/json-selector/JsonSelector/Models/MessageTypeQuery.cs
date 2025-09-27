using System.Collections.Generic;

namespace JsonSelector.Models
{
    /// <summary>
    /// Enhanced query request that supports different selectors per message type
    /// </summary>
    public class MessageTypeQueryRequest
    {
        /// <summary>
        /// Path to the field that identifies message type (e.g., "header.MsgType")
        /// </summary>
        public string MessageTypeField { get; set; } = "header.MsgType";

        /// <summary>
        /// Output columns - defines the unified schema
        /// </summary>
        public List<string> OutputColumns { get; set; } = new();

        /// <summary>
        /// Message type specific selectors
        /// Key = message type value (e.g., "8" for execution, "J" for allocation)
        /// Value = selector mappings for that message type
        /// </summary>
        public Dictionary<string, MessageTypeSelectors> MessageTypes { get; set; } = new();

        /// <summary>
        /// Optional global WHERE conditions applied to all message types
        /// </summary>
        public Dictionary<string, object> GlobalWhere { get; set; } = new();

        /// <summary>
        /// Output format: "json" or "csv"
        /// </summary>
        public string OutputFormat { get; set; } = "json";

        /// <summary>
        /// Optional limit on number of rows per message type
        /// </summary>
        public int? Limit { get; set; }
    }

    /// <summary>
    /// Selectors for a specific message type
    /// </summary>
    public class MessageTypeSelectors
    {
        /// <summary>
        /// Selector expressions for each output column
        /// Key = output column name
        /// Value = selector expression for this message type
        /// </summary>
        public Dictionary<string, string> Select { get; set; } = new();

        /// <summary>
        /// Optional WHERE conditions specific to this message type
        /// </summary>
        public Dictionary<string, object> Where { get; set; } = new();

        /// <summary>
        /// Description of this message type (optional)
        /// </summary>
        public string Description { get; set; }
    }

    /// <summary>
    /// Example configuration for FIX messages
    /// </summary>
    public static class FixQueryExamples
    {
        public static MessageTypeQueryRequest CreateExecutionAndAllocationQuery()
        {
            return new MessageTypeQueryRequest
            {
                MessageTypeField = "header.MsgType",
                OutputColumns = new List<string>
                {
                    "msg_type",
                    "order_id",
                    "symbol",
                    "side",
                    "quantity",
                    "price",
                    "trader",
                    "account",
                    "fund_allocations",
                    "total_allocated"
                },
                MessageTypes = new Dictionary<string, MessageTypeSelectors>
                {
                    ["8"] = new MessageTypeSelectors // Execution Report
                    {
                        Description = "Execution Report",
                        Select = new Dictionary<string, string>
                        {
                            ["msg_type"] = "header.MsgType",
                            ["order_id"] = "body.ClOrdID",
                            ["symbol"] = "body.Symbol",
                            ["side"] = "body.Side",
                            ["quantity"] = "body.LastQty",
                            ["price"] = "body.LastPx",
                            ["trader"] = "Parties[PartyRole=11].PartyID",
                            ["account"] = "Parties[PartyRole=1].PartyID",
                            ["fund_allocations"] = "AllocGrp:count()",
                            ["total_allocated"] = "AllocGrp:sum(AllocQty)"
                        },
                        Where = new Dictionary<string, object>
                        {
                            ["body.ExecType"] = "F" // Trades only
                        }
                    },
                    ["J"] = new MessageTypeSelectors // Allocation Instruction
                    {
                        Description = "Allocation Instruction",
                        Select = new Dictionary<string, string>
                        {
                            ["msg_type"] = "header.MsgType",
                            ["order_id"] = "OrdAllocGrp[0].ClOrdID", // First order in allocation
                            ["symbol"] = "body.Symbol",
                            ["side"] = "body.Side",
                            ["quantity"] = "body.Quantity",
                            ["price"] = "body.AvgPx",
                            ["trader"] = "null", // Not present in allocation messages
                            ["account"] = "AllocGrp[0].AllocAccount", // First allocation account
                            ["fund_allocations"] = "AllocGrp:count()",
                            ["total_allocated"] = "AllocGrp:sum(AllocQty)"
                        }
                    }
                }
            };
        }
    }
}