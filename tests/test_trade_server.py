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
def generate_trades(source=None, trade_date=None, count=5000):
    """Generate trade data - default 5000 rows for realistic testing"""
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
    for i in range(1, count + 1):  # Generate specified number of trades
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

@app.route('/securities', methods=['GET', 'POST'])
def securities():
    """Securities master data - instrument details"""
    securities_data = [
        {"Ticker": "AAPL", "SecurityName": "Apple Inc.", "Sector": "Technology", "Currency": "USD", "Exchange": "NASDAQ", "ISIN": "US0378331005", "Multiplier": 1},
        {"Ticker": "GOOGL", "SecurityName": "Alphabet Inc.", "Sector": "Technology", "Currency": "USD", "Exchange": "NASDAQ", "ISIN": "US02079K3059", "Multiplier": 1},
        {"Ticker": "MSFT", "SecurityName": "Microsoft Corp.", "Sector": "Technology", "Currency": "USD", "Exchange": "NASDAQ", "ISIN": "US5949181045", "Multiplier": 1},
        {"Ticker": "TSLA", "SecurityName": "Tesla Inc.", "Sector": "Automotive", "Currency": "USD", "Exchange": "NASDAQ", "ISIN": "US88160R1014", "Multiplier": 1},
        {"Ticker": "AMZN", "SecurityName": "Amazon.com Inc.", "Sector": "Consumer Discretionary", "Currency": "USD", "Exchange": "NASDAQ", "ISIN": "US0231351067", "Multiplier": 1},
        {"Ticker": "JPM", "SecurityName": "JPMorgan Chase", "Sector": "Financials", "Currency": "USD", "Exchange": "NYSE", "ISIN": "US46625H1005", "Multiplier": 1},
        {"Ticker": "GS", "SecurityName": "Goldman Sachs", "Sector": "Financials", "Currency": "USD", "Exchange": "NYSE", "ISIN": "US38141G1040", "Multiplier": 1},
    ]

    # Filter by ticker if requested
    if request.method == 'POST':
        body = request.get_json()
        where_clause = body.get('Where', '').upper()
        if 'TICKER' in where_clause:
            import re
            match = re.search(r'TICKER.*IN.*\((.*?)\)', where_clause)
            if match:
                tickers = [t.strip().strip('"').strip("'") for t in match.group(1).split(',')]
                securities_data = [s for s in securities_data if s['Ticker'] in tickers]

    return jsonify({"Result": securities_data})

@app.route('/fix_messages', methods=['GET', 'POST'])
def fix_messages():
    """Simulated FIX protocol messages with timing data"""
    base_time = datetime.now()

    fix_data = []
    for i in range(20):
        send_time = base_time + timedelta(seconds=i*2 + random.uniform(-0.5, 0.5))
        exec_time = send_time + timedelta(milliseconds=random.randint(50, 500))

        msg = {
            "MsgSeqNum": i + 1,
            "MsgType": "8",  # Tag 35: Execution Report
            "ExecType": random.choice(["0", "1", "2"]),  # Tag 150: New, Partial, Fill
            "OrdStatus": random.choice(["0", "1", "2"]),  # Tag 39: New, Partial, Filled
            "ClOrdID": f"CLO{i+1:05d}",  # Tag 11: Client Order ID
            "OrderID": f"ORD{random.randint(100000, 999999)}",  # Tag 37: Order ID
            "Symbol": random.choice(["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"]),  # Tag 55
            "Side": random.choice(["1", "2"]),  # Tag 54: Buy(1), Sell(2)
            "OrderQty": random.randint(100, 5000),  # Tag 38
            "LastQty": random.randint(100, 1000),  # Tag 32: Last executed quantity
            "LastPx": round(random.uniform(100.0, 500.0), 2),  # Tag 31: Last price
            "SendingTime": send_time.strftime("%Y%m%d-%H:%M:%S.%f")[:-3],  # Tag 52
            "TransactTime": exec_time.strftime("%Y%m%d-%H:%M:%S.%f")[:-3],  # Tag 60
            "AllocAccount": f"ACC{random.randint(1000, 9999)}",  # Tag 79
            "LatencyMs": int((exec_time - send_time).total_seconds() * 1000)
        }
        fix_data.append(msg)

    # Filter by tags if requested
    if request.method == 'POST':
        body = request.get_json()
        tags = body.get('tags', [])
        if tags:
            # Map FIX tag numbers to field names
            tag_map = {
                "35": "MsgType", "8": "MsgSeqNum", "79": "AllocAccount",
                "52": "SendingTime", "60": "TransactTime", "11": "ClOrdID",
                "37": "OrderID", "55": "Symbol", "54": "Side",
                "38": "OrderQty", "31": "LastPx", "32": "LastQty"
            }

            filtered_data = []
            for msg in fix_data:
                filtered_msg = {}
                for tag in tags:
                    field_name = tag_map.get(tag, tag)
                    if field_name in msg:
                        filtered_msg[field_name] = msg[field_name]
                # Always include computed fields
                filtered_msg["LatencyMs"] = msg["LatencyMs"]
                filtered_data.append(filtered_msg)
            fix_data = filtered_data

    return jsonify({"Result": fix_data})

