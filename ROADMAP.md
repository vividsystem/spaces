# ROADMAP
As I am building this for my week 9? project of Hackclub Siege I need to do this well prepared.
(its my first time using rust btw)
## Setup
- setup web framework using Axum?
- health endpoitn /health
- cors for frontend?
- logging using 'tracing' crate
- serde? for json serialization

## Data Models
- Space: id, name, created_at
- File: id, name, size, upload_date, space_id


## API Endpoints
~CRUD /api/spaces/:id~
POST /api/spaces/:id/files
GET /api/spaces/:id/files
GET /api/files/:id/download
DELETE /api/files/:id
user stuff?

## Storage Layer?
Store files in "uploads/" directory
-> `uploads/{space_id}/{file_id}_{filename}`
metadata (see datamodels)

file operations using `tokio::fs`?
