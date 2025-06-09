-- Video Analytics Platform Database Schema
-- This script initializes the database with all necessary tables

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Video streams table
CREATE TABLE IF NOT EXISTS video_streams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    source_url VARCHAR(500),
    source_type VARCHAR(50) NOT NULL, -- 'rtmp', 'webrtc', 'file', 'camera'
    status VARCHAR(50) NOT NULL DEFAULT 'inactive', -- 'active', 'inactive', 'error'
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Video segments table (for stored video chunks)
CREATE TABLE IF NOT EXISTS video_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID REFERENCES video_streams(id) ON DELETE CASCADE,
    file_path VARCHAR(500) NOT NULL,
    duration_seconds REAL NOT NULL,
    size_bytes BIGINT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Convert video_segments to hypertable for time-series optimization
SELECT create_hypertable('video_segments', 'timestamp', if_not_exists => TRUE);

-- Inference models table
CREATE TABLE IF NOT EXISTS inference_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    model_type VARCHAR(100) NOT NULL, -- 'object_detection', 'face_recognition', 'action_recognition'
    file_path VARCHAR(500) NOT NULL,
    config JSONB,
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Inference results table (time-series data)
CREATE TABLE IF NOT EXISTS inference_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID REFERENCES video_streams(id) ON DELETE CASCADE,
    model_id UUID REFERENCES inference_models(id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    frame_number BIGINT NOT NULL,
    confidence REAL NOT NULL,
    bounding_box JSONB, -- {x, y, width, height}
    detected_class VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Convert inference_results to hypertable
SELECT create_hypertable('inference_results', 'timestamp', if_not_exists => TRUE);

-- Analytics events table (time-series data)
CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID REFERENCES video_streams(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL, -- 'person_detected', 'vehicle_detected', 'alert_triggered'
    event_data JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    severity VARCHAR(20) DEFAULT 'info', -- 'info', 'warning', 'critical'
    processed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Convert analytics_events to hypertable
SELECT create_hypertable('analytics_events', 'timestamp', if_not_exists => TRUE);

-- Alerts table
CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID REFERENCES video_streams(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'info',
    status VARCHAR(50) NOT NULL DEFAULT 'open', -- 'open', 'acknowledged', 'closed'
    metadata JSONB,
    triggered_at TIMESTAMP WITH TIME ZONE NOT NULL,
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    acknowledged_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Sessions table for authentication
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) UNIQUE NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN DEFAULT TRUE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_video_streams_status ON video_streams(status);
CREATE INDEX IF NOT EXISTS idx_video_streams_created_by ON video_streams(created_by);
CREATE INDEX IF NOT EXISTS idx_video_segments_stream_id ON video_segments(stream_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_stream_id ON inference_results(stream_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_model_id ON inference_results(model_id);
CREATE INDEX IF NOT EXISTS idx_inference_results_detected_class ON inference_results(detected_class);
CREATE INDEX IF NOT EXISTS idx_analytics_events_stream_id ON analytics_events(stream_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_event_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_processed ON analytics_events(processed);
CREATE INDEX IF NOT EXISTS idx_alerts_stream_id ON alerts(stream_id);
CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);
CREATE INDEX IF NOT EXISTS idx_alerts_severity ON alerts(severity);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);

-- Create a default admin user (password: admin123 - change in production!)
INSERT INTO users (email, password_hash, role) 
VALUES ('admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj3L3jHG6LwC', 'admin')
ON CONFLICT (email) DO NOTHING;

-- Create some sample inference models
INSERT INTO inference_models (name, version, model_type, file_path, config, is_active)
VALUES 
    ('YOLOv8n', '1.0.0', 'object_detection', '/app/models/yolov8n.onnx', '{"input_size": [640, 640], "classes": 80}', true),
    ('Face Detection', '1.0.0', 'face_detection', '/app/models/face_detection.onnx', '{"confidence_threshold": 0.5}', true)
ON CONFLICT DO NOTHING; 