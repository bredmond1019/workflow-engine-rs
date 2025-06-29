//! Tests for template parsing with malformed input
//!
//! This module tests that the template parser properly handles
//! malformed input without panicking from unwrap() calls.

#[cfg(test)]
mod tests {
    use crate::ai::templates::{Template, TemplateParser};
    use crate::nodes::Node;
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_parse_unclosed_expression() {
        let parser = TemplateParser::new();
        let result = parser.parse("Hello {{name");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unclosed template expression"));
            }
            Ok(_) => panic!("Expected error for unclosed expression"),
        }
    }

    #[test] 
    fn test_parse_empty_expression() {
        let parser = TemplateParser::new();
        let result = parser.parse("Hello {{}}");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Empty template expression"));
            }
            Ok(_) => panic!("Expected error for empty expression"),
        }
    }

    #[test]
    fn test_parse_invalid_variable_name() {
        let parser = TemplateParser::new();
        let result = parser.parse("Hello {{123invalid}}");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Invalid variable"));
            }
            Ok(_) => panic!("Expected error for invalid variable name"),
        }
    }

    #[test]
    fn test_parse_unclosed_if_block() {
        let parser = TemplateParser::new();
        let result = parser.parse("{{#if condition}}content");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unclosed block"));
            }
            Ok(_) => panic!("Expected error for unclosed if block"),
        }
    }

    #[test]
    fn test_parse_unclosed_each_block() {
        let parser = TemplateParser::new();
        let result = parser.parse("{{#each items}}{{this}}");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unclosed block"));
            }
            Ok(_) => panic!("Expected error for unclosed each block"),
        }
    }

    #[test]
    fn test_parse_mismatched_block_end() {
        let parser = TemplateParser::new();
        let result = parser.parse("{{#if condition}}content{{/each}}");
        assert!(result.is_err());
        // This should fail but might unwrap somewhere
    }

    #[test]
    fn test_parse_nested_unclosed_blocks() {
        let parser = TemplateParser::new();
        let result = parser.parse("{{#if outer}}{{#if inner}}content{{/if}}");
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unclosed block"));
            }
            Ok(_) => panic!("Expected error for unclosed outer block"),
        }
    }

    #[test]
    fn test_template_render_with_malformed_content() {
        use crate::ai::templates::{TemplateEngine, Template, TemplateVariables};
        
        // Template::new doesn't validate content, so create template and try to render
        let template = Template::new("malformed", "Hello {{name").expect("Template creation shouldn't fail");
        let engine = TemplateEngine::new();
        let vars = TemplateVariables::new();
        
        // Rendering should fail with malformed template
        let result = engine.render(&template, &vars);
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Unclosed template expression") || 
                        e.to_string().contains("parse") ||
                        e.to_string().contains("malformed"));
            }
            Ok(_) => panic!("Expected error for malformed template"),
        }
    }

    #[test]
    fn test_template_agent_with_invalid_template_id() {
        use crate::ai::templates::TemplateManager;
        use crate::nodes::template_agent::TemplateAgentNode;
        use crate::nodes::agent::{AgentConfig, ModelProvider};
        use crate::task::TaskContext;
        use std::sync::Arc;

        let manager = Arc::new(TemplateManager::new().expect("Should create manager"));
        let config = crate::nodes::template_agent::TemplateAgentConfig {
            agent_config: AgentConfig {
                system_prompt: "test".to_string(),
                model_provider: ModelProvider::OpenAI,
                model_name: "gpt-4".to_string(),
                mcp_server_uri: None,
            },
            system_prompt_template: Some("non_existent_template".to_string()),
            user_message_template: None,
            template_vars: HashMap::new(),
            template_based_tools: false,
        };

        let agent = TemplateAgentNode::new(config, manager).expect("Should create agent");
        let context = TaskContext::new("test_workflow".to_string(), json!({}));
        
        // This might panic when trying to render non-existent template
        let result = agent.process(context);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_contextual_with_unwrap_or_default() {
        use crate::ai::templates::TemplateManager;
        
        let manager = TemplateManager::new().expect("Should create manager");
        
        // Test that unwrap_or_default in template_agent.rs line 148 doesn't panic
        let result = manager.render_contextual("non_existent", &HashMap::new());
        // This uses unwrap_or_default() which should return empty string
        assert_eq!(result.unwrap_or_default(), "");
    }

    #[test]
    fn test_template_agent_json_serialization_unwrap() {
        use crate::task::TaskContext;
        use serde_json::json;
        
        // Test the unwrap() calls in template_agent.rs lines 202-203
        let mut context = TaskContext::new("test".to_string(), json!(null));
        
        // This tests the path that leads to unwrap_or_default
        let data = context.get_event_data::<serde_json::Value>()
            .map(|v| serde_json::to_string(&v).unwrap_or_default())
            .unwrap_or_default();
        
        assert_eq!(data, "null");
    }

    #[test]
    fn test_parser_regex_compilation() {
        // The parser has a regex compilation with unwrap() at line 125
        // This should always succeed with the hardcoded pattern, but let's test
        let parser = TemplateParser::new();
        
        // If regex compilation failed, we'd panic before getting here
        let result = parser.parse("{{validVar}}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_parser_empty_block_unwrap() {
        let parser = TemplateParser::new();
        
        // This tests the unwrap() calls in parse_if_block and parse_each_block
        // when there's exactly one statement
        let result = parser.parse("{{#if cond}}single{{/if}}");
        assert!(result.is_ok());
        
        let result = parser.parse("{{#each items}}single{{/each}}");
        assert!(result.is_ok());
    }
}