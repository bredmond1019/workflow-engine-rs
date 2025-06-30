# AI Integration - Bringing Intelligence to Your Workflows

Welcome to AI Integration examples! This section shows you how to integrate powerful AI models from OpenAI and Anthropic into your workflows, creating intelligent, automated systems.

## 🎯 Learning Objectives

By completing these examples, you will:
- Integrate OpenAI and Anthropic AI models into workflows
- Understand prompt engineering best practices
- Learn token management and cost optimization
- Build multi-model workflows for different use cases
- Handle AI API errors and edge cases

## 📚 Examples in This Section

### 1. openai-agent
**File**: `openai-agent.rs`
**Concepts**: OpenAI API integration, GPT models, prompt engineering
**Time**: 20 minutes

Learn to integrate OpenAI's GPT models:
- Configure OpenAI agent nodes
- Write effective prompts
- Handle API responses and errors
- Optimize for different GPT models

```bash
cargo run --bin openai-agent
```

### 2. anthropic-agent
**File**: `anthropic-agent.rs`
**Concepts**: Anthropic Claude integration, model comparison, async processing
**Time**: 20 minutes

Explore Anthropic's Claude models:
- Set up Claude agent nodes
- Compare different Claude model capabilities
- Handle long-form content processing
- Manage async AI operations

```bash
cargo run --bin anthropic-agent
```

### 3. multi-model
**File**: `multi-model.rs`
**Concepts**: Model comparison, fallback strategies, parallel processing
**Time**: 30 minutes

Use multiple AI models together:
- Compare responses from different models
- Implement model fallback strategies
- Process queries in parallel
- Choose the best model for each task

```bash
cargo run --bin multi-model
```

### 4. prompt-engineering
**File**: `prompt-engineering.rs`
**Concepts**: Advanced prompting, few-shot learning, prompt templates
**Time**: 25 minutes

Master prompt engineering techniques:
- Design effective system prompts
- Use few-shot examples
- Create dynamic prompt templates
- Optimize prompts for different models

```bash
cargo run --bin prompt-engineering
```

### 5. token-management
**File**: `token-management.rs`
**Concepts**: Cost optimization, token counting, batch processing
**Time**: 25 minutes

Optimize AI usage costs:
- Track and manage token usage
- Implement cost-aware processing
- Batch requests efficiently
- Monitor spending and limits

```bash
cargo run --bin token-management
```

## 🛠 Setup

### 1. API Keys
Set up your AI service API keys:

```bash
# OpenAI API Key
export OPENAI_API_KEY="sk-your-openai-api-key"

# Anthropic API Key  
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### 2. Dependencies
Navigate to this directory and install dependencies:

```bash
cd examples/03-ai-integration
cargo build
```

### 3. Run Examples
Execute examples individually:

```bash
cargo run --bin openai-agent
cargo run --bin anthropic-agent
cargo run --bin multi-model
cargo run --bin prompt-engineering
cargo run --bin token-management
```

## 📖 Key Concepts

### AI Agent Nodes
Specialized nodes that integrate with AI services:

```rust
use workflow_engine_nodes::ai_agents::{OpenAIAgentNode, AnthropicAgentNode};

// Create OpenAI agent
let openai_agent = OpenAIAgentNode::new(openai_config)?;

// Create Anthropic agent  
let anthropic_agent = AnthropicAgentNode::new(anthropic_config);
```

### Agent Configuration
Configure AI agents for your specific use case:

```rust
use workflow_engine_core::nodes::agent::{AgentConfig, ModelProvider};

