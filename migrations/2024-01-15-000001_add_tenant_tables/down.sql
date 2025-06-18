-- Drop triggers
DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants;
DROP TRIGGER IF EXISTS update_service_permissions_updated_at ON service_permissions;

-- Drop RLS policies
DROP POLICY IF EXISTS events_tenant_isolation ON events;
DROP POLICY IF EXISTS event_store_tenant_isolation ON event_store;
DROP POLICY IF EXISTS event_snapshots_tenant_isolation ON event_snapshots;
DROP POLICY IF EXISTS event_subscriptions_tenant_isolation ON event_subscriptions;
DROP POLICY IF EXISTS event_projections_tenant_isolation ON event_projections;
DROP POLICY IF EXISTS event_dead_letter_queue_tenant_isolation ON event_dead_letter_queue;
DROP POLICY IF EXISTS agents_tenant_isolation ON agents;
DROP POLICY IF EXISTS users_tenant_isolation ON users;

-- Disable row-level security
ALTER TABLE events DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_store DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_snapshots DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_subscriptions DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_projections DISABLE ROW LEVEL SECURITY;
ALTER TABLE event_dead_letter_queue DISABLE ROW LEVEL SECURITY;
ALTER TABLE agents DISABLE ROW LEVEL SECURITY;
ALTER TABLE users DISABLE ROW LEVEL SECURITY;

-- Drop tenant_id columns
ALTER TABLE events DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE event_store DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE event_snapshots DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE event_subscriptions DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE event_projections DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE event_dead_letter_queue DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE agents DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE users DROP COLUMN IF EXISTS tenant_id;

-- Drop tables
DROP TABLE IF EXISTS service_permissions;
DROP TABLE IF EXISTS tenants;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();