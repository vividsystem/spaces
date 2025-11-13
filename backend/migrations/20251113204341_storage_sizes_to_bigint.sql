-- Add migration script here
ALTER TABLE files
ALTER COLUMN file_size_bytes TYPE BIGINT;

ALTER TABLE spaces
ALTER COLUMN total_size_used_bytes TYPE BIGINT;
