# spaces
a file sharing solution for you and your friends.

## Idea and Motivation
I was on a trip with friends recently. Not all of them had the same OS on their phone and we didn't want to send the files via messenger. So I had the idea to build this.
It is supposed to be file sharing managed into groups. I intend on adding auth in the future. I didn't have time to implement everything as this is MY *FIRST PROJECT IN RUST*. 

## Features
- file sharing for groups
- speeeeeed (cuz Rust)
- very nice logging in the backend
- metadata storage
- automatic file-deduplication (no file is stored more than once at the same time)


## Requirements
- UNIX OS? probably even windows
- Bun (or another TS/JS runtime) and Rust installed


## Deploy
```bash
cd backend
cargo build --release
cd ../web
bun run build
```

### Example Systemd services
#### Backend
```ini
[Unit]
Description=Spaces Backend
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/your-app
ExecStart=/home/your-user/your-app/target/release/your-binary-name
Restart=on-failure
RestartSec=5
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```
#### Web
```ini
[Unit]
Description=Spaces Web
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/your-project
ExecStart=/home/your-user/.bun/bin/bun run src/index.ts
Restart=on-failure
RestartSec=5
Environment="NODE_ENV=production"
Environment="PORT=3000"

[Install]
WantedBy=multi-user.target
```

## Configuration
[Frontend Configuration](./web/README.md)
[Backend Configuration](./backend/README.md)

