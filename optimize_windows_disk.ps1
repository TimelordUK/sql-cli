# Windows Disk Optimization Script
# Run as Administrator in PowerShell

Write-Host "Windows Disk Optimization for Development" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green

# Function to set page file
function Set-PageFile {
    param(
        [string]$Drive,
        [int]$InitialSize,
        [int]$MaximumSize
    )
    
    $computerSystem = Get-WmiObject -Class Win32_ComputerSystem -EnableAllPrivileges
    
    if ($Drive -eq "C") {
        # Reduce C: drive page file
        Write-Host "Reducing page file on C: drive..." -ForegroundColor Yellow
        $computerSystem.AutomaticManagedPagefile = $false
        $computerSystem.Put() | Out-Null
        
        $pageFileSetting = Get-WmiObject -Query "SELECT * FROM Win32_PageFileSetting WHERE Name='C:\\pagefile.sys'"
        if ($pageFileSetting) {
            $pageFileSetting.InitialSize = $InitialSize
            $pageFileSetting.MaximumSize = $MaximumSize
            $pageFileSetting.Put() | Out-Null
        } else {
            Set-WmiInstance -Class Win32_PageFileSetting -Arguments @{Name="C:\pagefile.sys"; InitialSize=$InitialSize; MaximumSize=$MaximumSize} | Out-Null
        }
    }
    elseif ($Drive -eq "D") {
        # Add page file to D: drive
        Write-Host "Adding page file to D: drive..." -ForegroundColor Yellow
        
        $pageFileSetting = Get-WmiObject -Query "SELECT * FROM Win32_PageFileSetting WHERE Name='D:\\pagefile.sys'"
        if ($pageFileSetting) {
            $pageFileSetting.InitialSize = $InitialSize
            $pageFileSetting.MaximumSize = $MaximumSize
            $pageFileSetting.Put() | Out-Null
        } else {
            Set-WmiInstance -Class Win32_PageFileSetting -Arguments @{Name="D:\pagefile.sys"; InitialSize=$InitialSize; MaximumSize=$MaximumSize} | Out-Null
        }
    }
}

# Step 1: Disable Windows Search Indexing on development folders
Write-Host "`nDisabling Windows Search indexing on development folders..." -ForegroundColor Yellow
$devFolders = @(
    "C:\Users\$env:USERNAME\.cargo",
    "C:\Users\$env:USERNAME\.rustup",
    "D:\WSL",
    "C:\Program Files\Docker"
)

foreach ($folder in $devFolders) {
    if (Test-Path $folder) {
        attrib +I "$folder" /S /D
        Write-Host "  Disabled indexing for: $folder" -ForegroundColor Gray
    }
}

# Step 2: Configure Page Files
Write-Host "`nConfiguring page files..." -ForegroundColor Yellow
Write-Host "  Setting minimal page file on C: (2GB-4GB)" -ForegroundColor Gray
Write-Host "  Setting large page file on D: (40GB-80GB)" -ForegroundColor Gray

# Minimal page file on C: for system stability
Set-PageFile -Drive "C" -InitialSize 2048 -MaximumSize 4096

# Large page file on D: for heavy workloads
Set-PageFile -Drive "D" -InitialSize 40960 -MaximumSize 81920

# Step 3: Disable Superfetch/SysMain for SSDs
Write-Host "`nDisabling SysMain (Superfetch) service..." -ForegroundColor Yellow
Stop-Service -Name "SysMain" -Force -ErrorAction SilentlyContinue
Set-Service -Name "SysMain" -StartupType Disabled

# Step 4: Configure Windows Defender exclusions
Write-Host "`nAdding Windows Defender exclusions for development..." -ForegroundColor Yellow
$exclusions = @(
    "D:\WSL",
    "C:\Users\$env:USERNAME\.cargo",
    "C:\Users\$env:USERNAME\.rustup",
    "$env:TEMP",
    "*.rs",
    "*.toml",
    "cargo.exe",
    "rustc.exe"
)

foreach ($exclusion in $exclusions) {
    if ($exclusion -like "*.*") {
        Add-MpPreference -ExclusionExtension $exclusion -ErrorAction SilentlyContinue
        Write-Host "  Added extension exclusion: $exclusion" -ForegroundColor Gray
    } elseif ($exclusion -like "*.exe") {
        Add-MpPreference -ExclusionProcess $exclusion -ErrorAction SilentlyContinue
        Write-Host "  Added process exclusion: $exclusion" -ForegroundColor Gray
    } else {
        Add-MpPreference -ExclusionPath $exclusion -ErrorAction SilentlyContinue
        Write-Host "  Added path exclusion: $exclusion" -ForegroundColor Gray
    }
}

# Step 5: Optimize disk for performance
Write-Host "`nOptimizing disk settings..." -ForegroundColor Yellow
fsutil behavior set DisableLastAccess 1
fsutil behavior set EncryptPagingFile 0

# Step 6: Clean temporary files
Write-Host "`nCleaning temporary files..." -ForegroundColor Yellow
Remove-Item "$env:TEMP\*" -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "  Cleaned Windows temp folder" -ForegroundColor Gray

# Run disk cleanup on C:
Write-Host "  Running disk cleanup on C: drive..." -ForegroundColor Gray
cleanmgr /d C: /sageset:1
cleanmgr /d C: /sagerun:1

Write-Host "`nOptimization complete!" -ForegroundColor Green
Write-Host "Please restart your computer for all changes to take effect." -ForegroundColor Yellow
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Run move_wsl_to_d.ps1 to move WSL to D: drive" -ForegroundColor White
Write-Host "2. Copy .wslconfig to C:\Users\$env:USERNAME\.wslconfig" -ForegroundColor White
Write-Host "3. Restart WSL with: wsl --shutdown" -ForegroundColor White
Write-Host "4. Restart Windows" -ForegroundColor White