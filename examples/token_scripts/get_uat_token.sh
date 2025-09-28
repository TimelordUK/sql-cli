#!/bin/bash
# Example script to get UAT JWT token
# This should output ONLY the token to stdout

# Example 1: Get from a secure credential store
# token=$(secret-tool lookup service myapp env uat)

# Example 2: Call an authentication endpoint
# token=$(curl -s -X POST https://auth.uat.example.com/token \
#   -H "Content-Type: application/json" \
#   -d '{"username":"'$UAT_USER'","password":"'$UAT_PASS'"}' | \
#   jq -r .access_token)

# Example 3: Use Azure CLI
# token=$(az account get-access-token \
#   --resource https://api.uat.example.com \
#   --query accessToken -o tsv)

# Example 4: OAuth2 client credentials
# token=$(curl -s -X POST https://auth.uat.example.com/oauth/token \
#   -d "client_id=${UAT_CLIENT_ID}" \
#   -d "client_secret=${UAT_CLIENT_SECRET}" \
#   -d "grant_type=client_credentials" | \
#   jq -r .access_token)

# For testing, just echo a dummy token
echo "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbnYiOiJ1YXQiLCJleHAiOjE3MzAwMDAwMDB9.uat_token_signature"