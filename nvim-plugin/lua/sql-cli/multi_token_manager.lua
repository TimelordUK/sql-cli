-- Multi-Token Manager for SQL-CLI
-- Handles multiple JWT tokens with independent refresh cycles

local M = {}

-- Store multiple token configurations
M.tokens = {}

-- Setup multiple tokens
-- Example config:
-- {
--   JWT_TOKEN = {
--     command = "~/.config/sql-cli/get_uat_token.sh",
--     refresh_interval = 840,  -- 14 minutes
--     auto_refresh = true,
--   },
--   JWT_TOKEN_PROD = {
--     command = "~/.config/sql-cli/get_prod_token.sh",
--     refresh_interval = 840,
--     auto_refresh = true,
--   }
-- }
function M.setup(configs)
  -- Clear any existing timers
  M.stop_all_timers()

  -- Setup each token
  for token_name, config in pairs(configs or {}) do
    M.tokens[token_name] = vim.tbl_extend("force", {
      name = token_name,
      command = config.command,
      endpoint = config.endpoint,
      refresh_interval = config.refresh_interval or 840,  -- 14 minutes default
      auto_refresh = config.auto_refresh ~= false,  -- Default true
      current_token = nil,
      last_refresh = 0,
      timer = nil,
      debug = config.debug or false,
    }, config)

    -- Start auto-refresh if enabled
    if M.tokens[token_name].auto_refresh then
      M.start_timer(token_name)
    end
  end

  -- Initial fetch for all tokens
  M.refresh_all()
end

-- Refresh a specific token
function M.refresh_token(token_name, callback)
  local token_config = M.tokens[token_name]
  if not token_config then
    vim.notify("Token " .. token_name .. " not configured", vim.log.levels.ERROR)
    if callback then callback(nil) end
    return
  end

  if token_config.command then
    M.fetch_token_from_command(token_name, callback)
  elseif token_config.endpoint then
    M.fetch_token_from_endpoint(token_name, callback)
  else
    vim.notify("No command or endpoint for " .. token_name, vim.log.levels.ERROR)
    if callback then callback(nil) end
  end
end

-- Fetch token using command
function M.fetch_token_from_command(token_name, callback)
  local token_config = M.tokens[token_name]
  if not token_config or not token_config.command then
    if callback then callback(nil) end
    return
  end

  -- Expand ~ to home directory
  local command = token_config.command:gsub("^~", vim.env.HOME or "")

  if token_config.debug then
    vim.notify("Running command for " .. token_name .. ": " .. command, vim.log.levels.INFO)
  end

  local output = {}
  vim.fn.jobstart(command, {
    stdout_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output, line)
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      if exit_code == 0 and #output > 0 then
        local token = table.concat(output, "")

        if token and token ~= "" then
          -- Update token state
          token_config.current_token = token
          token_config.last_refresh = os.time()

          -- Set environment variable
          vim.env[token_name] = token

          -- Show success notification
          local preview = token:sub(1, 20) .. "..." .. token:sub(-10)
          local msg = string.format("🔑 %s refreshed", token_name)

          -- Show in status line temporarily
          vim.api.nvim_echo({{msg, "WarningMsg"}}, false, {})

          -- Also show as notification
          vim.notify(
            string.format("%s refreshed: %s", token_name, preview),
            vim.log.levels.INFO
          )

          -- Optional: Update statusline if custom statusline is configured
          vim.g.sql_cli_last_token_refresh = {
            token = token_name,
            time = os.date("%H:%M:%S"),
            timestamp = os.time(),
            success = true
          }

          if callback then callback(token) end
        else
          vim.notify("Empty token received for " .. token_name, vim.log.levels.ERROR)
          if callback then callback(nil) end
        end
      else
        vim.notify("Failed to fetch " .. token_name .. " (exit: " .. exit_code .. ")", vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end
  })
end

