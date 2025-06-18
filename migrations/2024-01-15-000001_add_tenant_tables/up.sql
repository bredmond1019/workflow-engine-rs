-- Create tenants table for multi-tenancy support
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    database_schema VARCHAR(255) NOT NULL,
    isolation_mode VARCHAR(50) NOT NULL CHECK (isolation_mode IN ('schema', 'row_level', 'hybrid')),
    settings JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on tenant name for lookups
CREATE INDEX idx_tenants_name ON tenants(name);
CREATE INDEX idx_tenants_is_active ON tenants(is_active);

-- Create service permissions table
CREATE TABLE IF NOT EXISTS service_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    database_url VARCHAR(500) NOT NULL,
    allowed_operations TEXT[] NOT NULL DEFAULT '{}',
    resource_limits JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(service_name, tenant_id)
);

-- Create indexes for service permissions
CREATE INDEX idx_service_permissions_service_name ON service_permissions(service_name);
CREATE INDEX idx_service_permissions_tenant_id ON service_permissions(tenant_id);

-- Add tenant_id column to existing tables for row-level security
ALTER TABLE events ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE event_store ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE event_snapshots ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE event_subscriptions ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE event_projections ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE event_dead_letter_queue ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE agents ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE users ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- Create indexes on tenant_id for all tables
CREATE INDEX IF NOT EXISTS idx_events_tenant_id ON events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_event_store_tenant_id ON event_store(tenant_id);
CREATE INDEX IF NOT EXISTS idx_event_snapshots_tenant_id ON event_snapshots(tenant_id);
CREATE INDEX IF NOT EXISTS idx_event_subscriptions_tenant_id ON event_subscriptions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_event_projections_tenant_id ON event_projections(tenant_id);
CREATE INDEX IF NOT EXISTS idx_event_dead_letter_queue_tenant_id ON event_dead_letter_queue(tenant_id);
CREATE INDEX IF NOT EXISTS idx_agents_tenant_id ON agents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);

-- Enable row-level security on all tables
ALTER TABLE events ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_store ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_snapshots ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_projections ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_dead_letter_queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for tenant isolation
-- These policies check the current_setting for tenant_id
CREATE POLICY events_tenant_isolation ON events
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY event_store_tenant_isolation ON event_store
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY event_snapshots_tenant_isolation ON event_snapshots
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY event_subscriptions_tenant_isolation ON event_subscriptions
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY event_projections_tenant_isolation ON event_projections
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY event_dead_letter_queue_tenant_isolation ON event_dead_letter_queue
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY agents_tenant_isolation ON agents
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY users_tenant_isolation ON users
    FOR ALL
    USING (tenant_id IS NULL OR tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_service_permissions_updated_at BEFORE UPDATE ON service_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();