#!/bin/bash
# SQL-CLI Token Management Script
# Refreshes configured tokens and sets them as environment variables

set -e

CONFIG_FILE="${HOME}/.config/sql-cli/config.toml"
CACHE_DIR="${HOME}/.cache/sql-cli/tokens"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create cache directory if it doesn't exist
mkdir -p "$CACHE_DIR"

usage() {
    cat << EOF
SQL-CLI Token Management

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    refresh [TOKEN_NAME]    Refresh specific token or all tokens
    list                    List all configured tokens
    status                  Show token status and expiry
    clear                   Clear cached tokens
    export                  Export tokens as environment variables
    help                    Show this help message

Examples:
    $0 refresh              # Refresh all tokens
    $0 refresh JWT_TOKEN    # Refresh specific token
    $0 status              # Show token status
    $0 export > tokens.env # Export to file for sourcing

Configuration:
    Tokens are configured in: $CONFIG_FILE

    Example config:
    [tokens.tokens.JWT_TOKEN]
    description = "UAT environment token"
    refresh_command = "~/.config/sql-cli/get_uat_token.sh"
    lifetime = 3600
EOF
}

# Function to parse token config from TOML
get_token_config() {
    local token_name=$1
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}Config file not found: $CONFIG_FILE${NC}" >&2
        return 1
    fi

    # Extract refresh command for the token (basic parsing)
    # This is simplified - in production you'd use a proper TOML parser
    grep -A3 "\[tokens.tokens.$token_name\]" "$CONFIG_FILE" 2>/dev/null || true
}

# Function to refresh a token
refresh_token() {
    local token_name=$1
    local cache_file="$CACHE_DIR/${token_name}.token"
    local timestamp_file="$CACHE_DIR/${token_name}.timestamp"

    echo -e "${YELLOW}Refreshing token: $token_name${NC}"

    # Get refresh command from config
    local refresh_cmd=$(grep -A5 "\[tokens.tokens.$token_name\]" "$CONFIG_FILE" 2>/dev/null | \
                        grep "refresh_command" | \
                        sed 's/.*refresh_command = //; s/"//g; s/^~/$HOME/')

    if [ -z "$refresh_cmd" ]; then
        echo -e "${RED}No refresh command found for $token_name${NC}"
        return 1
    fi

    # Expand home directory if needed
    refresh_cmd="${refresh_cmd/#\~/$HOME}"

    echo "Running: $refresh_cmd"

    # Execute command and capture token
    if token_value=$(eval "$refresh_cmd" 2>/dev/null); then
        if [ -n "$token_value" ]; then
            # Save token to cache
            echo "$token_value" > "$cache_file"
            chmod 600 "$cache_file"
            date +%s > "$timestamp_file"

            # Also set as environment variable
            export "$token_name=$token_value"

            echo -e "${GREEN}✓ Token $token_name refreshed successfully${NC}"
            return 0
        else
            echo -e "${RED}✗ Command returned empty token${NC}"
            return 1
        fi
    else
        echo -e "${RED}✗ Failed to refresh token $token_name${NC}"
        return 1
    fi
}

# Function to list all configured tokens
list_tokens() {
    echo "Configured tokens:"
    echo "=================="

    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}Config file not found: $CONFIG_FILE${NC}"
        return 1
    fi

    # Extract token names
    grep "^\[tokens.tokens\." "$CONFIG_FILE" 2>/dev/null | \
        sed 's/\[tokens.tokens.//; s/\]//' | \
        while read -r token_name; do
            local desc=$(grep -A2 "\[tokens.tokens.$token_name\]" "$CONFIG_FILE" | \
                        grep "description" | \
                        sed 's/.*description = //; s/"//g')
            echo "  • $token_name: ${desc:-No description}"
        done
}

# Function to show token status
show_status() {
    echo "Token Status"
    echo "============"

    # Get all configured tokens
    grep "^\[tokens.tokens\." "$CONFIG_FILE" 2>/dev/null | \
        sed 's/\[tokens.tokens.//; s/\]//' | \
        while read -r token_name; do
            local cache_file="$CACHE_DIR/${token_name}.token"
            local timestamp_file="$CACHE_DIR/${token_name}.timestamp"

            echo -n "$token_name: "

            if [ -f "$cache_file" ] && [ -f "$timestamp_file" ]; then
                local timestamp=$(cat "$timestamp_file")
                local current_time=$(date +%s)
                local age=$((current_time - timestamp))

                # Get lifetime from config (default 3600)
                local lifetime=$(grep -A5 "\[tokens.tokens.$token_name\]" "$CONFIG_FILE" | \
                                grep "lifetime" | \
                                sed 's/.*lifetime = //' | \
                                grep -o '[0-9]*' || echo "3600")

                if [ "$age" -lt "$lifetime" ]; then
                    local remaining=$((lifetime - age))
                    echo -e "${GREEN}Valid (expires in ${remaining}s)${NC}"
                else
                    echo -e "${RED}Expired (${age}s old)${NC}"
                fi
            else
                echo -e "${YELLOW}Not cached${NC}"
            fi
        done
}

# Function to clear cached tokens
clear_tokens() {
    echo "Clearing cached tokens..."
    rm -f "$CACHE_DIR"/*.token "$CACHE_DIR"/*.timestamp
    echo -e "${GREEN}✓ Token cache cleared${NC}"
}

# Function to export tokens
export_tokens() {
    echo "# SQL-CLI Token Environment Variables"
    echo "# Generated: $(date)"
    echo ""

    for cache_file in "$CACHE_DIR"/*.token; do
        if [ -f "$cache_file" ]; then
            token_name=$(basename "$cache_file" .token)
            token_value=$(cat "$cache_file")
            echo "export ${token_name}='${token_value}'"
        fi
    done
}

# Function to refresh all tokens
refresh_all() {
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}Config file not found: $CONFIG_FILE${NC}"
        return 1
    fi

    # Get all configured tokens
    grep "^\[tokens.tokens\." "$CONFIG_FILE" 2>/dev/null | \
        sed 's/\[tokens.tokens.//; s/\]//' | \
        while read -r token_name; do
            refresh_token "$token_name"
        done
}

# Main command processing
case "${1:-help}" in
    refresh)
        if [ -n "$2" ]; then
            refresh_token "$2"
        else
            refresh_all
        fi
        ;;
    list)
        list_tokens
        ;;
    status)
        show_status
        ;;
    clear)
        clear_tokens
        ;;
    export)
        export_tokens
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo "Unknown command: $1"
        usage
        exit 1
        ;;
esac