# Example Production token getter script for Windows PowerShell
# This is an example that users can copy and customize

# For testing, return a mock token with timestamp
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
Write-Output "example-prod-token-$timestamp"

# Real-world examples with enhanced security:
#
# 1. Azure AD with subscription switch:
# az account set --subscription "Production"
# $token = az account get-access-token `
#   --resource https://api.prod.example.com `
#   --query accessToken -o tsv
# Write-Output $token
#
# 2. OAuth2 with client certificate:
# $cert = Get-Item Cert:\CurrentUser\My\THUMBPRINT
# $body = @{
#     grant_type = "client_credentials"
#     scope = "prod.api.access"
# }
# $response = Invoke-RestMethod -Uri "https://auth.example.com/oauth/token" `
#   -Method POST -Body $body -Certificate $cert
# Write-Output $response.access_token
#
# 3. Windows Credential Manager with MFA prompt:
# $cred = Get-StoredCredential -Target "PROD-API-Token"
# if (-not $cred) {
#     $mfaCode = Read-Host "Enter MFA code"
#     # Authenticate with MFA
#     $token = & "C:\Program Files\Company\MFAAuth.exe" --code $mfaCode
#     Write-Output $token
# } else {
#     Write-Output $cred.GetNetworkCredential().Password
# }
#
# 4. HashiCorp Vault:
# $env:VAULT_ADDR = "https://vault.company.com"
# $token = vault kv get -field=token secret/prod/api-tokens
# Write-Output $token