#!/bin/bash
# Example script to get Production JWT token
# This should output ONLY the token to stdout

# IMPORTANT: Use proper authentication for production!

# Example 1: Use HashiCorp Vault
# token=$(vault kv get -field=token secret/prod/jwt)

# Example 2: AWS Secrets Manager
# token=$(aws secretsmanager get-secret-value \
#   --secret-id prod/jwt-token \
#   --query SecretString --output text)

# Example 3: Use certificate-based auth
# token=$(curl -s --cert /path/to/client.crt \
#   --key /path/to/client.key \
#   https://auth.prod.example.com/token | \
#   jq -r .access_token)

# Example 4: Use Kerberos/SPNEGO
# token=$(curl -s --negotiate -u : \
#   https://auth.prod.example.com/api/token)

# For testing, just echo a dummy token
echo "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbnYiOiJwcm9kIiwiZXhwIjoxNzMwMDAwMDAwfQ.prod_token_signature"