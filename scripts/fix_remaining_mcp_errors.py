#!/usr/bin/env python3
"""
Fix remaining WorkflowError patterns in MCP crate.
"""

import re
from pathlib import Path

def fix_complex_config_errors(content):
    """Fix complex ConfigurationError patterns with format! macros."""
    
    # Find all ConfigurationError struct patterns
    pattern = r'WorkflowError::ConfigurationError\s*\{([^}]+)\}'
    
    def extract_field(field_content, field_name):
        """Extract field value from struct content."""
        # Try different patterns
        patterns = [
            rf'{field_name}:\s*format!\(([^)]+)\)',
            rf'{field_name}:\s*Some\(([^)]+)\)',
            rf'{field_name}:\s*"([^"]+)"\.to_string\(\)',
            rf'{field_name}:\s*([^,]+?)(?:,|\s*$)'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, field_content, re.DOTALL)
            if match:
                value = match.group(1).strip()
                # Handle format! specially
                if 'format!' in field_content and field_name in ['message', 'config_key', 'expected_format']:
                    return f'format!({value})'
                # Handle Some() values
                elif field_name == 'received_value' and 'Some(' in field_content:
                    return f'Some({value})'
                # String literals
                elif value.endswith('.to_string()'):
                    return value[:-12]  # Remove .to_string()
                else:
                    return value
        
        # Default values
        if field_name == 'received_value':
            return 'None'
        return '""'
    
    def replacer(match):
        field_content = match.group(1)
        
        # Extract each field
        message = extract_field(field_content, 'message')
        config_key = extract_field(field_content, 'config_key')
        config_source = extract_field(field_content, 'config_source')
        expected_format = extract_field(field_content, 'expected_format')
        received_value = extract_field(field_content, 'received_value')
        
        # Clean up config_source if it's a simple string
        if config_source == '"builder".to_string()' or config_source == '"builder"':
            config_source = '"builder"'
        
        return f'WorkflowError::configuration_error({message}, {config_key}, {config_source}, {expected_format}, {received_value})'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def fix_runtime_errors_in_metrics(content):
    """Fix RuntimeError patterns in metrics.rs."""
    # Pattern for RuntimeError in metrics
    pattern = r'\.map_err\(\|e\|\s*WorkflowError::RuntimeError\s*\{[^}]+\}\)'
    
    def replacer(match):
        # Extract the error message pattern
        inner = match.group(0)
        if 'Failed to record' in inner:
            return '.map_err(|e| WorkflowError::runtime_error(format!("Failed to record metric: {}", e), "mcp_metrics", "record_metric"))'
        else:
            return '.map_err(|e| WorkflowError::runtime_error(e.to_string(), "mcp_metrics", "metrics_operation"))'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def fix_validation_errors_in_tools(content):
    """Fix ValidationError patterns in tool files."""
    # Simple validation error
    pattern = r'WorkflowError::ValidationError\s*\{\s*message:\s*([^,}]+),\s*field:\s*None,\s*constraint:\s*None,\s*context:\s*None\s*\}'
    
    def replacer(match):
        message = match.group(1).strip()
        if message.startswith('format!'):
            return f'WorkflowError::validation_error_simple({message})'
        elif message.endswith('.to_string()'):
            message = message[:-12]
            if not (message.startswith('"') and message.endswith('"')):
                message = f'"{message}"'
            return f'WorkflowError::validation_error_simple({message})'
        else:
            return f'WorkflowError::validation_error_simple({message})'
    
    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def process_file(file_path):
    """Process a single file."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        
        # Apply fixes based on file
        if 'config_builder.rs' in str(file_path):
            content = fix_complex_config_errors(content)
        elif 'metrics.rs' in str(file_path):
            content = fix_runtime_errors_in_metrics(content)
        elif 'tools' in str(file_path):
            content = fix_validation_errors_in_tools(content)
            content = fix_complex_config_errors(content)  # Some tools have config errors too
        
        # General fixes for all files
        content = fix_complex_config_errors(content)
        
        if content != original:
            with open(file_path, 'w') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error in {file_path}: {e}")
        return False

def main():
    mcp_src = Path("/Users/brandon/Documents/Projects/ai-engineering/workflow-engine-rs/worktree-graphql/crates/workflow-engine-mcp/src")
    
    fixed = 0
    for rust_file in mcp_src.rglob("*.rs"):
        if process_file(rust_file):
            fixed += 1
            print(f"Fixed: {rust_file.name}")
    
    print(f"\nFixed {fixed} files")

if __name__ == "__main__":
    main()