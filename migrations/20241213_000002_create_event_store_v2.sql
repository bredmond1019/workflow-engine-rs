-- Enhanced Event Sourcing Schema Migration v2
-- Creates optimized event store tables with partitioning and performance enhancements

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create event store partitioned table for high-volume scenarios
CREATE TABLE IF NOT EXISTS event_store_partitioned (
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
    
    -- When the event occurred (business time)
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- When the event was recorded in the store (system time)
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Event schema version for backward compatibility
    schema_version INTEGER NOT NULL DEFAULT 1,
    
    -- Causation ID (event that caused this event)
    causation_id UUID,
    
    -- Correlation ID for tracing related events
    correlation_id UUID,
    
    -- Checksum for data integrity
    checksum VARCHAR(64),
    
    -- Partition key for time-based partitioning
    partition_date DATE GENERATED ALWAYS AS (DATE(occurred_at)) STORED,
    
    CONSTRAINT unique_aggregate_version_partitioned UNIQUE (aggregate_id, aggregate_version)
) PARTITION BY RANGE (occurred_at);

-- Create initial partitions for current and next months
CREATE TABLE event_store_current PARTITION OF event_store_partitioned
    FOR VALUES FROM (DATE_TRUNC('month', NOW())) TO (DATE_TRUNC('month', NOW() + INTERVAL '1 month'));

CREATE TABLE event_store_next PARTITION OF event_store_partitioned
    FOR VALUES FROM (DATE_TRUNC('month', NOW() + INTERVAL '1 month')) TO (DATE_TRUNC('month', NOW() + INTERVAL '2 months'));

-- Create default partition for overflow
CREATE TABLE event_store_default PARTITION OF event_store_partitioned DEFAULT;

-- Enhanced event snapshots with compression
CREATE TABLE IF NOT EXISTS event_snapshots_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL UNIQUE,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    compression_type VARCHAR(20) DEFAULT 'none',
    original_size INTEGER,
    compressed_size INTEGER
);

-- Event replay positions for streaming
CREATE TABLE IF NOT EXISTS event_replay_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consumer_name VARCHAR(255) NOT NULL UNIQUE,
    last_processed_event_id UUID,
    last_processed_position BIGINT NOT NULL DEFAULT 0,
    last_processed_at TIMESTAMPTZ,
    consumer_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Event saga management for distributed transactions
CREATE TABLE IF NOT EXISTS event_sagas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_type VARCHAR(100) NOT NULL,
    saga_data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'started',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    timeout_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    correlation_id UUID,
    compensating_actions JSONB DEFAULT '[]'::jsonb
);

