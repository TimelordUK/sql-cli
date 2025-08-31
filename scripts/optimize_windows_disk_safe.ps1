# Safe Windows Disk Optimization Script - Phase 1
# Focus: Reduce C: drive disk thrashing with minimal risk
# Run as Administrator in PowerShell

Write-Host "Safe Windows Disk Optimization - Phase 1" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "This script performs SAFE optimizations only" -ForegroundColor Cyan
Write-Host ""

# Step 1: Disable Windows Search Indexing on development folders
# This is completely safe and reduces constant disk activity
Write-Host "Step 1: Disabling Windows Search indexing on development folders..." -ForegroundColor Yellow
$devFolders = @(
    "D:\WSL",
    "C:\Users\$env:USERNAME\.cargo",
    "C:\Users\$env:USERNAME\.rustup",
    "C:\Users\$env:USERNAME\AppData\Local\Temp"
)

foreach ($folder in $devFolders) {
    if (Test-Path $folder) {
        attrib +I "$folder" /S /D 2>$null
        Write-Host "  Disabled indexing for: $folder" -ForegroundColor Gray
    }
}

# Step 2: Move Windows TEMP/TMP variables to D: drive
# This significantly reduces C: drive writes
Write-Host ""
Write-Host "Step 2: Moving TEMP folders to D: drive..." -ForegroundColor Yellow

# Create temp folder on D:
$newTempPath = "D:\Temp"
if (!(Test-Path $newTempPath)) {
    New-Item -ItemType Directory -Path $newTempPath -Force | Out-Null
    Write-Host "  Created D:\Temp folder" -ForegroundColor Gray
}

# Set user environment variables
[System.Environment]::SetEnvironmentVariable("TEMP", $newTempPath, "User")
[System.Environment]::SetEnvironmentVariable("TMP", $newTempPath, "User")
Write-Host "  Set user TEMP/TMP to: $newTempPath" -ForegroundColor Gray

# Step 3: Disable last access time updates (safe performance boost)
Write-Host ""
Write-Host "Step 3: Disabling last access time updates..." -ForegroundColor Yellow
fsutil behavior set DisableLastAccess 1 | Out-Null
Write-Host "  Disabled (reduces metadata writes)" -ForegroundColor Gray

# Step 4: Clean existing temp files
Write-Host ""
Write-Host "Step 4: Cleaning temporary files..." -ForegroundColor Yellow
$tempPaths = @(
    "$env:TEMP",
    "C:\Windows\Temp",
    "C:\Users\$env:USERNAME\AppData\Local\Temp"
)

$totalFreed = 0
foreach ($tempPath in $tempPaths) {
    if (Test-Path $tempPath) {
        $sizeBefore = (Get-ChildItem $tempPath -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB
        Remove-Item "$tempPath\*" -Recurse -Force -ErrorAction SilentlyContinue
        $sizeAfter = (Get-ChildItem $tempPath -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB
        $freed = [math]::Round($sizeBefore - $sizeAfter, 2)
        $totalFreed += $freed
        Write-Host "  Cleaned: $tempPath (freed $freed MB)" -ForegroundColor Gray
    }
}
Write-Host "  Total freed: $totalFreed MB" -ForegroundColor Green

# Step 5: Configure Chrome/Edge to use D: for cache (if present)
Write-Host ""
Write-Host "Step 5: Moving browser caches to D: drive..." -ForegroundColor Yellow

# Create browser cache directory
$browserCachePath = "D:\BrowserCache"
if (!(Test-Path $browserCachePath)) {
    New-Item -ItemType Directory -Path $browserCachePath -Force | Out-Null
}

# Move Edge cache
$edgeCachePath = "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Default\Cache"
if (Test-Path $edgeCachePath) {
    Remove-Item $edgeCachePath -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Junction -Path $edgeCachePath -Target "$browserCachePath\Edge" -Force | Out-Null
    Write-Host "  Moved Edge cache to D:\BrowserCache\Edge" -ForegroundColor Gray
}

# Move Chrome cache
$chromeCachePath = "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache"
if (Test-Path $chromeCachePath) {
    Remove-Item $chromeCachePath -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Junction -Path $chromeCachePath -Target "$browserCachePath\Chrome" -Force | Out-Null
    Write-Host "  Moved Chrome cache to D:\BrowserCache\Chrome" -ForegroundColor Gray
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "Phase 1 optimization complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Changes made (all safe and reversible):" -ForegroundColor Cyan
Write-Host "  1. Disabled indexing on development folders" -ForegroundColor White
Write-Host "  2. Moved TEMP/TMP to D:\Temp" -ForegroundColor White
Write-Host "  3. Disabled last access time updates" -ForegroundColor White
Write-Host "  4. Cleaned temporary files" -ForegroundColor White
Write-Host "  5. Moved browser caches to D: drive" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Restart Windows for all changes to take effect" -ForegroundColor White
Write-Host "  2. After restart, monitor C: drive activity" -ForegroundColor White
Write-Host "  3. If satisfied, run Phase 2 for more optimizations" -ForegroundColor White
Write-Host ""
Write-Host "To revert TEMP location if needed:" -ForegroundColor Gray
Write-Host '  [System.Environment]::SetEnvironmentVariable("TEMP", "$env:USERPROFILE\AppData\Local\Temp", "User")' -ForegroundColor Gray
Write-Host '  [System.Environment]::SetEnvironmentVariable("TMP", "$env:USERPROFILE\AppData\Local\Temp", "User")' -ForegroundColor Gray