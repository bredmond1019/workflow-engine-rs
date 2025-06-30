# AI System Workflow Diagrams

## Customer Support Workflow

```mermaid
graph TD
    Start([Customer Ticket Input]) --> AT[Analyze Ticket]
    
    AT --> |Parallel Processing| DI[Determine Intent]
    AT --> |Parallel Processing| FS[Filter Spam]
    AT --> |Parallel Processing| VT[Validate Ticket]
    
    DI --> TR{Ticket Router}
    FS --> TR
    VT --> TR
    
    TR --> |Route Based on Analysis| GR[Generate Response]
    TR -.-> |Complex/High Priority| ESC[Escalate Ticket]
    
    GR --> SR[Send Reply]
    ESC -.-> SR
    
    SR --> CT[Close Ticket]
    CT --> End([Ticket Resolved])
    
    style AT fill:#e1f5fe
    style TR fill:#fff3e0
    style GR fill:#e8f5e9
    style SR fill:#f3e5f5
    style DI fill:#fce4ec
    style FS fill:#fce4ec
    style VT fill:#fce4ec
    style ESC fill:#ffebee
```

### Workflow Details:
- **Entry Point**: Analyze Ticket - Extracts key information and triggers parallel processing
- **Parallel Tasks**: Intent determination, spam filtering, and validation run simultaneously
- **Router Node**: Makes intelligent routing decisions based on analysis results
- **Response Path**: Generates AI-powered responses and sends them to customers
- **Escalation Path**: Complex issues can be routed for manual handling

## Knowledge Base Workflow

```mermaid
graph TD
    Start([User Query Input]) --> QR[Query Router]
    
    QR --> |Parallel Processing| VQ[Validate Query]
    QR --> |Parallel Processing| FSQ[Filter Spam Query]
    
    VQ --> SR{Search Router}
    FSQ --> SR
    
    SR --> |Parallel Searches| NS[Notion Search]
    SR --> |Parallel Searches| HS[HelpScout Search]
    SR --> |Parallel Searches| SS[Slack Search]
    
    NS --> AK[Analyze Knowledge]
    HS --> AK
    SS --> AK
    
    AK --> |Sufficient Info?| GKR[Generate Knowledge Response]
    AK -.-> |Insufficient Info| FB[Fallback Response]
    
    GKR --> SKR[Send Knowledge Reply]
    FB -.-> SKR
    
    SKR --> End([Query Resolved])
    
    style QR fill:#e1f5fe
    style SR fill:#fff3e0
    style AK fill:#e8f5e9
    style GKR fill:#f3e5f5
    style VQ fill:#fce4ec
    style FSQ fill:#fce4ec
    style NS fill:#e3f2fd
    style HS fill:#e3f2fd
    style SS fill:#e3f2fd
    style FB fill:#ffebee
```

### Workflow Details:
- **Entry Point**: Query Router - Processes user queries and extracts keywords
- **Validation**: Parallel query validation and spam filtering
- **Search Router**: Orchestrates parallel searches across multiple knowledge sources
- **Knowledge Sources**: Searches Notion, HelpScout, and Slack simultaneously
- **Analysis**: Evaluates search results for completeness and relevance
- **Response Generation**: Creates comprehensive responses with source attribution

## Node Execution Patterns

### Customer Support Workflow Nodes

| Node | Type | Purpose | Execution Time |
|------|------|---------|----------------|
| AnalyzeTicket | Entry | Initial ticket analysis | ~100ms |
| DetermineIntent | Parallel | AI intent classification | ~500ms |
| FilterSpam | Parallel | Spam detection | ~200ms |
| ValidateTicket | Parallel | Data validation | ~50ms |
| TicketRouter | Router | Decision routing | ~10ms |
| GenerateResponse | Sequential | AI response generation | ~1000ms |
| SendReply | Sequential | Response delivery | ~200ms |

### Knowledge Base Workflow Nodes

| Node | Type | Purpose | Execution Time |
|------|------|---------|----------------|
| QueryRouter | Entry | Query preparation | ~50ms |
| ValidateQuery | Parallel | Query validation | ~50ms |
| FilterSpamQuery | Parallel | Spam detection | ~100ms |
| SearchRouter | Router | Search orchestration | ~10ms |
| NotionSearch | Parallel | Notion API search | ~2000ms |
| HelpscoutSearch | Parallel | HelpScout search | ~1500ms |
| SlackSearch | Parallel | Slack search | ~3000ms |
| AnalyzeKnowledge | Sequential | Result analysis | ~500ms |
| GenerateKnowledgeResponse | Sequential | Response synthesis | ~1000ms |
| SendKnowledgeReply | Sequential | Response delivery | ~200ms |

## Key Differences

### Customer Support Workflow
- **Focus**: Ticket processing and customer service automation
- **Inputs**: Support tickets with customer messages
- **Outputs**: Generated responses and ticket status updates
- **Use Cases**: Customer inquiries, technical support, billing questions

### Knowledge Base Workflow
- **Focus**: Multi-source knowledge search and retrieval
- **Inputs**: User queries for information
- **Outputs**: Comprehensive responses from multiple sources
- **Use Cases**: Documentation search, FAQ retrieval, team knowledge mining

## Performance Characteristics

### Parallelization Benefits
- Customer Support: Reduces processing from ~3s to ~1s per ticket
- Knowledge Base: Reduces search time from ~15s to ~5s per query

### Scalability
- Both workflows handle multiple requests concurrently
- Node-level parallelization for improved throughput
- Connection pooling for external service integration