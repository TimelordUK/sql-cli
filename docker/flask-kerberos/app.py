"""
Simple Flask app with Kerberos authentication example.
Demonstrates health endpoint and Kerberos ticket status.
"""
from flask import Flask, jsonify
import subprocess
import os
from datetime import datetime

app = Flask(__name__)


@app.route('/health')
def health():
    """Health check endpoint - returns Kerberos ticket status."""

    # Check if we have a valid Kerberos ticket
    try:
        result = subprocess.run(
            ['klist', '-s'],
            capture_output=True,
            timeout=5
        )
        has_ticket = result.returncode == 0
    except Exception as e:
        has_ticket = False

    # Get ticket details if available
    ticket_info = None
    if has_ticket:
        try:
            result = subprocess.run(
                ['klist'],
                capture_output=True,
                text=True,
                timeout=5
            )
            ticket_info = result.stdout
        except Exception:
            pass

    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.utcnow().isoformat(),
        'kerberos': {
            'has_valid_ticket': has_ticket,
            'ticket_cache': os.environ.get('KRB5CCNAME', 'default'),
            'details': ticket_info
        }
    })


@app.route('/')
def index():
    """Root endpoint - simple welcome message."""
    return jsonify({
        'message': 'Flask + Kerberos Demo',
        'endpoints': {
            '/health': 'Health check with Kerberos status',
            '/': 'This message'
        }
    })


if __name__ == '__main__':
    # Run Flask app
    app.run(host='0.0.0.0', port=5000, debug=False)
