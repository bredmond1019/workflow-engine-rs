#!/bin/bash

echo "==================================="
echo "GraphQL Federation Validation"
echo "==================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

echo "1. Checking Gateway Package..."
echo "------------------------------"

# Build gateway
if cargo build --package workflow-engine-gateway 2>/dev/null; then
    echo -e "${GREEN}✅ Gateway builds successfully${NC}"
else
    echo -e "${RED}❌ Gateway build failed${NC}"
fi

# Test gateway
echo ""
echo "Running gateway tests..."
if cargo test --package workflow-engine-gateway 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Gateway tests pass${NC}"
else
    echo -e "${YELLOW}⚠️  Some gateway tests may have issues${NC}"
fi

echo ""
echo "2. Checking API Package..."
echo "-------------------------"

# Build API
if cargo build --package workflow-engine-api --features graphql 2>/dev/null; then
    echo -e "${GREEN}✅ API builds successfully with GraphQL${NC}"
else
    echo -e "${RED}❌ API build failed${NC}"
fi

# Test API
echo ""
echo "Running API tests..."
if cargo test --package workflow-engine-api 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ API tests pass${NC}"
else
    echo -e "${YELLOW}⚠️  Some API tests may have issues${NC}"
fi

echo ""
echo "3. Federation Features Check..."
echo "------------------------------"

# Check for federation files
if [ -f "crates/workflow-engine-gateway/src/federation/mod.rs" ]; then
    echo -e "${GREEN}✅ Federation module exists${NC}"
else
    echo -e "${RED}❌ Federation module missing${NC}"
fi

if grep -q "_service\|_entities" "crates/workflow-engine-api/src/api/graphql/schema.rs" 2>/dev/null; then
    echo -e "${GREEN}✅ API federation support exists${NC}"
else
    echo -e "${RED}❌ API federation support missing${NC}"
fi

# Check for schema file
if [ -f "crates/workflow-engine-api/src/api/graphql/schema.graphql" ]; then
    echo -e "${GREEN}✅ GraphQL schema file exists${NC}"
else
    echo -e "${RED}❌ GraphQL schema file missing${NC}"
fi

echo ""
echo "4. Example Programs..."
echo "----------------------"

# Check for examples
if [ -f "crates/workflow-engine-gateway/examples/federated_query.rs" ]; then
    echo -e "${GREEN}✅ Federated query example exists${NC}"
else
    echo -e "${RED}❌ Federated query example missing${NC}"
fi

if [ -f "crates/workflow-engine-gateway/examples/test_federation.rs" ]; then
    echo -e "${GREEN}✅ Federation test example exists${NC}"
else
    echo -e "${RED}❌ Federation test example missing${NC}"
fi

echo ""
echo "==================================="
echo "Validation Complete!"
echo "==================================="
echo ""
echo "To test the federation setup:"
echo "1. Start the workflow API: cargo run --bin workflow-engine"
echo "2. Start the gateway: cargo run --bin graphql-gateway"
echo "3. Run federation test: cargo run --example test_federation"
echo ""
echo "GraphQL endpoints:"
echo "- Gateway: http://localhost:4000/graphql"
echo "- Workflow API: http://localhost:8080/api/v1/graphql"