-- Fetch token from HTTP endpoint
function M.fetch_token_from_endpoint(token_name, callback)
  local token_config = M.tokens[token_name]
  if not token_config or not token_config.endpoint then
    if callback then callback(nil) end
    return
  end

  -- Build curl command
  local curl_cmd = {"curl", "-s", token_config.endpoint}

  -- Add custom headers if provided
  if token_config.headers then
    for key, value in pairs(token_config.headers) do
      table.insert(curl_cmd, "-H")
      table.insert(curl_cmd, key .. ": " .. value)
    end
  end

  if token_config.debug then
    vim.notify("Fetching " .. token_name .. " from: " .. token_config.endpoint, vim.log.levels.INFO)
  end

  local output = {}
  vim.fn.jobstart(curl_cmd, {
    stdout_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output, line)
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      if exit_code == 0 and #output > 0 then
        local response = table.concat(output, "")

        -- Try to parse as JSON
        local ok, json = pcall(vim.json.decode, response)
        if ok and json then
          -- Extract token from JSON path
          local token = M.extract_from_json(json, token_config.token_path or "token")

          if token then
            -- Update token state
            token_config.current_token = token
            token_config.last_refresh = os.time()

            -- Set environment variable
            vim.env[token_name] = token

            -- Show success notification
            local preview = token:sub(1, 20) .. "..." .. token:sub(-10)
            local msg = string.format("🔑 %s refreshed", token_name)

            -- Show in status line temporarily
            vim.api.nvim_echo({{msg, "WarningMsg"}}, false, {})

            -- Also show as notification
            vim.notify(
              string.format("%s refreshed: %s", token_name, preview),
              vim.log.levels.INFO
            )

            -- Update statusline global
            vim.g.sql_cli_last_token_refresh = {
              token = token_name,
              time = os.date("%H:%M:%S"),
              timestamp = os.time(),
              success = true
            }

            if callback then callback(token) end
          else
            vim.notify("Token not found in response for " .. token_name, vim.log.levels.ERROR)
            if callback then callback(nil) end
          end
        else
          -- Assume raw token response
          local token = response:match("^%s*(.-)%s*$")  -- Trim whitespace
          if token and token ~= "" then
            token_config.current_token = token
            token_config.last_refresh = os.time()
            vim.env[token_name] = token

            local preview = token:sub(1, 20) .. "..." .. token:sub(-10)
            vim.notify(
              string.format("%s refreshed: %s", token_name, preview),
              vim.log.levels.INFO
            )

            if callback then callback(token) end
          else
            vim.notify("Invalid token response for " .. token_name, vim.log.levels.ERROR)
            if callback then callback(nil) end
          end
        end
      else
        vim.notify("Failed to fetch " .. token_name .. " (exit: " .. exit_code .. ")", vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end
  })
end

-- Extract value from JSON using dot notation path
function M.extract_from_json(json, path)
  local parts = vim.split(path, ".", {plain = true})
  local current = json

  for _, part in ipairs(parts) do
    if type(current) == "table" and current[part] then
      current = current[part]
    else
      return nil
    end
  end

  return current
end

-- Start auto-refresh timer for a token
function M.start_timer(token_name)
  local token_config = M.tokens[token_name]
  if not token_config then return end

  -- Stop existing timer if any
  if token_config.timer then
    vim.fn.timer_stop(token_config.timer)
  end

  -- Initial refresh
  M.refresh_token(token_name)

  -- Setup periodic refresh
  local interval_ms = token_config.refresh_interval * 1000
  token_config.timer = vim.fn.timer_start(interval_ms, function()
    M.refresh_token(token_name)
  end, {['repeat'] = -1})

  if token_config.debug then
    vim.notify(
      string.format("Started auto-refresh for %s every %d seconds", token_name, token_config.refresh_interval),
      vim.log.levels.INFO
    )
  end
end

-- Stop timer for a token
function M.stop_timer(token_name)
  local token_config = M.tokens[token_name]
  if token_config and token_config.timer then
    vim.fn.timer_stop(token_config.timer)
    token_config.timer = nil
  end
end

-- Stop all timers
function M.stop_all_timers()
  for token_name, _ in pairs(M.tokens) do
    M.stop_timer(token_name)
  end
end

-- Refresh all tokens
function M.refresh_all(callback)
  local tokens_to_refresh = vim.tbl_keys(M.tokens)
  local completed = 0

  for _, token_name in ipairs(tokens_to_refresh) do
    M.refresh_token(token_name, function()
      completed = completed + 1
      if completed == #tokens_to_refresh and callback then
        callback()
      end
    end)
  end
end

-- Get a specific token value
function M.get_token(token_name)
  local token_config = M.tokens[token_name]
  if token_config then
    -- Check if we need to refresh (token older than interval)
    local age = os.time() - token_config.last_refresh
    if age > token_config.refresh_interval then
      -- Token is stale, trigger refresh (but return current for now)
      M.refresh_token(token_name)
    end
    return token_config.current_token
  end
  -- Also check environment as fallback
  return vim.env[token_name]
end

