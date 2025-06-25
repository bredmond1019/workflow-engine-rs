#!/bin/bash

echo "=== Local Build Environment ==="
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Target: $(rustc -vV | grep host)"
echo "OS: $(uname -a)"
echo "Arch: $(uname -m)"
echo "================================"

echo
echo "=== Environment Variables ==="
env | grep -E "(RUST|CARGO|DATABASE|SQLX)" || echo "No relevant env vars found"
echo "================================"

echo
echo "=== Cargo Features ==="
cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "workflow-engine-app") | .features' 2>/dev/null || echo "Could not parse features"
echo "================================"

echo
echo "=== Build Command ==="
echo "Running: cargo build --release --bin workflow-engine --verbose"
echo "================================"