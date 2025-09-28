#!/usr/bin/env python3
"""Simple HTTP server to test Redis caching with trades CSV"""

from flask import Flask, send_file, jsonify
import time

app = Flask(__name__)

# Track number of requests to verify caching
request_count = 0

@app.route('/trades.csv')
def serve_trades():
    global request_count
    request_count += 1
    print(f"[Request #{request_count}] Serving trades.csv - if cache is working, should only see this once")

    # Add a small delay to simulate network latency
    time.sleep(0.5)

    return send_file('/home/me/dev/sql-cli/data/large_trades.csv',
                     mimetype='text/csv',
                     as_attachment=False)

@app.route('/status')
def status():
    return jsonify({
        'request_count': request_count,
        'message': f'Trades CSV has been requested {request_count} times'
    })

if __name__ == '__main__':
    print("Starting test server on http://localhost:5002")
    print("Trades available at: http://localhost:5002/trades.csv")
    print("Check status at: http://localhost:5002/status")
    app.run(host='0.0.0.0', port=5002, debug=True)