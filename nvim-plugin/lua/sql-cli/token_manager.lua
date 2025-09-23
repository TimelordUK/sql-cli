-- Token Manager for SQL-CLI
-- Handles fetching and refreshing JWT tokens from external services

local M = {}

-- Configuration
M.config = {
  token_endpoint = nil,  -- URL to fetch token from
  auto_refresh = false,  -- Auto-refresh before macro expansion
  refresh_interval = 14 * 60,  -- Refresh interval in seconds (14 minutes for 15-minute tokens)
  last_refresh = 0,
  current_token = nil,
  token_var_name = "JWT_TOKEN",  -- Environment variable name
  custom_headers = {},  -- Additional headers for token request
  token_path = "token",  -- JSON path to extract token from response
}

-- Initialize token manager with config
function M.setup(opts)
  M.config = vim.tbl_extend("force", M.config, opts or {})

  -- Check if endpoint is configured
  if M.config.token_endpoint then
    vim.notify("Token manager configured for: " .. M.config.token_endpoint, vim.log.levels.INFO)
  end
end

-- Fetch token from endpoint
function M.fetch_token(callback)
  if not M.config.token_endpoint then
    vim.notify("Token endpoint not configured. Set token_endpoint in setup()", vim.log.levels.WARN)
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
      vim.notify("Token config:\n" .. vim.inspect(M.config), vim.log.levels.INFO)
    end
  end, {
    nargs = "?",
    desc = "Configure or show token endpoint"
  })
end

return M