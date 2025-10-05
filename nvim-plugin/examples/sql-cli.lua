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
		},

        -- FIX Message Syntax Highlighting
        syntax = {
          patterns = {
            -- Message Types
            { pattern = [[\<ExecutionReport\>]], group = "FixExecReport",
              color = { gui = "#50fa7b", cterm = "Green", bold = true } },
            { pattern = [[\<AllocationReport\>]], group = "FixAllocReport",
              color = { gui = "#8be9fd", cterm = "Cyan", bold = true } },
            { pattern = [[\<NewOrderSingle\>]], group = "FixNewOrder",
              color = { gui = "#f1fa8c", cterm = "Yellow", bold = true } },
            { pattern = [[\<OrderCancelReject\>]], group = "FixCancelReject",
              color = { gui = "#ff5555", cterm = "Red", bold = true } },

            -- Order Status
            { pattern = [[\<PendingNew\>]], group = "FixStatPendingNew",
              color = { gui = "#f1fa8c", cterm = "Yellow" } },
            { pattern = [[\<New\>]], group = "FixStatNew",
              color = { gui = "#50fa7b", cterm = "Green" } },
            { pattern = [[\<PartiallyFilled\>]], group = "FixStatPartial",
              color = { gui = "#50fa7b", cterm = "Green" } },
            { pattern = [[\<Filled\>]], group = "FixStatFilled",
              color = { gui = "#50fa7b", cterm = "Green", bold = true } },
            { pattern = [[\<Canceled\>]], group = "FixStatCanceled",
              color = { gui = "#6272a4", cterm = "DarkGray" } },
            { pattern = [[\<Rejected\>]], group = "FixStatRejected",
              color = { gui = "#ff5555", cterm = "Red", bold = true } },

            -- Side (Buy/Sell)
            { pattern = [[\<Buy\>]], group = "FixSideBuy",
              color = { gui = "#50fa7b", cterm = "Green", bold = true } },
            { pattern = [[\<Sell\>]], group = "FixSideSell",
              color = { gui = "#ff5555", cterm = "Red", bold = true } },

            -- Exchanges
            { pattern = [[\<NYSE\>]], group = "FixExchNYSE",
              color = { gui = "#bd93f9", cterm = "Magenta" } },
            { pattern = [[\<NASDAQ\>]], group = "FixExchNASDAQ",
              color = { gui = "#ff79c6", cterm = "Magenta" } },
            { pattern = [[\<LSE\>]], group = "FixExchLSE",
              color = { gui = "#bd93f9", cterm = "Magenta" } },

            -- Instrument Types
            { pattern = [[\<NDS\>]], group = "FixInstNDS",
              color = { gui = "#00aaff", cterm = "Cyan" } },
            { pattern = [[\<NFD\>]], group = "FixInstNFD",
              color = { gui = "#ffaa00", cterm = "Yellow" } },
            { pattern = [[\<CDS\>]], group = "FixInstCDS",
              color = { gui = "#aa00ff", cterm = "Magenta" } },
            { pattern = [[\<IRS\>]], group = "FixInstIRS",
              color = { gui = "#00ffaa", cterm = "Green" } },
          }
        },
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
