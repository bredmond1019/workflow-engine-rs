-- Event Sourcing Schema Migration
-- Creates event store tables for comprehensive event sourcing architecture

-- Event store table for storing all system events
CREATE TABLE IF NOT EXISTS event_store (
    -- Unique event identifier
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Aggregate root identifier that this event belongs to
    aggregate_id UUID NOT NULL,
    
    -- Type of aggregate (workflow, ai_interaction, service_call, etc.)
    aggregate_type VARCHAR(100) NOT NULL,
    
    -- Event type/name (workflow_started, ai_request_sent, etc.)
    event_type VARCHAR(100) NOT NULL,
    
    -- Version of the event within the aggregate (for ordering and conflict detection)
    aggregate_version BIGINT NOT NULL,
    
    -- Event payload as JSON
    event_data JSONB NOT NULL,
    
    -- Metadata about the event (correlation_id, user_id, source, etc.)
    metadata JSONB DEFAULT '{}'::jsonb,
    
    -- When the event occurred
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- When the event was recorded in the store
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Event schema version for backward compatibility
    schema_version INTEGER NOT NULL DEFAULT 1,
    
    -- Causation ID (event that caused this event)
    causation_id UUID,
    
    -- Correlation ID for tracing related events
    correlation_id UUID,
    
    -- Checksum for data integrity
    checksum VARCHAR(64),
    
    CONSTRAINT unique_aggregate_version UNIQUE (aggregate_id, aggregate_version)
);

-- Event snapshots for performance optimization
CREATE TABLE IF NOT EXISTS event_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL UNIQUE,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Event subscriptions for real-time processing
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_name VARCHAR(255) NOT NULL UNIQUE,
    event_types TEXT[] NOT NULL,
    last_processed_position BIGINT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    filter_criteria JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dead letter queue for failed event processing
CREATE TABLE IF NOT EXISTS event_dead_letter_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_event_id UUID NOT NULL,
    event_data JSONB NOT NULL,
    error_message TEXT NOT NULL,
    error_details JSONB DEFAULT '{}'::jsonb,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    status VARCHAR(50) NOT NULL DEFAULT 'failed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_retry_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,
    
    FOREIGN KEY (original_event_id) REFERENCES event_store(id)
);

-- Event projections tracking table
CREATE TABLE IF NOT EXISTS event_projections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    projection_name VARCHAR(255) NOT NULL UNIQUE,
    last_processed_event_id UUID,
    last_processed_position BIGINT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (last_processed_event_id) REFERENCES event_store(id)
);

-- Indexes for optimal query performance
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_id ON event_store(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_type ON event_store(aggregate_type);
CREATE INDEX IF NOT EXISTS idx_event_store_event_type ON event_store(event_type);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at ON event_store(occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_store_correlation_id ON event_store(correlation_id);
CREATE INDEX IF NOT EXISTS idx_event_store_causation_id ON event_store(causation_id);
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_type_id ON event_store(aggregate_type, aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_store_type_occurred ON event_store(event_type, occurred_at);

-- Composite index for efficient aggregate reconstruction
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_reconstruction 
ON event_store(aggregate_id, aggregate_version);

-- GIN index for efficient metadata queries
CREATE INDEX IF NOT EXISTS idx_event_store_metadata ON event_store USING GIN(metadata);
CREATE INDEX IF NOT EXISTS idx_event_store_event_data ON event_store USING GIN(event_data);

-- Snapshot indexes
CREATE INDEX IF NOT EXISTS idx_event_snapshots_aggregate_id ON event_snapshots(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_snapshots_type ON event_snapshots(aggregate_type);

-- Dead letter queue indexes
CREATE INDEX IF NOT EXISTS idx_dlq_status ON event_dead_letter_queue(status);
CREATE INDEX IF NOT EXISTS idx_dlq_next_retry ON event_dead_letter_queue(next_retry_at) WHERE status = 'failed';

-- Event subscriptions indexes
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON event_subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_event_types ON event_subscriptions USING GIN(event_types);

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for auto-updating timestamps
CREATE TRIGGER update_event_subscriptions_updated_at 
    BEFORE UPDATE ON event_subscriptions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_event_projections_updated_at 
    BEFORE UPDATE ON event_projections 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate event checksum
CREATE OR REPLACE FUNCTION calculate_event_checksum(event_data JSONB, metadata JSONB)
RETURNS VARCHAR(64) AS $$
BEGIN
    RETURN encode(digest(event_data::text || metadata::text, 'sha256'), 'hex');
END;
$$ language 'plpgsql';

-- Trigger to automatically calculate checksums
CREATE OR REPLACE FUNCTION set_event_checksum()
RETURNS TRIGGER AS $$
BEGIN
    NEW.checksum = calculate_event_checksum(NEW.event_data, NEW.metadata);
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_event_store_checksum 
    BEFORE INSERT ON event_store 
    FOR EACH ROW EXECUTE FUNCTION set_event_checksum();

-- Grant permissions for the event store
GRANT ALL PRIVILEGES ON event_store TO aiworkflow;
GRANT ALL PRIVILEGES ON event_snapshots TO aiworkflow;
GRANT ALL PRIVILEGES ON event_subscriptions TO aiworkflow;
GRANT ALL PRIVILEGES ON event_dead_letter_queue TO aiworkflow;
GRANT ALL PRIVILEGES ON event_projections TO aiworkflow;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO aiworkflow;