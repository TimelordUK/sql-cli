-- Token Manager for SQL-CLI
-- Handles fetching and refreshing JWT tokens from external services

local M = {}

-- Configuration
M.config = {
  token_endpoint = nil,  -- URL to fetch token from (legacy)
  token_command = nil,   -- Command to run to get token (e.g., "pwsh -NoProfile -Command ./ExportJwt.ps1")
  auto_refresh = false,  -- Auto-refresh before macro expansion
  refresh_interval = 14 * 60,  -- Refresh interval in seconds (14 minutes for 15-minute tokens)
  last_refresh = 0,
  current_token = nil,
  token_var_name = "JWT_TOKEN",  -- Environment variable name
  custom_headers = {},  -- Additional headers for token request
  token_path = "token",  -- JSON path to extract token from response
  timer = nil,  -- Timer handle for auto-refresh
  debug = false,  -- Debug output
}

-- Initialize token manager with config
function M.setup(opts)
  M.config = vim.tbl_extend("force", M.config, opts or {})

  -- Stop existing timer if any
  if M.config.timer then
    vim.fn.timer_stop(M.config.timer)
    M.config.timer = nil
  end

  -- Check if endpoint or command is configured
  if M.config.token_command then
    vim.notify("Token manager configured with command: " .. M.config.token_command, vim.log.levels.INFO)
  elseif M.config.token_endpoint then
    vim.notify("Token manager configured for: " .. M.config.token_endpoint, vim.log.levels.INFO)
  end

  -- Start auto-refresh timer if enabled
  if M.config.auto_refresh and (M.config.token_command or M.config.token_endpoint) then
    M.start_timer()
  end
end

