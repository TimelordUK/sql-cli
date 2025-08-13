#!/bin/bash

# Test script to verify colors work on different terminals

echo "Testing color rendering for table highlighting..."
echo ""
echo "You should see three distinct shades:"
echo ""

# Test the colors we're using
printf "\033[48;2;50;50;50m  Dark Gray (Column)  \033[0m - RGB(50,50,50)\n"
printf "\033[48;5;8m  Light Gray (Row)    \033[0m - Color::DarkGray\n"
printf "\033[43;30m  Yellow (Crosshair)  \033[0m - Yellow bg, Black fg\n"

echo ""
echo "If all three are clearly different, the highlighting will work well!"
echo ""
echo "On Windows Terminal, PowerShell, and WSL these should all render correctly."