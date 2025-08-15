#!/bin/bash

clear
echo "═══════════════════════════════════════════════════════════════"
echo "          SQL CLI Action System Debug Tools"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "We have two tools to help understand the action system:"
echo ""
echo "1. action_logger  - Simple console output, shows key -> action mapping"
echo "2. action_debugger - Full TUI with history and state tracking"
echo ""
echo "Which would you like to run?"
echo ""
echo "  1) Simple Logger (recommended for first time)"
echo "  2) Full Debugger (TUI interface)"
echo "  3) Exit"
echo ""
read -p "Choice [1-3]: " choice

case $choice in
    1)
        echo ""
        echo "Starting Action Logger..."
        echo "This shows how each key maps to an action."
        echo ""
        sleep 1
        ./target/debug/action_logger
        ;;
    2)
        echo ""
        echo "Starting Action Debugger..."
        echo "This provides a full TUI to explore the action system."
        echo ""
        sleep 1
        ./target/debug/action_debugger
        ;;
    3)
        echo "Goodbye!"
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "Demo complete!"