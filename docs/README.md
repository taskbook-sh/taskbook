# Taskbook Documentation

Tasks, boards & notes for the command-line habitat.

Taskbook is a CLI application for managing tasks and notes organized into boards. It supports both local storage and encrypted server sync for accessing your tasks across multiple devices.

## Quick Start

```bash
# Create a task
tb --task "Review pull request"

# Create a task in a specific board with priority
tb --task @work "Deploy to production" p:3

# View your boards
tb

# Mark task as complete
tb --check 1

# Start working on a task
tb --begin 2
```

## Documentation

| Document | Description |
|----------|-------------|
| [Installation](installation.md) | How to install the client and server |
| [CLI Reference](cli-reference.md) | Complete command reference |
| [Configuration](configuration.md) | Client configuration options |
| [Server Setup](server.md) | Running the sync server |
| [Sync & Encryption](sync.md) | Setting up sync between devices |
| [Kubernetes Deployment](kubernetes.md) | Deploying the server to Kubernetes |
| [Observability](observability.md) | OpenTelemetry traces, metrics & logs |

## Features

- **Tasks & Notes**: Create tasks with priorities and notes with rich body content
- **Boards**: Organize items into custom boards
- **Interactive TUI**: Full-featured terminal UI with keyboard navigation
- **External Editor**: Compose and edit notes in your preferred editor (`$EDITOR`)
- **Timeline View**: See items chronologically
- **Search & Filter**: Find items by text or attributes
- **Sortable Boards**: Sort by ID, priority, or status
- **Archive**: Soft-delete with restore capability
- **Clipboard**: Copy item descriptions
- **Themes**: Customizable color schemes including Catppuccin
- **Server Sync**: Optional encrypted sync with real-time SSE notifications
- **End-to-End Encryption**: Your data is encrypted client-side with AES-256-GCM

## Architecture

```
~/.taskbook/              # Data directory
├── storage/
│   └── storage.json      # Active items
└── archive/
    └── archive.json      # Archived items

~/.taskbook.json          # Configuration file
~/.taskbook/credentials.json  # Server credentials (when using sync)
```

## Data Compatibility

This implementation uses the same data format as the original [Node.js taskbook](https://github.com/klaussinani/taskbook), allowing seamless migration from the original version.
