#!/usr/bin/env python3
"""
Mock REST API Server for SQL-CLI Testing
=========================================

This server simulates various REST API patterns to test SQL-CLI's WEB CTE features.
It perfectly replicates common enterprise REST API patterns.

Usage:
    uv run python tests/mock_api_server/server.py

Endpoints:
    /trades         - Trading data with Result wrapper
    /nested/data    - Deeply nested JSON for JSON_PATH testing
    /csv-data       - CSV data as JSON
    /auth/token     - OAuth token endpoint
    /protected/data - Protected endpoint requiring auth
"""

from flask import Flask, jsonify, request, Response
import csv
import json
import os
from datetime import datetime, timedelta
import random

app = Flask(__name__)

# In-memory storage
trades_data = []
users_data = []

def generate_trades(symbol_filter=None, count=100):
    """Generate realistic trade data."""
    symbols = ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA', 'META', 'NVDA']
    if symbol_filter:
        symbols = [s for s in symbols if s == symbol_filter]

    trades = []
    base_time = datetime.now() - timedelta(hours=24)

    for i in range(count):
        symbol = random.choice(symbols)
        base_price = {
            'AAPL': 150, 'GOOGL': 2800, 'MSFT': 380,
            'AMZN': 170, 'TSLA': 650, 'META': 350, 'NVDA': 450
        }.get(symbol, 100)

        trade = {
            'TradeId': f'T{i+1:04d}',
            'Symbol': symbol,
            'Price': base_price + random.uniform(-5, 5),
            'Quantity': random.randint(10, 500),
            'ExecutionTime': (base_time + timedelta(minutes=i)).isoformat(),
            'Side': random.choice(['BUY', 'SELL']),
            'OrderType': random.choice(['MARKET', 'LIMIT', 'STOP']),
            'Venue': random.choice(['NYSE', 'NASDAQ', 'BATS', 'ARCA'])
        }
        trades.append(trade)

    return trades

@app.route('/trades', methods=['GET', 'POST'])
def trades():
    """
    Trading endpoint that supports filtering via POST body.
    Returns data in format: { Result: [...] }
    """
    if request.method == 'POST':
        body = request.get_json()

        # Parse the body for filtering
        symbol_filter = None
        count = 100

        if body:
            # Simple WHERE clause parsing for demo
            if 'Where' in body:
                where = body['Where']
                for symbol in ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA']:
                    if f'Symbol = "{symbol}"' in where:
                        symbol_filter = symbol
                        break

            if 'Limit' in body:
                count = min(body['Limit'], 1000)

        trades = generate_trades(symbol_filter, count)
        return jsonify({'Result': trades})

    else:  # GET
        trades = generate_trades(count=50)
        return jsonify({'Result': trades})

@app.route('/nested/data', methods=['GET'])
def nested_data():
    """Returns deeply nested JSON for testing JSON_PATH."""
    return jsonify({
        'status': 'success',
        'metadata': {
            'version': '1.0',
            'timestamp': datetime.now().isoformat(),
            'recordCount': 3
        },
        'data': {
            'trades': {
                'Result': [
                    {'id': 1, 'symbol': 'AAPL', 'price': 150.50, 'volume': 1000},
                    {'id': 2, 'symbol': 'GOOGL', 'price': 2800.00, 'volume': 500},
                    {'id': 3, 'symbol': 'MSFT', 'price': 380.25, 'volume': 750},
                ]
            },
            'summary': {
                'totalVolume': 2250,
                'avgPrice': 1110.25
            }
        }
    })

@app.route('/api/v2/market-data', methods=['POST'])
def market_data():
    """
    Advanced market data endpoint with complex filtering.
    Expects body like:
    {
        "Select": ["field1", "field2"],
        "Where": "condition",
        "OrderBy": "field",
        "Limit": 100
    }
    """
    body = request.get_json()

    # Generate base data
    data = []
    for i in range(200):
        data.append({
            'timestamp': (datetime.now() - timedelta(minutes=i)).isoformat(),
            'symbol': random.choice(['AAPL', 'GOOGL', 'MSFT']),
            'bid': 100 + random.uniform(-10, 10),
            'ask': 100 + random.uniform(-10, 10),
            'volume': random.randint(100, 10000),
            'vwap': 100 + random.uniform(-5, 5)
        })

    # Apply filtering if specified
    if body and 'Limit' in body:
        data = data[:body['Limit']]

    return jsonify({
        'Result': data,
        'Metadata': {
            'Count': len(data),
            'Timestamp': datetime.now().isoformat()
        }
    })

