#!/usr/bin/env python3
"""
Simple Flask token server for testing SQL-CLI token management.
This mimics a typical JWT token service.
"""

from flask import Flask, jsonify
import jwt
import datetime
import os

app = Flask(__name__)

# Secret key for JWT encoding (use environment variable in production)
SECRET_KEY = os.environ.get('JWT_SECRET', 'your-secret-key-here')

@app.route('/token', methods=['GET'])
def get_token():
    """Generate a JWT token valid for 15 minutes."""

    # Token payload
    payload = {
        'user': 'sql-cli-user',
        'exp': datetime.datetime.utcnow() + datetime.timedelta(minutes=15),
        'iat': datetime.datetime.utcnow(),
        'permissions': ['read', 'write', 'execute']
    }

    # Generate token
    token = jwt.encode(payload, SECRET_KEY, algorithm='HS256')

    # Return as JSON
    return jsonify({
        'token': token,
        'expires_in': 900,  # 15 minutes in seconds
        'type': 'Bearer'
    })

@app.route('/token/complex', methods=['GET'])
def get_token_complex():
    """Example of nested response structure."""

    payload = {
        'user': 'sql-cli-user',
        'exp': datetime.datetime.utcnow() + datetime.timedelta(minutes=15),
        'iat': datetime.datetime.utcnow(),
    }

    token = jwt.encode(payload, SECRET_KEY, algorithm='HS256')

    return jsonify({
        'status': 'success',
        'data': {
            'access_token': token,
            'refresh_token': 'not-implemented',
            'expires_in': 900
        }
    })

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({'status': 'healthy'})

if __name__ == '__main__':
    # Run on localhost:5000 by default
    app.run(debug=True, port=5000)

"""
To run this server:
1. pip install flask pyjwt
2. python token_server.py
3. Test: curl http://localhost:5000/token

For the complex endpoint, configure nvim with:
  token_path = "data.access_token"
"""