#!/usr/bin/env python3
"""
Comprehensive fix for WorkflowError usage in workflow-engine-mcp crate.
"""

import os
import re
import sys
from pathlib import Path

def fix_configuration_errors(content):
    """Fix ConfigurationError struct-style to constructor method."""
    # Pattern for struct-style ConfigurationError
    pattern = r'WorkflowError::ConfigurationError\s*\{\s*' \
              r'message:\s*([^,]+?)\s*,\s*' \
              r'config_key:\s*([^,]+?)\s*,\s*' \
              r'config_source:\s*([^,]+?)\s*,\s*' \
              r'expected_format:\s*([^,]+?)\s*,\s*' \
              r'received_value:\s*([^,]+?)\s*,\s*' \
              r'source:\s*[^}]+?\s*\}'
    
    def replacer(match):
        message = match.group(1).strip()
        config_key = match.group(2).strip()
        config_source = match.group(3).strip()
        expected_format = match.group(4).strip()
        received_value = match.group(5).strip()
        
        # Remove .to_string() if present
        for i, val in enumerate([message, config_key, config_source, expected_format]):
            if val.endswith('.to_string()'):
                val = val[:-12]  # Remove .to_string()
                if val.startswith('"') and val.endswith('"'):
                    pass  # Keep quotes
                else:
                    val = f'"{val}"'
            locals()[['message', 'config_key', 'config_source', 'expected_format'][i]] = val
        
        return f'WorkflowError::configuration_error(\n                {message},\n                {config_key},\n                {config_source},\n                {expected_format},\n                {received_value}\n            )'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def fix_runtime_errors(content):
    """Fix RuntimeError struct-style to constructor method."""
    # Pattern for struct-style RuntimeError
    pattern = r'WorkflowError::RuntimeError\s*\{\s*' \
              r'message:\s*([^,]+?)\s*,\s*' \
              r'component:\s*([^,]+?)\s*,\s*' \
              r'operation:\s*([^,]+?)\s*,\s*' \
              r'source:\s*[^}]+?\s*\}'
    
    def replacer(match):
        message = match.group(1).strip()
        component = match.group(2).strip()
        operation = match.group(3).strip()
        
        # Remove .to_string() and format properly
        for i, val in enumerate([message, component, operation]):
            if val.endswith('.to_string()'):
                val = val[:-12]
            if not (val.startswith('"') and val.endswith('"')):
                if not val.startswith('format!') and not val.startswith('&'):
                    val = f'"{val}"'
            locals()[['message', 'component', 'operation'][i]] = val
        
        return f'WorkflowError::runtime_error({message}, {component}, {operation})'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def fix_mcp_errors(content):
    """Fix MCP-related error patterns."""
    # MCPConnectionError pattern matching
    content = re.sub(
        r'WorkflowError::MCPConnectionError\s*\{\s*\.\.\.?\s*\}',
        'e',  # Just use the error variable directly in pattern matching
        content
    )
    
    # MCPError pattern matching
    content = re.sub(
        r'WorkflowError::MCPError\s*\{\s*\.\.\.?\s*\}',
        'e',
        content
    )
    
    return content

def fix_validation_errors(content):
    """Fix ValidationError struct-style to constructor method."""
    # Simple validation error pattern
    pattern = r'WorkflowError::ValidationError\s*\{\s*' \
              r'message:\s*([^,}]+?)\s*,?\s*' \
              r'field:\s*None\s*,?\s*' \
              r'constraint:\s*None\s*,?\s*' \
              r'context:\s*None\s*\}'
    
    def replacer(match):
        message = match.group(1).strip()
        if message.endswith('.to_string()'):
            message = message[:-12]
        if not (message.startswith('"') and message.endswith('"')) and not message.startswith('format!'):
            message = f'"{message}"'
        return f'WorkflowError::validation_error_simple({message})'
    
    content = re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)
    
    # Full validation error pattern
    pattern = r'WorkflowError::ValidationError\s*\{\s*' \
              r'message:\s*([^,]+?)\s*,\s*' \
              r'field:\s*Some\(([^)]+?)\)\s*,\s*' \
              r'constraint:\s*Some\(([^)]+?)\)\s*,\s*' \
              r'context:\s*([^}]+?)\s*\}'
    
    def replacer(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        
        # Clean up values
        for i, val in enumerate([message, field, constraint]):
            if val.endswith('.to_string()'):
                val = val[:-12]
            if not (val.startswith('"') and val.endswith('"')):
                val = f'"{val}"'
            locals()[['message', 'field', 'constraint'][i]] = val
        
        return f'WorkflowError::validation_error({message}, Some({field}), Some({constraint}), {context})'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def fix_file(file_path):
    """Fix WorkflowError usage in a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_configuration_errors(content)
        content = fix_runtime_errors(content)
        content = fix_mcp_errors(content)
        content = fix_validation_errors(content)
        
        # Write back if changes were made
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
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

if __name__ == "__main__":
    main()