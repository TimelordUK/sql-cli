-- Test the SQL formatter function
package.path = package.path .. ";nvim-plugin/lua/?.lua;nvim-plugin/lua/?/init.lua"

-- Mock vim functions for testing
_G.vim = {
  split = function(str, sep)
    local result = {}
    for match in (str..sep):gmatch("(.-)"..sep) do
      table.insert(result, match)
    end
    return result
  end,
  notify = function(msg, level) print(msg) end,
  log = { levels = { INFO = 1, WARN = 2, ERROR = 3 } }
}

-- Load the module
local sql_cli = require('sql-cli')

-- Test queries
local test_queries = {
  -- Simple query
  "SELECT id, name, price FROM products WHERE price > 100 AND status = 'active' ORDER BY price DESC LIMIT 10",
  
  -- Query with JOIN
  "SELECT o.order_id, o.amount, c.name FROM orders o LEFT JOIN customers c ON o.customer_id = c.id WHERE o.status = 'active' ORDER BY o.created_date DESC",
  
  -- Query with CASE
  "SELECT id, CASE WHEN amount > 1000 THEN 'High' WHEN amount > 500 THEN 'Medium' ELSE 'Low' END as priority FROM orders WHERE status = 'pending'"
}

-- Test the formatter
for i, query in ipairs(test_queries) do
  print("\n--- Test Query " .. i .. " ---")
  print("Original:")
  print(query)
  print("\nFormatted:")
  local formatted = sql_cli.format_sql(query)
  print(formatted)
  print("\n" .. string.rep("-", 50))
end