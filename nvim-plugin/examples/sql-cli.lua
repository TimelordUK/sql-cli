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
      local token_manager = require('sql-cli.token_manager')
      token_manager.setup({
         -- Use the test shell script for now (Linux/Mac)
         -- token_command = vim.fn.expand("~/dev/sql-cli/export_jwt.sh"),

         -- For PowerShell Core (cross-platform pwsh):
         -- token_command = "pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),

         -- For Windows PowerShell 5.1:
         token_command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),

         -- For production script that fetches real tokens:
         -- token_command = "pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File C:\\Scripts\\GetAuthToken.ps1",

         -- Enable auto-refresh
         auto_refresh = true,

         -- Test with shorter interval (10 seconds for testing, change to 840 for production)
         refresh_interval = 600,  -- seconds

         -- Environment variable name
         token_var_name = "JWT_TOKEN",

         -- Enable debug to see timer activity (disable for normal use)
         debug = true,
      })

      -- Create token manager commands
      token_manager.create_commands()

      -- Notify that token manager is configured
      vim.notify("SQL-CLI Token Manager configured with 10-second test interval", vim.log.levels.INFO)
      vim.notify("Commands: :TokenConfig, :TokenRefresh, :TokenShow, :TokenDebug", vim.log.levels.INFO)
   end,
}


