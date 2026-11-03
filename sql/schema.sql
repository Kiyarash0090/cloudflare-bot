-- Database schema for Telegram-Bale Bot
-- Version: 1.0.0
-- Description: Core tables for session management, channel monitoring, and state tracking

-- ============================================
-- Telegram Sessions Table
-- ============================================
-- Stores serialized Telegram session data for user authentication
CREATE TABLE IF NOT EXISTS telegram_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE NOT NULL,
    session_data BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON telegram_sessions(user_id);

-- ============================================
-- Channels Table
-- ============================================
-- Stores Telegram channels to monitor and forward to Bale
CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    telegram_id INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    username TEXT,
    link TEXT,
    custom_max_size_mb INTEGER,  -- Custom file size limit (20-200MB)
    zip_max_size_mb INTEGER,     -- If set, compress all files and split
    enabled BOOLEAN NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    added_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_channels_telegram_id ON channels(telegram_id);
CREATE INDEX IF NOT EXISTS idx_channels_enabled ON channels(enabled);

-- ============================================
-- Monitoring State Table (Singleton)
-- ============================================
-- Tracks global monitoring on/off state
CREATE TABLE IF NOT EXISTS monitoring_state (
    id INTEGER PRIMARY KEY CHECK(id = 1),  -- Only one row allowed
    is_running BOOLEAN NOT NULL DEFAULT 0 CHECK(is_running IN (0, 1)),
    last_update_id INTEGER DEFAULT 0,
    started_at INTEGER,
    stopped_at INTEGER
);

-- Initialize with default state
INSERT OR IGNORE INTO monitoring_state (id, is_running) VALUES (1, 0);

-- ============================================
-- Pending Posts Table (Phase 2)
-- ============================================
-- Stores posts that couldn't be sent immediately (for download buttons)
CREATE TABLE IF NOT EXISTS pending_posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT UNIQUE NOT NULL,
    telegram_channel_id INTEGER NOT NULL,
    telegram_message_id INTEGER NOT NULL,
    file_id TEXT,
    file_type TEXT,  -- 'photo', 'document', 'video', etc.
    caption TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pending_token ON pending_posts(token);
CREATE INDEX IF NOT EXISTS idx_pending_expires ON pending_posts(expires_at);

-- ============================================
-- Views for convenience
-- ============================================
CREATE VIEW IF NOT EXISTS enabled_channels AS
SELECT * FROM channels WHERE enabled = 1;

CREATE VIEW IF NOT EXISTS monitoring_status AS
SELECT 
    (SELECT is_running FROM monitoring_state WHERE id = 1) as is_monitoring,
    (SELECT COUNT(*) FROM channels) as total_channels,
    (SELECT COUNT(*) FROM channels WHERE enabled = 1) as enabled_channels,
    (SELECT last_update_id FROM monitoring_state WHERE id = 1) as last_update_id;