@app.route('/csv-data', methods=['GET'])
def csv_data_endpoint():
    """Load a CSV file and return as JSON array wrapped in Result."""
    csv_file = 'data/sales_data.csv'

    if os.path.exists(csv_file):
        data = []
        with open(csv_file, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                data.append(row)
        return jsonify({'Result': data})

    # Return sample data if file doesn't exist
    return jsonify({
        'Result': [
            {'id': 1, 'product': 'Widget', 'quantity': 100, 'price': 9.99, 'region': 'North'},
            {'id': 2, 'product': 'Gadget', 'quantity': 50, 'price': 19.99, 'region': 'South'},
            {'id': 3, 'product': 'Doohickey', 'quantity': 75, 'price': 14.99, 'region': 'East'},
            {'id': 4, 'product': 'Thingamajig', 'quantity': 120, 'price': 7.99, 'region': 'West'},
        ]
    })

@app.route('/auth/token', methods=['POST'])
def auth_token():
    """Mock OAuth2 token endpoint."""
    body = request.get_data(as_text=True)

    # Check for client credentials
    if 'client_id=' in body and 'client_secret=' in body:
        return jsonify({
            'access_token': f'mock-jwt-token-{random.randint(1000, 9999)}',
            'token_type': 'Bearer',
            'expires_in': 3600,
            'scope': 'read write'
        })

    # Check for refresh token
    if 'refresh_token=' in body:
        return jsonify({
            'access_token': f'refreshed-token-{random.randint(1000, 9999)}',
            'token_type': 'Bearer',
            'expires_in': 3600
        })

    return jsonify({'error': 'invalid_grant', 'error_description': 'Invalid credentials'}), 401

@app.route('/protected/data', methods=['GET'])
def protected_data():
    """Protected endpoint that requires Bearer token authentication."""
    auth_header = request.headers.get('Authorization', '')

    if not auth_header.startswith('Bearer '):
        return jsonify({'error': 'unauthorized', 'message': 'Bearer token required'}), 401

    token = auth_header[7:]  # Remove 'Bearer ' prefix

    if 'mock-jwt-token' in token or 'refreshed-token' in token:
        return jsonify({
            'Result': [
                {'id': 1, 'confidential': 'alpha', 'value': 1000, 'classification': 'SECRET'},
                {'id': 2, 'confidential': 'beta', 'value': 2000, 'classification': 'TOP_SECRET'},
                {'id': 3, 'confidential': 'gamma', 'value': 1500, 'classification': 'CONFIDENTIAL'},
            ],
            'AccessLevel': 'FULL',
            'User': 'test-user'
        })

    return jsonify({'error': 'invalid_token', 'message': 'Token expired or invalid'}), 403

@app.route('/graphql', methods=['POST'])
def graphql():
    """Mock GraphQL endpoint."""
    body = request.get_json()

    if not body or 'query' not in body:
        return jsonify({'errors': [{'message': 'No query provided'}]}), 400

    # Simple mock response
    return jsonify({
        'data': {
            'trades': {
                'edges': [
                    {'node': {'id': '1', 'symbol': 'AAPL', 'price': 150.25}},
                    {'node': {'id': '2', 'symbol': 'GOOGL', 'price': 2800.50}},
                ],
                'pageInfo': {
                    'hasNextPage': True,
                    'endCursor': 'cursor123'
                }
            }
        }
    })

@app.route('/fix/executions', methods=['POST'])
def fix_executions():
    """
    FIX execution report endpoint with pipe-delimited repeated groups.
    Simulates parsing FIX messages from a log file.

    Expects POST body with:
    {
        "tags": ["35", "52", "11", "55", "54", "79", "80"],  // FIX tags to extract
        "filters": {
            "Symbol": "AAPL",  // Optional filters
            "Side": "1"
        }
    }

    Returns executions with pipe-delimited allocation groups:
    {
        "Result": [
            {
                "MsgType": "8",
                "SendingTime": "2024-01-15T09:30:15.123",
                "ClOrdID": "ORD-2024-001",
                "Symbol": "AAPL",
                "Side": "1",
                "AllocAccount": "FUND-A|FUND-B|FUND-C",
                "AllocQty": "400|300|300"
            },
            ...
        ]
    }
    """
    body = request.get_json() or {}

    # Load FIX executions from CSV
    csv_file = 'data/fix_executions.csv'
    executions = []

    if os.path.exists(csv_file):
        with open(csv_file, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                executions.append(row)
    else:
        # Fallback to hardcoded data if file doesn't exist
        executions = [
            {
                'MsgSeqNum': '1001',
                'SendingTime': '2024-01-15T09:30:15.123',
                'MsgType': '8',
                'ClOrdID': 'ORD-2024-001',
                'Symbol': 'AAPL',
                'Side': '1',
                'ExecType': 'F',
                'LastQty': '1000',
                'LastPx': '175.50',
                'AvgPx': '175.50',
                'ExecID': 'EXEC-001',
                'AllocAccount': 'FUND-A|FUND-B|FUND-C',
                'AllocQty': '400|300|300'
            },
            {
                'MsgSeqNum': '1003',
                'SendingTime': '2024-01-15T09:32:30.789',
                'MsgType': '8',
                'ClOrdID': 'ORD-2024-004',
                'Symbol': 'GOOGL',
                'Side': '1',
                'ExecType': 'F',
                'LastQty': '500',
                'LastPx': '140.75',
                'AvgPx': '140.75',
                'ExecID': 'EXEC-002',
                'AllocAccount': 'FUND-D|FUND-E',
                'AllocQty': '250|250'
            }
        ]

    # Apply filters if provided
    if 'filters' in body:
        filters = body['filters']
        filtered = []
        for exec in executions:
            match = True
            for key, value in filters.items():
                if exec.get(key) != str(value):
                    match = False
                    break
            if match:
                filtered.append(exec)
        executions = filtered

    # If specific tags requested, filter columns
    if 'tags' in body:
        # Map FIX tags to field names (simplified mapping)
        tag_map = {
            '35': 'MsgType',
            '52': 'SendingTime',
            '11': 'ClOrdID',
            '55': 'Symbol',
            '54': 'Side',
            '150': 'ExecType',
            '32': 'LastQty',
            '31': 'LastPx',
            '6': 'AvgPx',
            '17': 'ExecID',
            '79': 'AllocAccount',
            '80': 'AllocQty'
        }

        requested_fields = [tag_map.get(tag, tag) for tag in body['tags']]
        filtered_execs = []
        for exec in executions:
            filtered_exec = {field: exec.get(field, '') for field in requested_fields if field in exec}
            if filtered_exec:
                filtered_execs.append(filtered_exec)
        executions = filtered_execs

    return jsonify({'Result': executions})

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({
        'status': 'healthy',
        'service': 'mock-api-server',
        'version': '1.0.0',
        'timestamp': datetime.now().isoformat()
    })

@app.route('/streaming/prices', methods=['GET'])
def streaming_prices():
    """Server-sent events endpoint for real-time price updates."""
    def generate():
        for i in range(10):
            data = {
                'symbol': random.choice(['AAPL', 'GOOGL', 'MSFT']),
                'price': 100 + random.uniform(-10, 10),
                'timestamp': datetime.now().isoformat()
            }
            yield f"data: {json.dumps(data)}\n\n"

    return Response(generate(), mimetype='text/event-stream')

@app.route('/batch', methods=['POST'])
def batch_operations():
    """
    Batch operations endpoint.
    Expects: { "operations": [{"method": "GET", "path": "/trades"}, ...] }
    """
    body = request.get_json()

    if not body or 'operations' not in body:
        return jsonify({'error': 'operations array required'}), 400

    results = []
    for op in body['operations']:
        # Simplified batch processing
        if op.get('path') == '/trades':
            results.append({
                'status': 200,
                'body': {'Result': generate_trades(count=5)}
            })
        else:
            results.append({
                'status': 404,
                'body': {'error': 'not found'}
            })

    return jsonify({'results': results})

if __name__ == '__main__':
    import sys

    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5000

    print(f"""
╔══════════════════════════════════════════════════════════════╗
║           SQL-CLI Mock API Server                           ║
║                                                              ║
║  Running on: http://localhost:{port:<47}║
║                                                              ║
║  Endpoints:                                                  ║
║    GET/POST /trades           - Trading data                ║
║    POST     /fix/executions   - FIX execution reports       ║
║    GET      /nested/data      - Nested JSON                 ║
║    POST     /api/v2/market-data - Advanced filtering        ║
║    GET      /csv-data         - CSV as JSON                 ║
║    POST     /auth/token       - OAuth2 token                ║
║    GET      /protected/data  - Protected endpoint           ║
║    POST     /graphql          - GraphQL endpoint            ║
║    POST     /batch            - Batch operations            ║
║    GET      /streaming/prices - SSE price stream            ║
║    GET      /health           - Health check                ║
║                                                              ║
║  Press Ctrl+C to stop                                        ║
╚══════════════════════════════════════════════════════════════╝
    """)

    app.run(host='0.0.0.0', port=port, debug=True)