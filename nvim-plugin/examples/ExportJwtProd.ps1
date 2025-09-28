# PowerShell script to get Production JWT token
# Place at: ~/dev/sql-cli/ExportJwtProd.ps1

# IMPORTANT: This script should output ONLY the token to stdout
# No other output, logging, or Write-Host statements

# Example 1: Get from Windows Credential Manager
# $credential = Get-StoredCredential -Target "ProdAPI"
# $token = Get-AuthToken -Credential $credential -Environment "Production"

# Example 2: Call production auth endpoint
# $body = @{
#     grant_type = "client_credentials"
#     client_id = $env:PROD_CLIENT_ID
#     client_secret = $env:PROD_CLIENT_SECRET
# }
#
# $response = Invoke-RestMethod `
#     -Uri "https://auth.prod.example.com/oauth/token" `
#     -Method POST `
#     -Body $body `
#     -ContentType "application/x-www-form-urlencoded"
#
# Write-Output $response.access_token

# Example 3: Use Azure CLI for production
# $token = az account get-access-token `
#     --resource "https://prod.api.example.com" `
#     --subscription "Production" `
#     --query accessToken -o tsv
# Write-Output $token

# Example 4: Call existing corporate token service
# $tokenResponse = Invoke-RestMethod `
#     -Uri "http://internal-auth-service/api/token" `
#     -Method POST `
#     -Headers @{
#         "X-Environment" = "Production"
#         "X-Application" = "sql-cli"
#     } `
#     -UseDefaultCredentials
#
# Write-Output $tokenResponse.token

# For testing, return a dummy production token
Write-Output "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbnYiOiJwcm9kIiwiZXhwIjoxNzMwMDAwMDAwfQ.prod_token_signature_here"