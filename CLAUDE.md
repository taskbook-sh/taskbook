# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this codebase.

## Project Overview

Taskbook-rs is a Rust port of the Node.js [taskbook](https://github.com/klaussinani/taskbook) CLI application. It provides task and note management from the command line with board organization, priority levels, and timeline views.

## Build & Development

This project uses devenv for development environment management.

```bash
# Enter development shell
devenv shell

# Build debug
cargo build

# Build release
cargo build --release

# Run directly
cargo run -- --help
cargo run -- --task "My task"
cargo run              # Display board view

# Run tests
cargo test

# Check for issues
cargo clippy
```

The binary is named `tb` and installs to `target/release/tb` or `target/debug/tb`.

## Architecture

```
src/
├── main.rs         # CLI entry point using clap
├── lib.rs          # Library exports
├── commands.rs     # Routes CLI flags to taskbook methods
├── taskbook.rs     # Core business logic (CRUD operations)
├── storage.rs      # JSON file persistence with atomic writes
├── config.rs       # ~/.taskbook.json configuration
├── directory.rs    # Taskbook directory resolution
├── render.rs       # Terminal output with colored formatting
├── error.rs        # Error types using thiserror
└── models/
    ├── mod.rs      # StorageItem enum (Task | Note)
    ├── item.rs     # Item trait definition
    ├── task.rs     # Task struct with serde
    └── note.rs     # Note struct with serde
```

### Key Design Decisions

1. **Backward Compatible JSON Format**: Uses `#[serde(rename = "...")]` to match the original Node.js field names (`_id`, `_date`, `_isTask`, `isStarred`, etc.) for seamless data migration.

2. **Atomic Writes**: Storage operations write to a temp file first, then rename to prevent data corruption on crash.

3. **Directory Resolution Priority**:
   - `--taskbook-dir` CLI flag (highest)
   - `TASKBOOK_DIR` environment variable
   - `~/.taskbook.json` config file
   - Default `~/.taskbook/` (lowest)

4. **Storage Structure**:
   ```
   ~/.taskbook/
   ├── storage/storage.json   # Active items
   ├── archive/archive.json   # Deleted items
   └── .temp/                  # Atomic write temp files
   ```

## CLI Usage

```bash
tb                          # Display board view
tb --task "Description"     # Create task
tb --task @board "Desc"     # Create task in specific board
tb --task "Desc" p:2        # Create with priority (1=normal, 2=medium, 3=high)
tb --note "Description"     # Create note
tb --check <id> [id...]     # Toggle task complete
tb --begin <id> [id...]     # Toggle task in-progress
tb --star <id> [id...]      # Toggle starred
tb --delete <id> [id...]    # Delete to archive
tb --restore <id> [id...]   # Restore from archive
tb --edit @<id> "New desc"  # Edit description
tb --move @<id> board       # Move to board
tb --priority @<id> <1-3>   # Set priority
tb --find <term>            # Search items
tb --list <attributes>      # Filter (pending, done, task, note, starred)
tb --timeline               # Chronological view
tb --archive                # View archived items
tb --clear                  # Delete all completed tasks
tb --copy <id> [id...]      # Copy descriptions to clipboard
```

## Dependencies

- `clap` - CLI argument parsing
- `serde` / `serde_json` - JSON serialization
- `colored` - Terminal colors
- `chrono` - Date/time handling
- `dirs` - Home directory resolution
- `arboard` - Clipboard access
- `uuid` - Temp file naming
- `thiserror` - Error handling

## Testing Notes

The application uses the same data directory (`~/.taskbook/`) as the original Node.js version, allowing seamless switching between implementations during testing.
