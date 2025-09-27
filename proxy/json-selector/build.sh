#!/bin/bash

# Build script for JsonSelector ASP.NET service

echo "Building JsonSelector..."
cd JsonSelector
dotnet build

if [ $? -eq 0 ]; then
    echo "Build succeeded!"
else
    echo "Build failed!"
    exit 1
fi