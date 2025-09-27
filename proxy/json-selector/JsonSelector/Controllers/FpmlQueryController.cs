using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using JsonSelector.Models;
using JsonSelector.Services;

namespace JsonSelector.Controllers
{
    /// <summary>
    /// Controller for querying FIX messages with embedded FPML/XML data
    /// </summary>
    [ApiController]
    [Route("api/[controller]")]
    [Produces("text/csv")]
    public class FpmlQueryController : ControllerBase
    {
        private readonly EnhancedSelectorEvaluator _evaluator;

        public FpmlQueryController()
        {
            _evaluator = new EnhancedSelectorEvaluator();
        }

        /// <summary>
        /// Query FIX messages with FPML/XML fields
        /// </summary>
        /// <param name="file">JSON file containing FIX messages with FPML</param>
        /// <param name="query">Query with selectors including XML paths</param>
        /// <returns>CSV with flattened data from both JSON and XML</returns>
        /// <remarks>
        /// Example selectors for CDS/CDX with FPML:
        ///
        ///     {
        ///       "selectors": [
        ///         "header.MsgType",
        ///         "body.Symbol",
        ///         "body.Price",
        ///         "body.FpmlData@xml://trade/tradeHeader/tradeDate",
        ///         "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/effectiveDate/unadjustedDate",
        ///         "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/scheduledTerminationDate/unadjustedDate",
        ///         "body.FpmlData@xml://trade/product/creditDefaultSwap/protectionTerms/creditEvents/bankruptcy",
        ///         "body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/fixedRate"
        ///       ]
        ///     }
        ///
        /// </remarks>
        [HttpPost("upload")]
        [Consumes("multipart/form-data")]
        [ProducesResponseType(StatusCodes.Status200OK)]
        [ProducesResponseType(StatusCodes.Status400BadRequest)]
        public async Task<IActionResult> QueryWithFpml(
            IFormFile file,
            [FromForm] string query)
        {
            if (file == null || file.Length == 0)
                return BadRequest("No file uploaded");

            try
            {
                // Parse query
                var queryRequest = JsonConvert.DeserializeObject<QueryRequest>(query);
                if (queryRequest == null)
                    return BadRequest("Invalid query format");

                // Read file content
                string jsonContent;
                using (var reader = new StreamReader(file.OpenReadStream()))
                {
                    jsonContent = await reader.ReadToEndAsync();
                }

                // Parse JSON
                JToken root;
                jsonContent = jsonContent.Trim();

                if (jsonContent.StartsWith("["))
                {
                    root = JArray.Parse(jsonContent);
                }
                else if (jsonContent.StartsWith("{"))
                {
                    root = new JArray(JObject.Parse(jsonContent));
                }
                else
                {
                    return BadRequest("Invalid JSON format");
                }

                // Process with enhanced evaluator
                var response = ProcessFpmlQuery(root, queryRequest);

                // Convert to CSV
                var csv = ConvertToCsv(response);
                return Content(csv, "text/csv");
            }
            catch (Exception ex)
            {
                return StatusCode(500, new { error = ex.Message, type = ex.GetType().Name });
            }
        }

        /// <summary>
        /// Query with direct JSON input
        /// </summary>
        [HttpPost("direct")]
        [ProducesResponseType(StatusCodes.Status200OK)]
        [ProducesResponseType(StatusCodes.Status400BadRequest)]
        public IActionResult QueryDirect([FromBody] DirectFpmlQueryRequest request)
        {
            try
            {
                JToken root;
                if (request.Data is JArray)
                    root = (JArray)request.Data;
                else if (request.Data is JObject)
                    root = new JArray((JObject)request.Data);
                else
                    return BadRequest("Data must be JSON object or array");

                var response = ProcessFpmlQuery(root, request.Query);

                if (request.Query.OutputFormat?.ToLowerInvariant() == "csv")
                {
                    var csv = ConvertToCsv(response);
                    return Content(csv, "text/csv");
                }
                else
                {
                    return Ok(response);
                }
            }
            catch (Exception ex)
            {
                return StatusCode(500, new { error = ex.Message, type = ex.GetType().Name });
            }
        }

