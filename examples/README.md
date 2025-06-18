# AI Workflow System - Example Integrations

This directory contains example integrations demonstrating various use cases for the AI Workflow System.

## Examples Overview

### 1. Blog Content Pipeline (`1_blog_content_pipeline.py`)

A complete content creation workflow that demonstrates:
- Topic research using AI
- Blog post outline generation
- Full content creation
- Publishing to Notion
- Template usage

**Use Case**: Content marketing teams, technical writers, documentation automation

### 2. Customer Support Automation (`2_customer_support_automation.py`)

An automated customer support system that shows:
- Support ticket processing
- Intelligent categorization and routing
- Automated response generation
- Spam detection
- Escalation handling
- Real-time updates via WebSocket

**Use Case**: Customer support teams, help desk automation, ticket triage

### 3. Knowledge Base Search (`3_knowledge_base_search.py`)

A multi-source knowledge search system featuring:
- Cross-platform search (Notion, Slack, HelpScout)
- Result aggregation and ranking
- Unified response generation
- Search caching for performance
- Advanced filtering options

**Use Case**: Internal knowledge management, documentation search, support agent assistance

## Prerequisites

1. **AI Workflow System Running**
   ```bash
   cd ..
   cargo run
   ```

2. **Authentication Token**
   ```bash
   export AUTH_TOKEN="your-jwt-token"
   ```

3. **Python Dependencies**
   ```bash
   pip install requests websockets asyncio
   ```

4. **Optional: MCP Servers**
   - Notion MCP Server (for blog publishing)
   - Slack MCP Server (for knowledge search)
   - HelpScout MCP Server (for support tickets)

## Running the Examples

### Basic Usage

Run any example directly:

```bash
python 1_blog_content_pipeline.py
```

### With Environment Variables

Configure the API endpoint and token:

```bash
export API_BASE_URL="http://localhost:8080/api/v1"
export AUTH_TOKEN="your-jwt-token"
python 2_customer_support_automation.py
```

### Using Docker

If running the system in Docker:

```bash
export API_BASE_URL="http://localhost:8080/api/v1"
export WS_URL="ws://localhost:8080/ws"
python 3_knowledge_base_search.py
```

## Example Output

### Blog Content Pipeline
```
=== Blog Content Pipeline Example ===

Step 1: Researching topic...
Triggered workflow: 550e8400-e29b-41d4-a716-446655440000
Status: running - Progress: 33%
Status: running - Progress: 66%
Status: completed - Progress: 100%
Research completed. Summary: AI is revolutionizing software development...

Step 2: Generating blog content...
Blog post generated. Title: The AI Revolution in Software Development

Step 3: Saving to Notion...
Successfully published to Notion!
Page URL: https://notion.so/ai-revolution-software-dev

=== Pipeline Complete ===
```

### Customer Support Automation
```
=== Customer Support Automation Example ===

Processing ticket: TICKET-001 - Cannot login to my account
Workflow triggered: 650e8400-e29b-41d4-a716-446655440001

Ticket TICKET-001 processed:
  Category: technical_support
  Intent: login_issue
  Spam Score: 0.02
  Escalated: False
  Response: I understand you're having trouble logging in...

=== Customer Support Automation Report ===
Total tickets: 4
Successfully processed: 4
Category Distribution:
  technical_support: 1 (25.0%)
  feature_request: 1 (25.0%)
  billing: 1 (25.0%)
  spam: 1 (25.0%)
```

### Knowledge Base Search
```
=== Basic Knowledge Base Search Demo ===

🔍 Searching for: 'How to set up authentication in our API?'
Searching in: notion, slack, helpscout

📊 Search Results (8 total)
Sources searched: notion, slack
Search time: 2.45 seconds

1. API Authentication Guide
   Source: notion | Relevance: 0.95
   This comprehensive guide covers JWT authentication setup...

2. Auth Implementation Discussion
   Source: slack | Relevance: 0.87
   Thread from #engineering about best practices for API auth...

🤖 Generating unified response...
To set up authentication in our API, follow these steps...
```

## Customization

### Modifying Workflows

Edit the workflow names and inputs to match your setup:

```python
# Change workflow name
response = requests.post(
    f"{API_BASE_URL}/workflows/trigger",
    json={
        "workflow_name": "your_custom_workflow",  # Your workflow
        "inputs": {
            "custom_field": "value"
        }
    }
)
```

### Adding New Sources

Register new MCP servers and add them to searches:

```python
# Register new source
kb = KnowledgeBaseSearch()
results = kb.search(
    query="your query",
    sources=["notion", "slack", "your_new_source"]
)
```

### Custom Templates

Use your own workflow templates:

```python
# Trigger custom template
response = requests.post(
    f"{API_BASE_URL}/templates/trigger",
    json={
        "template_id": "your_template_id",
        "inputs": {...}
    }
)
```

## Troubleshooting

### Connection Errors
- Verify the AI Workflow System is running
- Check API_BASE_URL is correct
- Ensure authentication token is valid

### Missing Sources
- Verify MCP servers are running
- Check agent registration status
- Review available sources in the system

### Timeout Issues
- Increase timeout values in the examples
- Check system logs for errors
- Verify external services are responsive

## Next Steps

1. **Modify Examples**: Adapt these examples for your use cases
2. **Create New Workflows**: Build custom workflows for your needs
3. **Add Integrations**: Connect your own services via MCP
4. **Production Setup**: See deployment guide for production use

## Support

For questions or issues:
- Check the [API Documentation](../docs/API_REFERENCE.md)
- Review the [Getting Started Guide](../docs/GETTING_STARTED.md)
- Submit issues on GitHub