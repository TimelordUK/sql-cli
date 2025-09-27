#!/bin/bash

# Build script for JsonSelector ASP.NET service (Release mode)

echo "Building JsonSelector in Release mode..."
cd JsonSelector
dotnet build -c Release

if [ $? -eq 0 ]; then
    echo "Release build succeeded!"
    echo "Output: JsonSelector/bin/Release/net9.0/"
else
    echo "Release build failed!"
    exit 1
fi