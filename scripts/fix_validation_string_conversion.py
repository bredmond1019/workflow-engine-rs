#!/usr/bin/env python3
"""
Fix validation_error calls that incorrectly use Some() for parameters.
"""

import re
from pathlib import Path

def fix_validation_error_calls(content):
    """Fix validation_error calls with Some() parameters."""
    
    # Pattern: validation_error with Some() wrappers
    pattern = r'WorkflowError::validation_error\((.*?)\)'
    
    def fix_some_params(match):
        full_match = match.group(0)
        params = match.group(1)
        
        # If it contains Some(), we need to fix it
        if 'Some(' in params:
            # Replace Some("string".to_string()) with just "string"
            params = re.sub(r'Some\("([^"]+)"\.to_string\(\)\)', r'"\1"', params)
            # Replace Some(var.to_string()) with var
            params = re.sub(r'Some\(([^)]+)\.to_string\(\)\)', r'\1', params)
            
            return f'WorkflowError::validation_error({params})'
        
        return full_match
    
    # Apply fixes
    content = re.sub(pattern, fix_some_params, content, flags=re.DOTALL)
    
    return content

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        content = fix_validation_error_calls(content)
        
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