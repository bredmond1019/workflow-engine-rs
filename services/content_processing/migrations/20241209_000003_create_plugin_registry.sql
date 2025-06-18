-- Create plugin status enum
CREATE TYPE plugin_status AS ENUM ('active', 'inactive', 'error', 'updating');

-- Create plugin registry table
CREATE TABLE IF NOT EXISTS plugin_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    version VARCHAR(20) NOT NULL,
    description TEXT,
    author VARCHAR(100),
    repository_url TEXT,
    wasm_hash VARCHAR(64) NOT NULL,
    status plugin_status NOT NULL DEFAULT 'inactive',
    capabilities JSONB DEFAULT '[]'::jsonb,
    config_schema JSONB,
    last_health_check TIMESTAMPTZ,
    error_count INT DEFAULT 0,
    total_executions BIGINT DEFAULT 0,
    average_execution_time_ms FLOAT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_plugin_registry_name ON plugin_registry(name);
CREATE INDEX idx_plugin_registry_status ON plugin_registry(status);
CREATE INDEX idx_plugin_registry_capabilities_gin ON plugin_registry USING gin(capabilities);

-- Add trigger to update updated_at timestamp
CREATE TRIGGER set_timestamp_plugin_registry
    BEFORE UPDATE ON plugin_registry
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();

-- Create plugin execution logs table
CREATE TABLE IF NOT EXISTS plugin_execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id UUID REFERENCES plugin_registry(id) ON DELETE CASCADE,
    job_id UUID REFERENCES processing_jobs(id) ON DELETE CASCADE,
    execution_time_ms INT NOT NULL,
    memory_used_bytes BIGINT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    input_size_bytes INT,
    output_size_bytes INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for execution logs
CREATE INDEX idx_plugin_execution_logs_plugin_id ON plugin_execution_logs(plugin_id);
CREATE INDEX idx_plugin_execution_logs_job_id ON plugin_execution_logs(job_id);
CREATE INDEX idx_plugin_execution_logs_success ON plugin_execution_logs(success);
CREATE INDEX idx_plugin_execution_logs_created_at ON plugin_execution_logs(created_at);