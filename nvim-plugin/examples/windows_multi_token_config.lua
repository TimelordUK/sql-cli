return
{
   dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
   name = "sql-cli.nvim",
   lazy = false,
   config = function()
      require('sql-cli').setup({
         debug = true,
         command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
         output_format = "table",
		  query_history = {
			persist = true,       -- Enable auto save/load
			max_items = 100,      -- Maximum queries to keep
			auto_save = true,     -- Save after each query
		}
      })

      -- Token Manager Configuration
      -- local token_manager = require('sql-cli.token_manager')
      local multi_token_manager = require('sql-cli.multi_token_manager')

      multi_token_manager.setup({
      JWT_TOKEN = {
        command = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File $HOME\\dev\\sql-cli\\ExportJwt.ps1",
        -- refresh_interval = 840,  -- 14 minutes
         refresh_interval = 20,  -- 20 seconds for testing
         auto_refresh = true,
         debug = true,  -- Enable debug output to see what's happening
      },
      JWT_TOKEN_PROD = {
        command = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File $HOME\\dev\\sql-cli\\ExportJwtProd.ps1",
        refresh_interval = 840,
        auto_refresh = true,
        debug = false,  -- Less verbose for prod token
      },
     })
      -- Create token manager commands
     multi_token_manager.create_commands()

      -- Notify that multi-token manager is configured
     vim.notify("SQL-CLI Multi-Token Manager configured (JWT_TOKEN @ 20s, JWT_TOKEN_PROD @ 840s)", vim.log.levels.INFO)
     vim.notify("Commands: :TokenStatus, :TokenRefreshAll, :TokenRefresh [name]", vim.log.levels.INFO)
   end,
}
