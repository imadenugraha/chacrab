-- Add verification token to user_config for session validation
-- This allows validating that the master password is correct during login

ALTER TABLE user_config ADD COLUMN verification_token TEXT;
ALTER TABLE user_config ADD COLUMN verification_nonce TEXT;
