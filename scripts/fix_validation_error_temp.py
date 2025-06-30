#!/usr/bin/env python3
"""
Fix ValidationError_TEMP back to proper constructor methods.
"""

import re
from pathlib import Path

def fix_validation_error_temp(content):
    """Fix ValidationError_TEMP patterns to use proper constructors."""
    
    # Pattern 1: ValidationError_TEMP with field, value, constraint, context
    pattern1 = r'WorkflowError::ValidationError_TEMP\s*\{\s*' \
               r'message:\s*([^,]+?)\s*,\s*' \
               r'field:\s*([^,]+?)\s*,\s*' \
               r'value:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'constraint:\s*([^,]+?)\s*,\s*' \
               r'context:\s*([^}]+?)\s*\}'
    
    def replacer1(match):
        message = match.group(1).strip()
        field = match.group(2).strip()  
        value = match.group(3).strip()
        constraint = match.group(4).strip()
        context = match.group(5).strip()
        
        # Remove .to_string() if present
        for var in [message, field, constraint, context]:
            if var.endswith('.to_string()'):
                var = var[:-12]
        
        return f'WorkflowError::validation_error_with_value({message}, Some({field}), Some({constraint}), Some({context}), Some({value}))'
    
    content = re.sub(pattern1, replacer1, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 2: ValidationError_TEMP with field None (missing required field)
    pattern2 = r'WorkflowError::ValidationError_TEMP\s*\{\s*' \
               r'message:\s*([^,]+?)\s*,\s*' \
               r'field:\s*([^,]+?)\s*,\s*' \
               r'value:\s*None\s*,\s*' \
               r'constraint:\s*([^,]+?)\s*,\s*' \
               r'context:\s*([^}]+?)\s*\}'
    
    def replacer2(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        
        # Remove .to_string() if present
        for var in [message, field, constraint, context]:
            if var.endswith('.to_string()'):
                var = var[:-12]
        
        return f'WorkflowError::validation_error({message}, Some({field}), Some({constraint}), Some({context}))'
    
    content = re.sub(pattern2, replacer2, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        content = fix_validation_error_temp(content)
        
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
    for rust_file in mcp_src.rglob("*.rs"):
        if process_file(rust_file):
            fixed += 1
            print(f"Fixed: {rust_file.relative_to(mcp_src)}")
    
    print(f"\nFixed {fixed} files")

if __name__ == "__main__":
    main()