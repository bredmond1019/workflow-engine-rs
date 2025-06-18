#!/bin/bash
# Generate self-signed SSL certificates for development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SSL_DIR="$SCRIPT_DIR/ssl"

echo "Generating self-signed SSL certificates for development..."

# Create SSL directory
mkdir -p "$SSL_DIR"

# Generate private key
openssl genrsa -out "$SSL_DIR/key.pem" 2048

# Generate certificate signing request
openssl req -new -key "$SSL_DIR/key.pem" -out "$SSL_DIR/csr.pem" -subj "/C=US/ST=Development/L=Local/O=AI Workflow System/CN=localhost"

# Generate self-signed certificate (valid for 1 year)
openssl x509 -req -days 365 -in "$SSL_DIR/csr.pem" -signkey "$SSL_DIR/key.pem" -out "$SSL_DIR/cert.pem"

# Clean up CSR
rm "$SSL_DIR/csr.pem"

# Set appropriate permissions
chmod 600 "$SSL_DIR/key.pem"
chmod 644 "$SSL_DIR/cert.pem"

echo "SSL certificates generated successfully!"
echo "  - Certificate: $SSL_DIR/cert.pem"
echo "  - Private key: $SSL_DIR/key.pem"
echo ""
echo "Note: These are self-signed certificates for development only."
echo "Your browser will show a security warning - this is expected."