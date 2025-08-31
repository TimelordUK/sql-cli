# RustRover Memory Configuration Fix for WSL2

## Windows Side Configuration (Where RustRover Actually Runs)

### 1. Find the VM Options File
Look for one of these files on Windows:
- `C:\Users\<YourUsername>\AppData\Roaming\JetBrains\RustRover<version>\rustrover64.exe.vmoptions`
- Or in RustRover installation: `C:\Program Files\JetBrains\RustRover <version>\bin\rustrover64.exe.vmoptions`

### 2. Create/Edit Custom VM Options
**Easier method through IDE:**
1. In RustRover: Help → Edit Custom VM Options
2. This creates a user-specific file that overrides defaults

### 3. Recommended Settings for Large Projects
```
-Xms2048m
-Xmx8192m
-XX:ReservedCodeCacheSize=512m
-XX:+UseG1GC
-XX:SoftRefLRUPolicyMSPerMB=50
-XX:CICompilerCount=2
-XX:+HeapDumpOnOutOfMemoryError
-XX:-OmitStackTraceInFastThrow
-ea
-Dsun.io.useCanonCaches=false
-Djdk.http.auth.tunneling.disabledSchemes=""
-Djdk.attach.allowAttachSelf=true
-Djdk.module.illegalAccess.silent=true
-Dkotlinx.coroutines.debug=off
```

### 4. WSL2 Specific Optimizations
Add these for better WSL2 performance:
```
-Dfile.encoding=UTF-8
-Didea.max.intellisense.filesize=2500
-Didea.max.content.load.filesize=20000
-Didea.cycle.buffer.size=2048
```

## WSL2 Memory Configuration

### 1. Configure WSL2 Memory Limits
Create/edit `C:\Users\<YourUsername>\.wslconfig`:
```ini
[wsl2]
memory=16GB  # Adjust based on your system RAM (leave 4-8GB for Windows)
processors=8  # Number of processors to assign
swap=8GB
swapFile=C:\\temp\\wsl-swap.vhdx  # Optional custom swap location
```

### 2. Restart WSL2
In PowerShell (as Administrator):
```powershell
wsl --shutdown
# Wait a few seconds
wsl
```

## RustRover Project-Specific Settings

### 1. Limit Indexing Scope
- File → Settings → Project Structure
- Mark large directories as "Excluded" (like target/, node_modules/)

### 2. Disable Unnecessary Plugins
- File → Settings → Plugins
- Disable plugins you don't use

### 3. Rust Analyzer Settings
In settings.json or through UI:
```json
{
  "rust-analyzer.cargo.allFeatures": false,
  "rust-analyzer.checkOnSave.allTargets": false,
  "rust-analyzer.procMacro.enable": false,  // If not using proc macros
  "rust-analyzer.diagnostics.disabled": ["unresolved-proc-macro"]
}
```

## Quick Diagnostic Commands

Check current memory usage in WSL2:
```bash
# Check WSL2 memory
free -h

# Check process memory
ps aux | grep rust

# Monitor in real-time
htop
```

## Alternative: Use rust-analyzer with VS Code
If memory issues persist, consider VS Code with rust-analyzer:
- Much lighter memory footprint
- Better WSL2 integration
- Still excellent Rust support

## Emergency Actions if OOM Occurs
1. Clear RustRover caches: File → Invalidate Caches and Restart
2. Increase Windows pagefile size
3. Close other memory-intensive applications
4. Consider using RustRover directly on Windows (not through WSL2)

## Notes
- The 1500MB limit you mentioned is way too low for Rust projects
- Rust analyzer needs significant memory for type inference
- WSL2 adds overhead, so you need more memory than native
- Target folder can be huge - consider using a tmpfs mount for it