-- Example custom syntax configuration for trading data
-- Place this file anywhere and reference it in your Neovim config:
-- syntax = { custom_file = "~/.config/sql-cli/trading_syntax.lua" }

return {
  -- Trading side highlighting
  {
    pattern = [[\<Buy\>]],
    group = "Buy",
    color = {
      gui = "#50fa7b",    -- Bright green
      cterm = "Green",
      bold = true
    }
  },
  {
    pattern = [[\<Sell\>]],
    group = "Sell",
    color = {
      gui = "#ff5555",    -- Bright red
      cterm = "Red",
      bold = true
    }
  },

  -- Instrument types
  {
    pattern = [[\<NDS\>]],
    group = "NDS",
    color = {
      gui = "#8be9fd",    -- Cyan
      cterm = "Cyan"
    }
  },
  {
    pattern = [[\<NFD\>]],
    group = "NFD",
    color = {
      gui = "#f1fa8c",    -- Yellow
      cterm = "Yellow"
    }
  },
  {
    pattern = [[\<CDS\>]],
    group = "CDS",
    color = {
      gui = "#ff79c6",    -- Pink/Magenta
      cterm = "Magenta"
    }
  },
  {
    pattern = [[\<IRS\>]],
    group = "IRS",
    color = {
      gui = "#bd93f9",    -- Purple/Blue
      cterm = "Blue"
    }
  },

  -- Add more custom patterns as needed
  -- {
  --   pattern = [[\<YourPattern\>]],
  --   group = "GroupName",
  --   color = { gui = "#hexcolor", cterm = "ColorName" }
  -- },
}
