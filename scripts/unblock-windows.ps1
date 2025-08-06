# Unblock SQL CLI on Windows
# Run this script as Administrator if Windows Defender blocks the executable

$exePath = ".\sql-cli.exe"

if (Test-Path $exePath) {
    # Unblock the file
    Unblock-File -Path $exePath
    
    # Add exclusion for this specific file (requires admin)
    try {
        Add-MpPreference -ExclusionPath (Resolve-Path $exePath).Path
        Write-Host "Successfully unblocked sql-cli.exe" -ForegroundColor Green
        Write-Host "You may need to restart your terminal" -ForegroundColor Yellow
    } catch {
        Write-Host "Could not add Windows Defender exclusion (requires admin)" -ForegroundColor Yellow
        Write-Host "But the file has been unblocked for current user" -ForegroundColor Green
    }
} else {
    Write-Host "sql-cli.exe not found in current directory" -ForegroundColor Red
    Write-Host "Please run this script from the directory containing sql-cli.exe" -ForegroundColor Yellow
}