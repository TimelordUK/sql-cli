#!/bin/bash
#
# Docker entrypoint script
# 1. Performs initial Kerberos authentication
# 2. Starts background renewal process
# 3. Launches Flask application
#

set -e

# Configuration
RENEWAL_INTERVAL="${KRB5_RENEWAL_INTERVAL:-25200}"  # 7 hours in seconds
RENEWAL_SCRIPT="/usr/local/bin/kerberos_renew.sh"

echo "========================================="
echo "Flask + Kerberos Container Starting"
echo "========================================="
echo "Renewal interval: $RENEWAL_INTERVAL seconds ($(($RENEWAL_INTERVAL / 3600)) hours)"
echo ""

# Initial Kerberos authentication
echo "Performing initial Kerberos authentication..."
if ! $RENEWAL_SCRIPT; then
    echo "ERROR: Initial Kerberos authentication failed"
    echo "The container will continue, but Kerberos operations will fail"
    echo "Please check your keytab and principal configuration"
    # Don't exit - let the app run so you can debug
fi

echo ""
echo "Starting Kerberos ticket renewal daemon..."
# Start renewal process in background
(
    while true; do
        echo "Sleeping for $RENEWAL_INTERVAL seconds until next renewal..."
        sleep "$RENEWAL_INTERVAL"

        echo "Time to renew Kerberos ticket..."
        if ! $RENEWAL_SCRIPT; then
            echo "WARNING: Kerberos ticket renewal failed"
        fi
    done
) &

RENEWAL_PID=$!
echo "Renewal daemon started with PID: $RENEWAL_PID"

echo ""
echo "========================================="
echo "Starting Flask application..."
echo "========================================="
echo ""

# Start Flask app (this will run in foreground)
exec python /app/app.py
