#!/bin/bash
set -e

echo "Building Docker image with debug output..."

# Force ARM64 platform to match your local environment
docker build --platform linux/arm64 \
  --progress=plain \
  --no-cache \
  -f Dockerfile.simple \
  -t workflow-engine-debug:latest \
  .

echo "Build completed!"