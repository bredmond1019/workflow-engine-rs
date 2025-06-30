# PRD Phase 2: Pragmatic System Integration

## Document Status

- **Created**: January 2025
- **Type**: Pragmatic Implementation Plan
- **Timeline**: 8 weeks (with parallel tracks)
- **Dependencies**: Phase 1 MVP completion

## Executive Summary

This PRD takes a pragmatic approach to integrating the AI Personal Tutor and AI Workflow System. Rather than building everything at once, we'll create useful integrations incrementally, starting with simple HTTP-based communication and evolving toward sophisticated cross-system workflows. Each milestone delivers immediate value while building toward the complete vision.

## Goals

1. **Week 1-2**: Get systems talking via simple HTTP/MCP calls
2. **Week 3-4**: Build first cross-system workflow that delivers real value
3. **Week 5-6**: Add monitoring and debugging capabilities
4. **Week 7-8**: Create reusable patterns and beta-ready features

## Integration Architecture

### Pragmatic Starting Point

```
┌──────────────────┐         ┌──────────────────┐
│   AI Tutor       │         │ Workflow System  │
│   (Port 8000)    │◀──────▶│   (Port 8081)    │
│                  │  HTTP   │                  │
└──────────────────┘  /MCP   └──────────────────┘
         ↓                            ↓
         └────────────┬───────────────┘
                      ↓
              ┌──────────────┐
              │   Registry   │
              │ (Port 8080)  │
              └──────────────┘
```

### End State (Week 8)

```
┌─────────────────────────────────────────────────┐
│              Simple API Router                   │
│        (Nginx or Caddy, not custom)             │
└─────────────────────────────────────────────────┘
                      ↓
┌──────────────────┐         ┌──────────────────┐
│   AI Tutor       │◀──MCP──▶│ Workflow System  │
│  ┌────────────┐  │         │  ┌────────────┐  │
│  │Research WF │  │         │  │Orchestrator│  │
│  └────────────┘  │         │  └────────────┘  │
└──────────────────┘         └──────────────────┘
                      ↓
           ┌─────────────────────┐
           │     Monitoring      │
           │ (Prometheus + Logs) │
           └─────────────────────┘
```

## Week 1-2: Basic Integration

### Goal: Prove Cross-System Communication Works

#### 1. Service Registration

```python
# AI Tutor startup.py
async def startup():
    # Register with the registry
    async with httpx.AsyncClient() as client:
        await client.post(
            f"{REGISTRY_URL}/registry/agents",
            json={
                "name": "ai-tutor-orchestrator",
                "endpoint": "http://ai-tutor:8000",
                "capabilities": ["research", "summarize", "knowledge_graph"]
            },
            headers={"Authorization": f"Bearer {SERVICE_TOKEN}"}
        )

    # Start heartbeat
    asyncio.create_task(heartbeat_loop())
```

```rust
// Workflow System main.rs
async fn main() {
    // Register workflow capabilities
    let registry = RegistryClient::new(&env::var("REGISTRY_URL")?);
    registry.register(AgentRegistration {
        name: "workflow-orchestrator".to_string(),
        endpoint: "http://workflow-system:8081".to_string(),
        capabilities: vec![
            "workflow_execution".to_string(),
            "notion_integration".to_string(),
            "slack_integration".to_string(),
        ],
    }).await?;
}
```

#### 2. First Cross-System Call

```rust
// Workflow System: Call AI Tutor for research
pub struct ResearchNode;

#[async_trait]
impl Node for ResearchNode {
    async fn execute(&self, context: TaskContext) -> Result<TaskContext> {
        // Discover AI Tutor
        let registry = context.get::<RegistryClient>()?;
        let tutors = registry.discover("research").await?;
        let tutor_endpoint = &tutors[0].endpoint;

        // Make MCP call via HTTP
        let client = HttpMCPClient::new(tutor_endpoint);
        let response = client.call("research_topic", json!({
            "topic": context.get_input("topic")?,
            "max_sources": 5
        })).await?;

        context.set_output("research_result", response)
    }
}
```

#### Functional Requirements

1. Both systems must register on startup
2. Systems must discover each other via registry
3. First successful cross-system call must complete
4. Error handling must provide clear diagnostics

### Week 3-4: First Value-Delivering Workflow

### Goal: Build "Research to Documentation" Workflow

