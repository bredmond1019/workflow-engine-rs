#!/usr/bin/env python3
"""
Fix ValidationError struct syntax to tuple variant constructor calls
This addresses a critical TDD publication blocker discovered in Test 7
"""

import os
import re
import glob
from pathlib import Path

def fix_validation_error_syntax(content):
    """
    Convert ValidationError struct syntax to constructor method calls
    
    From: WorkflowError::ValidationError { message: ..., field: ..., value: ..., constraint: ..., context: ... }
    To: WorkflowError::validation_error(...) or WorkflowError::validation_error_with_value(...)
    """
    
    # Pattern for ValidationError with value field
    pattern_with_value = re.compile(
        r'WorkflowError::ValidationError\s*\{\s*'
        r'message:\s*([^,]+),\s*'
        r'field:\s*([^,]+),\s*'
        r'value:\s*(Some\([^)]+\)|None),\s*'
        r'constraint:\s*([^,]+),\s*'
        r'context:\s*([^,]+),?\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )
    
    def replace_with_value(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        value = match.group(3).strip()
        constraint = match.group(4).strip()
        context = match.group(5).strip()
        
        if value == "None":
            return f"""WorkflowError::validation_error(
                {message},
                {field},
                {constraint},
                {context}
            )"""
        else:
            return f"""WorkflowError::validation_error_with_value(
                {message},
                {field},
                {value},
                {constraint},
                {context}
            )"""
    
    # Apply the replacement
    fixed_content = pattern_with_value.sub(replace_with_value, content)
    
    return fixed_content

def main():
    print("🔧 Fixing ValidationError syntax in MCP crate...")
    
    # Find all Rust files in the MCP crate
    mcp_src_dir = "crates/workflow-engine-mcp/src"
    
    if not os.path.exists(mcp_src_dir):
        print(f"❌ Directory {mcp_src_dir} not found!")
        return
    
    rust_files = list(Path(mcp_src_dir).rglob("*.rs"))
    
    fixed_count = 0
    total_replacements = 0
    
    for file_path in rust_files:
        print(f"📝 Checking: {file_path}")
        
        # Read file content
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                original_content = f.read()
        except Exception as e:
            print(f"❌ Error reading {file_path}: {e}")
            continue
        
        # Check if file contains ValidationError struct syntax
        if "WorkflowError::ValidationError {" not in original_content:
            continue
        
        # Apply fixes
        fixed_content = fix_validation_error_syntax(original_content)
        
        # Count replacements
        replacements = original_content.count("WorkflowError::ValidationError {") - fixed_content.count("WorkflowError::ValidationError {")
        
        if replacements > 0:
            print(f"  ✅ Fixed {replacements} ValidationError patterns")
            
            # Create backup
            backup_path = f"{file_path}.backup"
            with open(backup_path, 'w', encoding='utf-8') as f:
                f.write(original_content)
            
            # Write fixed content
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            
            fixed_count += 1
            total_replacements += replacements
    
    print(f"\n🎉 Fixed {total_replacements} ValidationError patterns across {fixed_count} files")
    
    # Test compilation
    print("\n🧪 Testing compilation...")
    os.system("cargo check -p workflow-engine-mcp --message-format=short 2>&1 | head -10")

if __name__ == "__main__":
    main()