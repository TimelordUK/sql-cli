-- SQL CLI Export Module
-- Multiple export formats for query results

local M = {}

-- Get the full result table from buffer
local function get_table_data(bufnr, table_info)
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local data = {
    headers = {},
    rows = {}
  }

  -- Debug output
  if not table_info then
    vim.notify("Error: table_info is nil", vim.log.levels.ERROR)
    return data
  end

  if not table_info.header_row then
    vim.notify("Warning: table_info.header_row is nil", vim.log.levels.WARN)
  end

  if not table_info.column_positions or #table_info.column_positions == 0 then
    vim.notify("Warning: No column positions found", vim.log.levels.WARN)
  end

  -- Extract headers
  if table_info.header_row and lines[table_info.header_row] then
    local header_line = lines[table_info.header_row]
    for _, col_pos in ipairs(table_info.column_positions or {}) do
      local col_text = header_line:sub(col_pos.start, col_pos.stop)
      col_text = col_text:gsub("^%s*", ""):gsub("%s*$", "")
      table.insert(data.headers, col_text)
    end
  end

  -- Extract data rows
  if table_info.data_start and table_info.data_end then
    for row_idx = table_info.data_start, table_info.data_end do
      local line = lines[row_idx]
      if line and line:match("^|") then
        local row = {}
        for _, col_pos in ipairs(table_info.column_positions) do
          local value = line:sub(col_pos.start, col_pos.stop)
          value = value:gsub("^%s*", ""):gsub("%s*$", "")
          table.insert(row, value)
        end
        table.insert(data.rows, row)
      end
    end
  end

  return data
end