-- Show status of all tokens
function M.show_status()
  local lines = {"Multi-Token Manager Status", "=" .. string.rep("=", 40)}

  for token_name, token_config in pairs(M.tokens) do
    table.insert(lines, "")
    table.insert(lines, token_name .. ":")

    if token_config.current_token then
      local age = os.time() - token_config.last_refresh
      local remaining = token_config.refresh_interval - age

      table.insert(lines, "  Status: Active")
      table.insert(lines, string.format("  Age: %d seconds", age))
      table.insert(lines, string.format("  Next refresh: %d seconds", math.max(0, remaining)))

      local preview = token_config.current_token:sub(1, 30) .. "..."
      table.insert(lines, "  Token: " .. preview)
    else
      table.insert(lines, "  Status: Not fetched")
    end

    table.insert(lines, "  Auto-refresh: " .. (token_config.auto_refresh and "Yes" or "No"))

    if token_config.command then
      table.insert(lines, "  Source: Command - " .. token_config.command)
    elseif token_config.endpoint then
      table.insert(lines, "  Source: Endpoint - " .. token_config.endpoint)
    end
  end

  -- Create a floating window to show status
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)

  local width = 60
  local height = math.min(#lines, 30)

  vim.api.nvim_open_win(buf, true, {
    relative = 'editor',
    width = width,
    height = height,
    col = math.floor((vim.o.columns - width) / 2),
    row = math.floor((vim.o.lines - height) / 2),
    style = 'minimal',
    border = 'rounded',
    title = ' Token Status ',
    title_pos = 'center'
  })
end

-- Get statusline component
function M.statusline()
  local last = vim.g.sql_cli_last_token_refresh
  if not last then
    return ""
  end

  -- Show for 5 seconds after refresh
  local elapsed = os.time() - (last.timestamp or 0)
  if elapsed > 5 then
    return ""
  end

  if last.success then
    return string.format("%%#DiagnosticOk# 🔑 %s %%*", last.token)
  else
    return string.format("%%#DiagnosticError# ⚠ %s failed %%*", last.token)
  end
end

-- Get status for lualine or other statusline plugins
function M.get_status()
  local status_parts = {}

  for token_name, token_config in pairs(M.tokens) do
    if token_config.current_token then
      local age = os.time() - token_config.last_refresh
      local remaining = token_config.refresh_interval - age

      if remaining < 60 then
        -- Token expiring soon - show warning
        table.insert(status_parts, string.format("⚠ %s:%ds", token_name, remaining))
      elseif age < 5 then
        -- Just refreshed
        table.insert(status_parts, string.format("✓ %s", token_name))
      end
    else
      -- Token not available
      table.insert(status_parts, string.format("✗ %s", token_name))
    end
  end

  if #status_parts > 0 then
    return "🔑 " .. table.concat(status_parts, " ")
  end
  return ""
end

-- Create user commands
function M.create_commands()
  vim.api.nvim_create_user_command('TokenRefreshAll', function()
    M.refresh_all(function()
      vim.notify("All tokens refreshed", vim.log.levels.INFO)
    end)
  end, {desc = 'Refresh all tokens'})

  vim.api.nvim_create_user_command('TokenRefresh', function(opts)
    local token_name = opts.args
    if token_name and token_name ~= "" then
      M.refresh_token(token_name, function(token)
        if token then
          vim.notify(token_name .. " refreshed successfully", vim.log.levels.INFO)
        end
      end)
    else
      vim.notify("Usage: :TokenRefresh TOKEN_NAME", vim.log.levels.ERROR)
    end
  end, {
    nargs = 1,
    complete = function()
      return vim.tbl_keys(M.tokens)
    end,
    desc = 'Refresh specific token'
  })

  vim.api.nvim_create_user_command('TokenStatus', function()
    M.show_status()
  end, {desc = 'Show token status'})

  vim.api.nvim_create_user_command('TokenStop', function(opts)
    local token_name = opts.args
    if token_name and token_name ~= "" then
      M.stop_timer(token_name)
      vim.notify("Stopped auto-refresh for " .. token_name, vim.log.levels.INFO)
    else
      M.stop_all_timers()
      vim.notify("Stopped all auto-refresh timers", vim.log.levels.INFO)
    end
  end, {
    nargs = '?',
    complete = function()
      return vim.tbl_keys(M.tokens)
    end,
    desc = 'Stop auto-refresh timer(s)'
  })

  vim.api.nvim_create_user_command('TokenStart', function(opts)
    local token_name = opts.args
    if token_name and token_name ~= "" then
      M.start_timer(token_name)
    else
      for name, _ in pairs(M.tokens) do
        if M.tokens[name].auto_refresh then
          M.start_timer(name)
        end
      end
      vim.notify("Started auto-refresh for all configured tokens", vim.log.levels.INFO)
    end
  end, {
    nargs = '?',
    complete = function()
      return vim.tbl_keys(M.tokens)
    end,
    desc = 'Start auto-refresh timer(s)'
  })
end

return M