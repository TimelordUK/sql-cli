#!/usr/bin/env python3
"""
Test Trade Server for SQL-CLI Template Testing
Runs on port 5001 to match the macro examples
"""

from flask import Flask, request, jsonify
from datetime import datetime, timedelta
import random

app = Flask(__name__)

# Generate sample trade data
def generate_trades(source=None, trade_date=None):
    sources = [
        "Bloomberg_FIX_FX",
        "Bloomberg_FIX_Equity",
        "Reuters_FX",
        "Manual_Entry",
        "Internal_Trade_System"
    ]

    symbols = ["EUR/USD", "GBP/USD", "USD/JPY", "AAPL", "GOOGL", "MSFT", "GOLD", "OIL"]
    statuses = ["Executed", "Pending", "Cancelled", "Settled"]

    trades = []
    for i in range(1, 51):  # Generate 50 trades
        trade = {
            "TradeId": f"T{i:05d}",
            "Source": random.choice(sources) if not source else source,
            "Symbol": random.choice(symbols),
            "Quantity": random.randint(100, 10000),
            "Price": round(random.uniform(1.0, 1500.0), 4),
            "TradeDate": datetime.now().strftime("%Y-%m-%d"),
            "ExecutionTime": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "Status": random.choice(statuses),
            "Account": f"ACC{random.randint(1000, 9999)}",
            "Currency": random.choice(["USD", "EUR", "GBP"]),
            "Broker": random.choice(["Broker1", "Broker2", "Broker3"])
        }
        trades.append(trade)

    return trades

@app.route('/trades', methods=['GET', 'POST'])
def trades():
    """Main trades endpoint that accepts POST with select/where"""

    if request.method == 'POST':
        try:
            body = request.get_json()
            print(f"Received POST body: {body}")

            # Parse the where clause if provided
            where_clause = body.get('where', '')
            select_clause = body.get('select', '*')

            # Generate trades based on where clause
            source_filter = None
            if 'Source' in where_clause:
                # Extract source from where clause like: Source = "Bloomberg_FIX_FX"
                import re
                match = re.search(r'Source\s*=\s*"([^"]+)"', where_clause)
                if match:
                    source_filter = match.group(1)

            trades = generate_trades(source=source_filter)

            # Apply date filter if present
            if 'TradeDate' in where_clause:
                # For now, return all trades (in real implementation would filter by date)
                pass

            # Return in expected format
            return jsonify({"Result": trades})

        except Exception as e:
            print(f"Error processing request: {e}")
            return jsonify({"error": str(e)}), 400

    # GET request
    return jsonify({"Result": generate_trades()})

@app.route('/counterparty_trades', methods=['GET', 'POST'])
def counterparty_trades():
    """Counterparty trades for reconciliation"""
    return jsonify({"Result": generate_trades()})

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "healthy", "timestamp": datetime.now().isoformat()})

@app.route('/protected/data', methods=['GET'])
def protected_data():
    """Protected endpoint that checks for Bearer token"""
    auth_header = request.headers.get('Authorization')

    if not auth_header or not auth_header.startswith('Bearer '):
        return jsonify({"error": "Unauthorized"}), 401

    token = auth_header.replace('Bearer ', '')
    print(f"Received token: {token}")

    return jsonify({
        "Result": generate_trades(),
        "authenticated": True,
        "token_received": token[:10] + "..." if len(token) > 10 else token
    })

if __name__ == '__main__':
    print("""
╔══════════════════════════════════════════════════════════════╗
║           Test Trade Server for SQL-CLI Templates           ║
╠══════════════════════════════════════════════════════════════╣
║  Running on: http://localhost:5001                          ║
║                                                              ║
║  Endpoints:                                                  ║
║    POST /trades         - Trade data (select/where body)    ║
║    GET  /trades         - All trades                        ║
║    POST /counterparty_trades - For reconciliation           ║
║    GET  /health         - Health check                      ║
║    GET  /protected/data - Requires Bearer token             ║
║                                                              ║
║  Test with:                                                  ║
║    export JWT_TOKEN="test-token-123"                        ║
║    Then use your SQL macros in Neovim!                      ║
╚══════════════════════════════════════════════════════════════╝
    """)

    # Run on port 5001
    app.run(host='0.0.0.0', port=5001, debug=True)