-- Export to HTML table (works great in Outlook/Teams)
function M.yank_as_html(bufnr, table_info, open_browser)
  local data = { headers = {}, rows = {} }

  -- Check if we're in table navigation mode (focused on a specific table)
  local table_nav = require('sql-cli.table_nav')
  local use_clean_export = not table_nav.is_active()

  -- Try to get clean data from sql-cli only if NOT in table navigation mode
  -- (i.e., when exporting the entire/only result, not a specific table from multi-table results)
  local csv_str = use_clean_export and M.get_clean_export_data("csv") or nil

  if csv_str then
    -- Parse CSV to get structured data (handle quoted fields properly)
    local function parse_csv_line(line)
      local fields = {}
      local field = ""
      local in_quotes = false

      for i = 1, #line do
        local c = line:sub(i, i)
        if c == '"' then
          if in_quotes and line:sub(i+1, i+1) == '"' then
            -- Escaped quote
            field = field .. '"'
            i = i + 1
          else
            in_quotes = not in_quotes
          end
        elseif c == ',' and not in_quotes then
          table.insert(fields, field)
          field = ""
        else
          field = field .. c
        end
      end
      -- Add last field
      table.insert(fields, field)
      return fields
    end

    local lines = vim.split(csv_str, '\n', { plain = true })

    -- First line is headers
    if #lines > 0 then
      data.headers = parse_csv_line(lines[1])
    end

    -- Remaining lines are data
    for i = 2, #lines do
      if lines[i] and lines[i] ~= "" then
        local row = parse_csv_line(lines[i])
        if #row > 0 then
          table.insert(data.rows, row)
        end
      end
    end
  else
    -- Fallback to parsing from buffer
    data = get_table_data(bufnr, table_info)
  end

  -- Create full HTML document for browser viewing
  local full_html = {
    '<!DOCTYPE html>',
    '<html>',
    '<head>',
    '  <meta charset="UTF-8">',
    '  <title>SQL Query Results</title>',
    '  <style>',
    '    body { font-family: Arial, sans-serif; margin: 20px; }',
    '    table { border-collapse: collapse; width: auto; }',
    '    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }',
    '    th { background-color: #4CAF50; color: white; }',
    '    tr:nth-child(even) { background-color: #f2f2f2; }',
    '    tr:hover { background-color: #ddd; }',
    '    .info { margin: 20px 0; padding: 10px; background: #e7f3fe; border-left: 4px solid #2196F3; }',
    '  </style>',
    '</head>',
    '<body>',
    '  <div class="info">',
    '    <strong>Tip:</strong> Select the table below and copy (Ctrl+C/Cmd+C) to paste into Gmail, Outlook, or Teams.',
    '  </div>',
  }

  local html = {'<table border="1" cellpadding="4" cellspacing="0" style="border-collapse: collapse;">'}

  -- Add header row
  table.insert(html, '  <thead>')
  table.insert(html, '    <tr style="background-color: #f0f0f0;">')
  for _, header in ipairs(data.headers) do
    table.insert(html, string.format('      <th>%s</th>', header))
  end
  table.insert(html, '    </tr>')
  table.insert(html, '  </thead>')

  -- Add data rows
  table.insert(html, '  <tbody>')
  for _, row in ipairs(data.rows) do
    table.insert(html, '    <tr>')
    for _, value in ipairs(row) do
      -- Escape HTML special characters
      local escaped = value:gsub("&", "&amp;"):gsub("<", "&lt;"):gsub(">", "&gt;")
      table.insert(html, string.format('      <td>%s</td>', escaped))
    end
    table.insert(html, '    </tr>')
  end
  table.insert(html, '  </tbody>')
  table.insert(html, '</table>')

  local html_table = table.concat(html, '\n')

  if open_browser then
    -- Add table to full HTML
    table.insert(full_html, html_table)
    table.insert(full_html, '</body>')
    table.insert(full_html, '</html>')

    -- Write to temp file and open in browser
    -- Use a more predictable location for WSL and Windows
    local temp_file
    if vim.fn.has('wsl') == 1 then
      -- Save to Windows temp directory for easy access
      temp_file = '/mnt/c/temp/sql_export_' .. os.date('%Y%m%d_%H%M%S') .. '.html'
      -- Ensure directory exists
      vim.fn.system('mkdir -p /mnt/c/temp')
    elseif vim.fn.has('win32') == 1 then
      -- Use Windows temp directory
      local temp_dir = vim.fn.expand('$TEMP')
      temp_file = temp_dir .. '\\sql_export_' .. os.date('%Y%m%d_%H%M%S') .. '.html'
    else
      temp_file = os.tmpname() .. '.html'
    end

    -- Debug: Show where we're trying to write
    if vim.fn.has('win32') == 1 then
      vim.notify(string.format("Writing HTML to: %s", temp_file), vim.log.levels.INFO)
    end

    local file, err = io.open(temp_file, 'w')
    if file then
      file:write(table.concat(full_html, '\n'))
      file:close()

      -- Verify file was written
      if vim.fn.has('win32') == 1 then
        local file_exists = vim.fn.filereadable(temp_file) == 1
        if file_exists then
          vim.notify(string.format("HTML file written successfully: %s", temp_file), vim.log.levels.INFO)
        else
          vim.notify(string.format("Warning: File may not have been written: %s", temp_file), vim.log.levels.WARN)
        end
      end
    else
      vim.notify(string.format("Error writing HTML file: %s - %s", temp_file, err or "unknown error"), vim.log.levels.ERROR)
      return
    end

    if file then

      -- Open in default browser (cross-platform)
      local open_cmd
      if vim.fn.has('mac') == 1 then
        open_cmd = 'open'
      elseif vim.fn.has('unix') == 1 then
        open_cmd = 'xdg-open'
      elseif vim.fn.has('win32') == 1 then
        open_cmd = 'start'
      end

      if open_cmd then
        -- Windows 'start' command needs special handling
        if vim.fn.has('win32') == 1 then
          -- On Windows, use rundll32 to open URL in default browser
          -- This is more reliable than start command
          local windows_path = temp_file:gsub('/', '\\')
          local file_url = 'file:///' .. windows_path:gsub('\\', '/')
          vim.fn.system(string.format('rundll32 url.dll,FileProtocolHandler "%s"', file_url))

          -- Also notify user with the file path in case browser doesn't open
          vim.notify(string.format("HTML saved to: %s", windows_path), vim.log.levels.INFO)
        else
          vim.fn.system(string.format('%s "%s"', open_cmd, temp_file))
        end

        -- For WSL users, provide Windows path
        if vim.fn.has('wsl') == 1 then
          -- File is saved at C:\temp\sql_export_*.html
          local win_path = temp_file:gsub('/mnt/c/', 'C:\\'):gsub('/', '\\')
          vim.notify(string.format("HTML saved to: %s", win_path), vim.log.levels.INFO)
          vim.notify("Open this file in Windows browser, then copy table to Gmail", vim.log.levels.INFO)

          -- Copy the Windows path to clipboard for easy access
          vim.fn.setreg('+', win_path)
        else
          vim.notify(string.format("Opened HTML table in browser - copy from there to paste into Gmail", #data.rows), vim.log.levels.INFO)
        end
      end
    end
  else
    -- Just yank the HTML code
    vim.fn.setreg('+', html_table)
    vim.fn.setreg('"', html_table)
    vim.notify(string.format("Yanked %d rows as HTML table", #data.rows), vim.log.levels.INFO)
  end

  return html_table
end

-- Export to Markdown table
function M.yank_as_markdown(bufnr, table_info)
  local data = get_table_data(bufnr, table_info)

  local md = {}

  -- Header row
  local header_line = "| " .. table.concat(data.headers, " | ") .. " |"
  table.insert(md, header_line)

  -- Separator row
  local separators = {}
  for i = 1, #data.headers do
    table.insert(separators, "---")
  end
  table.insert(md, "| " .. table.concat(separators, " | ") .. " |")

  -- Data rows
  for _, row in ipairs(data.rows) do
    table.insert(md, "| " .. table.concat(row, " | ") .. " |")
  end

  local md_str = table.concat(md, '\n')
  vim.fn.setreg('+', md_str)
  vim.fn.setreg('"', md_str)

  vim.notify(string.format("Yanked %d rows as Markdown table", #data.rows), vim.log.levels.INFO)
  return md_str
end

-- Export to Tab-Separated Values (perfect for Excel)
function M.yank_as_tsv(bufnr, table_info)
  -- Check if fuzzy filter is active and has filtered data
  local fuzzy = require('sql-cli.fuzzy_filter')
  local filtered_data = fuzzy.get_filtered_rows()

  if filtered_data then
    -- Export filtered data
    local tsv = {}

    -- Header row
    table.insert(tsv, table.concat(filtered_data.headers, "\t"))

    -- Data rows
    for _, row in ipairs(filtered_data.rows) do
      table.insert(tsv, table.concat(row, "\t"))
    end

    local tsv_str = table.concat(tsv, '\n')

    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      tsv_str = tsv_str:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', tsv_str)
    vim.fn.setreg('"', tsv_str)

    vim.notify(string.format("✅ Yanked %d filtered rows as TSV (ready for Excel)", #filtered_data.rows), vim.log.levels.INFO)
    return tsv_str
  end

  -- Check if we're in table navigation mode (focused on a specific table)
  local table_nav = require('sql-cli.table_nav')
  local use_clean_export = not table_nav.is_active()

  -- Get clean TSV data by running query with TSV output (only if not in table nav mode)
  local tsv_str = use_clean_export and M.get_clean_export_data("tsv") or nil

  if tsv_str then
    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      tsv_str = tsv_str:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', tsv_str)
    vim.fn.setreg('"', tsv_str)

    -- Count rows for notification
    local row_count = select(2, tsv_str:gsub('\r?\n', '\n'))
    vim.notify(string.format("✓ Fetched clean TSV from sql-cli: %d rows yanked (paste into Excel)", row_count), vim.log.levels.INFO)
    return tsv_str
  else
    -- Fallback to old method if new one fails
    local data = get_table_data(bufnr, table_info)
    local tsv = {}

    -- Header row
    table.insert(tsv, table.concat(data.headers, "\t"))

    -- Data rows
    for _, row in ipairs(data.rows) do
      table.insert(tsv, table.concat(row, "\t"))
    end

    local tsv_str_fallback = table.concat(tsv, '\n')

    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      tsv_str_fallback = tsv_str_fallback:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', tsv_str_fallback)
    vim.fn.setreg('"', tsv_str_fallback)

    vim.notify(string.format("⚠ Fallback: Yanked %d rows as TSV from display (may contain artifacts)", #data.rows), vim.log.levels.WARN)
    return tsv_str_fallback
  end
end

-- Get clean export data by re-running query with specific output format
function M.get_clean_export_data(format)
  -- Get the state and config from the main module
  local ok, sql_cli = pcall(require, 'sql-cli')
  if not ok then
    return nil
  end

  local state = sql_cli.state
  local config = sql_cli.config

  if not state or not config then
    return nil
  end

  -- Get the last query
  local query = state:get_last_query()
  if not query or query == "" then
    return nil
  end

  -- Build command with specific output format
  local utils = require('sql-cli.utils')
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    return nil
  end

  local cmd_parts = { command_path }

  -- Add data file if set
  local data_file = state:get_data_file()
  if data_file then
    table.insert(cmd_parts, vim.fn.shellescape(data_file))
  end

  -- Add query
  local is_script = query:match("%sGO%s") or query:match("^GO%s") or query:match("%sGO$")
  local is_multiline = query:find("\n") ~= nil

  if is_script or is_multiline then
    -- Save to temp file for script execution or multi-line queries
    local temp_file = vim.fn.tempname() .. ".sql"
    local file = io.open(temp_file, "w")
    if not file then
      return nil
    end
    file:write(query)
    file:close()

    table.insert(cmd_parts, "-f")
    table.insert(cmd_parts, vim.fn.shellescape(temp_file))
  else
    -- Direct single-line query
    table.insert(cmd_parts, "-q")
    table.insert(cmd_parts, vim.fn.shellescape(query))
  end

  -- Add output format
  table.insert(cmd_parts, "-o")
  table.insert(cmd_parts, format)

  -- Run command and capture output
  local cmd = table.concat(cmd_parts, " ")

  -- Debug option to show command
  if vim.g.sql_cli_debug_export then
    vim.notify(string.format("Export command: %s", cmd), vim.log.levels.DEBUG)
  end

  local output = vim.fn.system(cmd)

  if vim.v.shell_error ~= 0 then
    if vim.g.sql_cli_debug_export then
      vim.notify(string.format("Export failed: %s", vim.v.shell_error), vim.log.levels.ERROR)
    end
    return nil
  end

  -- Remove the timing line at the end (e.g., "# Query completed: 4 rows in 27.877807ms")
  -- This appears in stderr but sometimes gets mixed into output
  output = output:gsub("#%s*Query%s+completed:.-\n?$", "")

  -- Also remove any trailing whitespace
  output = output:gsub("%s+$", "")

  return output
end

-- Export to SQL INSERT statements
function M.yank_as_insert(bufnr, table_info, table_name)
  local data = get_table_data(bufnr, table_info)

  -- Ask for table name if not provided
  if not table_name or table_name == "" then
    table_name = "table_name"
  end

  local inserts = {}

  -- Generate INSERT statements
  for _, row in ipairs(data.rows) do
    local values = {}
    for _, value in ipairs(row) do
      -- Detect if value is numeric or needs quotes
      if value == "NULL" or value == "" then
        table.insert(values, "NULL")
      elseif value:match("^%-?%d+%.?%d*$") then
        -- Numeric value
        table.insert(values, value)
      else
        -- String value - escape single quotes
        local escaped = value:gsub("'", "''")
        table.insert(values, string.format("'%s'", escaped))
      end
    end

    local insert = string.format("INSERT INTO %s (%s) VALUES (%s);",
      table_name,
      table.concat(data.headers, ", "),
      table.concat(values, ", ")
    )
    table.insert(inserts, insert)
  end

  local sql_str = table.concat(inserts, '\n')
  vim.fn.setreg('+', sql_str)
  vim.fn.setreg('"', sql_str)

  vim.notify(string.format("Yanked %d INSERT statements for table '%s'", #data.rows, table_name), vim.log.levels.INFO)
  return sql_str
end

-- Generate CREATE TABLE statement from schema
function M.yank_create_table(bufnr, table_info, table_name)
  local data = get_table_data(bufnr, table_info)

  if not table_name or table_name == "" then
    table_name = "table_name"
  end

  local columns = {}

  -- Analyze first few rows to determine column types
  for col_idx, header in ipairs(data.headers) do
    local col_type = "VARCHAR(255)"  -- Default type
    local all_numeric = true
    local all_integer = true
    local max_length = 0

    -- Check first 10 rows to determine type
    for row_idx = 1, math.min(10, #data.rows) do
      local value = data.rows[row_idx][col_idx]
      if value and value ~= "" and value ~= "NULL" then
        max_length = math.max(max_length, #value)

        if not value:match("^%-?%d+%.?%d*$") then
          all_numeric = false
          all_integer = false
        elseif value:match("%.") then
          all_integer = false
        end
      end
    end

    -- Determine column type
    if all_integer then
      col_type = "INTEGER"
    elseif all_numeric then
      col_type = "DECIMAL(10,2)"
    else
      col_type = string.format("VARCHAR(%d)", math.max(max_length * 2, 50))
    end

    -- Clean column name (remove spaces, special chars)
    local clean_name = header:gsub("[^%w_]", "_"):lower()
    table.insert(columns, string.format("  %s %s", clean_name, col_type))
  end

  local create_sql = string.format("CREATE TABLE %s (\n%s\n);",
    table_name,
    table.concat(columns, ",\n")
  )

  vim.fn.setreg('+', create_sql)
  vim.fn.setreg('"', create_sql)

  vim.notify(string.format("Yanked CREATE TABLE statement for '%s'", table_name), vim.log.levels.INFO)
  return create_sql
end

-- Export to CSV (proper escaping)
function M.yank_as_csv(bufnr, table_info)
  -- Check if fuzzy filter is active and has filtered data
  local fuzzy = require('sql-cli.fuzzy_filter')
  local filtered_data = fuzzy.get_filtered_rows()

  if filtered_data then
    -- Export filtered data as CSV
    local csv = {}

    -- Header row
    local header_csv = {}
    for _, h in ipairs(filtered_data.headers) do
      -- Quote if contains comma, quote, or newline
      if h:match('[,"\n\r]') then
        table.insert(header_csv, '"' .. h:gsub('"', '""') .. '"')
      else
        table.insert(header_csv, h)
      end
    end
    table.insert(csv, table.concat(header_csv, ","))

    -- Data rows
    for _, row in ipairs(filtered_data.rows) do
      local row_csv = {}
      for _, cell in ipairs(row) do
        -- Quote if contains comma, quote, or newline
        if cell:match('[,"\n\r]') then
          table.insert(row_csv, '"' .. cell:gsub('"', '""') .. '"')
        else
          table.insert(row_csv, cell)
        end
      end
      table.insert(csv, table.concat(row_csv, ","))
    end

    local csv_str = table.concat(csv, '\n')

    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      csv_str = csv_str:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', csv_str)
    vim.fn.setreg('"', csv_str)

    vim.notify(string.format("✅ Yanked %d filtered rows as CSV", #filtered_data.rows), vim.log.levels.INFO)
    return csv_str
  end

  -- Check if we're in table navigation mode (focused on a specific table)
  local table_nav = require('sql-cli.table_nav')
  local use_clean_export = not table_nav.is_active()

  -- Get clean CSV data by running query with CSV output (only if not in table nav mode)
  local csv_str = use_clean_export and M.get_clean_export_data("csv") or nil

  if csv_str then
    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      csv_str = csv_str:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', csv_str)
    vim.fn.setreg('"', csv_str)

    -- Count rows for notification
    local row_count = select(2, csv_str:gsub('\n', '\n'))
    vim.notify(string.format("✓ Fetched clean CSV from sql-cli: %d rows yanked", row_count), vim.log.levels.INFO)
    return csv_str
  else
    -- Fallback to old method if new one fails
    local data = get_table_data(bufnr, table_info)
    local csv = {}

    -- Helper to escape CSV values
    local function escape_csv(value)
      if value:match('[,"\n]') then
        return '"' .. value:gsub('"', '""') .. '"'
      end
      return value
    end

    -- Header row
    local headers = {}
    for _, h in ipairs(data.headers) do
      table.insert(headers, escape_csv(h))
    end
    table.insert(csv, table.concat(headers, ","))

    -- Data rows
    for _, row in ipairs(data.rows) do
      local values = {}
      for _, v in ipairs(row) do
        table.insert(values, escape_csv(v))
      end
      table.insert(csv, table.concat(values, ","))
    end

    local csv_str_fallback = table.concat(csv, '\n')

    -- Convert line endings for Windows clipboard
    if vim.fn.has('win32') == 1 then
      csv_str_fallback = csv_str_fallback:gsub('\n', '\r\n')
    end

    vim.fn.setreg('+', csv_str_fallback)
    vim.fn.setreg('"', csv_str_fallback)

    vim.notify(string.format("⚠ Fallback: Yanked %d rows as CSV from display (may contain artifacts)", #data.rows), vim.log.levels.WARN)
    return csv_str_fallback
  end
end

-- Show export menu
function M.show_export_menu(bufnr, table_info)
  local options = {
    "1. Open HTML in Browser (for Gmail/Teams copy)",
    "2. HTML Table Code (raw HTML)",
    "3. Markdown Table",
    "4. Tab-Separated (Excel)",
    "5. CSV (Proper escaping)",
    "6. SQL INSERT Statements",
    "7. CREATE TABLE Statement",
  }

  vim.ui.select(options, {
    prompt = "Select export format:",
  }, function(choice)
    if not choice then return end

    local choice_num = tonumber(choice:match("^(%d+)"))
    if choice_num == 1 then
      M.yank_as_html(bufnr, table_info, true)  -- Open in browser
    elseif choice_num == 2 then
      M.yank_as_html(bufnr, table_info, false) -- Just yank HTML
    elseif choice_num == 3 then
      M.yank_as_markdown(bufnr, table_info)
    elseif choice_num == 4 then
      M.yank_as_tsv(bufnr, table_info)
    elseif choice_num == 5 then
      M.yank_as_csv(bufnr, table_info)
    elseif choice_num == 6 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_as_insert(bufnr, table_info, name)
        end
      end)
    elseif choice_num == 7 then
      vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
        if name then
          M.yank_create_table(bufnr, table_info, name)
        end
      end)
    end
  end)
end

return M