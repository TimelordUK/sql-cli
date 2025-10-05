-- Complete FIX Trading Setup Example
-- This shows a real-world configuration for querying FIX messages with syntax highlighting
--
-- Prerequisites:
-- 1. Your FIX data source (CSV/JSON/DB) with resolved enum values
-- 2. sql-cli built and in PATH or specified below

return {
  dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
  name = "sql-cli.nvim",
  lazy = false,
  config = function()
    require('sql-cli').setup({
      command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
      output_format = "table",

      table_output = {
        max_col_width = 30,
        style = "utf8",  -- Nice box drawing for FIX messages
      },

      -- FIX Message Syntax Highlighting
      syntax = {
        patterns = {
          -- ========================================
          -- MESSAGE TYPES (MsgType/35)
          -- ========================================
          { pattern = [[\<ExecutionReport\>]], group = "FixExecReport",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          { pattern = [[\<AllocationReport\>]], group = "FixAllocReport",
            color = { gui = "#8be9fd", cterm = "Cyan", bold = true } },

          { pattern = [[\<AllocationInstruction\>]], group = "FixAllocInstruction",
            color = { gui = "#8be9fd", cterm = "Cyan" } },

          { pattern = [[\<NewOrderSingle\>]], group = "FixNewOrder",
            color = { gui = "#f1fa8c", cterm = "Yellow", bold = true } },

          { pattern = [[\<OrderCancelRequest\>]], group = "FixCancelReq",
            color = { gui = "#ffb86c", cterm = "Yellow" } },

          { pattern = [[\<OrderCancelReject\>]], group = "FixCancelReject",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          { pattern = [[\<BusinessMessageReject\>]], group = "FixReject",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          -- ========================================
          -- ORDER STATUS (OrdStatus/39)
          -- ========================================
          -- Pending states (yellow/warning)
          { pattern = [[\<PendingNew\>]], group = "FixStatPendingNew",
            color = { gui = "#f1fa8c", cterm = "Yellow" } },

          { pattern = [[\<PendingCancel\>]], group = "FixStatPendingCancel",
            color = { gui = "#f1fa8c", cterm = "Yellow" } },

          { pattern = [[\<PendingReplace\>]], group = "FixStatPendingReplace",
            color = { gui = "#f1fa8c", cterm = "Yellow" } },

          -- Active states (green)
          { pattern = [[\<New\>]], group = "FixStatNew",
            color = { gui = "#50fa7b", cterm = "Green" } },

          { pattern = [[\<PartiallyFilled\>]], group = "FixStatPartial",
            color = { gui = "#50fa7b", cterm = "Green" } },

          -- Complete states (bold green)
          { pattern = [[\<Filled\>]], group = "FixStatFilled",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          { pattern = [[\<DoneForDay\>]], group = "FixStatDoneForDay",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          -- Inactive states (red/gray)
          { pattern = [[\<Canceled\>]], group = "FixStatCanceled",
            color = { gui = "#6272a4", cterm = "DarkGray" } },

          { pattern = [[\<Rejected\>]], group = "FixStatRejected",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          { pattern = [[\<Expired\>]], group = "FixStatExpired",
            color = { gui = "#6272a4", cterm = "DarkGray" } },

          -- ========================================
          -- SIDE (Side/54)
          -- ========================================
          { pattern = [[\<Buy\>]], group = "FixSideBuy",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          { pattern = [[\<Sell\>]], group = "FixSideSell",
            color = { gui = "#ff5555", cterm = "Red", bold = true } },

          { pattern = [[\<Cross\>]], group = "FixSideCross",
            color = { gui = "#bd93f9", cterm = "Magenta" } },

          -- ========================================
          -- EXECUTION TYPE (ExecType/150)
          -- ========================================
          { pattern = [[\<New\s*$]], group = "FixExecNew",  -- 'New' at end of line (avoid conflicts)
            color = { gui = "#50fa7b", cterm = "Green" } },

          { pattern = [[\<PartialFill\>]], group = "FixExecPartialFill",
            color = { gui = "#8be9fd", cterm = "Cyan" } },

          { pattern = [[\<Fill\>]], group = "FixExecFill",
            color = { gui = "#50fa7b", cterm = "Green", bold = true } },

          { pattern = [[\<Trade\>]], group = "FixExecTrade",
            color = { gui = "#50fa7b", cterm = "Green" } },

          { pattern = [[\<Canceled\s*$]], group = "FixExecCanceled",
            color = { gui = "#6272a4", cterm = "DarkGray" } },

          { pattern = [[\<Replaced\>]], group = "FixExecReplaced",
            color = { gui = "#ffb86c", cterm = "Yellow" } },

          -- ========================================
          -- EXCHANGES
          -- ========================================
          { pattern = [[\<NYSE\>]], group = "FixExchNYSE",
            color = { gui = "#bd93f9", cterm = "Magenta" } },

          { pattern = [[\<NASDAQ\>]], group = "FixExchNASDAQ",
            color = { gui = "#ff79c6", cterm = "Magenta" } },

          { pattern = [[\<LSE\>]], group = "FixExchLSE",
            color = { gui = "#bd93f9", cterm = "Magenta" } },

          { pattern = [[\<XETRA\>]], group = "FixExchXETRA",
            color = { gui = "#ff79c6", cterm = "Magenta" } },

          -- ========================================
          -- INSTRUMENT TYPES
          -- ========================================
          { pattern = [[\<NDS\>]], group = "FixInstNDS",
            color = { gui = "#00aaff", cterm = "Cyan" } },

          { pattern = [[\<NFD\>]], group = "FixInstNFD",
            color = { gui = "#ffaa00", cterm = "Yellow" } },

          { pattern = [[\<CDS\>]], group = "FixInstCDS",
            color = { gui = "#aa00ff", cterm = "Magenta" } },

          { pattern = [[\<IRS\>]], group = "FixInstIRS",
            color = { gui = "#00ffaa", cterm = "Green" } },

          -- ========================================
          -- TIME IN FORCE (TimeInForce/59)
          -- ========================================
          { pattern = [[\<Day\>]], group = "FixTIFDay",
            color = { gui = "#8be9fd", cterm = "Cyan" } },

          { pattern = [[\<GTC\>]], group = "FixTIFGTC",
            color = { gui = "#8be9fd", cterm = "Cyan", bold = true } },

          { pattern = [[\<IOC\>]], group = "FixTIFIOC",
            color = { gui = "#f1fa8c", cterm = "Yellow" } },

          { pattern = [[\<FOK\>]], group = "FixTIFFOK",
            color = { gui = "#ffb86c", cterm = "Yellow" } },
        }
      },
    })

    -- Example query mappings for FIX data
    -- Bind these to convenient keys for quick access
    local function query_fix_executions()
      -- Example query that resolves FIX enums to readable names
      local query = [[
        SELECT
          CASE msg_type
            WHEN '8' THEN 'ExecutionReport'
            WHEN 'J' THEN 'AllocationReport'
            WHEN 'D' THEN 'NewOrderSingle'
            ELSE msg_type
          END as msg_type,
          symbol,
          CASE side
            WHEN '1' THEN 'Buy'
            WHEN '2' THEN 'Sell'
            WHEN '3' THEN 'Cross'
          END as side,
          CASE ord_status
            WHEN '0' THEN 'New'
            WHEN '1' THEN 'PartiallyFilled'
            WHEN '2' THEN 'Filled'
            WHEN '4' THEN 'Canceled'
            WHEN '8' THEN 'Rejected'
            WHEN 'A' THEN 'PendingNew'
          END as status,
          CASE exec_type
            WHEN '0' THEN 'New'
            WHEN '1' THEN 'PartialFill'
            WHEN '2' THEN 'Fill'
            WHEN '4' THEN 'Canceled'
          END as exec_type,
          last_qty,
          last_px,
          exchange
        FROM fix_messages
        WHERE msg_type = '8'
        ORDER BY transact_time DESC
        LIMIT 100
      ]]

      -- Set this to the register and let user execute
      vim.fn.setreg('+', query)
      vim.notify("FIX executions query copied to clipboard", vim.log.levels.INFO)
    end

    -- Optional: Create user command for quick FIX queries
    vim.api.nvim_create_user_command('FixExecutions', query_fix_executions, {})

    vim.notify("SQL-CLI configured with FIX message syntax highlighting", vim.log.levels.INFO)
  end,
}
