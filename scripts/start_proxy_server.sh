#!/bin/bash

# Start the JSON selector proxy server for FIX message querying
# This server provides endpoints for parsing JSON data with custom selectors

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROXY_PROJECT="${PROJECT_ROOT}/proxy/json-selector/JsonSelector/JsonSelector.csproj"

echo "Starting JSON Selector Proxy Server..."
echo "Project: ${PROXY_PROJECT}"
echo "Server will run on: http://localhost:5050"
echo ""

# Check if dotnet is installed
if ! command -v dotnet &> /dev/null; then
    echo "Error: .NET SDK is not installed"
    echo "Please install .NET SDK from: https://dotnet.microsoft.com/download"
    exit 1
fi

# Check if the project file exists
if [ ! -f "$PROXY_PROJECT" ]; then
    echo "Error: Project file not found at $PROXY_PROJECT"
    exit 1
fi

echo "Building and running the proxy server..."
echo "Press Ctrl+C to stop the server"
echo "----------------------------------------"

# Run the server
cd "${PROJECT_ROOT}"
dotnet run --project "$PROXY_PROJECT"