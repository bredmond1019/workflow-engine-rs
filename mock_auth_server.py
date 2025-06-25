#!/usr/bin/env python3
from flask import Flask, request, jsonify
from flask_cors import CORS
import base64
import json
import time

app = Flask(__name__)
CORS(app)

@app.route('/auth/token', methods=['POST'])
def auth_token():
    data = request.get_json()
    sub = data.get('sub', 'user')
    role = data.get('role', 'Admin')
    
    # Create mock JWT payload
    payload = {
        'sub': sub,
        'role': role,
        'exp': int(time.time()) + 86400,  # 24 hours
        'iat': int(time.time())
    }
    
    # Create a mock JWT token (not cryptographically secure, just for testing)
    header = base64.b64encode(json.dumps({'alg': 'HS256', 'typ': 'JWT'}).encode()).decode()
    payload_encoded = base64.b64encode(json.dumps(payload).encode()).decode()
    signature = 'mock_signature'
    
    token = f"{header}.{payload_encoded}.{signature}"
    
    return jsonify({
        'access_token': token,
        'token_type': 'Bearer',
        'expires_in': 86400
    })

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'status': 'ok'})

@app.route('/api/v1/workflows/templates', methods=['GET'])
def workflow_templates():
    return jsonify([
        {
            'id': 'customer-support',
            'name': 'Customer Support Workflow',
            'description': 'Handle customer inquiries'
        },
        {
            'id': 'knowledge-base',
            'name': 'Knowledge Base Workflow',
            'description': 'Search and retrieve information'
        }
    ])

if __name__ == '__main__':
    print("Mock auth server running on http://localhost:8080")
    app.run(port=8080, debug=False)