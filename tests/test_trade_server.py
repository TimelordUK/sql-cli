#!/usr/bin/env python3
"""
Test Trade Server for SQL-CLI Template Testing
Runs on port 5001 to match the macro examples
"""

from flask import Flask, request, jsonify
from datetime import datetime, timedelta
import random
import time
import hashlib
import traceback

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

            # Handle both uppercase and lowercase field names
            where_clause = body.get('Where') or body.get('where', '')
            select_clause = body.get('Select') or body.get('select', '*')

            print(f"Where clause: {where_clause}")
            print(f"Select clause: {select_clause}")

            # Generate trades based on where clause
            source_filter = None
            if 'Source' in where_clause:
                # Extract source from where clause like: Source = "Bloomberg"
                import re
                match = re.search(r'Source\s*=\s*"([^"]+)"', where_clause)
                if match:
                    source_filter = match.group(1)
                    print(f"Filtered by source: {source_filter}")

            # Parse the select clause to return only requested fields
            selected_fields = []
            if select_clause and select_clause != '*':
                # Split by comma and clean up field names
                selected_fields = [f.strip() for f in select_clause.split(',')]
                print(f"Selected fields: {selected_fields}")

            # Generate mock trades
            all_trades = []

            # Map the requested fields to our mock data structure
            field_mapping = {
                'Source': 'Source',
                'PlatformOrderId': 'OrderId',
                'BloomberTicker': 'Ticker',
                'SignedQuantity': 'Quantity',
                'BuySell': 'Side',
                'Price': 'Price'
            }

            # Generate some mock trades with the requested structure
            for i in range(1, 11):  # Generate 10 trades
                trade = {
                    'Source': source_filter if source_filter else random.choice(["Bloomberg", "Reuters", "TradeWeb"]),
                    'OrderId': f"ORD{random.randint(100000, 999999)}",
                    'Ticker': random.choice(["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"]),
                    'Quantity': random.randint(-5000, 5000),  # Signed quantity
                    'Side': random.choice(["Buy", "Sell"]),
                    'Price': round(random.uniform(100.0, 500.0), 2),
                    'TradeDate': datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                }

                # If specific fields are selected, return only those
                if selected_fields:
                    filtered_trade = {}
                    for field in selected_fields:
                        # Map the field name or use as-is
                        mapped_field = field_mapping.get(field, field)
                        if mapped_field in trade:
                            # Use the original field name in response
                            if field == 'PlatformOrderId':
                                filtered_trade[field] = trade['OrderId']
                            elif field == 'BloomberTicker':
                                filtered_trade[field] = trade['Ticker']
                            elif field == 'SignedQuantity':
                                filtered_trade[field] = trade['Quantity']
                            elif field == 'BuySell':
                                filtered_trade[field] = trade['Side']
                            else:
                                filtered_trade[field] = trade.get(mapped_field, '')
                    all_trades.append(filtered_trade)
                else:
                    all_trades.append(trade)

            # Apply date filter if present
            if 'TradeDate' in where_clause:
                # For now, return all trades (in real implementation would filter by date)
                print("Date filter detected but not applied in mock")

            # Return in expected format
            return jsonify({"Result": all_trades})

        except Exception as e:
            print(f"Error processing request: {e}")
            import traceback
            traceback.print_exc()
            return jsonify({"error": str(e)}), 400

    # GET request
    return jsonify({"Result": generate_trades()})

@app.route('/counterparty_trades', methods=['GET', 'POST'])
def counterparty_trades():
    """Counterparty trades for reconciliation"""
    return jsonify({"Result": generate_trades()})

@app.route('/token', methods=['GET'])
def get_token():
    """Generate a test token that expires in 15 minutes"""
    # Create a simple token (not a real JWT, just for testing)
    timestamp = str(int(time.time()))
    expires = int(time.time()) + 900  # 15 minutes from now

    # Create a simple hash for the token
    token_data = f"test-token-{timestamp}-{expires}"
    token_hash = hashlib.sha256(token_data.encode()).hexdigest()[:32]

    # Format as a fake JWT-like token
    token = f"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.{token_hash}.{timestamp}"

    return jsonify({
        "token": token,
        "expires_in": 900,
        "expires_at": datetime.fromtimestamp(expires).isoformat(),
        "type": "Bearer"
    })

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
║    GET  /token          - Get test JWT token (15 min expiry)║
║    POST /trades         - Trade data (select/where body)    ║
║    GET  /trades         - All trades                        ║
║    POST /counterparty_trades - For reconciliation           ║
║    GET  /health         - Health check                      ║
║    GET  /protected/data - Requires Bearer token             ║
║                                                              ║
║  Test token refresh:                                         ║
║    curl http://localhost:5001/token                         ║
║                                                              ║
║  Configure Neovim:                                          ║
║    token = {                                                ║
║      token_endpoint = 'http://localhost:5001/token'        ║
║      auto_refresh = true                                    ║
║    }                                                         ║
╚══════════════════════════════════════════════════════════════╝
    """)

    # Run on port 5001
    app.run(host='0.0.0.0', port=5001, debug=True)