#!/bin/bash
# Test script that generates a mock JWT token with timestamp
# In production, this would fetch a real token from your auth service

# Get current timestamp
timestamp=$(date +"%Y-%m-%d %H:%M:%S")
unix_time=$(date +%s)

# Create a simple mock JWT-like token with timestamp embedded
# Real JWT would have proper header.payload.signature structure
header=$(echo -n '{"alg":"HS256","typ":"JWT"}' | base64 | tr -d '\n')
payload=$(echo -n "{\"sub\":\"test-user\",\"iat\":$unix_time,\"exp\":$(($unix_time + 900)),\"timestamp\":\"$timestamp\"}" | base64 | tr -d '\n')
# Include seconds in signature so we can see it change
signature="test-sig-$(date +%H%M%S)"

# Output ONLY the token to stdout - nothing else
echo "$header.$payload.$signature"