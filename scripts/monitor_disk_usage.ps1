# Disk Usage Monitoring Script
# Run in PowerShell to identify what's causing 100% disk usage

Write-Host "Disk Usage Monitor - Finding the culprit" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "Press Ctrl+C to stop monitoring" -ForegroundColor Yellow
Write-Host ""

while ($true) {
    Clear-Host
    Write-Host "=== DISK USAGE REPORT ===" -ForegroundColor Cyan
    Write-Host "Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
    Write-Host ""
    
    # Get disk activity
    $diskCounters = Get-Counter "\PhysicalDisk(*)\% Disk Time", "\PhysicalDisk(*)\Disk Bytes/sec" -ErrorAction SilentlyContinue
    
    Write-Host "DISK ACTIVITY:" -ForegroundColor Yellow
    foreach ($counter in $diskCounters.CounterSamples) {
        if ($counter.InstanceName -ne "_total" -and $counter.CookedValue -gt 0) {
            $diskName = $counter.InstanceName
            $activity = [math]::Round($counter.CookedValue, 2)
            
            if ($counter.Path -like "*% Disk Time*" -and $activity -gt 50) {
                Write-Host "  Disk $diskName]: $activity% busy" -ForegroundColor Red
            } elseif ($counter.Path -like "*Bytes/sec*" -and $activity -gt 1MB) {
                $mbPerSec = [math]::Round($activity / 1MB, 2)
                Write-Host "  Disk $diskName]: $mbPerSec MB/s" -ForegroundColor Yellow
            }
        }
    }
    
    Write-Host ""
    Write-Host "TOP DISK USERS (by I/O):" -ForegroundColor Yellow
    
    # Get processes with highest disk usage
    $processes = Get-Process | Where-Object { $_.Id -ne 0 } | ForEach-Object {
        try {
            $ioCounters = Get-Counter "\Process($($_.ProcessName))\IO Data Bytes/sec" -ErrorAction SilentlyContinue
            if ($ioCounters) {
                [PSCustomObject]@{
                    Name = $_.ProcessName
                    PID = $_.Id
                    Memory = [math]::Round($_.WorkingSet64 / 1GB, 2)
                    IORate = [math]::Round($ioCounters.CounterSamples[0].CookedValue / 1MB, 2)
                }
            }
        } catch { }
    } | Where-Object { $_.IORate -gt 0 } | Sort-Object IORate -Descending | Select-Object -First 10
    
    foreach ($proc in $processes) {
        $color = if ($proc.IORate -gt 10) { "Red" } elseif ($proc.IORate -gt 1) { "Yellow" } else { "Gray" }
        Write-Host ("  {0,-30} PID:{1,-8} Mem:{2,6}GB  I/O:{3,8}MB/s" -f $proc.Name, $proc.PID, $proc.Memory, $proc.IORate) -ForegroundColor $color
    }
    
    Write-Host ""
    Write-Host "MEMORY USAGE:" -ForegroundColor Yellow
    $mem = Get-WmiObject Win32_OperatingSystem
    $totalMem = [math]::Round($mem.TotalVisibleMemorySize / 1MB, 2)
    $freeMem = [math]::Round($mem.FreePhysicalMemory / 1MB, 2)
    $usedMem = $totalMem - $freeMem
    $usedPercent = [math]::Round(($usedMem / $totalMem) * 100, 1)
    
    Write-Host "  Total: $totalMem GB" -ForegroundColor Gray
    Write-Host "  Used:  $usedMem GB ($usedPercent%)" -ForegroundColor $(if ($usedPercent -gt 80) { "Red" } else { "Green" })
    Write-Host "  Free:  $freeMem GB" -ForegroundColor Gray
    
    # Check for specific problematic services
    Write-Host ""
    Write-Host "PROBLEMATIC SERVICES:" -ForegroundColor Yellow
    
    $services = @(
        @{Name="WSearch"; Display="Windows Search"},
        @{Name="SysMain"; Display="Superfetch/SysMain"},
        @{Name="DiagTrack"; Display="Diagnostics Tracking"},
        @{Name="wsappx"; Display="Windows Store Apps"}
    )
    
    foreach ($svc in $services) {
        $service = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
        if ($service -and $service.Status -eq "Running") {
            $proc = Get-Process -Name $svc.Name -ErrorAction SilentlyContinue
            if ($proc) {
                $mem = [math]::Round($proc.WorkingSet64 / 1MB, 2)
                Write-Host "  $($svc.Display): Running (${mem}MB)" -ForegroundColor Yellow
            } else {
                Write-Host "  $($svc.Display): Running" -ForegroundColor Yellow
            }
        }
    }
    
    # Check WSL2 specific
    $wslProcess = Get-Process -Name "vmmem" -ErrorAction SilentlyContinue
    if ($wslProcess) {
        $wslMem = [math]::Round($wslProcess.WorkingSet64 / 1GB, 2)
        Write-Host ""
        Write-Host "WSL2 MEMORY:" -ForegroundColor Yellow
        Write-Host "  vmmem process: $wslMem GB" -ForegroundColor $(if ($wslMem -gt 40) { "Red" } else { "Green" })
    }
    
    Start-Sleep -Seconds 3
}