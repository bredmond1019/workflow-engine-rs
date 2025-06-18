# AI Workflow System Tutorial Series

Welcome to the AI Workflow System tutorial series! This comprehensive guide will take you from beginner to expert in building production-ready AI workflow orchestration systems using Rust.

## 🎯 What You'll Learn

Through these tutorials, you'll master:

- **Core Architecture**: Understanding workflow nodes, TaskContext, and the Model Context Protocol (MCP)
- **AI Integration**: Building AI-powered workflows with OpenAI, Anthropic, and custom agents
- **External Services**: Connecting to external APIs through MCP with HelpScout, Notion, and Slack integrations
- **Event Sourcing**: Implementing event-driven architectures with state persistence and replay capabilities
- **Microservices**: Scaling with distributed services including Content Processing, Knowledge Graph, and Realtime Communication
- **Production Deployment**: Monitoring, testing, security, and scaling patterns for enterprise use

## 📚 Tutorial Overview

### Tutorial 1: Getting Started with Workflows
**For**: Developers new to the system
**Time**: 45 minutes

Build your first AI workflow from scratch using the current TaskContext API. Learn core concepts like nodes, workflow execution, and data flow through a practical customer feedback analysis example.

### Tutorial 2: Understanding Nodes and Data Flow
**For**: Developers building custom workflows
**Time**: 60 minutes

Master the Node trait and workflow patterns. Build a multi-step workflow with proper error handling, data validation, and node communication using real examples from the codebase.

### Tutorial 3: AI-Powered Automation
**For**: Developers integrating AI capabilities
**Time**: 90 minutes

Integrate AI agents, token management, and streaming responses. Build an automated email summarizer using OpenAI/Anthropic APIs with proper token budgeting and error handling.

### Tutorial 4: External Service Integration with MCP
**For**: Developers connecting external systems
**Time**: 75 minutes

Master the Model Context Protocol (MCP) framework. Build custom service connectors for HelpScout, Notion, and Slack using HTTP, WebSocket, and stdio transports with connection pooling.

### Tutorial 5: Event Sourcing and State Management
**For**: Developers building stateful systems
**Time**: 90 minutes

Implement event-driven architectures with the built-in event store. Learn state persistence, event replay, projections, and cross-service event routing patterns.

### Tutorial 6: Microservices Integration
**For**: Architects building distributed systems
**Time**: 105 minutes

Integrate the Content Processing, Knowledge Graph, and Realtime Communication services. Learn service discovery, inter-service communication, and fault tolerance patterns.

### Tutorial 7: Testing and Performance
**For**: Developers ensuring quality
**Time**: 90 minutes

Implement comprehensive testing strategies including unit tests, integration tests with external MCP servers, load testing, and chaos engineering for resilient systems.

### Tutorial 8: Production Best Practices
**For**: DevOps and operations teams
**Time**: 120 minutes

Deploy production-ready systems with monitoring, observability, security, and scaling. Learn Docker containerization, Prometheus metrics, and operational runbooks.

## 🛤️ Suggested Learning Paths

### New Developer Path
**Goal**: Build your first AI workflow and understand core concepts
1. **Tutorial 1**: Getting Started with Workflows (45 min)
2. **Tutorial 2**: Understanding Nodes and Data Flow (60 min)
3. **Tutorial 3**: AI-Powered Automation (90 min)

**Total time**: ~3.25 hours

### Integration Developer Path
**Goal**: Connect external services and build distributed workflows
1. **Tutorial 1**: Getting Started with Workflows (45 min)
2. **Tutorial 4**: External Service Integration with MCP (75 min)
3. **Tutorial 6**: Microservices Integration (105 min)
4. **Tutorial 7**: Testing and Performance (90 min)

**Total time**: ~5.25 hours

### System Architect Path
**Goal**: Design and implement production-grade distributed systems
1. **Tutorial 5**: Event Sourcing and State Management (90 min)
2. **Tutorial 6**: Microservices Integration (105 min)
3. **Tutorial 7**: Testing and Performance (90 min)
4. **Tutorial 8**: Production Best Practices (120 min)

**Total time**: ~6.75 hours

### Operations Engineer Path
**Goal**: Deploy, monitor, and maintain the system in production
1. **Tutorial 7**: Testing and Performance (90 min)
2. **Tutorial 8**: Production Best Practices (120 min)
3. **Review**: Monitoring and observability setup

**Total time**: ~3.5 hours

