CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY NOT NULL,
    space_id TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    stored_filename TEXT NOT NULL,  -- Actual filename on disk (with UUID)
    file_size_bytes INTEGER NOT NULL,
    mime_type TEXT,
    upload_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP,
    download_count INTEGER NOT NULL DEFAULT 0,
    checksum TEXT,  -- SHA256 hash for integrity verification
    
    -- Foreign key constraint
    FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

-- faster lookups
CREATE INDEX IF NOT EXISTS idx_files_space_id ON files(space_id);
CREATE INDEX IF NOT EXISTS idx_files_upload_date ON files(upload_date DESC);
CREATE INDEX IF NOT EXISTS idx_files_original_filename ON files(original_filename);
