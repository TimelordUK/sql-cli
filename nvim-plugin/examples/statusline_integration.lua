-- Example statusline integration for multi-token manager

-- For lualine users
local lualine_config = {
  sections = {
    lualine_x = {
      -- Add token status to lualine
      {
        function()
          local multi_token = require('sql-cli.multi_token_manager')
          return multi_token.get_status()
        end,
        icon = '🔑',
      },
    },
  },
}

-- For native vim statusline
-- Add to your config:
vim.o.statusline = vim.o.statusline .. '%{luaeval("require(\'sql-cli.multi_token_manager\').statusline()")}'

-- Or build a custom statusline function
function _G.custom_statusline()
  local parts = {}

  -- Your other statusline components...
  table.insert(parts, '%f')  -- filename
  table.insert(parts, '%m')  -- modified flag

  -- Add token status
  local multi_token = require('sql-cli.multi_token_manager')
  local token_status = multi_token.get_status()
  if token_status ~= "" then
    table.insert(parts, token_status)
  end

  -- More components...
  table.insert(parts, '%l:%c')  -- line:column

  return table.concat(parts, ' ')
end

vim.o.statusline = '%!v:lua.custom_statusline()'

-- For displaying refresh notifications in command line
-- The multi_token_manager already does this with:
-- vim.api.nvim_echo({{msg, "WarningMsg"}}, false, {})

-- You can also create a floating notification for token refresh
vim.api.nvim_create_autocmd("User", {
  pattern = "TokenRefreshed",
  callback = function(args)
    local token_name = args.data.token
    local success = args.data.success

    if success then
      -- Show a temporary floating window
      local buf = vim.api.nvim_create_buf(false, true)
      local msg = string.format("🔑 %s token refreshed", token_name)
      vim.api.nvim_buf_set_lines(buf, 0, -1, false, {msg})

      local width = #msg + 4
      local win = vim.api.nvim_open_win(buf, false, {
        relative = 'editor',
        width = width,
        height = 1,
        col = vim.o.columns - width - 2,
        row = vim.o.lines - 3,
        style = 'minimal',
        border = 'rounded',
      })

      -- Auto-close after 3 seconds
      vim.defer_fn(function()
        if vim.api.nvim_win_is_valid(win) then
          vim.api.nvim_win_close(win, true)
        end
      end, 3000)
    end
  end
})

-- Example using vim.notify with nvim-notify plugin
-- If you have nvim-notify installed, tokens will show as nice notifications
if pcall(require, 'notify') then
  vim.notify = require('notify')

  -- Configure notify for token messages
  require('notify').setup({
    render = 'compact',
    stages = 'fade_in_slide_out',
    timeout = 2000,  -- Show for 2 seconds
    on_open = function(win)
      -- Make token notifications less intrusive
      if vim.api.nvim_win_is_valid(win) then
        vim.api.nvim_win_set_config(win, { focusable = false })
      end
    end,
  })
end