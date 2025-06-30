#!/usr/bin/env python3
"""
Fix WorkflowError usage in workflow-engine-mcp crate to use new constructor methods.
"""

import os
import re
import sys
from pathlib import Path

# Define the error transformations
ERROR_TRANSFORMATIONS = {
    # ValidationError variants
    r'WorkflowError::ValidationError\s*\{\s*message:\s*([^,}]+),?\s*field:\s*([^,}]+),?\s*constraint:\s*([^,}]+),?\s*context:\s*([^}]+)\s*\}': 
        r'WorkflowError::validation_error(\1, \2, \3, \4)',
    r'WorkflowError::ValidationError\s*\{\s*message:\s*([^,}]+),?\s*field:\s*None,?\s*constraint:\s*None,?\s*context:\s*None\s*\}':
        r'WorkflowError::validation_error(\1, None, None, None)',
    
    # ConfigurationError variants
    r'WorkflowError::ConfigurationError\s*\{\s*key:\s*([^,}]+),?\s*value:\s*([^,}]+),?\s*expected:\s*([^}]+)\s*\}':
        r'WorkflowError::configuration_error(\1, \2, \3)',
    
    # ProcessingError variants
    r'WorkflowError::ProcessingError\s*\{\s*message:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::processing_error(\1, \2)',
    
    # NetworkError variants
    r'WorkflowError::NetworkError\s*\{\s*url:\s*([^,}]+),?\s*status_code:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::network_error(\1, \2, \3)',
    
    # DatabaseError variants
    r'WorkflowError::DatabaseError\s*\{\s*operation:\s*([^,}]+),?\s*table:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::database_error(\1, \2, \3)',
    
    # AiError variants
    r'WorkflowError::AiError\s*\{\s*provider:\s*([^,}]+),?\s*model:\s*([^,}]+),?\s*error_type:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::ai_error(\1, \2, \3, \4)',
    
    # StorageError variants
    r'WorkflowError::StorageError\s*\{\s*path:\s*([^,}]+),?\s*operation:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::storage_error(\1, \2, \3)',
    
    # ServiceError variants
    r'WorkflowError::ServiceError\s*\{\s*service:\s*([^,}]+),?\s*operation:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::service_error(\1, \2, \3)',
    
    # TemplateError variants
    r'WorkflowError::TemplateError\s*\{\s*template:\s*([^,}]+),?\s*line:\s*([^,}]+),?\s*column:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::template_error(\1, \2, \3, \4)',
    
    # Simple tuple variants (already correct, but check for any struct usage)
    r'WorkflowError::Timeout\s*\{\s*duration_ms:\s*([^}]+)\s*\}':
        r'WorkflowError::Timeout(\1)',
    r'WorkflowError::NotFound\s*\{\s*resource_type:\s*([^,}]+),?\s*id:\s*([^}]+)\s*\}':
        r'WorkflowError::NotFound(\1, \2)',
    r'WorkflowError::Unauthorized\s*\{\s*reason:\s*([^}]+)\s*\}':
        r'WorkflowError::Unauthorized(\1)',
    
    # McpError variants
    r'WorkflowError::McpClientError\s*\{\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::McpClientError(\1)',
    r'WorkflowError::McpServerError\s*\{\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::McpServerError(\1)',
    r'WorkflowError::TransportError\s*\{\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::TransportError(\1)',
    r'WorkflowError::CodecError\s*\{\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::CodecError(\1)',
    r'WorkflowError::McpProtocolError\s*\{\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::McpProtocolError(\1)',
    r'WorkflowError::McpToolError\s*\{\s*tool:\s*([^,}]+),?\s*details:\s*([^}]+)\s*\}':
        r'WorkflowError::McpToolError(\1, \2)',
}

def fix_file(file_path):
    """Fix WorkflowError usage in a single file."""
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    changes_made = False
    
    # Apply all transformations
    for pattern, replacement in ERROR_TRANSFORMATIONS.items():
        new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
        if new_content != content:
            changes_made = True
            content = new_content
    
    # Write back if changes were made
    if changes_made:
        with open(file_path, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    mcp_src = Path("/Users/brandon/Documents/Projects/ai-engineering/workflow-engine-rs/worktree-graphql/crates/workflow-engine-mcp/src")
    
    if not mcp_src.exists():
        print(f"Error: MCP source directory not found: {mcp_src}")
        sys.exit(1)
    
    fixed_files = []
    total_files = 0
    
    # Process all Rust files
    for rust_file in mcp_src.rglob("*.rs"):
        total_files += 1
        if fix_file(rust_file):
            fixed_files.append(rust_file)
            print(f"Fixed: {rust_file.relative_to(mcp_src.parent.parent)}")
    
    print(f"\nProcessed {total_files} files, fixed {len(fixed_files)} files")
    
    if fixed_files:
        print("\nFixed files:")
        for f in fixed_files:
            print(f"  - {f.relative_to(mcp_src.parent.parent)}")

if __name__ == "__main__":
    main()