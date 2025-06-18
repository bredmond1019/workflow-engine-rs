-- Create content metadata table
CREATE TABLE IF NOT EXISTS content_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    source_url TEXT,
    content_type VARCHAR(50) NOT NULL,
    format VARCHAR(50) NOT NULL,
    size_bytes BIGINT,
    hash VARCHAR(64) NOT NULL,
    quality_score FLOAT,
    difficulty_level VARCHAR(20),
    concepts JSONB DEFAULT '[]'::jsonb,
    embeddings vector(1536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_content_metadata_content_type ON content_metadata(content_type);
CREATE INDEX idx_content_metadata_quality_score ON content_metadata(quality_score);
CREATE INDEX idx_content_metadata_difficulty_level ON content_metadata(difficulty_level);
CREATE INDEX idx_content_metadata_created_at ON content_metadata(created_at);
CREATE INDEX idx_content_metadata_hash ON content_metadata(hash);
CREATE INDEX idx_content_metadata_concepts_gin ON content_metadata USING gin(concepts);

-- Create vector similarity search index
CREATE INDEX idx_content_metadata_embeddings_ivfflat ON content_metadata 
USING ivfflat (embeddings vector_cosine_ops) 
WITH (lists = 100);

-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_timestamp_content_metadata
    BEFORE UPDATE ON content_metadata
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();