# Token Script Testing Guide

## Test Scripts Provided

### 1. `export_jwt.sh` (Linux/Mac/WSL)
- Bash script that generates a test JWT token
- Updates signature with current time (HHmmss) so you can see it change
- Run directly: `./export_jwt.sh`

### 2. `ExportJwt.ps1` (Windows PowerShell)
- PowerShell script that generates identical test JWT tokens
- Works with both PowerShell 5.1 and PowerShell Core 7+
- Updates signature with current time so you can see changes

## Running PowerShell Script on Windows

### Method 1: PowerShell Core (pwsh) - Recommended
```powershell
# Full command with all flags
pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File .\ExportJwt.ps1

# Shorter version if execution policy allows
pwsh -NoProfile -File .\ExportJwt.ps1
```

### Method 2: Windows PowerShell 5.1
```powershell
# Full command with all flags
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File .\ExportJwt.ps1

# As a command (useful for testing)
powershell.exe -NoProfile -Command "& { .\ExportJwt.ps1 }"
```

### Flags Explained:
- `-NoProfile` - Don't load PowerShell profile (faster, no side effects)
- `-NonInteractive` - Don't prompt for input
- `-ExecutionPolicy Bypass` - Allow script execution without changing system policy
- `-File` - Execute a script file

## Testing in Neovim

### For Windows:
Edit `~/.config/nvim/lua/plugins/sql-cli.lua` and uncomment the appropriate line:

```lua
-- For PowerShell Core (recommended):
token_command = "pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),

-- For Windows PowerShell 5.1:
token_command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File " .. vim.fn.expand("~/dev/sql-cli/ExportJwt.ps1"),
```

### Test Commands in Neovim:
1. `:TokenRefresh` - Manually run the script and show token preview
2. `:TokenShow` - Display current token details
3. `:TokenConfig` - Show configuration and timer status
4. `:TokenDebug` - Toggle debug mode to see timer activity

## Token Output Format

Both scripts generate tokens with this structure:
```
header.payload.signature

Where signature = "test-sig-HHMMSS"
```

Example:
- At 14:30:45: `...test-sig-143045`
- At 14:30:55: `...test-sig-143055`

This makes it easy to verify that new tokens are being fetched.

## Troubleshooting Windows

### Script Not Running?
1. Check execution policy: `Get-ExecutionPolicy`
2. Try with full path: `C:\full\path\to\ExportJwt.ps1`
3. Verify PowerShell version: `$PSVersionTable.PSVersion`

### No Output?
1. Run script directly in PowerShell to see errors
2. Remove `$ErrorActionPreference = "SilentlyContinue"` temporarily
3. Check if Write-Host works: `pwsh -NoProfile -Command "Write-Host 'test'"`

### Timer Not Working?
1. Enable debug: `:TokenDebug`
2. Check config: `:TokenConfig`
3. Manually test: `:TokenRefresh`
4. Check Neovim messages: `:messages`

## Production Usage

For your actual token script at work:

1. Create a script that outputs ONLY the token
2. Test it manually first: `pwsh -NoProfile -File YourScript.ps1`
3. Configure in Neovim with appropriate refresh interval (e.g., 840 seconds for 14 minutes)
4. Disable debug mode for production use