-- Performance-optimized indexes for partitioned table
CREATE INDEX IF NOT EXISTS idx_event_store_part_aggregate_id ON event_store_partitioned(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_store_part_aggregate_type ON event_store_partitioned(aggregate_type);
CREATE INDEX IF NOT EXISTS idx_event_store_part_event_type ON event_store_partitioned(event_type);
CREATE INDEX IF NOT EXISTS idx_event_store_part_occurred_at ON event_store_partitioned(occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_store_part_correlation_id ON event_store_partitioned(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_event_store_part_causation_id ON event_store_partitioned(causation_id) WHERE causation_id IS NOT NULL;

-- Composite indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_event_store_part_aggregate_reconstruction 
ON event_store_partitioned(aggregate_id, aggregate_version);

CREATE INDEX IF NOT EXISTS idx_event_store_part_type_time 
ON event_store_partitioned(event_type, occurred_at);

CREATE INDEX IF NOT EXISTS idx_event_store_part_correlation_time 
ON event_store_partitioned(correlation_id, occurred_at) WHERE correlation_id IS NOT NULL;

-- GIN indexes for efficient metadata and payload queries
CREATE INDEX IF NOT EXISTS idx_event_store_part_metadata 
ON event_store_partitioned USING GIN(metadata);

CREATE INDEX IF NOT EXISTS idx_event_store_part_event_data 
ON event_store_partitioned USING GIN(event_data);

-- BRIN index for time-based queries (efficient for large datasets)
CREATE INDEX IF NOT EXISTS idx_event_store_part_occurred_brin 
ON event_store_partitioned USING BRIN(occurred_at);

-- Indexes for supporting tables
CREATE INDEX IF NOT EXISTS idx_event_snapshots_v2_aggregate_id ON event_snapshots_v2(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_snapshots_v2_type ON event_snapshots_v2(aggregate_type);
CREATE INDEX IF NOT EXISTS idx_event_snapshots_v2_created_at ON event_snapshots_v2(created_at);

CREATE INDEX IF NOT EXISTS idx_replay_positions_consumer ON event_replay_positions(consumer_name);
CREATE INDEX IF NOT EXISTS idx_replay_positions_type ON event_replay_positions(consumer_type);
CREATE INDEX IF NOT EXISTS idx_replay_positions_status ON event_replay_positions(status);

CREATE INDEX IF NOT EXISTS idx_sagas_type ON event_sagas(saga_type);
CREATE INDEX IF NOT EXISTS idx_sagas_status ON event_sagas(status);
CREATE INDEX IF NOT EXISTS idx_sagas_correlation ON event_sagas(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_sagas_timeout ON event_sagas(timeout_at) WHERE timeout_at IS NOT NULL;

-- Function for automatic partition creation
CREATE OR REPLACE FUNCTION create_monthly_partition(target_date DATE)
RETURNS VOID AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    start_date := DATE_TRUNC('month', target_date)::DATE;
    end_date := (DATE_TRUNC('month', target_date) + INTERVAL '1 month')::DATE;
    partition_name := 'event_store_' || TO_CHAR(target_date, 'YYYY_MM');
    
    -- Check if partition already exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_class WHERE relname = partition_name
    ) THEN
        EXECUTE format('CREATE TABLE %I PARTITION OF event_store_partitioned
                       FOR VALUES FROM (%L) TO (%L)',
                       partition_name, start_date, end_date);
        
        RAISE NOTICE 'Created partition % for date range % to %', 
                     partition_name, start_date, end_date;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to automatically create next month's partition
CREATE OR REPLACE FUNCTION ensure_future_partitions()
RETURNS VOID AS $$
BEGIN
    -- Create partition for next month if it doesn't exist
    PERFORM create_monthly_partition(NOW() + INTERVAL '1 month');
    -- Create partition for month after next
    PERFORM create_monthly_partition(NOW() + INTERVAL '2 months');
END;
$$ LANGUAGE plpgsql;

-- Function for archiving old partitions
CREATE OR REPLACE FUNCTION archive_old_partitions(months_to_keep INTEGER DEFAULT 12)
RETURNS VOID AS $$
DECLARE
    cutoff_date DATE;
    partition_record RECORD;
    archive_table_name TEXT;
BEGIN
    cutoff_date := (DATE_TRUNC('month', NOW()) - INTERVAL '1 month' * months_to_keep)::DATE;
    
    FOR partition_record IN
        SELECT schemaname, tablename 
        FROM pg_tables 
        WHERE tablename LIKE 'event_store_[0-9][0-9][0-9][0-9]_[0-9][0-9]'
        AND schemaname = 'public'
    LOOP
        -- Extract date from partition name
        DECLARE
            partition_date DATE;
        BEGIN
            partition_date := TO_DATE(
                SUBSTRING(partition_record.tablename FROM 'event_store_([0-9]{4}_[0-9]{2})'),
                'YYYY_MM'
            );
            
            IF partition_date < cutoff_date THEN
                archive_table_name := partition_record.tablename || '_archived';
                
                -- Move to archive schema or rename
                EXECUTE format('ALTER TABLE %I RENAME TO %I',
                              partition_record.tablename, archive_table_name);
                
                RAISE NOTICE 'Archived partition % to %', 
                           partition_record.tablename, archive_table_name;
            END IF;
        EXCEPTION
            WHEN OTHERS THEN
                RAISE WARNING 'Failed to archive partition %: %', 
                            partition_record.tablename, SQLERRM;
        END;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Enhanced trigger for automatic checksum calculation
CREATE OR REPLACE FUNCTION set_event_checksum_v2()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate checksum of event data and metadata combined
    NEW.checksum = encode(
        digest(
            COALESCE(NEW.event_data::text, '') || 
            COALESCE(NEW.metadata::text, '') ||
            COALESCE(NEW.aggregate_id::text, '') ||
            COALESCE(NEW.event_type, ''),
            'sha256'
        ), 
        'hex'
    );
    
    -- Ensure recorded_at is set
    IF NEW.recorded_at IS NULL THEN
        NEW.recorded_at = NOW();
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for checksum calculation
CREATE TRIGGER set_event_store_checksum_v2
    BEFORE INSERT ON event_store_partitioned 
    FOR EACH ROW EXECUTE FUNCTION set_event_checksum_v2();

-- Trigger for updating timestamps on replay positions
CREATE TRIGGER update_replay_positions_updated_at 
    BEFORE UPDATE ON event_replay_positions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Grant permissions for all new tables
GRANT ALL PRIVILEGES ON event_store_partitioned TO aiworkflow;
GRANT ALL PRIVILEGES ON event_store_current TO aiworkflow;
GRANT ALL PRIVILEGES ON event_store_next TO aiworkflow;
GRANT ALL PRIVILEGES ON event_store_default TO aiworkflow;
GRANT ALL PRIVILEGES ON event_snapshots_v2 TO aiworkflow;
GRANT ALL PRIVILEGES ON event_replay_positions TO aiworkflow;
GRANT ALL PRIVILEGES ON event_sagas TO aiworkflow;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO aiworkflow;

-- Record this migration
SELECT record_migration(
    '20241213_000002',
    'create_event_store_v2',
    calculate_migration_checksum('Enhanced event store with partitioning and performance optimizations'),
    0 -- Will be updated by migration runner
);