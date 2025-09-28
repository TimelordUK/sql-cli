#!/bin/bash
# Example Production token getter script for testing
# This is an example that users can copy and customize

# For testing, return a mock token with timestamp
echo "example-prod-token-$(date +%Y%m%d-%H%M%S)"

# Real-world examples with enhanced security:
#
# 1. Azure AD with MFA:
# az account get-access-token \
#   --resource https://api.prod.example.com \
#   --subscription "Production" \
#   --query accessToken -o tsv
#
# 2. OAuth2 with client certificate:
# curl -s -X POST https://auth.example.com/oauth/token \
#   --cert ${PROD_CERT_PATH} \
#   --key ${PROD_KEY_PATH} \
#   -d "grant_type=client_credentials" \
#   -d "scope=prod.api.access" \
#   | jq -r '.access_token'
#
# 3. HashiCorp Vault:
# vault kv get -field=token secret/prod/api-tokens
#
# 4. AWS with MFA:
# aws sts assume-role \
#   --role-arn arn:aws:iam::987654321:role/prod-access \
#   --role-session-name sql-cli-prod \
#   --serial-number arn:aws:iam::123456789:mfa/user \
#   --token-code $(read -p "Enter MFA code: " && echo $REPLY) \
#   --query 'Credentials.SessionToken' \
#   --output text