#!/usr/bin/env python3
"""
Fix validation_error calls that use Some() wrappers.
"""

import re
from pathlib import Path

def fix_validation_error_some(content):
    """Fix validation_error calls with Some() wrappers."""
    
    # Pattern for validation_error with Some() parameters
    pattern = r'WorkflowError::validation_error\(([^,]+),\s*Some\(([^)]+)\),\s*Some\(([^)]+)\),\s*Some\(([^)]+)\)\)'
    
    def replacer(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        
        # Remove .to_string() if present
        for i, val in enumerate([message, field, constraint, context]):
            if val.endswith('.to_string()'):
                val = val[:-12]
                # Add quotes if it's a literal string
                if not val.startswith('"') and not val.startswith('format!'):
                    val = f'"{val}"'
            locals()[['message', 'field', 'constraint', 'context'][i]] = val
        
        return f'WorkflowError::validation_error({message}, {field}, {constraint}, {context})'
    
    content = re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)
    
    # Also fix validation_error_with_value patterns
    pattern2 = r'WorkflowError::validation_error_with_value\(([^,]+),\s*Some\(([^)]+)\),\s*Some\(([^)]+)\),\s*Some\(([^)]+)\),\s*Some\(([^)]+)\)\)'
    
    def replacer2(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        value = match.group(5).strip()
        
        # Remove .to_string() if present
        for i, val in enumerate([message, field, constraint, context]):
            if val.endswith('.to_string()'):
                val = val[:-12]
                if not val.startswith('"') and not val.startswith('format!'):
                    val = f'"{val}"'
            locals()[['message', 'field', 'constraint', 'context'][i]] = val
        
        # Value stays as Some()
        return f'WorkflowError::validation_error_with_value({message}, {field}, {constraint}, {context}, Some({value}))'
    
    content = re.sub(pattern2, replacer2, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        content = fix_validation_error_some(content)
        
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