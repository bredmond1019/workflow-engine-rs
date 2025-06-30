#!/usr/bin/env python3
"""
Fix the mangled format! calls in config_builder.rs
"""

import re

def fix_config_builder():
    file_path = "/Users/brandon/Documents/Projects/ai-engineering/workflow-engine-rs/worktree-graphql/crates/workflow-engine-mcp/src/config_builder.rs"
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Fix all the mangled format! patterns
    replacements = [
        # WebSocket empty URL
        (
            r'return Err\(WorkflowError::configuration_error\(format!\(format!\("WebSocket URL cannot be empty for server \'{[^)]*\)\)', 
            'return Err(WorkflowError::configuration_error(\n                                format!("WebSocket URL cannot be empty for server \'{}\'", name),\n                                format!("servers.{}.transport.url", name),\n                                "builder",\n                                "non-empty WebSocket URL",\n                                Some(url.clone())\n                            ))'
        ),
        # WebSocket URL prefix
        (
            r'return Err\(WorkflowError::configuration_error\(format!\(format!\("WebSocket URL must start with \'ws://\' or \'wss://\' for server \'{[^)]*\)\)',
            'return Err(WorkflowError::configuration_error(\n                            format!("WebSocket URL must start with \'ws://\' or \'wss://\' for server \'{}\'", name),\n                            format!("servers.{}.transport.url", name),\n                            "builder",\n                            "URL starting with ws:// or wss://",\n                            Some(url.clone())\n                        ))'
        ),
        # HTTP empty URL
        (
            r'return Err\(WorkflowError::configuration_error\(format!\(format!\("HTTP base URL cannot be empty for server \'{[^)]*\)\)',
            'return Err(WorkflowError::configuration_error(\n                                format!("HTTP base URL cannot be empty for server \'{}\'", name),\n                                format!("servers.{}.transport.base_url", name),\n                                "builder",\n                                "non-empty HTTP URL",\n                                Some(base_url.clone())\n                            ))'
        ),
        # HTTP URL prefix
        (
            r'return Err\(WorkflowError::configuration_error\(format!\(format!\("HTTP base URL must start with \'http://\' or \'https://\' for server \'{[^)]*\)\)',
            'return Err(WorkflowError::configuration_error(\n                            format!("HTTP base URL must start with \'http://\' or \'https://\' for server \'{}\'", name),\n                            format!("servers.{}.transport.base_url", name),\n                            "builder",\n                            "URL starting with http:// or https://",\n                            Some(base_url.clone())\n                        ))'
        ),
        # Stdio empty command
        (
            r'return Err\(WorkflowError::configuration_error\(format!\(format!\("Stdio command cannot be empty for server \'{[^)]*\)\)',
            'return Err(WorkflowError::configuration_error(\n                                format!("Stdio command cannot be empty for server \'{}\'", name),\n                                format!("servers.{}.transport.command", name),\n                                "builder",\n                                "non-empty command",\n                                Some(command.clone())\n                            ))'
        ),
    ]
    
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    # Fix the specific lines that got messed up
    # Line 291-298
    content = re.sub(
        r'return Err\(WorkflowError::configuration_error\(format!\(format!\("WebSocket URL cannot be empty for server \'\{[^}]*\), "", "", "", None\)\'"[^)]*\)',
        '''return Err(WorkflowError::configuration_error(
                                format!("WebSocket URL cannot be empty for server '{}'", name),
                                format!("servers.{}.transport.url", name),
                                "builder",
                                "non-empty WebSocket URL",
                                Some(url.clone())
                            ))''',
        content
    )
    
    # Write back
    with open(file_path, 'w') as f:
        f.write(content)
    
    print("Fixed config_builder.rs")

if __name__ == "__main__":
    fix_config_builder()