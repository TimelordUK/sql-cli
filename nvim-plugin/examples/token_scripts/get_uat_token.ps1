# Example UAT token getter script for Windows PowerShell
# This is an example that users can copy and customize

# For testing, return a mock token with timestamp
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
Write-Output "example-uat-token-$timestamp"

# Real-world examples:
#
# 1. Azure AD token:
# $token = az account get-access-token `
#   --resource https://api.uat.example.com `
#   --query accessToken -o tsv
# Write-Output $token
#
# 2. OAuth2 flow:
# $body = @{
#     grant_type = "client_credentials"
#     client_id = $env:UAT_CLIENT_ID
#     client_secret = $env:UAT_CLIENT_SECRET
# }
# $response = Invoke-RestMethod -Uri "https://auth.example.com/oauth/token" `
#   -Method POST -Body $body
# Write-Output $response.access_token
#
# 3. Windows Credential Manager:
# $cred = Get-StoredCredential -Target "UAT-API-Token"
# Write-Output $cred.GetNetworkCredential().Password
#
# 4. Corporate SSO tool:
# $token = & "C:\Program Files\Company\AuthTool.exe" get-token --env uat
# Write-Output $token