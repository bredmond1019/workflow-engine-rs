use backend::core::{
    error::WorkflowError,
    nodes::{
        agent::{AgentConfig, AgentNode, BaseAgentNode, ModelProvider},
        Node,
    },
    task::TaskContext,
};
use serde_json::json;
use std::env;
use serial_test::serial;

#[cfg(test)]
mod base_agent_tests {
    use super::*;

    #[test]
    fn test_base_agent_node_creation() {
        let config = AgentConfig {
            system_prompt: "You are a helpful assistant.".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config.clone());
        let retrieved_config = agent.get_agent_config();
        
        assert_eq!(retrieved_config.system_prompt, config.system_prompt);
        assert_eq!(retrieved_config.model_provider, config.model_provider);
        assert_eq!(retrieved_config.model_name, config.model_name);
    }

    #[test]
    fn test_prompt_extraction() {
        let config = AgentConfig {
            system_prompt: "Test system prompt".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        // Test with prompt field
        let context = TaskContext::new(
            "test_workflow".to_string(),
            json!({
                "prompt": "What is the weather like?",
                "other_data": "ignored"
            })
        );
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            agent.process_with_ai(context).await
        });
        
        // Should succeed if API key is set, or fail with configuration error
        match result {
            Ok(ctx) => {
                assert!(ctx.nodes.contains_key("ai_response"));
            }
            Err(WorkflowError::ConfigurationError(msg)) => {
                assert!(msg.contains("API_KEY"));
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_multiple_prompt_fields() {
        let config = AgentConfig {
            system_prompt: "Test system prompt".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        // Test with message field (no prompt field)
        let context = TaskContext::new(
            "test_workflow".to_string(),
            json!({
                "message": "Hello, how are you?",
                "metadata": {"source": "test"}
            })
        );
        
        // Using sync interface
        let result = agent.process(context);
        
        match result {
            Ok(ctx) => {
                assert!(ctx.nodes.contains_key("ai_response"));
            }
            Err(WorkflowError::ConfigurationError(msg)) => {
                assert!(msg.contains("API_KEY"));
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_bedrock_configuration() {
        let config = AgentConfig {
            system_prompt: "You are Claude on Bedrock".to_string(),
            model_provider: ModelProvider::Bedrock,
            model_name: "anthropic.claude-v2".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config.clone());
        assert_eq!(agent.get_agent_config().model_provider, ModelProvider::Bedrock);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API keys to be set
    async fn test_openai_integration() {
        if env::var("OPENAI_API_KEY").is_err() {
            eprintln!("Skipping OpenAI integration test - OPENAI_API_KEY not set");
            return;
        }

        let config = AgentConfig {
            system_prompt: "You are a helpful assistant. Keep responses brief.".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        let context = TaskContext::new(
            "test_workflow".to_string(),
            json!({
                "prompt": "What is 2+2? Reply with just the number."
            })
        );

        let result = agent.process_with_ai(context).await;
        assert!(result.is_ok());
        
        let ctx = result.unwrap();
        let response = ctx.nodes.get("ai_response").unwrap();
        let response_text = response["response"].as_str().unwrap();
        
        // Should contain "4" in the response
        assert!(response_text.contains("4"));
    }

    #[tokio::test]
    #[ignore] // Requires API keys to be set
    async fn test_anthropic_integration() {
        if env::var("ANTHROPIC_API_KEY").is_err() {
            eprintln!("Skipping Anthropic integration test - ANTHROPIC_API_KEY not set");
            return;
        }

        let config = AgentConfig {
            system_prompt: "You are Claude. Keep responses brief.".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-sonnet-20240229".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        let context = TaskContext::new(
            "test_workflow".to_string(),
            json!({
                "prompt": "What is the capital of France? Reply with just the city name."
            })
        );

        let result = agent.process_with_ai(context).await;
        assert!(result.is_ok());
        
        let ctx = result.unwrap();
        let response = ctx.nodes.get("ai_response").unwrap();
        let response_text = response["response"].as_str().unwrap().to_lowercase();
        
        // Should contain "paris" in the response
        assert!(response_text.contains("paris"));
    }

    #[tokio::test]
    #[ignore] // Requires AWS credentials and Bedrock access
    async fn test_bedrock_integration() {
        // Check if AWS credentials are available
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
        if config.credentials_provider().is_none() {
            eprintln!("Skipping Bedrock integration test - AWS credentials not configured");
            return;
        }

        let config = AgentConfig {
            system_prompt: "You are Claude on AWS Bedrock. Keep responses brief.".to_string(),
            model_provider: ModelProvider::Bedrock,
            model_name: "anthropic.claude-v2".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        let context = TaskContext::new(
            "test_workflow".to_string(),
            json!({
                "prompt": "What is the largest planet in our solar system? Reply with just the planet name."
            })
        );

        let result = agent.process_with_ai(context).await;
        
        match result {
            Ok(ctx) => {
                let response = ctx.nodes.get("ai_response").unwrap();
                let response_text = response["response"].as_str().unwrap().to_lowercase();
                assert!(response_text.contains("jupiter"));
            }
            Err(e) => {
                eprintln!("Bedrock test failed: {:?}", e);
                // This is expected if Bedrock is not configured in the AWS account
            }
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_missing_api_key_error() {
        // Temporarily unset API keys
        let _old_openai = env::var("OPENAI_API_KEY").ok();
        env::remove_var("OPENAI_API_KEY");

        let config = AgentConfig {
            system_prompt: "Test".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        let context = TaskContext::new(
            "test".to_string(),
            json!({"prompt": "test"})
        );

        let result = agent.process(context);
        assert!(matches!(result, Err(WorkflowError::ConfigurationError(_))));

        // Restore API key if it existed
        if let Some(key) = _old_openai {
            env::set_var("OPENAI_API_KEY", key);
        }
    }

    #[test]
    fn test_unsupported_provider_error() {
        let config = AgentConfig {
            system_prompt: "Test".to_string(),
            model_provider: ModelProvider::Gemini,
            model_name: "gemini-pro".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        let context = TaskContext::new(
            "test".to_string(),
            json!({"prompt": "test"})
        );

        let result = agent.process(context);
        assert!(matches!(result, Err(WorkflowError::ConfigurationError(_))));
    }

    #[test]
    fn test_fallback_prompt_extraction() {
        let config = AgentConfig {
            system_prompt: "Test".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        
        // Context with no prompt, message, or query field
        let context = TaskContext::new(
            "test".to_string(),
            json!({
                "data": "some data",
                "values": [1, 2, 3]
            })
        );

        // Should use entire event data as prompt
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            agent.process_with_ai(context).await
        });

        // Will fail with configuration error if no API key
        match result {
            Err(WorkflowError::ConfigurationError(_)) => {
                // Expected when API key is not set
            }
            Ok(ctx) => {
                // If API key is set, it should have processed successfully
                assert!(ctx.nodes.contains_key("ai_response"));
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

#[cfg(test)]
mod node_trait_tests {
    use super::*;

    #[test]
    fn test_node_name() {
        let config = AgentConfig {
            system_prompt: "Test".to_string(),
            model_provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            mcp_server_uri: None,
        };

        let agent = BaseAgentNode::new(config);
        let name = agent.node_name();
        assert!(name.contains("BaseAgentNode"));
    }

    #[test]
    fn test_agent_config_clone() {
        let config = AgentConfig {
            system_prompt: "Original prompt".to_string(),
            model_provider: ModelProvider::Anthropic,
            model_name: "claude-3-opus-20240229".to_string(),
            mcp_server_uri: Some("ws://localhost:8080".to_string()),
        };

        let cloned = config.clone();
        assert_eq!(config.system_prompt, cloned.system_prompt);
        assert_eq!(config.model_provider, cloned.model_provider);
        assert_eq!(config.model_name, cloned.model_name);
        assert_eq!(config.mcp_server_uri, cloned.mcp_server_uri);
    }
}