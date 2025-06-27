#!/bin/bash

# Test script for GraphQL Gateway health endpoints

echo "Testing GraphQL Gateway health endpoints..."

# Start the gateway in background
cargo run --bin graphql-gateway &
GATEWAY_PID=$!

# Function to cleanup on exit
cleanup() {
    echo "Stopping gateway..."
    kill $GATEWAY_PID 2>/dev/null
}
trap cleanup EXIT

# Wait for gateway to start
echo "Waiting for gateway to start..."
sleep 3

# Test basic health endpoint
echo "Testing /health endpoint..."
HEALTH_RESPONSE=$(curl -s -w "%{http_code}" http://localhost:4000/health)
HEALTH_CODE=${HEALTH_RESPONSE: -3}
HEALTH_BODY=${HEALTH_RESPONSE%???}

echo "Health endpoint response code: $HEALTH_CODE"
echo "Health endpoint response body: $HEALTH_BODY"

if [ "$HEALTH_CODE" = "200" ]; then
    echo "✅ Basic health endpoint working"
else
    echo "❌ Basic health endpoint failed"
fi

# Test detailed health endpoint
echo "Testing /health/detailed endpoint..."
DETAILED_RESPONSE=$(curl -s -w "%{http_code}" http://localhost:4000/health/detailed)
DETAILED_CODE=${DETAILED_RESPONSE: -3}
DETAILED_BODY=${DETAILED_RESPONSE%???}

echo "Detailed health endpoint response code: $DETAILED_CODE"
echo "Detailed health endpoint response body: $DETAILED_BODY"

if [ "$DETAILED_CODE" = "200" ] || [ "$DETAILED_CODE" = "206" ]; then
    echo "✅ Detailed health endpoint working"
else
    echo "❌ Detailed health endpoint failed"
fi

# Test GraphQL endpoint
echo "Testing GraphQL health query..."
GRAPHQL_QUERY='{"query": "{ health }"}'
GRAPHQL_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$GRAPHQL_QUERY" http://localhost:4000/graphql)

echo "GraphQL health query response: $GRAPHQL_RESPONSE"

if echo "$GRAPHQL_RESPONSE" | grep -q "GraphQL Gateway is healthy"; then
    echo "✅ GraphQL health query working"
else
    echo "❌ GraphQL health query failed"
fi

echo "Health endpoint testing complete!"