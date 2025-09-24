#!/usr/bin/env pwsh
# Test script that generates a mock JWT token with timestamp
# In production, this would fetch a real token from your auth service
#
# To run this script without loading PowerShell profile:
# pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File .\ExportJwt.ps1
#
# Or as a one-liner command:
# pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& { .\ExportJwt.ps1 }"

# Ensure no extra output
$ErrorActionPreference = "SilentlyContinue"

# Get current timestamp
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

# For Unix timestamp, handle different PowerShell versions
if ($PSVersionTable.PSVersion.Major -ge 7) {
    # PowerShell Core/7+ has built-in Unix time support
    $unixTime = [int](Get-Date -UFormat %s)
} else {
    # Windows PowerShell 5.1 needs manual calculation
    $unixTime = [int]([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())
}

# Create a simple mock JWT-like token with timestamp embedded
# Real JWT would have proper header.payload.signature structure
$header = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('{"alg":"HS256","typ":"JWT"}'))

# Create payload with proper JSON
$payloadJson = @{
    sub = "test-user"
    iat = $unixTime
    exp = $unixTime + 900
    timestamp = $timestamp
} | ConvertTo-Json -Compress

$payload = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($payloadJson))

# Include seconds in signature so we can see it change with each refresh
$signature = "test-sig-" + (Get-Date -Format "HHmmss")

# Clean up base64 padding if needed (standard JWT doesn't use padding)
$header = $header.TrimEnd('=')
$payload = $payload.TrimEnd('=')

# Output ONLY the token to stdout - nothing else
# Use Write-Host to ensure it goes to stdout, not the success stream
$token = "$header.$payload.$signature"
Write-Host $token