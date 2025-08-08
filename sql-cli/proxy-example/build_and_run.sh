#!/bin/bash

echo "Building SQL to Q Translator Demo..."
echo ""

# Check if .NET is installed
if ! command -v dotnet &> /dev/null; then
    echo "dotnet CLI not found. Trying with csc..."
    
    if command -v csc &> /dev/null; then
        csc Program.cs -out:SqlToQDemo.exe
        if [ $? -eq 0 ]; then
            echo "Build successful with csc!"
            echo "Running demo..."
            echo ""
            mono SqlToQDemo.exe
        else
            echo "Build failed with csc"
        fi
    else
        echo "Neither dotnet nor csc found. Please install .NET SDK or Mono."
        exit 1
    fi
else
    # Create a simple project file if it doesn't exist
    if [ ! -f "SqlToQDemo.csproj" ]; then
        cat > SqlToQDemo.csproj << 'EOF'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net6.0</TargetFramework>
  </PropertyGroup>
</Project>
EOF
    fi
    
    # Build and run with dotnet
    dotnet build
    if [ $? -eq 0 ]; then
        echo ""
        echo "Build successful!"
        echo "Running demo..."
        echo ""
        dotnet run
    else
        echo "Build failed"
        exit 1
    fi
fi