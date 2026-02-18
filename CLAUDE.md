# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this codebase.

## Project Overview

Taskbook-rs is a Rust port of the Node.js [taskbook](https://github.com/klaussinani/taskbook) CLI application. It provides task and note management from the command line with board organization, priority levels, and timeline views. It supports both local file storage and remote server sync with client-side encryption.

## Build & Development

This project uses devenv for development environment management.

```bash
# Enter development shell
devenv shell

# Build debug
cargo build

# Build release
cargo build --release

# Run the client
cargo run --package taskbook-client -- --help
cargo run --package taskbook-client -- --task "My task"
cargo run --package taskbook-client              # Display board view

# Run the server
cargo run --package taskbook-server

# Run tests
cargo test

# Check for issues
cargo clippy
```

The client binary is named `tb` and the server binary is `tb-server`.

## Architecture

This is a Cargo workspace with three crates:

```
crates/
├── taskbook-common/        # Shared types, models, encryption
│   └── src/
│       ├── lib.rs          # Module exports
│       ├── api.rs          # Shared API request/response types
│       ├── board.rs        # Board name handling, normalization
│       ├── encryption.rs   # AES-256-GCM encrypt/decrypt
│       ├── error.rs        # CommonError type
│       └── models/         # StorageItem, Task, Note, Item trait
│
├── taskbook-client/        # CLI + TUI binary (tb)
│   └── src/
│       ├── main.rs         # CLI entry point using clap
│       ├── commands.rs     # Routes CLI flags to taskbook methods + migrate
│       ├── taskbook.rs     # Core business logic (CRUD operations)
│       ├── api_client.rs   # HTTP client for server communication
│       ├── auth.rs         # Register, login, logout, status commands
│       ├── credentials.rs  # Encryption key and token management
│       ├── config.rs       # ~/.taskbook.json configuration (with sync section)
│       ├── directory.rs    # Taskbook directory resolution
│       ├── render.rs       # Terminal output with colored formatting
│       ├── editor.rs       # External editor support for notes
│       ├── error.rs        # Error types using thiserror
│       ├── storage/
│       │   ├── mod.rs      # StorageBackend trait
│       │   ├── local.rs    # LocalStorage (file-based)
│       │   └── remote.rs   # RemoteStorage (HTTP + encryption)
│       └── tui/            # Interactive TUI (ratatui + crossterm)
│
└── taskbook-server/        # Server binary (tb-server)
    └── src/
        ├── main.rs         # Axum server entry point
        ├── config.rs       # Server config from environment variables
        ├── db.rs           # PostgreSQL connection pool (sqlx)
        ├── router.rs       # Route definitions and AppState
        ├── auth.rs         # Argon2id password hashing
        ├── error.rs        # ServerError → HTTP response mapping
        ├── middleware.rs    # Auth middleware (Bearer token extraction)
        ├── rate_limit.rs   # Per-IP sliding window rate limiter
        ├── handlers/
        │   ├── user.rs     # POST /register, POST /login, DELETE /logout, GET /me
        │   ├── items.rs    # GET/PUT /items, GET/PUT /items/archive
        │   ├── events.rs   # GET /events (SSE real-time sync notifications)
        │   └── health.rs   # GET /health
        └── migrations/
            ├── 001_initial.sql          # Users, sessions, items tables
            └── 002_add_session_index.sql # Session lookup index
```

### Key Design Decisions

1. **StorageBackend Trait**: `Taskbook` business logic is storage-agnostic via `Box<dyn StorageBackend>`. Backend selection is config-driven — local file storage by default, remote server when `sync.enabled = true`.

2. **Client-Side Encryption**: All data is encrypted with AES-256-GCM before being sent to the server. The 32-byte encryption key is generated on registration and never leaves the client. Each item is encrypted individually with a unique random nonce.

3. **Backward Compatible JSON Format**: Uses `#[serde(rename = "...")]` to match the original Node.js field names (`_id`, `_date`, `_isTask`, `isStarred`, etc.) for seamless data migration.

4. **Atomic Writes**: Local storage operations write to a temp file first, then rename to prevent data corruption on crash.

5. **Directory Resolution Priority**:
   - `--taskbook-dir` CLI flag (highest)
   - `TASKBOOK_DIR` environment variable
   - `~/.taskbook.json` config file
   - Default `~/.taskbook/` (lowest)

6. **Storage Structure (Local)**:
   ```
   ~/.taskbook/
   ├── storage/storage.json   # Active items
   ├── archive/archive.json   # Deleted items
   ├── credentials.json       # Server token + encryption key
   └── .temp/                 # Atomic write temp files
   ```

## CLI Usage

```bash
# Task management
tb                          # Display board view (TUI)
tb --task "Description"     # Create task
tb --task @board "Desc"     # Create task in specific board
tb --task "Desc" p:2        # Create with priority (1=normal, 2=medium, 3=high)
tb --note "Description"     # Create note
tb --note                   # Create note in external editor ($EDITOR)
tb --check <id> [id...]     # Toggle task complete
tb --begin <id> [id...]     # Toggle task in-progress
tb --star <id> [id...]      # Toggle starred
tb --delete <id> [id...]    # Delete to archive
tb --restore <id> [id...]   # Restore from archive
tb --edit @<id> "New desc"  # Edit description
tb --edit-note @<id>        # Edit note in external editor ($EDITOR)
tb --move @<id> board       # Move to board
tb --priority @<id> <1-3>   # Set priority
tb --find <term>            # Search items
tb --list <attributes>      # Filter (pending, done, task, note, starred)
tb --timeline               # Chronological view
tb --archive                # View archived items
tb --clear                  # Delete all completed tasks
tb --copy <id> [id...]      # Copy descriptions to clipboard

# Server sync
tb --register --server <url> --username <name> --email <email> --password <pass>
tb --login --server <url> --username <name> --password <pass> --key <base64>
tb --logout
tb --status
tb --migrate                # Push local data to server

# Mode
tb --cli                    # Force non-interactive CLI mode
```

## Server Configuration

The server reads configuration from environment variables:

```bash
TB_HOST=0.0.0.0              # Listen address
TB_PORT=8080                  # Listen port
TB_DB_HOST=localhost          # Database hostname
TB_DB_PORT=5432               # Database port (default: 5432)
TB_DB_NAME=taskbook           # Database name
TB_DB_USER=postgres           # Database username
TB_DB_PASSWORD=secret         # Database password
TB_SESSION_EXPIRY_DAYS=30     # Session token expiry (default: 30)
TB_CORS_ORIGINS=              # Allowed CORS origins, comma-separated
RUST_LOG=info                 # Log level (trace, debug, info, warn, error)
```

### Running with Docker

```bash
docker build -f Dockerfile.server -t tb-server .
docker run -p 8080:8080 \
  -e TB_DB_HOST=host.docker.internal \
  -e TB_DB_NAME=taskbook \
  -e TB_DB_USER=postgres \
  -e TB_DB_PASSWORD=secret \
  tb-server
```

## API Endpoints

All under `/api/v1/`:

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | /health | No | Health check |
| POST | /register | No | Create account |
| POST | /login | No | Get session token |
| DELETE | /logout | Yes | Invalidate session |
| GET | /me | Yes | Get user info |
| GET | /items | Yes | Get encrypted items |
| PUT | /items | Yes | Replace all items |
| GET | /items/archive | Yes | Get encrypted archive |
| PUT | /items/archive | Yes | Replace all archived items |
| GET | /events | Yes | SSE stream for real-time sync notifications |

## Dependencies

### Client (taskbook-client)
- `clap` - CLI argument parsing
- `serde` / `serde_json` - JSON serialization
- `colored` - Terminal colors
- `chrono` - Date/time handling
- `dirs` - Home directory resolution
- `arboard` - Clipboard access
- `uuid` - Temp file naming
- `thiserror` - Error handling
- `ratatui` / `crossterm` - Terminal UI
- `reqwest` - HTTP client for server sync
- `base64` - Encoding encrypted data
- `rpassword` - Secure password input
- `fs2` - File locking for local storage

### Common (taskbook-common)
- `aes-gcm` - AES-256-GCM encryption
- `rand` - Cryptographic random number generation

### Server (taskbook-server)
- `axum` - HTTP framework
- `sqlx` - PostgreSQL async driver with migrations
- `argon2` - Password hashing (Argon2id)
- `tower-http` - HTTP middleware (CORS, tracing, body limits)
- `tracing` / `tracing-subscriber` - Structured logging
- `tokio-stream` / `futures-util` - SSE event streaming

## Testing Notes

The application uses the same data directory (`~/.taskbook/`) as the original Node.js version, allowing seamless switching between implementations during testing.
