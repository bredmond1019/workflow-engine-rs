-- Development seed data for AI Workflow System
-- This file is only loaded in development environments

-- Insert test agents
INSERT INTO agents (id, name, description, capabilities, created_at, updated_at) VALUES
('00000000-0000-0000-0000-000000000001', 'Test OpenAI Agent', 'Development OpenAI agent for testing', '["completion", "chat", "embeddings"]', NOW(), NOW()),
('00000000-0000-0000-0000-000000000002', 'Test Claude Agent', 'Development Claude agent for testing', '["completion", "chat", "analysis"]', NOW(), NOW()),
('00000000-0000-0000-0000-000000000003', 'Test Local Agent', 'Development local agent for testing', '["data_processing", "file_operations"]', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert test agent configurations
INSERT INTO agent_configurations (agent_id, config_key, config_value, created_at) VALUES
('00000000-0000-0000-0000-000000000001', 'model', 'gpt-4', NOW()),
('00000000-0000-0000-0000-000000000001', 'temperature', '0.7', NOW()),
('00000000-0000-0000-0000-000000000002', 'model', 'claude-3-opus', NOW()),
('00000000-0000-0000-0000-000000000002', 'max_tokens', '4096', NOW()),
('00000000-0000-0000-0000-000000000003', 'mode', 'development', NOW())
ON CONFLICT (agent_id, config_key) DO NOTHING;

-- Insert test workflows
INSERT INTO workflows (id, name, description, definition, created_at, updated_at) VALUES
('00000000-0000-0000-0000-000000000001', 'Test Research Workflow', 'Development workflow for testing research capabilities', 
'{
  "nodes": [
    {"id": "start", "type": "trigger", "config": {"event": "research_request"}},
    {"id": "search", "type": "notion_search", "config": {"query": "${input.query}"}},
    {"id": "analyze", "type": "ai_analysis", "config": {"agent_id": "00000000-0000-0000-0000-000000000001"}},
    {"id": "complete", "type": "response", "config": {"format": "markdown"}}
  ],
  "edges": [
    {"from": "start", "to": "search"},
    {"from": "search", "to": "analyze"},
    {"from": "analyze", "to": "complete"}
  ]
}', NOW(), NOW()),
('00000000-0000-0000-0000-000000000002', 'Test Support Workflow', 'Development workflow for testing customer support', 
'{
  "nodes": [
    {"id": "start", "type": "trigger", "config": {"event": "support_ticket"}},
    {"id": "fetch", "type": "helpscout_fetch", "config": {"ticket_id": "${input.ticket_id}"}},
    {"id": "assist", "type": "ai_assistance", "config": {"agent_id": "00000000-0000-0000-0000-000000000002"}},
    {"id": "respond", "type": "helpscout_respond", "config": {}}
  ],
  "edges": [
    {"from": "start", "to": "fetch"},
    {"from": "fetch", "to": "assist"},
    {"from": "assist", "to": "respond"}
  ]
}', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert test events
INSERT INTO events (id, event_type, event_data, correlation_id, created_at) VALUES
('00000000-0000-0000-0000-000000000001', 'workflow.started', 
'{"workflow_id": "00000000-0000-0000-0000-000000000001", "user": "test@example.com"}', 
'dev-correlation-001', NOW() - INTERVAL '1 hour'),
('00000000-0000-0000-0000-000000000002', 'workflow.completed', 
'{"workflow_id": "00000000-0000-0000-0000-000000000001", "duration_ms": 3500}', 
'dev-correlation-001', NOW() - INTERVAL '59 minutes'),
('00000000-0000-0000-0000-000000000003', 'agent.invoked', 
'{"agent_id": "00000000-0000-0000-0000-000000000001", "action": "analyze_text"}', 
'dev-correlation-002', NOW() - INTERVAL '30 minutes'),
('00000000-0000-0000-0000-000000000004', 'error.occurred', 
'{"error": "Rate limit exceeded", "service": "openai", "retry_after": 60}', 
'dev-correlation-003', NOW() - INTERVAL '15 minutes')
ON CONFLICT (id) DO NOTHING;

-- Insert test sessions
INSERT INTO sessions (id, user_id, token, expires_at, created_at) VALUES
('00000000-0000-0000-0000-000000000001', 'dev-user-001', 'dev-token-001', NOW() + INTERVAL '7 days', NOW()),
('00000000-0000-0000-0000-000000000002', 'dev-user-002', 'dev-token-002', NOW() + INTERVAL '7 days', NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert test session data
INSERT INTO session_data (session_id, key, value, created_at) VALUES
('00000000-0000-0000-0000-000000000001', 'theme', 'dark', NOW()),
('00000000-0000-0000-0000-000000000001', 'last_workflow', '00000000-0000-0000-0000-000000000001', NOW()),
('00000000-0000-0000-0000-000000000002', 'preferences', '{"notifications": true, "auto_save": false}', NOW())
ON CONFLICT (session_id, key) DO NOTHING;

-- Create development indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_dev_events_correlation ON events(correlation_id);
CREATE INDEX IF NOT EXISTS idx_dev_events_type_time ON events(event_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_dev_sessions_user ON sessions(user_id);

-- Development helper views
CREATE OR REPLACE VIEW recent_events AS
SELECT 
    id,
    event_type,
    event_data,
    correlation_id,
    created_at,
    NOW() - created_at as age
FROM events
WHERE created_at > NOW() - INTERVAL '24 hours'
ORDER BY created_at DESC;

CREATE OR REPLACE VIEW workflow_metrics AS
SELECT 
    w.name as workflow_name,
    COUNT(DISTINCT e.correlation_id) as execution_count,
    AVG(CASE 
        WHEN e.event_type = 'workflow.completed' 
        THEN (e.event_data->>'duration_ms')::int 
    END) as avg_duration_ms,
    MAX(e.created_at) as last_run
FROM workflows w
LEFT JOIN events e ON e.event_data->>'workflow_id' = w.id::text
WHERE e.event_type IN ('workflow.started', 'workflow.completed')
GROUP BY w.id, w.name;

-- Grant permissions for development
GRANT ALL ON ALL TABLES IN SCHEMA public TO aiworkflow;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO aiworkflow;

-- Development notification
DO $$
BEGIN
    RAISE NOTICE 'Development seed data loaded successfully!';
    RAISE NOTICE 'Test agents: 3 created';
    RAISE NOTICE 'Test workflows: 2 created';
    RAISE NOTICE 'Test events: 4 created';
    RAISE NOTICE 'Test sessions: 2 created';
END $$;