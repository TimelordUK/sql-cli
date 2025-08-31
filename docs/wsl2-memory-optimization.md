# WSL2 Memory Optimization Guide

## Current Setup Analysis
- Total RAM: 64GB
- WSL2 allocated: 40GB
- Windows left with: 24GB
- Problem: Windows is memory-starved with modern IDEs

## Recommended Configurations

### Option 1: Balanced Configuration (Recommended)
```ini
[wsl2]
# More balanced split for 64GB system
memory=28GB  # Reduced from 40GB
processors=20  # Reduced from 24
swap=16GB  # Slightly reduced

# Enable memory reclaim (critical for efficiency)
pageReporting=true  # Return unused memory to Windows
sparseVhd=true  # Sparse VHD for efficient disk usage

# Keep these optimizations
vmIdleTimeout=60000
networkingMode=mirrored
dnsTunneling=true
```

### Option 2: Dynamic Memory Management
```ini
[wsl2]
# Let WSL2 use memory more dynamically
memory=32GB  # Set a reasonable cap
processors=20

# Critical: Enable memory reclaim features
pageReporting=true  # IMPORTANT: Returns unused memory to Windows
sparseVhd=true
autoMemoryReclaim=gradual  # New in recent Windows versions

# Swap on fast drive
swap=24GB
swapFile=D:\\WSL\\swap.vhdx  # Use fastest NVMe
```

### Option 3: Development-Specific Tuning
```ini
[wsl2]
memory=30GB
processors=16  # Give Windows more CPU headroom

# Docker-specific optimizations
kernelCommandLine = cgroup_no_v1=all systemd.unified_cgroup_hierarchy=1
pageReporting=true
sparseVhd=true

# Aggressive memory reclaim for Docker
autoMemoryReclaim=dropcache  # Aggressive cache dropping
```

## Memory-Saving Tips

### 1. WSL2 Side Optimizations

```bash
# Add to ~/.bashrc or ~/.zshrc

# Drop caches periodically (safe operation)
alias drop-caches='echo 3 | sudo tee /proc/sys/vm/drop_caches'

# Check WSL2 memory usage
alias wsl-mem='free -h && echo "---" && ps aux --sort=-%mem | head -10'

# Compact WSL2 VHD (run from PowerShell as admin)
# wsl --shutdown
# diskpart
# select vdisk file="C:\Users\[username]\AppData\Local\Packages\...\ext4.vhdx"
# compact vdisk
```

### 2. Docker Memory Management

Create `/etc/docker/daemon.json` in WSL2:
```json
{
  "storage-driver": "overlay2",
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "default-ulimits": {
    "memlock": {
      "Hard": -1,
      "Soft": -1
    }
  }
}
```

Docker Compose memory limits:
```yaml
services:
  myservice:
    mem_limit: 2g
    mem_reservation: 1g
```

### 3. Windows Side Optimizations

#### Disable Windows Features You Don't Need:
- Windows Search Indexing (if using Everything search)
- Windows Defender real-time scanning for WSL2 folders
- Superfetch/SysMain service
- Windows Updates during work hours

#### RustRover/IntelliJ Optimizations:
Edit `rustRover64.exe.vmoptions`:
```
-Xms2g
-Xmx6g  # Limit max heap to 6GB
-XX:ReservedCodeCacheSize=512m
-XX:+UseG1GC
-XX:SoftRefLRUPolicyMSPerMB=50
-XX:+UnlockDiagnosticVMOptions
-XX:G1PeriodicGCInterval=300000
```

#### Chrome/Edge Flags:
```
chrome://flags
- Enable "Automatic tab discarding"
- Enable "Tab Freeze"
- Limit renderer processes
```

### 4. Monitoring Tools

#### Windows:
```powershell
# Check WSL2 memory usage
wsl --status
Get-Process *wsl* | Select-Object Name, @{n='RAM(GB)';e={$_.WorkingSet64/1GB}}

# RustRover memory
Get-Process rustRover* | Select-Object Name, @{n='RAM(GB)';e={$_.WorkingSet64/1GB}}
```

#### WSL2:
```bash
# Install and use htop
sudo apt install htop
htop

# Memory pressure
cat /proc/pressure/memory

# See biggest memory users
ps aux --sort=-%mem | head -20
```

## Hardware Upgrade Considerations

### If Upgrading to 128GB:
- 2x 64GB DDR5 is often better than 4x 32GB (leaves room for future)
- Ensure matched pairs for dual-channel
- Check QVL (Qualified Vendor List) for Ryzen 9 7950X
- DDR5-5600 is sweet spot for Ryzen 7000

### Alternative: Dedicated Linux Workstation
- Keep Windows machine for IDEs only (32GB)
- Dedicated Linux dev server (32-64GB)
- Use remote development in RustRover/VS Code

## Quick Fix Script

Save as `optimize-wsl.ps1` and run as Administrator:
```powershell
# Compact WSL2 VHD
Write-Host "Shutting down WSL2..."
wsl --shutdown

# Wait for shutdown
Start-Sleep -Seconds 5

# Find and compact VHDs
$vhds = Get-ChildItem -Path "$env:LOCALAPPDATA\Packages" -Filter "ext4.vhdx" -Recurse -ErrorAction SilentlyContinue
foreach ($vhd in $vhds) {
    Write-Host "Compacting $($vhd.FullName)..."
    & diskpart /s <(echo "select vdisk file=`"$($vhd.FullName)`"`ncompact vdisk`nexit")
}

Write-Host "Restarting WSL2..."
wsl --distribution Ubuntu echo "WSL2 Started"
```

## Recommended Immediate Actions

1. **Reduce WSL2 memory to 28-30GB**
2. **Enable pageReporting=true** (critical!)
3. **Limit RustRover to 6GB max heap**
4. **Use swap on fastest NVMe**
5. **Run memory cleanup weekly**

## Long-term Solutions

1. **128GB RAM upgrade** (2x64GB DDR5-5600)
2. **Dedicated Linux machine** for heavy development
3. **Remote development** with local IDE
4. **Container-based development** with resource limits

## Memory Usage Baseline

After optimizations, you should see:
- Windows: 20-30GB used (with IDEs)
- WSL2: 15-25GB used (with Docker)
- Free: 10-15GB buffer

This gives breathing room for both environments.