-- Migration: Initial schema
-- Version: 0001
-- Applied: 2026-11-03

-- Telegram Sessions
CREATE TABLE telegram_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE NOT NULL,
    session_data BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_sessions_user ON telegram_sessions(user_id);

-- Channels
CREATE TABLE channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    telegram_id INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    username TEXT,
    link TEXT,
    custom_max_size_mb INTEGER,
    zip_max_size_mb INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    added_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_channels_telegram_id ON channels(telegram_id);
CREATE INDEX idx_channels_enabled ON channels(enabled);

-- Monitoring State (singleton)
CREATE TABLE monitoring_state (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    is_running BOOLEAN NOT NULL DEFAULT 0 CHECK(is_running IN (0, 1)),
    last_update_id INTEGER DEFAULT 0,
    started_at INTEGER,
    stopped_at INTEGER
);

INSERT INTO monitoring_state (id, is_running) VALUES (1, 0);

-- Views
CREATE VIEW enabled_channels AS
SELECT * FROM channels WHERE enabled = 1;

CREATE VIEW monitoring_status AS
SELECT 
    (SELECT is_running FROM monitoring_state WHERE id = 1) as is_monitoring,
    (SELECT COUNT(*) FROM channels) as total_channels,
    (SELECT COUNT(*) FROM channels WHERE enabled = 1) as enabled_channels,
    (SELECT last_update_id FROM monitoring_state WHERE id = 1) as last_update_id;
