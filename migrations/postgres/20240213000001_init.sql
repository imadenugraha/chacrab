-- ChaCrab Database Schema for PostgreSQL
-- Single-user password manager

-- User configuration table (single row)
CREATE TABLE IF NOT EXISTS user_config (
    id BIGSERIAL PRIMARY KEY,
    salt TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Credentials table
CREATE TABLE IF NOT EXISTS credentials (
    id BIGSERIAL PRIMARY KEY,
    label TEXT NOT NULL UNIQUE,
    url TEXT,
    enc_username TEXT NOT NULL,
    enc_password TEXT NOT NULL,
    nonce_username TEXT NOT NULL,
    nonce_password TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_credentials_label ON credentials(label);
CREATE INDEX IF NOT EXISTS idx_credentials_created ON credentials(created_at DESC);