-- Fetch token from command
function M.fetch_token_from_command(callback)
  if not M.config.token_command then
    if callback then callback(nil) end
    return
  end

  if M.config.debug then
    vim.notify("Running token command: " .. M.config.token_command, vim.log.levels.INFO)
  end

  vim.fn.jobstart(M.config.token_command, {
    stdout_buffered = true,
    on_stdout = function(_, data, _)
      if data and #data > 0 then
        -- Get the token from stdout (should be just the token, nothing else)
        local token = nil
        for _, line in ipairs(data) do
          if line and line ~= "" then
            token = line:match("^%s*(.-)%s*$")  -- Trim whitespace
            break  -- Take first non-empty line
          end
        end

        if token then
          M.config.current_token = token
          M.config.last_refresh = os.time()
          vim.env[M.config.token_var_name] = token

          if M.config.debug then
            vim.notify("Token refreshed from command (length: " .. #token .. ")", vim.log.levels.INFO)
          else
            vim.notify("Token refreshed successfully", vim.log.levels.INFO)
          end

          if callback then callback(token) end
        else
          vim.notify("No token output from command", vim.log.levels.ERROR)
          if callback then callback(nil) end
        end
      end
    end,
    on_stderr = function(_, data, _)
      if data and #data > 0 and data[1] ~= "" then
        vim.notify("Error from token command: " .. table.concat(data, "\n"), vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end,
    on_exit = function(_, exit_code, _)
      if exit_code ~= 0 then
        vim.notify("Token command failed with exit code: " .. exit_code, vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end
  })
end

-- Fetch token from endpoint (legacy HTTP method)
function M.fetch_token_from_endpoint(callback)
  if not M.config.token_endpoint then
    if callback then callback(nil) end
    return
  end

  -- Use curl to fetch token
  local cmd = string.format('curl -s "%s"', M.config.token_endpoint)

  -- Add custom headers if any
  for key, value in pairs(M.config.custom_headers) do
    cmd = cmd .. string.format(' -H "%s: %s"', key, value)
  end

  vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    on_stdout = function(_, data, _)
      if data and #data > 0 then
        local response = table.concat(data, "\n")

        -- Try to parse JSON response
        local ok, json = pcall(vim.json.decode, response)
        if ok and json then
          -- Extract token from response
          local token = nil

          -- Simple path extraction (supports "token" or "data.token" style paths)
          if M.config.token_path:find("%.") then
            -- Nested path
            local parts = vim.split(M.config.token_path, ".")
            local current = json
            for _, part in ipairs(parts) do
              if current and type(current) == "table" then
                current = current[part]
              else
                break
              end
            end
            token = current
          else
            -- Direct path
            token = json[M.config.token_path]
          end

          if token then
            M.config.current_token = token
            M.config.last_refresh = os.time()

            -- Set environment variable
            vim.env[M.config.token_var_name] = token

            vim.notify("Token refreshed successfully", vim.log.levels.INFO)
            if callback then callback(token) end
          else
            vim.notify("Token not found in response at path: " .. M.config.token_path, vim.log.levels.ERROR)
            if callback then callback(nil) end
          end
        else
          -- Maybe it's just a plain token string
          if response and response ~= "" and not response:match("^%s*<") then
            M.config.current_token = response:match("^%s*(.-)%s*$")
            M.config.last_refresh = os.time()
            vim.env[M.config.token_var_name] = M.config.current_token
            vim.notify("Token refreshed successfully", vim.log.levels.INFO)
            if callback then callback(M.config.current_token) end
          else
            vim.notify("Failed to parse token response", vim.log.levels.ERROR)
            if callback then callback(nil) end
          end
        end
      end
    end,
    on_stderr = function(_, data, _)
      if data and #data > 0 and data[1] ~= "" then
        vim.notify("Error fetching token: " .. table.concat(data, "\n"), vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end,
    on_exit = function(_, exit_code, _)
      if exit_code ~= 0 then
        vim.notify("Token fetch failed with exit code: " .. exit_code, vim.log.levels.ERROR)
        if callback then callback(nil) end
      end
    end
  })
end

-- Unified fetch token function (chooses command or endpoint)
function M.fetch_token(callback)
  if M.config.token_command then
    M.fetch_token_from_command(callback)
  elseif M.config.token_endpoint then
    M.fetch_token_from_endpoint(callback)
  else
    vim.notify("No token source configured. Set token_command or token_endpoint in setup()", vim.log.levels.WARN)
    if callback then callback(nil) end
  end
end

-- Timer callback for auto-refresh
function M.timer_callback()
  if M.config.debug then
    vim.notify("Timer triggered for token refresh", vim.log.levels.INFO)
  end

  if M.needs_refresh() then
    M.fetch_token(function(token)
      if token and M.config.debug then
        vim.notify("Timer: Token refreshed (length: " .. #token .. ")", vim.log.levels.INFO)
      end
    end)
  end
end

-- Start the auto-refresh timer
function M.start_timer()
  -- Stop existing timer if any
  if M.config.timer then
    vim.fn.timer_stop(M.config.timer)
  end

  -- Convert seconds to milliseconds for timer
  local interval_ms = M.config.refresh_interval * 1000

  vim.notify(string.format("Starting token refresh timer (interval: %d seconds)", M.config.refresh_interval), vim.log.levels.INFO)

  -- Create repeating timer
  M.config.timer = vim.fn.timer_start(interval_ms, function()
    M.timer_callback()
  end, { ["repeat"] = -1 })  -- -1 means repeat indefinitely

  -- Also do an immediate refresh
  M.fetch_token(function(token)
    if token then
      vim.notify("Initial token fetch successful", vim.log.levels.INFO)
    end
  end)
end

-- Stop the auto-refresh timer
function M.stop_timer()
  if M.config.timer then
    vim.fn.timer_stop(M.config.timer)
    M.config.timer = nil
    vim.notify("Token refresh timer stopped", vim.log.levels.INFO)
  end
end

-- Check if token needs refresh
function M.needs_refresh()
  if not M.config.current_token then
    return true
  end

  local elapsed = os.time() - M.config.last_refresh
  return elapsed >= M.config.refresh_interval
end

-- Get current token, optionally refreshing if needed
function M.get_token(callback)
  if M.config.auto_refresh and M.needs_refresh() then
    M.fetch_token(callback)
  else
    if callback then
      callback(M.config.current_token or vim.env[M.config.token_var_name])
    end
    return M.config.current_token or vim.env[M.config.token_var_name]
  end
end

-- Manual refresh command
function M.refresh_token()
  M.fetch_token(function(token)
    if token then
      vim.notify("Token: " .. token:sub(1, 20) .. "...", vim.log.levels.INFO)
    end
  end)
end

-- Create vim commands
function M.create_commands()
  vim.api.nvim_create_user_command("TokenRefresh", function()
    M.refresh_token()
  end, { desc = "Refresh JWT token from configured endpoint" })

  vim.api.nvim_create_user_command("TokenShow", function()
    local token = M.config.current_token or vim.env[M.config.token_var_name]
    if token then
      vim.notify("Current token: " .. token:sub(1, 30) .. "...", vim.log.levels.INFO)
    else
      vim.notify("No token available", vim.log.levels.WARN)
    end
  end, { desc = "Show current JWT token" })

  vim.api.nvim_create_user_command("TokenConfig", function(opts)
    if opts.args ~= "" then
      -- Parse endpoint URL from args
      M.config.token_endpoint = opts.args
      vim.notify("Token endpoint set to: " .. opts.args, vim.log.levels.INFO)
    else
      -- Show current config
      local config_display = {
        token_source = M.config.token_command and "command" or (M.config.token_endpoint and "endpoint" or "none"),
        token_command = M.config.token_command,
        token_endpoint = M.config.token_endpoint,
        auto_refresh = M.config.auto_refresh,
        refresh_interval = M.config.refresh_interval .. " seconds",
        timer_active = M.config.timer ~= nil,
        last_refresh = M.config.last_refresh > 0 and os.date("%Y-%m-%d %H:%M:%S", M.config.last_refresh) or "never",
        has_token = M.config.current_token ~= nil,
      }
      vim.notify("Token config:\n" .. vim.inspect(config_display), vim.log.levels.INFO)
    end
  end, {
    nargs = "?",
    desc = "Configure or show token endpoint"
  })

  vim.api.nvim_create_user_command("TokenStartTimer", function()
    M.start_timer()
  end, { desc = "Start token auto-refresh timer" })

  vim.api.nvim_create_user_command("TokenStopTimer", function()
    M.stop_timer()
  end, { desc = "Stop token auto-refresh timer" })

  vim.api.nvim_create_user_command("TokenSetCommand", function(opts)
    if opts.args and opts.args ~= "" then
      M.config.token_command = opts.args
      vim.notify("Token command set to: " .. opts.args, vim.log.levels.INFO)
      -- Clear endpoint if command is set
      M.config.token_endpoint = nil
    else
      vim.notify("Usage: :TokenSetCommand <command>", vim.log.levels.WARN)
    end
  end, {
    nargs = 1,
    desc = "Set token fetch command (e.g., pwsh -NoProfile ./ExportJwt.ps1)"
  })

  vim.api.nvim_create_user_command("TokenDebug", function(opts)
    M.config.debug = not M.config.debug
    vim.notify("Token debug mode: " .. (M.config.debug and "ON" or "OFF"), vim.log.levels.INFO)
  end, { desc = "Toggle token manager debug mode" })
end

return M