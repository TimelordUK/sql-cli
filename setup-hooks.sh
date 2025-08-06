#!/bin/bash

# Script to set up git hooks for the project

echo "Setting up git hooks..."

# Configure git to use the .githooks directory
git config core.hooksPath .githooks

echo "✅ Git hooks configured!"
echo ""
echo "The pre-commit hook will now:"
echo "  • Check Rust formatting with 'cargo fmt --check'"
echo "  • Prevent commits if code is not formatted"
echo ""
echo "To format your code before committing:"
echo "  cd sql-cli && cargo fmt"
echo ""
echo "To disable hooks temporarily:"
echo "  git commit --no-verify"
echo ""
echo "To disable hooks permanently:"
echo "  git config --unset core.hooksPath"