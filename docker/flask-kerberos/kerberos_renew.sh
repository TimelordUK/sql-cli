#!/bin/bash
#
# Kerberos ticket renewal script
# Authenticates using keytab and logs the result
#

set -e

# Configuration
PRINCIPAL="${KRB5_PRINCIPAL:-user@EXAMPLE.COM}"
KEYTAB="${KRB5_KEYTAB:-/etc/krb5.keytab}"
LOGFILE="${KRB5_LOGFILE:-/var/log/krb5_renew.log}"

# Ensure log directory exists
mkdir -p "$(dirname "$LOGFILE")"

# Log function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOGFILE"
}

log "=== Kerberos Ticket Renewal Started ==="
log "Principal: $PRINCIPAL"
log "Keytab: $KEYTAB"

# Check if keytab exists
if [ ! -f "$KEYTAB" ]; then
    log "ERROR: Keytab file not found at $KEYTAB"
    log "Please create a keytab file and mount it to the container"
    exit 1
fi

# Get ticket using keytab
if kinit -kt "$KEYTAB" "$PRINCIPAL" 2>&1 | tee -a "$LOGFILE"; then
    log "SUCCESS: Kerberos ticket renewed successfully"

    # Display ticket information
    log "Current ticket status:"
    klist 2>&1 | tee -a "$LOGFILE"
else
    log "ERROR: Failed to renew Kerberos ticket"
    exit 1
fi

log "=== Kerberos Ticket Renewal Completed ==="