@app.route('/parent_orders', methods=['GET', 'POST'])
def parent_orders():
    """Parent orders with child executions for VWAP calculation"""
    parent_orders = []

    for parent_id in range(1, 6):
        parent_ticker = random.choice(["AAPL", "GOOGL", "MSFT"])
        parent_qty = random.randint(5000, 20000)
        child_count = random.randint(5, 15)

        base_price = random.uniform(100.0, 500.0)
        base_time = datetime.now() - timedelta(hours=2) + timedelta(minutes=parent_id*10)

        # Create parent order
        parent = {
            "ParentOrderID": f"PARENT{parent_id:03d}",
            "Ticker": parent_ticker,
            "TotalQuantity": parent_qty,
            "Side": random.choice(["Buy", "Sell"]),
            "OrderTime": base_time.strftime("%Y-%m-%d %H:%M:%S"),
            "Status": "Completed",
            "ChildExecutions": []
        }

        remaining_qty = parent_qty
        for child_id in range(child_count):
            child_qty = min(remaining_qty, random.randint(100, 1000))
            remaining_qty -= child_qty

            # Price walks randomly
            child_price = base_price + random.uniform(-5.0, 5.0)
            child_time = base_time + timedelta(seconds=child_id*30 + random.randint(0, 20))

            child = {
                "ChildOrderID": f"CHILD{parent_id:03d}_{child_id+1:02d}",
                "ParentOrderID": f"PARENT{parent_id:03d}",
                "Ticker": parent_ticker,
                "Quantity": child_qty,
                "Price": round(child_price, 2),
                "ExecutionTime": child_time.strftime("%Y-%m-%d %H:%M:%S"),
                "Venue": random.choice(["NASDAQ", "NYSE", "ARCA", "BATS"])
            }
            parent["ChildExecutions"].append(child)
            parent_orders.append(child)  # Also add to flat list

            if remaining_qty <= 0:
                break

    return jsonify({"Result": parent_orders})

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
╔════════════════════════════════════════════════════════════════╗
║         Test Trade Server for SQL-CLI Temp Tables Demo        ║
╠════════════════════════════════════════════════════════════════╣
║  Running on: http://localhost:5001                            ║
║                                                                ║
║  Endpoints:                                                    ║
║    GET  /token               - Test JWT token (15 min expiry) ║
║    POST /trades              - Trade data (select/where body) ║
║    GET  /trades              - All trades                     ║
║    POST /counterparty_trades - Counterparty trades            ║
║    POST /securities          - Securities master data         ║
║    POST /fix_messages        - FIX protocol messages          ║
║    POST /parent_orders       - Parent/child order executions  ║
║    GET  /health              - Health check                   ║
║    GET  /protected/data      - Requires Bearer token          ║
║                                                                ║
║  Test Examples:                                                ║
║    examples/cte_flask_dependency.sql - Multi-stage analysis   ║
║                                                                ║
║  Usage:                                                        ║
║    sql-cli -f examples/cte_flask_dependency.sql               ║
║    sql-cli -f examples/cte_flask_dependency.sql --execute-statement 4 ║
║                                                                ║
║  In Neovim: \\sq (all) or \\sx (statement at cursor)          ║
╚════════════════════════════════════════════════════════════════╝
    """)

    # Run on port 5001
    app.run(host='0.0.0.0', port=5001, debug=True)