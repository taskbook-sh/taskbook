# taskbook

A Rust port of [taskbook](https://github.com/klaussinani/taskbook) - tasks, boards & notes for the command-line habitat.

## Features

- Create and manage tasks and notes organized into boards
- Priority levels and progress tracking
- Timeline and archive views
- Customizable themes (including Catppuccin)
- **Optional server sync** with end-to-end encryption (AES-256-GCM)

## Installation

Download the latest binary from [releases](https://github.com/taskbook-sh/taskbook/releases) or build from source:

```bash
cargo install --path crates/taskbook-client
```

### Nix Flake

```nix
{
  inputs.taskbook.url = "github:taskbook-sh/taskbook";

  # Add to your packages
  environment.systemPackages = [ inputs.taskbook.packages.${system}.default ];
}
```

## Usage

```bash
tb                          # Display board view
tb --task "Description"     # Create task
tb --task @board "Desc"     # Create task in specific board
tb --task "Desc" p:2        # Create with priority (1=normal, 2=medium, 3=high)
tb --note "Description"     # Create note
tb --note                   # Create note in external editor
tb --edit-note @<id>        # Edit note in external editor
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

## Server Sync

Optionally sync your tasks across devices with encrypted server storage:

```bash
# Register an account (interactive)
tb --register

# Migrate existing local data to server
tb --migrate

# On another device, login with your encryption key
tb --login

# Check sync status
tb --status

# Logout and return to local-only mode
tb --logout
```

All data is encrypted client-side with AES-256-GCM before being sent to the server. Your encryption key is generated locally during registration and never leaves your device.

See [Server Setup](docs/server.md) for running your own server.

## Configuration

Configuration is stored in `~/.taskbook.json`:

```json
{
  "taskbookDirectory": "~",
  "displayCompleteTasks": true,
  "displayProgressOverview": true,
  "theme": "catppuccin-macchiato",
  "sync": {
    "enabled": false,
    "serverUrl": "http://localhost:8080"
  }
}
```

### Themes

Available preset themes: `default`, `catppuccin-macchiato`, `catppuccin-mocha`, `catppuccin-frappe`, `catppuccin-latte`, `high-contrast`

Or define custom RGB colors - see [Configuration docs](docs/configuration.md).

## Documentation

See the [docs](docs/) folder for detailed documentation:

- [Installation](docs/installation.md)
- [CLI Reference](docs/cli-reference.md)
- [Configuration](docs/configuration.md)
- [Server Setup](docs/server.md)
- [Sync & Encryption](docs/sync.md)
- [Kubernetes Deployment](docs/kubernetes.md)

## Data Compatibility

This implementation uses the same data format and directory (`~/.taskbook/`) as the original Node.js version, allowing seamless migration.

## License

MIT - see [LICENSE](LICENSE)

## Credits

Original project by [Klaus Sinani](https://github.com/klaussinani/taskbook)
