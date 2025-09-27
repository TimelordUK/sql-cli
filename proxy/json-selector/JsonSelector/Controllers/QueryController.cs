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
    public class QueryController : ControllerBase
    {
        /// <summary>
        /// Query JSON data with file upload
        /// </summary>
        [HttpPost("upload")]
        public async Task<IActionResult> QueryWithUpload(
            [FromForm] IFormFile file,
            [FromForm] string query)
        {
            if (file == null || file.Length == 0)
                return BadRequest("No file uploaded");

            try
            {
                // Parse query from form data
                var queryRequest = JsonConvert.DeserializeObject<QueryRequest>(query);
                if (queryRequest == null)
                    return BadRequest("Invalid query format");

                // Read file content
                string jsonContent;
                using (var reader = new StreamReader(file.OpenReadStream()))
                {
                    jsonContent = await reader.ReadToEndAsync();
                }

                // Parse JSON - support both single object and array
                JToken root;
                jsonContent = jsonContent.Trim();

                if (jsonContent.StartsWith("["))
                {
                    root = JArray.Parse(jsonContent);
                }
                else if (jsonContent.StartsWith("{"))
                {
                    // Single object - wrap in array for consistent processing
                    var obj = JObject.Parse(jsonContent);
                    root = new JArray(obj);
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
                    root = array;
                }

                // Process query
                var response = ProcessQuery(root, queryRequest);

                // Return appropriate format
                if (queryRequest.OutputFormat?.ToLowerInvariant() == "csv")
                {
                    var csv = ConvertToCsv(response);
                    return File(Encoding.UTF8.GetBytes(csv), "text/csv", "result.csv");
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
        /// Query JSON data directly (for testing)
        /// </summary>
        [HttpPost("direct")]
        public IActionResult QueryDirect([FromBody] DirectQueryRequest request)
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

                var response = ProcessQuery(root, request.Query);

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

        private QueryResponse ProcessQuery(JToken root, QueryRequest query)
        {
            var response = new QueryResponse
            {
                Columns = query.Select.Keys.ToList(),
                Format = query.OutputFormat ?? "json"
            };

            var items = root as JArray ?? new JArray(root);

            // Apply WHERE filters
            if (query.Where != null && query.Where.Any())
            {
                items = FilterItems(items, query.Where);
            }

            // Apply LIMIT
            if (query.Limit.HasValue && query.Limit.Value > 0)
            {
                items = new JArray(items.Take(query.Limit.Value));
            }

            // Apply SELECT
            foreach (var item in items)
            {
                var row = new List<object>();

                foreach (var selector in query.Select)
                {
                    try
                    {
                        var value = SelectorEvaluator.Evaluate(item, selector.Value);
                        row.Add(value);
                    }
                    catch
                    {
                        // If selector fails, add null
                        row.Add(null);
                    }
                }

                response.Rows.Add(row);
            }

            response.RowCount = response.Rows.Count;
            return response;
        }

        private JArray FilterItems(JArray items, Dictionary<string, object> whereConditions)
        {
            var filtered = new JArray();

            foreach (var item in items)
            {
                bool include = true;

                foreach (var condition in whereConditions)
                {
                    try
                    {
                        var actualValue = SelectorEvaluator.Evaluate(item, condition.Key);

                        // Handle different condition types
                        if (condition.Value is JObject condObj)
                        {
                            // Complex condition (e.g., regex, range)
                            if (condObj.ContainsKey("regex"))
                            {
                                var pattern = condObj["regex"].ToString();
                                var regex = new System.Text.RegularExpressions.Regex(pattern);
                                include = actualValue != null && regex.IsMatch(actualValue.ToString());
                            }
                            else if (condObj.ContainsKey("in"))
                            {
                                var values = condObj["in"] as JArray;
                                include = values != null && values.Any(v =>
                                    v.ToString() == actualValue?.ToString());
                            }
                        }
                        else if (condition.Value is JArray array)
                        {
                            // IN condition
                            include = array.Any(v => v.ToString() == actualValue?.ToString());
                        }
                        else
                        {
                            // Simple equality
                            include = actualValue?.ToString() == condition.Value?.ToString();
                        }

                        if (!include) break;
                    }
                    catch
                    {
                        include = false;
                        break;
                    }
                }

                if (include)
                    filtered.Add(item);
            }

            return filtered;
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

    public class DirectQueryRequest
    {
        public JToken Data { get; set; }
        public QueryRequest Query { get; set; }
    }
}