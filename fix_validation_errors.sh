#!/bin/bash

# Fix ValidationError struct syntax to use proper constructor methods
# This script converts old struct syntax to new tuple variant with constructor

echo "🔧 Fixing ValidationError syntax across MCP crate..."

# Find all files with ValidationError struct syntax
FILES=$(rg "WorkflowError::ValidationError \{" crates/workflow-engine-mcp/src/ -l)

for file in $FILES; do
    echo "📝 Fixing: $file"
    
    # Use sed to replace the struct syntax with constructor method
    # This is a complex pattern, so we'll do it step by step
    
    # First, backup the file
    cp "$file" "$file.backup"
    
    # Replace multi-line ValidationError struct patterns with validation_error constructor
    sed -i '' '
    # Start of ValidationError pattern
    /WorkflowError::ValidationError {/{
        # Read the next lines until we find the closing }
        :a
        N
        /})/!ba
        # Now we have the full pattern, replace it
        s/WorkflowError::ValidationError {\n[[:space:]]*message:[[:space:]]*\([^,]*\),[[:space:]]*\n[[:space:]]*field:[[:space:]]*\([^,]*\),[[:space:]]*\n[[:space:]]*value:[[:space:]]*None,[[:space:]]*\n[[:space:]]*constraint:[[:space:]]*\([^,]*\),[[:space:]]*\n[[:space:]]*context:[[:space:]]*\([^,]*\),[[:space:]]*\n[[:space:]]*}/WorkflowError::validation_error(\n                \1,\n                \2,\n                \3,\n                \4\n            )/g
    }' "$file"
    
    echo "  ✅ Updated ValidationError patterns in $file"
done

echo "🎉 ValidationError syntax fixes complete!"

# Test compilation
echo "🧪 Testing compilation..."
cargo check -p workflow-engine-mcp --message-format=short 2>&1 | head -20