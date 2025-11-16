-- as i am storing files by their checksum as filename this no longer will be neccessary
ALTER TABLE files
DROP COLUMN stored_filename;

CREATE INDEX IF NOT EXISTS idx_files_checksum ON files(checksum);
