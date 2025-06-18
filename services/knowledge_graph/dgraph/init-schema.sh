#!/bin/sh
# Script to initialize DGraph with the GraphQL schema

set -e

echo "Waiting for DGraph to be ready..."
until curl -f http://dgraph-alpha:8080/health > /dev/null 2>&1; do
  echo "DGraph is not ready yet. Retrying in 5 seconds..."
  sleep 5
done

echo "DGraph is ready. Uploading GraphQL schema..."

# Upload the GraphQL schema to DGraph
curl -X POST http://dgraph-alpha:8080/admin/schema \
  -H "Content-Type: application/graphql" \
  --data-binary "@/schema.graphql"

if [ $? -eq 0 ]; then
  echo "Schema uploaded successfully!"
else
  echo "Failed to upload schema"
  exit 1
fi

# Create initial indexes for vector similarity search
echo "Creating vector similarity indexes..."
curl -X POST http://dgraph-alpha:8080/alter -d '{
  "schema": "
    Concept.embedding: [float] @index(hnsw(metric:\"euclidean\", exponent:\"4\", maxLevels:\"12\")) .
  "
}'

echo "Schema initialization complete!"