#!/bin/bash
# Script to test GraphQL Federation connectivity

set -e

GATEWAY_URL="http://localhost:4000/graphql"

echo "🧪 Testing GraphQL Federation..."
echo "================================"

# Test 1: Gateway Health Check
echo ""
echo "1️⃣ Testing Gateway Health..."
if curl -s -f "http://localhost:4000/health" > /dev/null; then
    echo "✅ Gateway is healthy"
else
    echo "❌ Gateway health check failed"
    exit 1
fi

# Test 2: Simple Query
echo ""
echo "2️⃣ Testing Simple Workflow Query..."
SIMPLE_QUERY='{"query":"{ workflows { id name status } }"}'
RESPONSE=$(curl -s -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d "$SIMPLE_QUERY")

if echo "$RESPONSE" | grep -q "workflows"; then
    echo "✅ Simple query successful"
    echo "   Response: $(echo $RESPONSE | jq -c '.data.workflows' 2>/dev/null || echo $RESPONSE)"
else
    echo "❌ Simple query failed"
    echo "   Response: $RESPONSE"
fi

# Test 3: Federated Query
echo ""
echo "3️⃣ Testing Federated Query..."
FEDERATED_QUERY='{
  "query": "query TestFederation($workflowId: ID!) { 
    workflow(id: $workflowId) { 
      id 
      name 
      nodes { id type }
    } 
  }",
  "variables": { "workflowId": "test-123" }
}'

RESPONSE=$(curl -s -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d "$FEDERATED_QUERY")

if echo "$RESPONSE" | grep -q "workflow"; then
    echo "✅ Federated query successful"
    echo "   Response: $(echo $RESPONSE | jq -c '.data.workflow' 2>/dev/null || echo $RESPONSE)"
else
    echo "❌ Federated query failed"
    echo "   Response: $RESPONSE"
fi

# Test 4: Service Discovery
echo ""
echo "4️⃣ Testing Service Discovery..."
SERVICE_QUERY='{"query":"{ _service { sdl } }"}'
RESPONSE=$(curl -s -X POST "http://localhost:8080/api/v1/graphql" \
    -H "Content-Type: application/json" \
    -d "$SERVICE_QUERY")

if echo "$RESPONSE" | grep -q "_service"; then
    echo "✅ Service discovery working"
else
    echo "❌ Service discovery failed"
fi

# Test 5: Entity Resolution
echo ""
echo "5️⃣ Testing Entity Resolution..."
ENTITY_QUERY='{
  "query": "query ResolveEntities($representations: [_Any!]!) { 
    _entities(representations: $representations) { 
      ... on Workflow { id name status } 
    } 
  }",
  "variables": {
    "representations": [
      { "__typename": "Workflow", "id": "workflow-1" }
    ]
  }
}'

RESPONSE=$(curl -s -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d "$ENTITY_QUERY")

if echo "$RESPONSE" | grep -q "_entities"; then
    echo "✅ Entity resolution working"
    echo "   Response: $(echo $RESPONSE | jq -c '.data._entities' 2>/dev/null || echo $RESPONSE)"
else
    echo "❌ Entity resolution failed"
    echo "   Response: $RESPONSE"
fi

echo ""
echo "================================"
echo "🎉 Federation Testing Complete!"
echo ""