### Fast Track (Experienced Rust + AI Developers)
**Goal**: Quickly understand system-specific patterns
1. **Tutorial 2**: Understanding Nodes (skim - 30 min)
2. **Tutorial 4**: MCP Integration (75 min)
3. **Tutorial 5**: Event Sourcing (90 min)
4. **Tutorial 8**: Production Best Practices (120 min)

**Total time**: ~5.25 hours

## 📋 Prerequisites

### System Requirements
- **Rust**: Version 1.75 or higher (for latest async features)
- **PostgreSQL**: Version 14 or higher (for event store and repositories)
- **Docker**: Version 20.10 or higher (for services and MCP servers)
- **Python**: Version 3.10+ (for MCP server implementations)
- **Optional**: Dgraph for Knowledge Graph service
- **Memory**: Minimum 8GB RAM, 16GB recommended for full microservices
- **Disk**: 4GB free space for dependencies, builds, and databases

### Knowledge Prerequisites
- **For Beginners**: Basic programming concepts
- **For Developers**: Familiarity with async programming helpful
- **For Operations**: Container and database administration experience

### Development Environment
Before starting, ensure you have:
```bash
# Verify Rust installation
rustc --version  # Should show 1.75+
cargo --version

# Verify Docker installation
docker --version  # Should show 20.10+
docker-compose --version

# Verify PostgreSQL
psql --version  # Should show 14+

# Clone the repository
git clone [repository-url]
cd ai-system-rust

# Quick setup (from CLAUDE.md)
./scripts/quick-start.sh

# Or manual setup
cargo build
docker-compose up -d
./scripts/start_test_servers.sh
```

## 🚀 How to Use These Tutorials

### Tutorial Structure
Each tutorial follows a consistent structure:
1. **Learning Objectives**: Clear goals for what you'll achieve
2. **Conceptual Overview**: Understanding the "why" before the "how"
3. **Hands-On Practice**: Step-by-step implementation
4. **Real-World Example**: Practical application of concepts
5. **Exercises**: Challenges to reinforce learning
6. **Next Steps**: Where to go from here

### Tips for Success
- **Type the Code**: Don't copy-paste - typing helps retention
- **Experiment**: Modify examples to see what happens
- **Read Errors**: Error messages are learning opportunities
- **Use the REPL**: Test small code snippets as you learn
- **Join the Community**: Ask questions in our Discord/Slack

### Getting Help
- **Quick Reference**: See the [Quick Reference Card](./quick-reference.md)
- **API Documentation**: Full API reference at `/docs/API_REFERENCE.md`
- **Troubleshooting**: Common issues in each tutorial's troubleshooting section
- **Community Support**: Discord server at [discord.gg/ai-workflow]

## 🔗 Additional Resources

### Documentation
- [Architecture Overview](../ARCHITECTURE.md)
- [API Reference](../API_REFERENCE.md)
- [MCP Protocol Specification](../MCP_INTEGRATION.md)
- [Contributing Guide](../../CONTRIBUTING.md)

### Example Projects
- [Basic Workflow](../../examples/basic-workflow.rs) - Simple knowledge base search
- [AI Research Workflow](../../examples/ai-research-workflow.rs) - AI-powered research automation
- [HTTP MCP Client Demo](../../examples/http_mcp_client_demo.rs) - External service integration
- [Multi-Service Integration](../../examples/multi-service-integration.rs) - Microservices patterns
- [Customer Support Automation](../../examples/2_customer_support_automation.py) - End-to-end workflow
- [Token Usage Demo](../../examples/token_usage_demo.rs) - AI cost management

### External Resources
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Model Context Protocol Spec](https://github.com/anthropics/mcp)

## 🎯 Learning Outcomes

By completing this tutorial series, you will be able to:

- **Build** production-ready AI workflow systems using the current TaskContext and Node APIs
- **Integrate** AI agents with proper token management and streaming responses
- **Connect** external services through MCP with HTTP, WebSocket, and stdio transports
- **Implement** event-driven architectures with state persistence and replay capabilities
- **Deploy** microservices with Content Processing, Knowledge Graph, and Realtime Communication
- **Test** systems comprehensively with unit, integration, load, and chaos testing
- **Monitor** production systems with Prometheus metrics, correlation tracking, and observability
- **Scale** horizontally with connection pooling, circuit breakers, and fault tolerance

## 📝 Feedback and Contributions

We'd love to hear from you! If you:
- Find errors or unclear explanations
- Have suggestions for improvements
- Want to contribute examples or tutorials
- Build something cool with the system

Please open an issue or submit a PR on our GitHub repository.

---

Ready to begin? Start with [Tutorial 1: Understanding the Basics](./01-understanding-basics.md) and begin your journey into AI workflow orchestration!

Happy learning! 🚀