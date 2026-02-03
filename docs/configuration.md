# Configuration

Client configuration is stored in `~/.taskbook.json`. The file is created automatically with default values on first run.

## Configuration File

```json
{
  "taskbookDirectory": "~",
  "displayCompleteTasks": true,
  "displayProgressOverview": true,
  "theme": "default",
  "sync": {
    "enabled": false,
    "serverUrl": "http://localhost:8080"
  }
}
```

## Options

### taskbookDirectory

**Type**: `string`
**Default**: `"~"`

Directory where taskbook stores its data. The actual storage location is `<taskbookDirectory>/.taskbook/`.

```json
{
  "taskbookDirectory": "~"
}
```

Results in data stored at `~/.taskbook/storage/storage.json`.

You can also specify this directory:
- Via CLI: `tb --taskbook-dir /path/to/dir`
- Via environment variable: `TASKBOOK_DIR=/path/to/dir`

Priority order (highest to lowest):
1. `--taskbook-dir` CLI flag
2. `TASKBOOK_DIR` environment variable
3. `taskbookDirectory` in config file
4. Default (`~`)

### displayCompleteTasks

**Type**: `boolean`
**Default**: `true`

Whether to show completed tasks in the board view.

```json
{
  "displayCompleteTasks": false
}
```

When `false`, completed tasks are hidden from the board view but still exist and can be seen with `tb --list done`.

### displayProgressOverview

**Type**: `boolean`
**Default**: `true`

Whether to show the progress statistics at the bottom of the board view.

```json
{
  "displayProgressOverview": false
}
```

### theme

**Type**: `string` or `object`
**Default**: `"default"`

Color theme for the terminal output. Can be a preset name or custom color definitions.

#### Preset Themes

```json
{
  "theme": "catppuccin-macchiato"
}
```

Available presets:

| Theme | Description |
|-------|-------------|
| `default` | Neutral colors, works on most terminals |
| `catppuccin-macchiato` | [Catppuccin](https://catppuccin.com/) Macchiato variant |
| `catppuccin-mocha` | Catppuccin Mocha (darkest) |
| `catppuccin-frappe` | Catppuccin Frappé |
| `catppuccin-latte` | Catppuccin Latte (light theme) |
| `high-contrast` | High contrast for accessibility |

#### Custom Theme

Define custom RGB colors for each element:

```json
{
  "theme": {
    "muted": { "r": 140, "g": 140, "b": 140 },
    "success": { "r": 134, "g": 239, "b": 172 },
    "warning": { "r": 253, "g": 224, "b": 71 },
    "error": { "r": 252, "g": 129, "b": 129 },
    "info": { "r": 147, "g": 197, "b": 253 },
    "pending": { "r": 216, "g": 180, "b": 254 },
    "starred": { "r": 253, "g": 224, "b": 71 }
  }
}
```

| Color | Used For |
|-------|----------|
| `muted` | Secondary text (IDs, labels, completed task text) |
| `success` | Checkmarks, completed counts, normal priority |
| `warning` | In-progress indicators, medium priority |
| `error` | Error messages, high priority |
| `info` | Notes, information text |
| `pending` | Pending task icons and counts |
| `starred` | Star indicators |

### sync

**Type**: `object`
**Default**: `{ "enabled": false, "serverUrl": "http://localhost:8080" }`

Configuration for server sync. This is typically managed automatically by the `--register` and `--login` commands.

```json
{
  "sync": {
    "enabled": true,
    "serverUrl": "https://taskbook.example.com"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | `boolean` | Whether sync is active |
| `serverUrl` | `string` | URL of the sync server |

When `enabled` is `true`, all task operations are synced to the server. The client stores encrypted data locally as a cache and syncs with the server on each operation.

See [Sync & Encryption](sync.md) for setup instructions.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TASKBOOK_DIR` | Override taskbook data directory |

## Data Storage

```
~/.taskbook/
├── storage/
│   └── storage.json      # Active items (JSON)
├── archive/
│   └── archive.json      # Archived items (JSON)
└── credentials.json      # Server credentials (when using sync)
```

### storage.json Format

Items are stored as a JSON object with string IDs as keys:

```json
{
  "1": {
    "_id": 1,
    "_date": "Mon Jan 06 2025",
    "_timestamp": 1736193600000,
    "_description": "Task description",
    "_isStarred": false,
    "_boards": ["My Board"],
    "_isTask": true,
    "isComplete": false,
    "inProgress": false,
    "priority": 1
  }
}
```

This format is compatible with the original Node.js taskbook for easy migration.

## Example Configurations

### Minimal

```json
{}
```

All defaults are used.

### Work Setup

```json
{
  "taskbookDirectory": "~/work",
  "displayCompleteTasks": false,
  "theme": "catppuccin-mocha"
}
```

### With Server Sync

```json
{
  "taskbookDirectory": "~",
  "displayCompleteTasks": true,
  "displayProgressOverview": true,
  "theme": "catppuccin-macchiato",
  "sync": {
    "enabled": true,
    "serverUrl": "https://taskbook.example.com"
  }
}
```

### High Contrast

```json
{
  "theme": "high-contrast",
  "displayProgressOverview": true
}
```
