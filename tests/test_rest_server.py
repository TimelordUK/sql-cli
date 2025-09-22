#!/usr/bin/env python3
"""Simple REST API server for testing WEB CTE features with JSON_PATH and POST body."""

from flask import Flask, jsonify, request
import csv
import json
import os

app = Flask(__name__)

# In-memory storage for testing
trades_data = []

@app.route('/trades', methods=['GET', 'POST'])
def trades():
    """Endpoint that returns data in the format: { Result: [...] }"""
    global trades_data

    if request.method == 'POST':
        # Parse the body to filter trades
        body = request.get_json()

        # If no trades loaded yet, load from CSV
        if not trades_data:
            load_trades_data()

        # Simple filtering based on body content
        filtered = trades_data.copy()

        if body and 'Where' in body:
            # Very basic WHERE clause parsing (for demo only)
            where = body['Where']
            if 'Symbol = "AAPL"' in where:
                filtered = [t for t in filtered if t.get('Symbol') == 'AAPL']
            elif 'Symbol = "GOOGL"' in where:
                filtered = [t for t in filtered if t.get('Symbol') == 'GOOGL']

        # Return in the expected format with Result wrapper
        return jsonify({'Result': filtered[:100]})  # Limit to 100 records

    else:  # GET
        if not trades_data:
            load_trades_data()
        return jsonify({'Result': trades_data[:100]})

@app.route('/nested/data', methods=['GET'])
def nested_data():
    """Returns deeply nested JSON for testing JSON_PATH."""
    return jsonify({
        'status': 'success',
        'metadata': {
            'version': '1.0',
            'timestamp': '2024-01-15'
        },
        'data': {
            'trades': {
                'Result': [
                    {'id': 1, 'symbol': 'AAPL', 'price': 150.50, 'volume': 1000},
                    {'id': 2, 'symbol': 'GOOGL', 'price': 2800.00, 'volume': 500},
                    {'id': 3, 'symbol': 'MSFT', 'price': 380.25, 'volume': 750},
                ]
            }
        }
    })

@app.route('/csv-data', methods=['GET'])
def csv_data_endpoint():
    """Load a CSV file and return as JSON array wrapped in Result."""
    csv_file = 'data/sales_data.csv'
    if not os.path.exists(csv_file):
        # Create sample data if file doesn't exist
        return jsonify({
            'Result': [
                {'id': 1, 'product': 'Widget', 'quantity': 100, 'price': 9.99},
                {'id': 2, 'product': 'Gadget', 'quantity': 50, 'price': 19.99},
                {'id': 3, 'product': 'Doohickey', 'quantity': 75, 'price': 14.99},
            ]
        })

    # Read CSV file
    data = []
    with open(csv_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            data.append(row)

    return jsonify({'Result': data})

@app.route('/auth/token', methods=['POST'])
def auth_token():
    """Mock authentication endpoint."""
    # Check for client credentials in body
    body = request.get_data(as_text=True)
    if 'client_id=' in body and 'client_secret=' in body:
        return jsonify({
            'access_token': 'mock-jwt-token-12345',
            'token_type': 'Bearer',
            'expires_in': 3600
        })
    return jsonify({'error': 'Invalid credentials'}), 401

@app.route('/protected/data', methods=['GET'])
def protected_data():
    """Protected endpoint that requires authentication."""
    auth_header = request.headers.get('Authorization', '')
    if 'Bearer mock-jwt-token' in auth_header:
        return jsonify({
            'Result': [
                {'id': 1, 'secret': 'data1', 'value': 1000},
                {'id': 2, 'secret': 'data2', 'value': 2000},
            ]
        })
    return jsonify({'error': 'Unauthorized'}), 401

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({'status': 'healthy', 'service': 'test-rest-server'})

def load_trades_data():
    """Load sample trades data."""
    global trades_data
    trades_data = [
        {'TradeId': 'T001', 'Symbol': 'AAPL', 'Price': 150.25, 'Quantity': 100, 'ExecutionTime': '2024-01-15T10:00:00'},
        {'TradeId': 'T002', 'Symbol': 'GOOGL', 'Price': 2800.50, 'Quantity': 50, 'ExecutionTime': '2024-01-15T10:01:00'},
        {'TradeId': 'T003', 'Symbol': 'AAPL', 'Price': 151.00, 'Quantity': 200, 'ExecutionTime': '2024-01-15T10:02:00'},
        {'TradeId': 'T004', 'Symbol': 'MSFT', 'Price': 380.75, 'Quantity': 75, 'ExecutionTime': '2024-01-15T10:03:00'},
        {'TradeId': 'T005', 'Symbol': 'AAPL', 'Price': 149.50, 'Quantity': 150, 'ExecutionTime': '2024-01-15T10:04:00'},
        {'TradeId': 'T006', 'Symbol': 'TSLA', 'Price': 650.00, 'Quantity': 25, 'ExecutionTime': '2024-01-15T10:05:00'},
        {'TradeId': 'T007', 'Symbol': 'GOOGL', 'Price': 2799.00, 'Quantity': 30, 'ExecutionTime': '2024-01-15T10:06:00'},
        {'TradeId': 'T008', 'Symbol': 'AAPL', 'Price': 150.75, 'Quantity': 80, 'ExecutionTime': '2024-01-15T10:07:00'},
    ]

if __name__ == '__main__':
    print("Starting test REST server on http://localhost:5000")
    print("Endpoints:")
    print("  GET/POST http://localhost:5000/trades - Returns {Result: [...]}")
    print("  GET http://localhost:5000/nested/data - Returns nested JSON")
    print("  GET http://localhost:5000/csv-data - Returns CSV as JSON")
    print("  POST http://localhost:5000/auth/token - Mock auth")
    print("  GET http://localhost:5000/protected/data - Protected endpoint")
    print("  GET http://localhost:5000/health - Health check")
    app.run(port=5000, debug=True)