-- ChaCrab Database Schema for SQLite
-- Single-user password manager

-- User configuration table (single row)
CREATE TABLE IF NOT EXISTS user_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    salt TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Credentials table
CREATE TABLE IF NOT EXISTS credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT NOT NULL UNIQUE,
    url TEXT,
    enc_username TEXT NOT NULL,
    enc_password TEXT NOT NULL,
    nonce_username TEXT NOT NULL,
    nonce_password TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_credentials_label ON credentials(label);
CREATE INDEX IF NOT EXISTS idx_credentials_created ON credentials(created_at DESC);
