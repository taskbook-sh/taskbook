# taskbook — Rust workspace with three crates:
#
#   taskbook-common  shared models (Task/Note/StorageItem), board + due-date parsing,
#                    AES-256-GCM encryption, API request/response types
#   taskbook-client  `tb` binary: TUI (default), CLI (flags/--cli), MCP stdio server (--mcp),
#                    local JSON storage or remote sync
#   taskbook-server  `tb-server` binary: axum + Postgres (sqlx) sync server, env-configured
#
# Start with `just` (lists recipes)

set dotenv-load := true

# Scratch taskbook + config so experiments never touch ~/.taskbook or ~/.config/taskbook
sandbox := justfile_directory() / "target" / "sandbox"

# Local server env, matching the postgres service in docker-compose.yml
export TB_DB_HOST := env("TB_DB_HOST", "localhost")
export TB_DB_PORT := env("TB_DB_PORT", "5432")
export TB_DB_NAME := env("TB_DB_NAME", "taskbook")
export TB_DB_USER := env("TB_DB_USER", "taskbook")
export TB_DB_PASSWORD := env("TB_DB_PASSWORD", "taskbook")
export TB_PORT := env("TB_PORT", "8080")
export RUST_LOG := env("RUST_LOG", "info")

server_url := "http://localhost:" + TB_PORT

default:
    @just --list --unsorted

# Show the dependency tree of one crate (common | client | server)
deps crate="client":
    cargo tree -p taskbook-{{ crate }} --depth 1

# Open rustdoc for the workspace (private items included, useful for learning)
doc:
    cargo doc --workspace --no-deps --document-private-items --open

# ─── Build & check (mirrors .github/workflows/ci.yml) ─────────────────────────

build:
    cargo build --workspace

release:
    cargo build --workspace --release

check:
    cargo check --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace -- -D warnings

# Everything CI runs on a PR
ci: fmt-check clippy test

# ─── Tests ────────────────────────────────────────────────────────────────────

# All tests, or filter by name: `just test due`
test *filter:
    cargo test --workspace {{ filter }}

# Tests in one crate, optionally one module: `just test-mod common board::`
test-mod crate module="":
    cargo test -p taskbook-{{ crate }} {{ module }}

# Re-run tests on change (needs cargo-watch)
watch crate="common":
    cargo watch -x "test -p taskbook-{{ crate }}"

# ─── Client (tb) ──────────────────────────────────────────────────────────────

# Run tb with arbitrary args against your real ~/.taskbook
tb *args:
    cargo run -q -p taskbook-client -- {{ args }}

# TUI on the bundled sample data (sample/.taskbook)
sample *args:
    cargo run -q -p taskbook-client -- --taskbook-dir sample {{ args }}

# CLI board view of the sample data
sample-cli:
    cargo run -q -p taskbook-client -- --taskbook-dir sample --cli

# tb against a throwaway dir + config (target/sandbox) — safe for experiments
sandbox *args:
    mkdir -p {{ sandbox }}/config
    XDG_CONFIG_HOME={{ sandbox }}/config TASKBOOK_DIR={{ sandbox }} \
        cargo run -q -p taskbook-client -- {{ args }}

# Seed the sandbox with a few items, then show the board
sandbox-seed: sandbox-reset
    just sandbox --task @coding Read taskbook-common p:2
    just sandbox --task @coding Rebuild models/task.rs
    just sandbox --note @notes The client binary is tb, the server is tb-server
    just sandbox --cli

sandbox-reset:
    rm -rf {{ sandbox }}

# Run tb as an MCP stdio server over the sandbox
mcp:
    mkdir -p {{ sandbox }}/config
    XDG_CONFIG_HOME={{ sandbox }}/config TASKBOOK_DIR={{ sandbox }} \
        cargo run -q -p taskbook-client -- --mcp

# Smoke-test MCP: send initialize + tools/list and print the responses
mcp-smoke:
    printf '%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"just","version":"0"}}}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        | just mcp

# ─── Server (tb-server) ───────────────────────────────────────────────────────

# Start only Postgres from docker-compose
db-up:
    docker compose up -d postgres

db-down:
    docker compose down

# Drop the Postgres volume too
db-nuke:
    docker compose down -v

# psql shell into the compose database
db-shell:
    docker compose exec postgres psql -U {{ TB_DB_USER }} {{ TB_DB_NAME }}

# Run tb-server from source against the compose Postgres (migrations run on startup)
server: db-up
    cargo run -p taskbook-server

health:
    curl -fsS {{ server_url }}/api/v1/health && echo

metrics:
    curl -fsS {{ server_url }}/metrics

# Point the sandbox client at the local server and register a user
sandbox-register user="dev" password="devpassword":
    just sandbox --register --server {{ server_url }} --username {{ user }} --email {{ user }}@example.com --password {{ password }}

sandbox-status:
    just sandbox --status

# Whole stack from published images (postgres + server + client)
up:
    docker compose up -d postgres server

down:
    docker compose down

logs:
    docker compose logs -f server

# ─── Containers ───────────────────────────────────────────────────────────────

docker-build:
    docker build -f Dockerfile.server -t taskbook-server:dev .
    docker build -f Dockerfile.client -t taskbook-client:dev .

# ─── Release ──────────────────────────────────────────────────────────────────

# Preview a version bump without committing/pushing (scripts/release.sh expects git, not jj)
release-dry version:
    scripts/release.sh --dry-run {{ version }}

clean:
    cargo clean
