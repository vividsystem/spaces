CREATE TABLE IF NOT EXISTS spaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    total_size_used_bytes INTEGER NOT NULL DEFAULT 0,
		access_code TEXT -- optional: maybe something in the future like passwords
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_spaces_created_at ON spaces(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_spaces_access_code ON spaces(access_code) WHERE access_code IS NOT NULL;
