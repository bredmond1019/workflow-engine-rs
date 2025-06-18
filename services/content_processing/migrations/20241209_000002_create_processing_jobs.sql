-- Create processing job status enum
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');

-- Create processing job priority enum  
CREATE TYPE job_priority AS ENUM ('low', 'medium', 'high', 'critical');

-- Create processing jobs table
CREATE TABLE IF NOT EXISTS processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID REFERENCES content_metadata(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    priority job_priority NOT NULL DEFAULT 'medium',
    options JSONB DEFAULT '{}'::jsonb,
    result JSONB,
    error_message TEXT,
    retry_count INT DEFAULT 0,
    max_retries INT DEFAULT 3,
    worker_id VARCHAR(100),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_processing_jobs_status ON processing_jobs(status);
CREATE INDEX idx_processing_jobs_priority ON processing_jobs(priority);
CREATE INDEX idx_processing_jobs_content_id ON processing_jobs(content_id);
CREATE INDEX idx_processing_jobs_job_type ON processing_jobs(job_type);
CREATE INDEX idx_processing_jobs_created_at ON processing_jobs(created_at);
CREATE INDEX idx_processing_jobs_worker_id ON processing_jobs(worker_id);

-- Create composite index for job queue queries
CREATE INDEX idx_processing_jobs_queue ON processing_jobs(status, priority DESC, created_at ASC)
WHERE status IN ('pending', 'running');

-- Add trigger to update updated_at timestamp
CREATE TRIGGER set_timestamp_processing_jobs
    BEFORE UPDATE ON processing_jobs
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();

-- Create job metrics table for tracking performance
CREATE TABLE IF NOT EXISTS job_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES processing_jobs(id) ON DELETE CASCADE,
    metric_name VARCHAR(100) NOT NULL,
    metric_value FLOAT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for metrics queries
CREATE INDEX idx_job_metrics_job_id ON job_metrics(job_id);
CREATE INDEX idx_job_metrics_metric_name ON job_metrics(metric_name);
CREATE INDEX idx_job_metrics_created_at ON job_metrics(created_at);