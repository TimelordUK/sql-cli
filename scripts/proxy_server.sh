#!/bin/bash

# Advanced script to manage the JSON selector proxy server

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROXY_PROJECT="${PROJECT_ROOT}/proxy/json-selector/JsonSelector/JsonSelector.csproj"
PID_FILE="${PROJECT_ROOT}/.proxy_server.pid"
LOG_FILE="${PROJECT_ROOT}/proxy_server.log"
PORT=5050

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 {start|stop|restart|status|logs|test}"
    echo ""
    echo "Commands:"
    echo "  start    - Start the proxy server in the background"
    echo "  stop     - Stop the proxy server"
    echo "  restart  - Restart the proxy server"
    echo "  status   - Check if the server is running"
    echo "  logs     - Show server logs"
    echo "  test     - Run a test query against the server"
    exit 1
}

check_dotnet() {
    if ! command -v dotnet &> /dev/null; then
        echo -e "${RED}Error: .NET SDK is not installed${NC}"
        echo "Please install .NET SDK from: https://dotnet.microsoft.com/download"
        exit 1
    fi
}

check_project() {
    if [ ! -f "$PROXY_PROJECT" ]; then
        echo -e "${RED}Error: Project file not found at $PROXY_PROJECT${NC}"
        exit 1
    fi
}

is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PID_FILE"
        fi
    fi
    return 1
}

start_server() {
    if is_running; then
        echo -e "${YELLOW}Proxy server is already running (PID: $(cat $PID_FILE))${NC}"
        return 1
    fi

    echo -e "${GREEN}Starting JSON Selector Proxy Server...${NC}"
    echo "Log file: $LOG_FILE"

    cd "$PROJECT_ROOT"
    nohup dotnet run --project "$PROXY_PROJECT" > "$LOG_FILE" 2>&1 &
    PID=$!
    echo $PID > "$PID_FILE"

    # Wait a moment for the server to start
    sleep 3

    # Check if server started successfully
    if is_running; then
        echo -e "${GREEN}✓ Server started successfully (PID: $PID)${NC}"
        echo -e "${GREEN}✓ Server running on: http://localhost:${PORT}${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to start server${NC}"
        echo "Check logs with: $0 logs"
        return 1
    fi
}

stop_server() {
    if ! is_running; then
        echo -e "${YELLOW}Proxy server is not running${NC}"
        return 1
    fi

    PID=$(cat "$PID_FILE")
    echo -e "${YELLOW}Stopping proxy server (PID: $PID)...${NC}"

    kill $PID
    sleep 2

    if is_running; then
        echo -e "${YELLOW}Server didn't stop gracefully, forcing...${NC}"
        kill -9 $PID
    fi

    rm -f "$PID_FILE"
    echo -e "${GREEN}✓ Server stopped${NC}"
}

check_status() {
    if is_running; then
        PID=$(cat "$PID_FILE")
        echo -e "${GREEN}✓ Proxy server is running (PID: $PID)${NC}"

        # Try to check if the server is responding
        if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${PORT}/health" | grep -q "200\|404"; then
            echo -e "${GREEN}✓ Server is responding on port ${PORT}${NC}"
        else
            echo -e "${YELLOW}⚠ Server may not be responding correctly${NC}"
        fi
    else
        echo -e "${RED}✗ Proxy server is not running${NC}"
    fi
}

show_logs() {
    if [ -f "$LOG_FILE" ]; then
        echo "=== Proxy Server Logs ==="
        tail -n 50 "$LOG_FILE"
    else
        echo -e "${YELLOW}No log file found${NC}"
    fi
}

test_server() {
    if ! is_running; then
        echo -e "${RED}Server is not running. Start it first with: $0 start${NC}"
        return 1
    fi

    echo -e "${GREEN}Testing proxy server...${NC}"

    # Check if test data exists
    TEST_FILE="${PROJECT_ROOT}/data/fix_messages.json"
    if [ ! -f "$TEST_FILE" ]; then
        echo -e "${RED}Test file not found: $TEST_FILE${NC}"
        return 1
    fi

    # Run test query
    echo "Sending test query to http://localhost:${PORT}/api/messagequery/example/fix"
    RESPONSE=$(curl -s -X POST "http://localhost:${PORT}/api/messagequery/example/fix" \
        -F "file=@${TEST_FILE}" \
        -w "\nHTTP_STATUS:%{http_code}")

    HTTP_STATUS=$(echo "$RESPONSE" | grep HTTP_STATUS | cut -d':' -f2)
    BODY=$(echo "$RESPONSE" | grep -v HTTP_STATUS)

    if [ "$HTTP_STATUS" = "200" ]; then
        echo -e "${GREEN}✓ Test successful!${NC}"
        echo "Response preview:"
        echo "$BODY" | head -n 10
    else
        echo -e "${RED}✗ Test failed (HTTP $HTTP_STATUS)${NC}"
        echo "Response: $BODY"
    fi
}

# Main script logic
check_dotnet
check_project

case "$1" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        stop_server
        sleep 1
        start_server
        ;;
    status)
        check_status
        ;;
    logs)
        show_logs
        ;;
    test)
        test_server
        ;;
    *)
        usage
        ;;
esac