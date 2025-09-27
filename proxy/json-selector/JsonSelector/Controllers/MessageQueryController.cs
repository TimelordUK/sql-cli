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
    [ApiController]
    [Route("api/[controller]")]
    public class MessageQueryController : ControllerBase
    {
        /// <summary>
        /// Query messages with type-specific selectors via file upload
        /// </summary>
        /// <param name="file">JSON file containing messages</param>
        /// <param name="query">JSON string with query parameters</param>
        /// <returns>CSV formatted results</returns>
        [HttpPost("upload")]
        [Consumes("multipart/form-data")]
        [ProducesResponseType(StatusCodes.Status200OK)]
        [ProducesResponseType(StatusCodes.Status400BadRequest)]
        public async Task<IActionResult> QueryMessagesWithUpload(
            IFormFile file,
            [FromForm] string query)
        {
            if (file == null || file.Length == 0)
                return BadRequest("No file uploaded");

            try
            {
                // Parse query from form data
                var queryRequest = JsonConvert.DeserializeObject<MessageTypeQueryRequest>(query);
                if (queryRequest == null)
                    return BadRequest("Invalid query format");

                // Read file content
                string jsonContent;
                using (var reader = new StreamReader(file.OpenReadStream()))
                {
                    jsonContent = await reader.ReadToEndAsync();
                }

                // Parse messages
                var messages = ParseMessages(jsonContent);

                // Process query
                var response = ProcessMessageTypeQuery(messages, queryRequest);

                // Return appropriate format
                if (queryRequest.OutputFormat?.ToLowerInvariant() == "csv")
                {
                    var csv = ConvertToCsv(response);
                    return File(Encoding.UTF8.GetBytes(csv), "text/csv", "messages.csv");
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
        /// Query with example FIX configuration
        /// </summary>
        /// <param name="file">JSON file containing FIX messages</param>
        /// <returns>CSV formatted results</returns>
        [HttpPost("example/fix")]
        [Consumes("multipart/form-data")]
        [ProducesResponseType(StatusCodes.Status200OK)]
        [ProducesResponseType(StatusCodes.Status400BadRequest)]
        public async Task<IActionResult> QueryFixExample(IFormFile file)
        {
            if (file == null || file.Length == 0)
                return BadRequest("No file uploaded");

            try
            {
                // Use the example FIX query configuration
                var queryRequest = FixQueryExamples.CreateExecutionAndAllocationQuery();

                // Read file content
                string jsonContent;
                using (var reader = new StreamReader(file.OpenReadStream()))
                {
                    jsonContent = await reader.ReadToEndAsync();
                }

                // Parse messages
                var messages = ParseMessages(jsonContent);

                // Process query
                var response = ProcessMessageTypeQuery(messages, queryRequest);

                // Always return CSV for the example
                var csv = ConvertToCsv(response);
                return File(Encoding.UTF8.GetBytes(csv), "text/csv", "fix_messages.csv");
            }
            catch (Exception ex)
            {
                return StatusCode(500, new { error = ex.Message, type = ex.GetType().Name });
            }
        }

        /// <summary>
        /// Query with JSON body containing both data and query
        /// </summary>
        [HttpPost("json")]
        public async Task<IActionResult> QueryMessagesWithJson([FromBody] MessageQueryJsonRequest request)
        {
            if (request == null || request.Data == null || request.Query == null)
                return BadRequest("Invalid request - both data and query are required");

            try
            {
                // Parse messages from data field
                var messages = ParseMessages(JsonConvert.SerializeObject(request.Data));

                // Process query
                var response = ProcessMessageTypeQuery(messages, request.Query);

                // Return appropriate format
                if (request.Query.OutputFormat?.ToLowerInvariant() == "csv")
                {
                    var csv = ConvertToCsv(response);
                    return File(Encoding.UTF8.GetBytes(csv), "text/csv", "messages.csv");
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
        /// Query with FIX example using JSON body
        /// </summary>
        [HttpPost("json/fix")]
        public async Task<IActionResult> QueryFixExampleJson([FromBody] JArray data)
        {
            if (data == null || data.Count == 0)
                return BadRequest("No data provided");

            try
            {
                // Use the example FIX query configuration
                var queryRequest = FixQueryExamples.CreateExecutionAndAllocationQuery();

                // Process query
                var response = ProcessMessageTypeQuery(data, queryRequest);

                // Always return CSV for the example
                var csv = ConvertToCsv(response);
                return File(Encoding.UTF8.GetBytes(csv), "text/csv", "fix_messages.csv");
            }
            catch (Exception ex)
            {
                return StatusCode(500, new { error = ex.Message, type = ex.GetType().Name });
            }
        }

        private JArray ParseMessages(string jsonContent)
        {
            jsonContent = jsonContent.Trim();

            if (jsonContent.StartsWith("["))
            {
                return JArray.Parse(jsonContent);
            }
            else if (jsonContent.StartsWith("{"))
            {
                // Single object - wrap in array
                var obj = JObject.Parse(jsonContent);
                return new JArray(obj);
            }
            else
            {
                // Try JSONL format (one JSON object per line)
                var lines = jsonContent.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
                var array = new JArray();
                foreach (var line in lines)
                {
                    if (!string.IsNullOrWhiteSpace(line))
                    {
                        array.Add(JObject.Parse(line));
                    }
                }
                return array;
            }
        }

        private QueryResponse ProcessMessageTypeQuery(JArray messages, MessageTypeQueryRequest query)
        {
            var response = new QueryResponse
            {
                Columns = query.OutputColumns,
                Format = query.OutputFormat ?? "json"
            };

            foreach (var message in messages)
            {
                try
                {
                    // Get message type
                    var messageType = SelectorEvaluator.Evaluate(message, query.MessageTypeField)?.ToString();

                    if (string.IsNullOrEmpty(messageType))
                        continue;

                    // Check if we have selectors for this message type
                    if (!query.MessageTypes.ContainsKey(messageType))
                        continue;

                    var typeConfig = query.MessageTypes[messageType];

                    // Apply message-type-specific WHERE conditions
                    if (typeConfig.Where != null && typeConfig.Where.Any())
                    {
                        bool include = true;
                        foreach (var condition in typeConfig.Where)
                        {
                            var actualValue = SelectorEvaluator.Evaluate(message, condition.Key);
                            if (actualValue?.ToString() != condition.Value?.ToString())
                            {
                                include = false;
                                break;
                            }
                        }
                        if (!include) continue;
                    }

                    // Apply global WHERE conditions
                    if (query.GlobalWhere != null && query.GlobalWhere.Any())
                    {
                        bool include = true;
                        foreach (var condition in query.GlobalWhere)
                        {
                            var actualValue = SelectorEvaluator.Evaluate(message, condition.Key);
                            if (actualValue?.ToString() != condition.Value?.ToString())
                            {
                                include = false;
                                break;
                            }
                        }
                        if (!include) continue;
                    }

                    // Build row using unified column schema
                    var row = new List<object>();
                    foreach (var column in query.OutputColumns)
                    {
                        object value = null;

                        if (typeConfig.Select.ContainsKey(column))
                        {
                            var selector = typeConfig.Select[column];

                            // Handle special "null" selector for intentional nulls
                            if (selector.ToLower() == "null")
                            {
                                value = null;
                            }
                            else
                            {
                                try
                                {
                                    value = SelectorEvaluator.Evaluate(message, selector);
                                }
                                catch
                                {
                                    value = null;
                                }
                            }
                        }

                        row.Add(value);
                    }

                    response.Rows.Add(row);

                    // Check limit per message type if needed
                    if (query.Limit.HasValue && query.Limit.Value > 0)
                    {
                        var typeCount = response.Rows.Count(r =>
                            r.Count > 0 && r[0]?.ToString() == messageType);
                        if (typeCount >= query.Limit.Value)
                        {
                            // Skip further messages of this type
                            continue;
                        }
                    }
                }
                catch (Exception ex)
                {
                    // Log error but continue processing other messages
                    Console.WriteLine($"Error processing message: {ex.Message}");
                }
            }

            response.RowCount = response.Rows.Count;
            return response;
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
    /// Request model for JSON body queries
    /// </summary>
    public class MessageQueryJsonRequest
    {
        public JArray Data { get; set; }
        public MessageTypeQueryRequest Query { get; set; }
    }
}