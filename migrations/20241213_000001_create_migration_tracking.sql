-- Migration tracking system for versioned database migrations
-- This creates the infrastructure to track applied migrations

-- Create migration tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
    id SERIAL PRIMARY KEY,
    version VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checksum VARCHAR(64),
    execution_time_ms INTEGER
);

-- Create migration history table for rollback support
CREATE TABLE IF NOT EXISTS migration_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    operation VARCHAR(20) NOT NULL, -- 'apply' or 'rollback'
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    applied_by VARCHAR(255),
    execution_time_ms INTEGER,
    rollback_sql TEXT
);

-- Index for efficient migration version queries
CREATE INDEX IF NOT EXISTS idx_schema_migrations_version ON schema_migrations(version);
CREATE INDEX IF NOT EXISTS idx_migration_history_version ON migration_history(version);
CREATE INDEX IF NOT EXISTS idx_migration_history_applied_at ON migration_history(applied_at);

-- Function to calculate migration checksum
CREATE OR REPLACE FUNCTION calculate_migration_checksum(migration_content TEXT)
RETURNS VARCHAR(64) AS $$
BEGIN
    RETURN encode(digest(migration_content, 'sha256'), 'hex');
END;
$$ language 'plpgsql';

-- Function to record migration application
CREATE OR REPLACE FUNCTION record_migration(
    p_version VARCHAR(255),
    p_name VARCHAR(255),
    p_checksum VARCHAR(64),
    p_execution_time_ms INTEGER
) RETURNS VOID AS $$
BEGIN
    INSERT INTO schema_migrations (version, name, checksum, execution_time_ms)
    VALUES (p_version, p_name, p_checksum, p_execution_time_ms);
    
    INSERT INTO migration_history (version, name, operation, execution_time_ms)
    VALUES (p_version, p_name, 'apply', p_execution_time_ms);
END;
$$ language 'plpgsql';

-- Function to check if migration is already applied
CREATE OR REPLACE FUNCTION is_migration_applied(p_version VARCHAR(255))
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS(SELECT 1 FROM schema_migrations WHERE version = p_version);
END;
$$ language 'plpgsql';

-- Grant permissions
GRANT ALL PRIVILEGES ON schema_migrations TO aiworkflow;
GRANT ALL PRIVILEGES ON migration_history TO aiworkflow;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO aiworkflow;