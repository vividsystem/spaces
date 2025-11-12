-- Add migration script here
ALTER TABLE files
ADD UNIQUE (stored_filename);