#### Use Case

User provides a topic → AI Tutor researches → Workflow System creates Notion page

#### Implementation

```yaml
# Workflow Definition
name: research_to_documentation
description: Research a topic and create documentation
steps:
  - id: research
    type: cross_system
    system: ai-tutor
    operation: research_workflow
    input:
      topic: "{{ input.topic }}"
      difficulty: "{{ input.difficulty | default('intermediate') }}"
      max_sources: 10

  - id: create_notion_page
    type: node
    node: NotionClientNode
    input:
      title: "Research: {{ input.topic }}"
      content: |
        # {{ input.topic }}

        ## Summary
        {{ steps.research.output.summary }}

        ## Key Insights
        {{ steps.research.output.key_points }}

        ## Sources
        {{ steps.research.output.sources }}
      parent_id: "{{ env.NOTION_RESEARCH_FOLDER }}"
```

#### API Endpoint

```rust
// Simple trigger endpoint
#[post("/api/v1/workflows/trigger")]
async fn trigger_workflow(
    req: Json<TriggerRequest>,
    auth: JwtAuth,
) -> Result<Json<TriggerResponse>> {
    let workflow = workflows.get(&req.workflow_name)?;
    let instance_id = workflow.start(req.input.clone()).await?;

    Ok(Json(TriggerResponse {
        instance_id,
        status_url: format!("/api/v1/workflows/status/{}", instance_id),
    }))
}
```

#### Functional Requirements

5. Complete workflow must execute end-to-end
6. Status endpoint must show progress
7. Errors must be captured and reported
8. Results must be accessible via API

### Week 5-6: Monitoring and Debugging

### Goal: Know What's Happening Across Systems

#### 1. Simple Metrics Collection

```rust
// Add to both systems
use prometheus::{Encoder, TextEncoder, Counter, Histogram};

lazy_static! {
    static ref CROSS_SYSTEM_CALLS: Counter = Counter::new(
        "cross_system_calls_total", "Total cross-system calls"
    ).unwrap();

    static ref CALL_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("cross_system_call_duration", "Call duration in seconds")
    ).unwrap();
}

// Instrument calls
let timer = CALL_DURATION.start_timer();
let result = make_cross_system_call().await;
timer.observe_duration();
CROSS_SYSTEM_CALLS.inc();
```

#### 2. Correlation IDs

```python
# AI Tutor: Add correlation ID to all requests
@app.middleware("http")
async def add_correlation_id(request: Request, call_next):
    correlation_id = request.headers.get("X-Correlation-ID", str(uuid.uuid4()))
    request.state.correlation_id = correlation_id

    response = await call_next(request)
    response.headers["X-Correlation-ID"] = correlation_id
    return response

# Include in logs
logger.info(f"Processing research request", extra={
    "correlation_id": request.state.correlation_id,
    "topic": topic
})
```

#### 3. Simple Distributed Tracing

```yaml
# docker-compose addition
jaeger:
  image: jaegertracing/all-in-one:latest
  ports:
    - "16686:16686" # UI
    - "14268:14268" # Collector
```

#### Functional Requirements

9. Both systems must expose Prometheus metrics
10. All cross-system calls must include correlation IDs
11. Logs must be searchable by correlation ID
12. Basic Grafana dashboard must show system health

### Week 7-8: Patterns and Polish

### Goal: Make Integration Reusable and Beta-Ready

#### 1. Workflow Templates

```rust
// Pre-built workflow patterns
pub fn create_standard_workflows() -> HashMap<String, Workflow> {
    let mut workflows = HashMap::new();

    // Research → Documentation
    workflows.insert(
        "research_to_docs".to_string(),
        WorkflowBuilder::new()
            .add_cross_system_node("research", "ai-tutor")
            .add_node::<NotionClientNode>()
            .build()
    );

    // Research → Slack Summary
    workflows.insert(
        "research_to_slack".to_string(),
        WorkflowBuilder::new()
            .add_cross_system_node("research", "ai-tutor")
            .add_node::<SlackClientNode>()
            .build()
    );

    // Complex: Research → Analysis → Multiple Outputs
    workflows.insert(
        "research_pipeline".to_string(),
        WorkflowBuilder::new()
            .add_cross_system_node("research", "ai-tutor")
            .add_node::<AnthropicAgentNode>()  // Analysis
            .add_parallel_nodes(vec![
                TypeId::of::<NotionClientNode>(),
                TypeId::of::<SlackClientNode>(),
            ])
            .build()
    );

    workflows
}
```