let config = AgentConfig {
    system_prompt: "You are a helpful assistant specialized in...".to_string(),
    model_provider: ModelProvider::OpenAI,
    model_name: "gpt-4".to_string(),
    mcp_server_uri: None,
};
```

### Prompt Engineering Patterns
Effective prompting techniques:

1. **Clear Instructions**: Be specific about what you want
2. **Context Setting**: Provide relevant background information
3. **Output Format**: Specify the desired response format
4. **Examples**: Use few-shot examples for complex tasks
5. **Chain of Thought**: Break down complex reasoning

### Error Handling
Handle AI service errors gracefully:

```rust
match ai_agent.process(context) {
    Ok(result) => {
        // Process successful response
    }
    Err(WorkflowError::ApiError { message }) => {
        // Handle API errors (rate limits, authentication, etc.)
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## 🎓 What You'll Learn

### After openai-agent:
- OpenAI API integration basics
- GPT model capabilities and limitations
- Basic prompt engineering
- Error handling for AI services

### After anthropic-agent:
- Anthropic Claude integration
- Model comparison and selection
- Async AI processing patterns
- Long-form content handling

### After multi-model:
- Using multiple AI providers
- Model fallback strategies
- Parallel AI processing
- Response comparison and selection

### After prompt-engineering:
- Advanced prompting techniques
- Dynamic prompt generation
- Few-shot learning patterns
- Prompt optimization methods

### After token-management:
- Cost-aware AI usage
- Token counting and monitoring
- Batch processing optimization
- Budget management strategies

## 🔧 Model Capabilities

### OpenAI Models
- **GPT-4**: Most capable, best for complex reasoning
- **GPT-3.5-turbo**: Fast and cost-effective for simpler tasks
- **GPT-4-turbo**: Balanced performance and cost

### Anthropic Models
- **Claude-3-opus**: Most capable, excellent for complex tasks
- **Claude-3-sonnet**: Balanced performance and speed
- **Claude-3-haiku**: Fast and cost-effective

### Use Case Recommendations
- **Complex Analysis**: GPT-4 or Claude-3-opus
- **Content Generation**: GPT-4-turbo or Claude-3-sonnet
- **Simple Classification**: GPT-3.5-turbo or Claude-3-haiku
- **Code Generation**: GPT-4 (specialized for coding)
- **Long Documents**: Claude models (larger context windows)

## 💡 Best Practices

### Prompt Design
1. **Be Specific**: Clear, detailed instructions work better
2. **Provide Context**: Include relevant background information
3. **Use Examples**: Few-shot examples improve performance
4. **Specify Format**: Define the expected output structure
5. **Test Iteratively**: Refine prompts based on results

### Error Handling
1. **Retry Logic**: Implement exponential backoff for rate limits
2. **Fallback Models**: Use cheaper models as fallbacks
3. **Graceful Degradation**: Handle API failures gracefully
4. **Monitor Usage**: Track costs and token consumption

### Performance Optimization
1. **Batch Processing**: Group similar requests
2. **Caching**: Cache responses for repeated queries
3. **Model Selection**: Choose the right model for each task
4. **Parallel Processing**: Use multiple models concurrently

## 🧪 Testing

### Unit Tests
Test AI integrations with mock responses:

```bash
# Test individual agents
cargo test openai_agent_tests
cargo test anthropic_agent_tests

# Test all AI integration examples
cargo test --package ai-integration
```

### Integration Tests
Test with real AI services (requires API keys):

```bash
# Run integration tests (requires API keys)
cargo test -- --ignored

# Test specific AI provider
cargo test openai_integration -- --ignored
cargo test anthropic_integration -- --ignored
```

## 🐛 Troubleshooting

### Common Issues

1. **API Key Errors**
   - Verify environment variables are set
   - Check API key validity and permissions
   - Ensure sufficient credits/quota

2. **Rate Limiting**
   - Implement exponential backoff
   - Use model-specific rate limits
   - Consider upgrading API tier

3. **Token Limits**
   - Monitor input/output token counts
   - Split large inputs into chunks
   - Use models with larger context windows

4. **Model Availability**
   - Check model names and versions
   - Verify access permissions
   - Have fallback models ready

### Getting Help

1. **Check API Documentation**: OpenAI and Anthropic docs
2. **Monitor Usage**: Use provider dashboards
3. **Review Logs**: Enable debug logging with `RUST_LOG=debug`
4. **Test Incrementally**: Start with simple prompts

## 🔗 Additional Resources

- **[OpenAI API Documentation](https://platform.openai.com/docs)**
- **[Anthropic API Documentation](https://docs.anthropic.com/)**
- **[Prompt Engineering Guide](https://www.promptingguide.ai/)**
- **[AI Model Comparison](https://artificialanalysis.ai/)**

## 📊 Cost Management

### Token Estimation
Approximate token counts for planning:
- **English Text**: ~4 characters per token
- **Code**: ~2-3 characters per token
- **JSON/Structured**: ~5-6 characters per token

### Cost Optimization Tips
1. **Use Cheaper Models**: Start with less expensive options
2. **Optimize Prompts**: Shorter prompts = lower costs
3. **Cache Results**: Avoid repeated API calls
4. **Batch Processing**: Reduce per-request overhead
5. **Monitor Usage**: Set up billing alerts

---

**Ready to integrate AI?** Start with the [OpenAI agent example](openai-agent.rs) and explore the power of AI-driven workflows!