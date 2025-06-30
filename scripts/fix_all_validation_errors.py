#!/usr/bin/env python3
"""
Fix all ValidationError patterns in MCP crate.
"""

import re
from pathlib import Path

def fix_validation_errors(content):
    """Fix all ValidationError patterns."""
    
    # Pattern 1: ValidationError with field, value, constraint, context
    pattern1 = r'WorkflowError::ValidationError\s*\{\s*' \
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
        if message.endswith('.to_string()'):
            message = message[:-12]
            if not (message.startswith('"') and message.endswith('"')):
                message = f'"{message}"'
        
        # Handle field
        if not field.startswith('Some('):
            if field.endswith('.to_string()'):
                field = field[:-12]
            field = f'Some({field})'
        
        # Handle constraint  
        if not constraint.startswith('Some('):
            if constraint.endswith('.to_string()'):
                constraint = constraint[:-12]
            constraint = f'Some({constraint})'
            
        # Handle context
        if not context.startswith('Some('):
            if context.endswith('.to_string()'):
                context = context[:-12]
            context = f'Some({context})'
        
        return f'WorkflowError::validation_error_with_value({message}, {field}, {constraint}, {context}, Some({value}))'
    
    content = re.sub(pattern1, replacer1, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 2: Simple ValidationError with message only
    pattern2 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,}]+?)\s*,?\s*' \
               r'field:\s*None\s*,?\s*' \
               r'value:\s*None\s*,?\s*' \
               r'constraint:\s*None\s*,?\s*' \
               r'context:\s*None\s*,?\s*\}'
    
    def replacer2(match):
        message = match.group(1).strip()
        if message.endswith('.to_string()'):
            message = message[:-12]
            if not (message.startswith('"') and message.endswith('"')) and not message.startswith('format!'):
                message = f'"{message}"'
        return f'WorkflowError::validation_error_simple({message})'
    
    content = re.sub(pattern2, replacer2, content, flags=re.MULTILINE | re.DOTALL)
    
    # Pattern 3: ValidationError with field but no value
    pattern3 = r'WorkflowError::ValidationError\s*\{\s*' \
               r'message:\s*([^,]+?)\s*,\s*' \
               r'field:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'value:\s*None\s*,\s*' \
               r'constraint:\s*Some\(([^)]+?)\)\s*,\s*' \
               r'context:\s*([^}]+?)\s*\}'
    
    def replacer3(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        constraint = match.group(3).strip()
        context = match.group(4).strip()
        
        if message.endswith('.to_string()'):
            message = message[:-12]
            
        return f'WorkflowError::validation_error({message}, Some({field}), Some({constraint}), {context})'
    
    content = re.sub(pattern3, replacer3, content, flags=re.MULTILINE | re.DOTALL)
    
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
    for rust_file in mcp_src.rglob("*.rs"):
        if process_file(rust_file):
            fixed += 1
            print(f"Fixed: {rust_file.relative_to(mcp_src)}")
    
    print(f"\nFixed {fixed} files")

if __name__ == "__main__":
    main()