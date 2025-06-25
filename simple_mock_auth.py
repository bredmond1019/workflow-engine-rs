#!/usr/bin/env python3
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import time

class MockAuthHandler(BaseHTTPRequestHandler):
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()
    
    def do_POST(self):
        if self.path == '/auth/token':
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            
            # Create a properly formatted JWT token
            import base64
            
            # JWT header
            header = base64.urlsafe_b64encode(json.dumps({
                'alg': 'HS256',
                'typ': 'JWT'
            }).encode()).decode().rstrip('=')
            
            # JWT payload
            payload = base64.urlsafe_b64encode(json.dumps({
                'sub': data.get('sub', 'user'),
                'role': data.get('role', 'Admin'),
                'exp': int(time.time()) + 86400,
                'iat': int(time.time())
            }).encode()).decode().rstrip('=')
            
            # Mock signature
            signature = 'mock_signature_123'
            
            # Combine to create JWT
            token = f"{header}.{payload}.{signature}"
            
            response = {
                'access_token': token,
                'token_type': 'Bearer',
                'expires_in': 86400
            }
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'ok'}).encode())
        elif self.path == '/api/v1/workflows/templates':
            templates = [
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
            ]
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(templates).encode())
    
    def log_message(self, format, *args):
        # Suppress request logging
        pass

if __name__ == '__main__':
    server = HTTPServer(('localhost', 8080), MockAuthHandler)
    print('Mock auth server running on http://localhost:8080')
    server.serve_forever()