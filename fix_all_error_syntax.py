#!/usr/bin/env python3
"""
Fix all error struct syntax to tuple variant constructor calls
This completes the critical TDD publication blocker fix from Test 7
"""

import os
import re
import glob
from pathlib import Path

# Map of error types to their constructor methods
ERROR_CONSTRUCTORS = {
    'ValidationError': {
        'simple': 'validation_error_simple',
        'full': 'validation_error',
        'with_value': 'validation_error_with_value'
    },
    'ConfigurationError': {
        'simple': 'configuration_error_simple', 
        'full': 'configuration_error'
    },
    'ProcessingError': {
        'simple': 'processing_error_simple',
        'full': 'processing_error',
        'with_context': 'processing_error_with_context'
    },
    'DatabaseError': {
        'simple': 'database_error_simple',
        'full': 'database_error'
    },
    'ApiError': {
        'simple': 'api_error_simple',
        'full': 'api_error'
    },
    'MCPError': {
        'simple': 'mcp_error_simple',
        'full': 'mcp_error'
    },
    'MCPConnectionError': {
        'simple': 'mcp_connection_error_simple',
        'full': 'mcp_connection_error',
        'with_retry': 'mcp_connection_error_with_retry'
    },
    'MCPProtocolError': {
        'simple': 'mcp_protocol_error_simple',
        'full': 'mcp_protocol_error'
    },
    'MCPTransportError': {
        'simple': 'mcp_transport_error_simple',
        'full': 'mcp_transport_error'
    },
    'SerializationError': {
        'simple': 'serialization_error_simple',
        'full': 'serialization_error'
    },
    'DeserializationError': {
        'simple': 'deserialization_error_simple',
        'full': 'deserialization_error'
    },
    'RegistryError': {
        'simple': 'registry_error_simple',
        'full': 'registry_error'
    },
    'CrossSystemError': {
        'simple': 'cross_system_error_simple',
        'full': 'cross_system_error'
    }
}

def fix_error_syntax(content):
    """
    Convert all error struct syntax to constructor method calls
    """
    
    for error_type, constructors in ERROR_CONSTRUCTORS.items():
        # Pattern for this error type with struct syntax
        pattern = re.compile(
            rf'WorkflowError::{error_type}\s*\{{[^}}]+\}}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_error(match):
            error_text = match.group(0)
            
            # Extract field values
            message_match = re.search(r'message:\s*([^,\n}]+)', error_text)
            
            if not message_match:
                # If no message field, this might be a malformed pattern
                return error_text
            
            message = message_match.group(1).strip()
            
            # For now, use simple constructor for all - this covers most cases
            # More sophisticated detection could be added if needed
            simple_constructor = constructors.get('simple')
            if simple_constructor:
                return f"WorkflowError::{simple_constructor}({message})"
            else:
                # Fallback to first available constructor
                first_constructor = list(constructors.values())[0]
                return f"WorkflowError::{first_constructor}({message})"
        
        content = pattern.sub(replace_error, content)
    
    return content

def main():
    print("🔧 Fixing all error syntax in MCP crate...")
    
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
        
        # Check if file contains error struct syntax
        has_error_structs = any(f"WorkflowError::{error_type} {{" in original_content 
                               for error_type in ERROR_CONSTRUCTORS.keys())
        
        if not has_error_structs:
            continue
        
        # Apply fixes
        fixed_content = fix_error_syntax(original_content)
        
        # Count total error struct patterns before and after
        before_count = sum(original_content.count(f"WorkflowError::{error_type} {{") 
                          for error_type in ERROR_CONSTRUCTORS.keys())
        after_count = sum(fixed_content.count(f"WorkflowError::{error_type} {{") 
                         for error_type in ERROR_CONSTRUCTORS.keys())
        
        replacements = before_count - after_count
        
        if replacements > 0:
            print(f"  ✅ Fixed {replacements} error patterns")
            
            # Create backup
            backup_path = f"{file_path}.backup2"
            with open(backup_path, 'w', encoding='utf-8') as f:
                f.write(original_content)
            
            # Write fixed content
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            
            fixed_count += 1
            total_replacements += replacements
    
    print(f"\n🎉 Fixed {total_replacements} error patterns across {fixed_count} files")
    
    # Test compilation
    print("\n🧪 Testing compilation...")
    os.system("cargo check -p workflow-engine-mcp --message-format=short 2>&1 | head -15")

if __name__ == "__main__":
    main()