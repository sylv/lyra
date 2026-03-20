ALTER TABLE users ADD COLUMN last_seen_at INTEGER;

UPDATE users
SET last_seen_at = created_at
WHERE password_hash IS NOT NULL;
