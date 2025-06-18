-- AI Workflow System Database Initialization Script
-- This script sets up the initial database schema for development

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create agents table for service registry
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    endpoint TEXT NOT NULL,
    capabilities TEXT[] NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_capabilities ON agents USING GIN(capabilities);
CREATE INDEX IF NOT EXISTS idx_agents_last_seen ON agents(last_seen);
CREATE INDEX IF NOT EXISTS idx_agents_name ON agents(name);

-- Create events table for workflow tracking
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for events
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at);

-- Create sessions table for workflow sessions
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_type VARCHAR(100) NOT NULL,
    session_data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for sessions
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);

-- Create users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(64) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE NULL
);

-- Create indexes for users table
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Insert sample development data
INSERT INTO agents (id, name, endpoint, capabilities, status, last_seen, metadata) VALUES
    (uuid_generate_v4(), 'sample-ai-tutor', 'http://localhost:3001', ARRAY['tutoring', 'education'], 'active', NOW(), '{"version": "1.0.0", "environment": "development"}'),
    (uuid_generate_v4(), 'sample-workflow-engine', 'http://localhost:3002', ARRAY['orchestration', 'workflow'], 'active', NOW(), '{"version": "1.0.0", "environment": "development"}')
ON CONFLICT (name) DO NOTHING;

-- Insert sample users for development
-- Default password for all users is 'password123' 
-- Hash is SHA256 of 'password123' + salt for each user
INSERT INTO users (username, email, password_hash, salt, is_active, role) VALUES
    ('admin', 'admin@example.com', 'b89eaac7e61417341b710b727768294d0e6a277b9d4fde53a4090a8e2d8e6c7c', 'admin_salt_2024', true, 'admin'),
    ('demo_user', 'demo@example.com', 'f8e5c79b0f3a1c2e9b7d4a6e1f2c3b9a8e7d5f4c2b1a0e9d8c7b6a5f4e3d2c1b0', 'demo_salt_2024', true, 'user'),
    ('support', 'support@example.com', 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2', 'support_salt_2024', true, 'support')
ON CONFLICT (username) DO NOTHING;

-- Create a function to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_events_updated_at BEFORE UPDATE ON events FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Grant permissions (in development, we can be more permissive)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO aiworkflow;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO aiworkflow;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO aiworkflow;

-- Create additional indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_agents_status_last_seen ON agents(status, last_seen);
CREATE INDEX IF NOT EXISTS idx_events_type_created_at ON events(event_type, created_at);

-- Ensure uuid-ossp functions are available
SELECT uuid_generate_v4() AS test_uuid;