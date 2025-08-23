# PowerShell script to move WSL2 from C: to D: drive
# Run this in Windows PowerShell as Administrator

Write-Host "WSL2 Migration Script - Moving to D: Drive" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green

# Step 1: Check current WSL distributions
Write-Host "`nCurrent WSL distributions:" -ForegroundColor Yellow
wsl --list --verbose

# Step 2: Get distribution name
$distroName = Read-Host "`nEnter the name of your WSL distribution (usually 'Ubuntu' or similar)"

# Validate distribution name
if ([string]::IsNullOrWhiteSpace($distroName)) {
    Write-Host "Error: Distribution name cannot be empty!" -ForegroundColor Red
    exit 1
}

# Check if the distribution exists
$wslList = wsl --list --quiet
if ($wslList -notcontains $distroName) {
    Write-Host "Error: Distribution '$distroName' not found!" -ForegroundColor Red
    Write-Host "Available distributions:" -ForegroundColor Yellow
    wsl --list --quiet
    exit 1
}

# Step 3: Create backup directory on D: drive
$backupPath = "D:\WSL\Backups"
$installPath = "D:\WSL\Installations"

Write-Host "`nCreating directories on D: drive..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $backupPath | Out-Null
New-Item -ItemType Directory -Force -Path $installPath | Out-Null

# Step 4: Export the distribution
$backupFile = "$backupPath\$distroName-backup.tar"
Write-Host "`nExporting $distroName to $backupFile..." -ForegroundColor Yellow
Write-Host "This may take several minutes depending on size..." -ForegroundColor Cyan
wsl --export $distroName $backupFile

if ($LASTEXITCODE -ne 0) {
    Write-Host "Export failed! Exiting..." -ForegroundColor Red
    exit 1
}

Write-Host "Export completed successfully!" -ForegroundColor Green

# Step 5: Unregister the distribution from C: drive
Write-Host "`nUnregistering $distroName from C: drive..." -ForegroundColor Yellow
Write-Host "WARNING: This will remove the distribution from C:" -ForegroundColor Red
$confirm = Read-Host "Continue? (yes/no)"

if ($confirm -eq "yes") {
    wsl --unregister $distroName
    Write-Host "Unregistered successfully!" -ForegroundColor Green
} else {
    Write-Host "Cancelled. Your backup is at: $backupFile" -ForegroundColor Yellow
    exit 0
}

# Step 6: Import the distribution to D: drive
$installDir = "$installPath\$distroName"
Write-Host "`nImporting $distroName to $installDir..." -ForegroundColor Yellow
Write-Host "This may take several minutes..." -ForegroundColor Cyan
wsl --import $distroName $installDir $backupFile --version 2

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nMigration completed successfully!" -ForegroundColor Green
    Write-Host "Your WSL distribution is now on D: drive" -ForegroundColor Green
    
    # Step 7: Set as default if needed
    $setDefault = Read-Host "`nSet $distroName as default distribution? (yes/no)"
    if ($setDefault -eq "yes") {
        wsl --set-default $distroName
        Write-Host "Set as default distribution!" -ForegroundColor Green
    }
    
    Write-Host "`nYou can now start WSL with: wsl" -ForegroundColor Cyan
    Write-Host "Backup kept at: $backupFile" -ForegroundColor Yellow
} else {
    Write-Host "Import failed! Your backup is at: $backupFile" -ForegroundColor Red
    Write-Host "You can manually import with:" -ForegroundColor Yellow
    Write-Host "wsl --import $distroName $installDir $backupFile --version 2" -ForegroundColor White
}