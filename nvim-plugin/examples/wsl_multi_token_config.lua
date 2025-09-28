return {
  dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
  name = "sql-cli.nvim",
  lazy = false,
  config = function()
    require('sql-cli').setup({
      command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
      output_format = "table",
      debug = true,
      query_history = {
        persist = true,
        max_items = 100,
        auto_save = true,
      }
    })

    -- Multi-Token Manager Configuration
    local multi_token_manager = require('sql-cli.multi_token_manager')

    multi_token_manager.setup({
      -- UAT Token
      JWT_TOKEN = {
        -- Linux/WSL command - using shell script
        command = vim.fn.expand("~/.config/sql-cli/get_uat_token.sh"),
        -- Or use example script for testing
        -- command = vim.fn.expand("~/dev/sql-cli/examples/token_scripts/get_uat_token.sh"),

        refresh_interval = 20,  -- 20 seconds for testing (use 840 for production)
        auto_refresh = true,
        debug = true,  -- Enable debug to see refresh activity
      },

      -- Production Token
      JWT_TOKEN_PROD = {
        command = vim.fn.expand("~/.config/sql-cli/get_prod_token.sh"),
        -- Or use example script for testing
        -- command = vim.fn.expand("~/dev/sql-cli/examples/token_scripts/get_prod_token.sh"),

        refresh_interval = 840,  -- 14 minutes
        auto_refresh = true,
        debug = false,
      },

      -- Optional: Add more tokens as needed
      -- AZURE_TOKEN = {
      --   command = "az account get-access-token --resource https://api.example.com --query accessToken -o tsv",
      --   refresh_interval = 3600,
      --   auto_refresh = true,
      -- },
    })

    -- Create user commands
    multi_token_manager.create_commands()

    -- Optional: Add to statusline (if using lualine)
    -- require('lualine').setup({
    --   sections = {
    --     lualine_x = {
    --       function() return require('sql-cli.multi_token_manager').get_status() end,
    --     },
    --   },
    -- })

    -- Show startup notifications
    vim.defer_fn(function()
      vim.notify("SQL-CLI Multi-Token Manager configured", vim.log.levels.INFO)
      vim.notify("  • JWT_TOKEN: 20s refresh (debug mode)", vim.log.levels.INFO)
      vim.notify("  • JWT_TOKEN_PROD: 14min refresh", vim.log.levels.INFO)
      vim.notify("Commands: :TokenStatus, :TokenRefreshAll, :TokenRefresh [name]", vim.log.levels.INFO)
    end, 500)
  end,
}


