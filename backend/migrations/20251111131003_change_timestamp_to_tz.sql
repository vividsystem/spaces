ALTER TABLE spaces 
ALTER COLUMN created_at TYPE timestamptz USING created_at AT TIME ZONE 'UTC',
ALTER COLUMN updated_at TYPE timestamptz USING created_at AT TIME ZONE 'UTC';

ALTER TABLE files
ALTER COLUMN upload_date TYPE timestamptz USING upload_date AT TIME ZONE 'UTC',
ALTER COLUMN last_accessed TYPE timestamptz USING last_accessed AT TIME ZONE 'UTC';

