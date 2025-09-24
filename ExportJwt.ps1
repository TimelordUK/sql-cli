#!/usr/bin/env pwsh
# Test script that generates a mock JWT token with timestamp
# In production, this would fetch a real token from your auth service

# Get current timestamp
$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$unixTime = [int][double]::Parse((Get-Date -UFormat %s))

# Create a simple mock JWT-like token with timestamp embedded
# Real JWT would have proper header.payload.signature structure
$header = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('{"alg":"HS256","typ":"JWT"}'))
$payload = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes("{`"sub`":`"test-user`",`"iat`":$unixTime,`"exp`":$($unixTime + 900),`"timestamp`":`"$timestamp`"}`"))
$signature = "test-signature-" + (Get-Date -Format "HHmmss")

# Output ONLY the token to stdout - nothing else
Write-Output "$header.$payload.$signature"