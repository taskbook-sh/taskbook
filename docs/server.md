# Server Setup

The taskbook server (`tb-server`) provides sync capabilities for accessing your tasks across multiple devices. All data is encrypted client-side before being sent to the server.

## Requirements

- PostgreSQL 14 or later
- Network connectivity between clients and server

## Quick Start with Docker Compose

The easiest way to run the server is with Docker Compose:

```bash
# Clone the repository
git clone https://github.com/taskbook-sh/taskbook.git
cd taskbook

# Start the server and database
docker compose up -d

# Check logs
docker compose logs -f server
```

This starts:
- PostgreSQL database on port 5432
- Taskbook server on port 8080

## Manual Setup

### 1. Create PostgreSQL Database

```bash
createdb taskbook
```

Or with psql:

```sql
CREATE DATABASE taskbook;
```

### 2. Configure Environment Variables

The server is configured entirely via environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `TB_DB_HOST` | Yes | - | Database hostname |
| `TB_DB_PORT` | No | `5432` | Database port |
| `TB_DB_NAME` | Yes | - | Database name |
| `TB_DB_USER` | Yes | - | Database username |
| `TB_DB_PASSWORD` | Yes | - | Database password |
| `TB_HOST` | No | `0.0.0.0` | Server bind address |
| `TB_PORT` | No | `8080` | Server port |
| `TB_SESSION_EXPIRY_DAYS` | No | `30` | Session token lifetime in days |
| `TB_CORS_ORIGINS` | No | (none) | Allowed CORS origins, comma-separated |
| `RUST_LOG` | No | `info` | Log level (trace, debug, info, warn, error) |

### 3. Run the Server

```bash
export TB_DB_HOST=localhost
export TB_DB_NAME=taskbook
export TB_DB_USER=postgres
export TB_DB_PASSWORD=secret

./tb-server
```

The server automatically runs database migrations on startup.

## Docker

### Build the Image

```bash
docker build -f Dockerfile.server -t taskbook-server .
```

### Run with Docker

```bash
docker run -d \
  --name taskbook-server \
  -p 8080:8080 \
  -e TB_DB_HOST=host.docker.internal \
  -e TB_DB_NAME=taskbook \
  -e TB_DB_USER=postgres \
  -e TB_DB_PASSWORD=secret \
  taskbook-server
```

## docker-compose.yml

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: taskbook
      POSTGRES_PASSWORD: taskbook
      POSTGRES_DB: taskbook
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U taskbook"]
      interval: 5s
      timeout: 5s
      retries: 5

  server:
    build:
      context: .
      dockerfile: Dockerfile.server
    environment:
      TB_HOST: "0.0.0.0"
      TB_PORT: "8080"
      TB_DB_HOST: postgres
      TB_DB_PORT: "5432"
      TB_DB_NAME: taskbook
      TB_DB_USER: taskbook
      TB_DB_PASSWORD: taskbook
      TB_SESSION_EXPIRY_DAYS: "30"
      RUST_LOG: "info"
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  postgres_data:
```

## API Endpoints

The server exposes the following REST API:

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/register` | Create new account |
| `POST` | `/api/v1/login` | Login and get session token |
| `DELETE` | `/api/v1/logout` | Invalidate session |
| `GET` | `/api/v1/me` | Get current user info |

Registration and login endpoints are rate-limited to 10 requests per IP per 60 seconds.

### Items

All item endpoints require `Authorization: Bearer <token>` header.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/items` | Get all active items |
| `PUT` | `/api/v1/items` | Replace all active items (max 10,000 items) |
| `GET` | `/api/v1/items/archive` | Get archived items |
| `PUT` | `/api/v1/items/archive` | Replace archived items (max 10,000 items) |

Request body size is limited to 10 MB.

### Real-time Sync

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/events` | SSE stream for real-time sync notifications |

The events endpoint provides Server-Sent Events (SSE) that notify connected clients when data changes. Events include a `data_changed` event type with either `"items"` or `"archive"` as the data payload. The server sends keep-alive pings every 15 seconds.

### Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/health` | Health check |

## Database Schema

The server creates these tables automatically:

```sql
-- User accounts
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username    VARCHAR(64) UNIQUE NOT NULL,
    email       VARCHAR(255) UNIQUE NOT NULL,
    password    TEXT NOT NULL,  -- Argon2id hash
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Session tokens
CREATE TABLE sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token       VARCHAR(128) UNIQUE NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Encrypted items
CREATE TABLE items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_key    VARCHAR(64) NOT NULL,
    data        BYTEA NOT NULL,     -- Encrypted JSON
    nonce       BYTEA NOT NULL,     -- AES-GCM nonce
    archived    BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, item_key, archived)
);

CREATE INDEX idx_items_user ON items(user_id, archived);
```

## Security Considerations

### Encryption

- All item data is encrypted client-side using AES-256-GCM
- The encryption key is generated on the client during registration
- The server never sees the encryption key or plaintext data
- Only the item ID and metadata (archived status, timestamps) are visible to the server

### Rate Limiting

- Registration and login endpoints are rate-limited per IP address
- 10 requests per 60-second sliding window
- Returns HTTP 429 (Too Many Requests) when exceeded

### Authentication

- Passwords are hashed using Argon2id
- Session tokens are cryptographically random 256-bit values (base64url-encoded)
- Tokens expire after configurable number of days (default 30)
- User deletion cascades to sessions and items

### Network

- Always use HTTPS in production
- Consider placing behind a reverse proxy (nginx, Caddy, Traefik)

## Reverse Proxy Example

### nginx

```nginx
server {
    listen 443 ssl http2;
    server_name taskbook.example.com;

    ssl_certificate /etc/letsencrypt/live/taskbook.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/taskbook.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```
taskbook.example.com {
    reverse_proxy localhost:8080
}
```

## Monitoring

### Health Check

```bash
curl http://localhost:8080/api/v1/health
# {"status":"ok"}
```

### Logs

Set `RUST_LOG` for different log levels:

```bash
RUST_LOG=debug ./tb-server    # Verbose
RUST_LOG=info ./tb-server     # Normal
RUST_LOG=warn ./tb-server     # Quiet
```

## Backup

Back up the PostgreSQL database regularly:

```bash
pg_dump -U taskbook taskbook > backup.sql
```

Note: The backup contains encrypted data. Without users' encryption keys, the item data cannot be decrypted.
