#!/usr/bin/env python3
"""
Fix ValidationError struct-style to use constructor methods.
"""

import re
from pathlib import Path

def fix_validation_errors(content):
    """Fix all ValidationError patterns."""
    
    # Pattern 1: Full ValidationError with all fields
    pattern1 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,]+?)\s*,\s*' \
               r'field:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'value:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'constraint:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'context:\s*([^}]+?)\s*\}'
    
    def replacer1(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        value = match.group(3).strip()
        constraint = match.group(4).strip()
        context = match.group(5).strip()
        
        # Clean up .to_string() calls
        for var in [message, field, value, constraint]:
            if var.endswith('.to_string()'):
                var = var[:-12]
        
        # Use validation_error_with_value since we have value
        return f'WorkflowError::validation_error_with_value({message}, Some({field}), Some({constraint}), {context}, Some({value}))'
    
    content = re.sub(pattern1, replacer1, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 2: ValidationError without value field
    pattern2 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,]+?)\s*,\s*' \
               r'field:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'constraint:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'context:\s*([^}]+?)\s*\}'
    
    def replacer2(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        
        return f'WorkflowError::validation_error({message}, Some({field}), Some({constraint}), {context})'
    
    content = re.sub(pattern2, replacer2, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 3: Simple ValidationError with just message
    pattern3 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,}]+?)\s*,?\s*' \
               r'field:\s*None\s*,?\s*' \
               r'constraint:\s*None\s*,?\s*' \
               r'context:\s*None\s*,?\s*' \
               r'value:\s*None\s*,?\s*\}'
    
    def replacer3(match):
        message = match.group(1).strip()
        if message.endswith('.to_string()'):
            message = message[:-12]
        return f'WorkflowError::validation_error_simple({message})'
    
    content = re.sub(pattern3, replacer3, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 4: ValidationError without value field (alternate)
    pattern4 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,}]+?)\s*,?\s*' \
               r'field:\s*None\s*,?\s*' \
               r'constraint:\s*None\s*,?\s*' \
               r'context:\s*None\s*,?\s*\}'
    
    def replacer4(match):
        message = match.group(1).strip()
        if message.endswith('.to_string()'):
            message = message[:-12]
        return f'WorkflowError::validation_error_simple({message})'
    
    content = re.sub(pattern4, replacer4, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        content = fix_validation_errors(content)
        
        if content != original:
            with open(file_path, 'w') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    mcp_src = Path("/Users/brandon/Documents/Projects/ai-engineering/workflow-engine-rs/worktree-graphql/crates/workflow-engine-mcp/src")
    
    fixed = 0
    # Process all tool files
    for rust_file in (mcp_src / "server" / "customer_support" / "tools").glob("*.rs"):
        if process_file(rust_file):
            fixed += 1
            print(f"Fixed: {rust_file.name}")
    
    # Process other files
    for rust_file in (mcp_src / "server" / "knowledge_base" / "tools").glob("*.rs"):
        if process_file(rust_file):
            fixed += 1
            print(f"Fixed: {rust_file.name}")
    
    print(f"\nFixed {fixed} files")

if __name__ == "__main__":
    main()