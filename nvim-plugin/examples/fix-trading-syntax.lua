-- FIX Trading Message Syntax Highlighting Example
-- Shows how to add custom color coding for FIX protocol messages
--
-- Place this in your lazy.nvim config or after the sql-cli.setup() call

return {
  dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
  name = "sql-cli.nvim",
  lazy = false,
  config = function()
    require('sql-cli').setup({
      command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
      output_format = "table",

      -- Custom syntax highlighting for FIX messages
      syntax = {
        patterns = {
          -- Message Types (MsgType field values)
          -- Execution types
          { pattern = [[\<ExecutionReport\>]], group = "FixExecutionReport",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },
          { pattern = [[\<AllocationReport\>]], group = "FixAllocationReport",
            color = { gui = "#8be9fd", cterm = "Cyan", bold = true } },
          { pattern = [[\<OrderCancelReject\>]], group = "FixOrderCancelReject",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },
          { pattern = [[\<NewOrderSingle\>]], group = "FixNewOrderSingle",
            color = { gui = "#f1fa8c", cterm = "Yellow", bold = true } },

          -- Order Status (OrdStatus field)
          { pattern = [[\<PendingNew\>]], group = "FixPendingNew",
            color = { gui = "#f1fa8c", cterm = "Yellow" } },
          { pattern = [[\<New\>]], group = "FixNew",
            color = { gui = "#50fa7b", cterm = "Green" } },
          { pattern = [[\<PartiallyFilled\>]], group = "FixPartiallyFilled",
            color = { gui = "#8be9fd", cterm = "Cyan" } },
          { pattern = [[\<Filled\>]], group = "FixFilled",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },
          { pattern = [[\<Canceled\>]], group = "FixCanceled",
            color = { gui = "#6272a4", cterm = "DarkGray" } },
          { pattern = [[\<Rejected\>]], group = "FixRejected",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          -- Side (Buy/Sell)
          { pattern = [[\<Buy\>]], group = "FixBuy",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },
          { pattern = [[\<Sell\>]], group = "FixSell",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          -- Exchange codes (if you have specific exchanges)
          { pattern = [[\<NYSE\>]], group = "FixExchangeNYSE",
            color = { gui = "#bd93f9", cterm = "Magenta" } },
          { pattern = [[\<NASDAQ\>]], group = "FixExchangeNASDAQ",
            color = { gui = "#ff79c6", cterm = "Magenta" } },
          { pattern = [[\<LSE\>]], group = "FixExchangeLSE",
            color = { gui = "#bd93f9", cterm = "Magenta" } },

          -- Common FIX field tags (if you show tag=value format)
          { pattern = [[35=]], group = "FixTagMsgType",
            color = { gui = "#f1fa8c", cterm = "Yellow", bold = true } },
          { pattern = [[54=]], group = "FixTagSide",
            color = { gui = "#8be9fd", cterm = "Cyan", bold = true } },
          { pattern = [[39=]], group = "FixTagOrdStatus",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          -- Instrument types (if applicable)
          { pattern = [[\<NDS\>]], group = "FixInstrumentNDS",
            color = { gui = "#00aaff", cterm = "Cyan" } },
          { pattern = [[\<NFD\>]], group = "FixInstrumentNFD",
            color = { gui = "#ffaa00", cterm = "Yellow" } },
          { pattern = [[\<CDS\>]], group = "FixInstrumentCDS",
            color = { gui = "#aa00ff", cterm = "Magenta" } },
          { pattern = [[\<IRS\>]], group = "FixInstrumentIRS",
            color = { gui = "#00ffaa", cterm = "Green" } },
        }
      },
    })
  end,
}
