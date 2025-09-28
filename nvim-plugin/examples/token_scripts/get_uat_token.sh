#!/bin/bash
# Example UAT token getter script for testing
# This is an example that users can copy and customize

# For testing, return a mock token with timestamp
echo "example-uat-token-$(date +%Y%m%d-%H%M%S)"

# Real-world examples:
#
# 1. Azure AD token:
# az account get-access-token \
#   --resource https://api.uat.example.com \
#   --query accessToken -o tsv
#
# 2. OAuth2 flow:
# curl -s -X POST https://auth.example.com/oauth/token \
#   -d "grant_type=client_credentials" \
#   -d "client_id=${UAT_CLIENT_ID}" \
#   -d "client_secret=${UAT_CLIENT_SECRET}" \
#   | jq -r '.access_token'
#
# 3. AWS STS:
# aws sts assume-role \
#   --role-arn arn:aws:iam::123456789:role/uat-access \
#   --role-session-name sql-cli-session \
#   --query 'Credentials.SessionToken' \
#   --output text