#!/bin/bash

# Run script for JsonSelector ASP.NET service

echo "Starting JsonSelector service..."
cd JsonSelector
dotnet run

# Service will run until interrupted with Ctrl+C