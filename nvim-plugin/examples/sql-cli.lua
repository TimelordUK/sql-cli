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
            -- Multi-leg trades (ORDER0001.1, ORDER0001.2, etc.)
            { pattern = [[\w\+\.\d\+]], group = "FixMultiLeg",
              color = { gui = "Cyan", cterm = "Cyan", bold = true } },

            -- Message Types (case-insensitive to match Executionreport, etc.)
            { pattern = [[\c\<ExecutionReport\>]], group = "FixExecReport",
              color = { gui = "Green", cterm = "Green", bold = true } },  -- #50fa7b
            { pattern = [[\c\<AllocationReport\>]], group = "FixAllocReport",
              color = { gui = "Cyan", cterm = "Cyan", bold = true } },  -- #8be9fd
            { pattern = [[\c\<NewOrderSingle\>]], group = "FixNewOrder",
              color = { gui = "Yellow", cterm = "Yellow", bold = true } },  -- #f1fa8c
            { pattern = [[\c\<OrderCancelReject\>]], group = "FixCancelReject",
              color = { gui = "Red", cterm = "Red", bold = true } },  -- #ff5555

            -- Order Status (case-insensitive: Pendingnew, New, Partial, Filled, Cancelled)
            -- Different colors for each state to show what's going on
            { pattern = [[\c\<Pendingnew\>]], group = "FixStatPendingNew",
              color = { gui = "Yellow", cterm = "Yellow" } },  -- #f1fa8c - Warning/waiting
            { pattern = [[\c\<New\>]], group = "FixStatNew",
              color = { gui = "Cyan", cterm = "Cyan" } },  -- #8be9fd - Active/submitted
            { pattern = [[\c\<Partial\>]], group = "FixStatPartial",
              color = { gui = "Magenta", cterm = "Magenta" } },  -- #bd93f9 - In progress
            { pattern = [[\c\<Filled\>]], group = "FixStatFilled",
              color = { gui = "Green", cterm = "Green", bold = true } },  -- #50fa7b - Complete
            { pattern = [[\c\<Cancelled\>]], group = "FixStatCanceled",
              color = { gui = "DarkGray", cterm = "DarkGray" } },  -- #6272a4 - Inactive
            { pattern = [[\c\<Canceled\>]], group = "FixStatCanceled2",
              color = { gui = "DarkGray", cterm = "DarkGray" } },  -- US spelling variant
            { pattern = [[\c\<Rejected\>]], group = "FixStatRejected",
              color = { gui = "Red", cterm = "Red", bold = true } },  -- #ff5555 - Error

            -- Side (Buy/Sell) - Green/Red for quick visual
            { pattern = [[\c\<Buy\>]], group = "FixSideBuy",
              color = { gui = "Green", cterm = "Green", bold = true } },  -- #50fa7b
            { pattern = [[\c\<Sell\>]], group = "FixSideSell",
              color = { gui = "Red", cterm = "Red", bold = true } },  -- #ff5555

            -- Bloomberg Yellow Key Codes
            { pattern = [[\c\<Index\>]], group = "BBGIndex",
              color = { gui = "Yellow", cterm = "Yellow", bold = true } },
            { pattern = [[\c\<Comdty\>]], group = "BBGComdty",
              color = { gui = "Yellow", cterm = "Yellow", bold = true } },
            { pattern = [[\c\<Equity\>]], group = "BBGEquity",
              color = { gui = "Cyan", cterm = "Cyan" } },
            { pattern = [[\c\<Govt\>]], group = "BBGGovt",
              color = { gui = "Green", cterm = "Green" } },
            { pattern = [[\c\<Corp\>]], group = "BBGCorp",
              color = { gui = "Magenta", cterm = "Magenta" } },
            { pattern = [[\c\<Curncy\>]], group = "BBGCurncy",
              color = { gui = "Cyan", cterm = "Cyan", bold = true } },

            -- Futures codes (e.g., FVZ5, ESH4) - 2-3 letters + month code + digit
            -- This is conservative to avoid false positives
            { pattern = [[\<[A-Z]\{2,3\}[FGHJKMNQUVXZ]\d\>]], group = "FuturesCode",
              color = { gui = "Yellow", cterm = "Yellow" } },

            -- Exchanges
            { pattern = [[\c\<NYSE\>]], group = "FixExchNYSE",
              color = { gui = "Magenta", cterm = "Magenta" } },  -- #bd93f9
            { pattern = [[\c\<NASDAQ\>]], group = "FixExchNASDAQ",
              color = { gui = "Magenta", cterm = "Magenta" } },  -- #ff79c6
            { pattern = [[\c\<LSE\>]], group = "FixExchLSE",
              color = { gui = "Magenta", cterm = "Magenta" } },  -- #bd93f9

            -- Instrument Types
            { pattern = [[\c\<NDS\>]], group = "FixInstNDS",
              color = { gui = "Cyan", cterm = "Cyan" } },  -- #00aaff
            { pattern = [[\c\<NFD\>]], group = "FixInstNFD",
              color = { gui = "Yellow", cterm = "Yellow" } },  -- #ffaa00
            { pattern = [[\c\<CDS\>]], group = "FixInstCDS",
              color = { gui = "Magenta", cterm = "Magenta" } },  -- #aa00ff
            { pattern = [[\c\<IRS\>]], group = "FixInstIRS",
              color = { gui = "Green", cterm = "Green" } },  -- #00ffaa
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