#### 2. Simple API Gateway

```nginx
# nginx.conf - Pragmatic routing
upstream ai_tutor {
    server ai-tutor:8000;
}

upstream workflow_system {
    server workflow-system:8081;
}

server {
    listen 80;

    # Route by path prefix
    location /api/v1/research {
        proxy_pass http://ai_tutor;
        proxy_set_header X-Correlation-ID $request_id;
    }

    location /api/v1/workflows {
        proxy_pass http://workflow_system;
        proxy_set_header X-Correlation-ID $request_id;
    }

    # Unified endpoints
    location /api/v1/unified/research-and-document {
        # Route to workflow system which orchestrates
        proxy_pass http://workflow_system/api/v1/workflows/trigger;
        proxy_set_header X-Workflow-Name "research_to_docs";
    }
}
```

#### 3. Beta-Ready Features

```python
# Usage tracking
@app.middleware("http")
async def track_usage(request: Request, call_next):
    start_time = time.time()
    response = await call_next(request)
    duration = time.time() - start_time

    # Simple usage tracking
    await redis_client.hincrby(
        f"usage:{request.state.user_id}",
        date.today().isoformat(),
        1
    )

    return response

# Rate limiting
from slowapi import Limiter
limiter = Limiter(key_func=lambda req: req.state.user_id)

@app.post("/api/v1/research")
@limiter.limit("10/hour")
async def research_endpoint(request: Request):
    # Implementation
    pass
```

#### Functional Requirements

13. At least 3 workflow templates must be production-ready
14. API gateway must route all traffic correctly
15. Usage tracking must work for billing preparation
16. Rate limiting must prevent abuse

## Success Metrics

### Technical Metrics

1. **Integration Success Rate**: > 99% of cross-system calls succeed
2. **End-to-End Latency**: < 5s for research-to-documentation workflow
3. **System Availability**: > 99.5% uptime during beta
4. **Error Recovery**: Automatic retry with exponential backoff

### User Metrics

5. **First Workflow Success**: Users can run workflow within 10 minutes
6. **Documentation Quality**: 90% of users find docs helpful
7. **API Adoption**: 30% of beta users try the API
8. **Feature Usage**: All 3 workflow templates used by beta users

## Pragmatic Decisions

### What We're NOT Building

1. **Complex API Gateway**: Use Nginx/Caddy, not custom
2. **Perfect Monitoring**: Good enough to debug issues
3. **Full Auth0**: JWT tokens are sufficient for beta
4. **Kubernetes**: Docker Compose is fine for beta scale

### What We ARE Building

1. **Real Value**: Workflows that solve actual problems
2. **Simple Integration**: HTTP/MCP that just works
3. **Debugging Tools**: Correlation IDs and basic tracing
4. **Clear Documentation**: Examples that work out-of-box

## Beta Launch Checklist

### Week 8 Deliverables

- [ ] 3 working workflow templates
- [ ] Public API documentation
- [ ] Quick start guide with examples
- [ ] Docker Compose for easy deployment
- [ ] Basic monitoring dashboard
- [ ] Usage tracking for billing prep
- [ ] Support channel (Discord/Slack)
- [ ] Feedback collection mechanism

## Risk Management

### Risk: Integration Complexity

**Mitigation**: Start with HTTP, add complexity gradually

### Risk: Performance Issues

**Mitigation**: Set clear SLAs for beta (5s response time acceptable)

### Risk: Debugging Difficulties

**Mitigation**: Correlation IDs from day 1, add tracing incrementally

## Next Steps (Post-Phase 2)

### Phase 2.5: Production Hardening

- Add Auth0 integration
- Implement circuit breakers
- Add comprehensive monitoring
- Performance optimization

### Phase 3: Product Development

- Select 2-3 products from original list
- Build product-specific UIs
- Implement billing
- Scale to 100+ users

## Conclusion

This pragmatic Phase 2 approach delivers working integration in 8 weeks by focusing on real value over perfect architecture. We build just enough infrastructure to support useful workflows, then iterate based on actual usage. The key is maintaining momentum while building toward the larger vision.
