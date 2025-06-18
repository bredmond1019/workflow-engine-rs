-- Create agents table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    endpoint VARCHAR(500) NOT NULL,
    capabilities TEXT[] NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for capabilities (GIN) and status columns
CREATE INDEX IF NOT EXISTS idx_agents_capabilities ON agents USING GIN(capabilities);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_last_seen ON agents(last_seen);
CREATE INDEX IF NOT EXISTS idx_agents_name ON agents(name);