        /// <summary>
        /// Example query for Bloomberg CDS/CDX messages
        /// </summary>
        [HttpPost("example/cds")]
        [Consumes("multipart/form-data")]
        [ProducesResponseType(StatusCodes.Status200OK)]
        [ProducesResponseType(StatusCodes.Status400BadRequest)]
        public async Task<IActionResult> QueryCdsExample(IFormFile file)
        {
            // Example query for CDS with FPML
            var queryRequest = new QueryRequest
            {
                Select = new Dictionary<string, string>
                {
                    // FIX message fields
                    {"MsgType", "header.MsgType"},
                    {"SenderCompID", "header.SenderCompID"},
                    {"SendingTime", "header.SendingTime"},
                    {"Symbol", "body.Symbol"},
                    {"SecurityType", "body.SecurityType"},
                    {"Side", "body.Side"},
                    {"OrderQty", "body.OrderQty"},
                    {"Price", "body.Price"},

                    // FPML fields from embedded XML
                    {"TradeDate", "body.FpmlData@xml://trade/tradeHeader/tradeDate"},
                    {"TradeId", "body.FpmlData@xml://trade/tradeHeader/partyTradeIdentifier/tradeId"},
                    {"ReferenceEntity", "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/referenceInformation/referenceEntity/entityName"},
                    {"ReferenceObligation", "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/referenceInformation/referenceObligation/primaryObligorReference"},
                    {"EffectiveDate", "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/effectiveDate/unadjustedDate"},
                    {"TerminationDate", "body.FpmlData@xml://trade/product/creditDefaultSwap/generalTerms/scheduledTerminationDate/unadjustedDate"},
                    {"FixedRate", "body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/fixedRate"},
                    {"DayCountFraction", "body.FpmlData@xml://trade/product/creditDefaultSwap/feeLeg/periodicPayment/fixedAmountCalculation/dayCountFraction"},
                    {"Bankruptcy", "body.FpmlData@xml://trade/product/creditDefaultSwap/protectionTerms/creditEvents/bankruptcy"},
                    {"FailureToPay", "body.FpmlData@xml://trade/product/creditDefaultSwap/protectionTerms/creditEvents/failureToPay"},
                    {"RestructuringType", "body.FpmlData@xml://trade/product/creditDefaultSwap/protectionTerms/creditEvents/restructuring/restructuringType"}
                },
                OutputFormat = "csv"
            };

            // Use the upload endpoint logic
            string query = JsonConvert.SerializeObject(queryRequest);
            return await QueryWithFpml(file, query);
        }

        private QueryResponse ProcessFpmlQuery(JToken root, QueryRequest query)
        {
            var response = new QueryResponse
            {
                Columns = new List<string>(),
                Rows = new List<List<object>>()
            };

            var items = root as JArray;
            if (items == null || items.Count == 0)
                return response;

            // Get column names from select dictionary
            foreach (var kvp in query.Select)
            {
                response.Columns.Add(kvp.Key);
            }

            // Process each item
            foreach (var item in items)
            {
                var row = new List<object>();

                foreach (var kvp in query.Select)
                {
                    var value = _evaluator.Evaluate(item, kvp.Value);
                    row.Add(value);
                }

                response.Rows.Add(row);
            }

            return response;
        }

        private string GetColumnName(string selector)
        {
            // Handle XML selectors
            if (selector.Contains("@xml:"))
            {
                var parts = selector.Split("@xml:");
                var jsonPart = parts[0].Split('.').Last();
                var xmlPart = parts[1]
                    .Replace("//", "")
                    .Replace("/", "_")
                    .Replace(":", "_")
                    .Replace("[", "_")
                    .Replace("]", "")
                    .Replace("@", "");

                return $"{jsonPart}_{xmlPart}";
            }

            // Regular selector
            var segments = selector.Split('.');
            return segments[segments.Length - 1];
        }

        private string ConvertToCsv(QueryResponse response)
        {
            var sb = new StringBuilder();

            // Headers
            sb.AppendLine(string.Join(",", response.Columns.Select(c => $"\"{c}\"")));

            // Rows
            foreach (var row in response.Rows)
            {
                var values = row.Select(v =>
                {
                    if (v == null) return "";
                    var str = v.ToString();

                    // Handle lists/arrays
                    if (v is List<object> list)
                    {
                        str = string.Join("; ", list.Select(i => i?.ToString() ?? ""));
                    }

                    // Escape quotes and wrap in quotes if contains comma or quote
                    if (str.Contains(",") || str.Contains("\"") || str.Contains("\n"))
                    {
                        return $"\"{str.Replace("\"", "\"\"")}\"";
                    }
                    return str;
                });
                sb.AppendLine(string.Join(",", values));
            }

            return sb.ToString();
        }
    }

    /// <summary>
    /// Request model for direct FPML querying
    /// </summary>
    public class DirectFpmlQueryRequest
    {
        /// <summary>
        /// JSON data containing FIX messages with FPML
        /// </summary>
        public JToken Data { get; set; }

        /// <summary>
        /// Query parameters including XML selectors
        /// </summary>
        public QueryRequest Query { get; set; }
    }
}