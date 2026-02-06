# CLI Reference

## Synopsis

```
tb [OPTIONS] [INPUT...]
```

## Display Commands

### Board View (default)

```bash
tb
```

Displays all items organized by board. This is the default when no options are provided.

### Timeline View

```bash
tb --timeline
tb -i
```

Displays items in chronological order by creation date.

### Archive View

```bash
tb --archive
tb -a
```

Displays archived (deleted) items.

## Creating Items

### Create Task

```bash
tb --task <description>
tb -t <description>
```

Creates a new task. Tasks can be checked off when completed.

**Board assignment**: Prefix board names with `@`:

```bash
tb --task @work "Review pull request"
tb --task @coding @review "Implement feature X"  # Multiple boards
```

**Priority**: Append `p:N` where N is 1 (normal), 2 (medium), or 3 (high):

```bash
tb --task "Fix critical bug" p:3        # High priority
tb --task @work "Update docs" p:2       # Medium priority
```

**Combined example**:

```bash
tb --task @work @urgent "Deploy hotfix" p:3
```

### Create Note

```bash
tb --note <description>
tb -n <description>
```

Creates a note. Notes are for reference and cannot be checked off.

```bash
tb --note "API endpoint: https://api.example.com"
tb --note @meetings "Standup at 9am daily"
```

## Modifying Items

### Check/Uncheck Task

```bash
tb --check <id> [id...]
tb -c <id> [id...]
```

Toggles the completion status of tasks. Multiple IDs can be specified.

```bash
tb --check 1
tb --check 1 2 3
```

### Begin/Pause Task

```bash
tb --begin <id> [id...]
tb -b <id> [id...]
```

Toggles the in-progress status of tasks. In-progress tasks are highlighted differently in the board view.

```bash
tb --begin 2
tb --begin 4 5
```

### Star/Unstar Item

```bash
tb --star <id> [id...]
tb -s <id> [id...]
```

Toggles the starred status. Works on both tasks and notes.

```bash
tb --star 1
tb --star 1 3 5
```

### Edit Description

```bash
tb --edit @<id> <new description>
tb -e @<id> <new description>
```

Edits an item's description. The ID must be prefixed with `@`.

```bash
tb --edit @3 "Updated task description"
```

### Set Priority

```bash
tb --priority @<id> <1-3>
tb -p @<id> <1-3>
```

Sets task priority: 1 = normal, 2 = medium, 3 = high.

```bash
tb --priority @5 3    # Set to high priority
tb --priority @5 1    # Set to normal priority
```

### Move to Board

```bash
tb --move @<id> <board>
tb -m @<id> <board>
```

Moves an item to a different board. Opens a board picker if the board name is omitted.

```bash
tb --move @3 work
tb --move @3 @personal   # @ prefix is optional for board name
```

## Deleting and Restoring

### Delete Item

```bash
tb --delete <id> [id...]
tb -d <id> [id...]
```

Moves items to the archive. They can be restored later.

```bash
tb --delete 4
tb --delete 1 2 3
```

### Restore from Archive

```bash
tb --restore <id> [id...]
tb -r <id> [id...]
```

Restores archived items back to the active board.

```bash
tb --restore 4
tb --restore 10 11 12
```

### Clear Completed

```bash
tb --clear
```

Permanently deletes all completed (checked) tasks from all boards.

## Searching and Filtering

### Find Items

```bash
tb --find <search term>
tb -f <search term>
```

Searches for items containing the search term in their description.

```bash
tb --find documentation
tb --find "pull request"
```

### List by Attributes

```bash
tb --list <attributes>
tb -l <attributes>
```

Filters items by one or more attributes:

| Attribute | Description |
|-----------|-------------|
| `pending` | Incomplete tasks |
| `done` | Completed tasks |
| `task` | All tasks |
| `note` | All notes |
| `starred` | Starred items |

```bash
tb --list pending
tb --list done starred
tb --list task pending    # Pending tasks only
```

## Clipboard

### Copy to Clipboard

```bash
tb --copy <id> [id...]
tb -y <id> [id...]
```

Copies item descriptions to the system clipboard. Multiple items are joined with newlines.

```bash
tb --copy 1
tb --copy 1 2 3
```

## Server Commands

These commands are used for syncing with a remote server. See [Sync & Encryption](sync.md) for details.

### Register Account

```bash
tb --register [OPTIONS]
```

Creates a new account on the sync server. Prompts interactively for missing values.

| Option | Description |
|--------|-------------|
| `--server <url>` | Server URL |
| `--username <name>` | Username |
| `--email <email>` | Email address |
| `--password <pass>` | Password (prompted securely if omitted) |

```bash
# Fully interactive (recommended)
tb --register

# Partial - prompts for password
tb --register --server https://taskbook.example.com --username alice --email alice@example.com
```

### Login

```bash
tb --login [OPTIONS]
```

Logs in to an existing account. Prompts interactively for missing values.

| Option | Description |
|--------|-------------|
| `--server <url>` | Server URL |
| `--username <name>` | Username |
| `--password <pass>` | Password (prompted securely if omitted) |
| `--key <base64>` | Encryption key from registration |

```bash
# Fully interactive
tb --login

# With server specified
tb --login --server https://taskbook.example.com
```

### Logout

```bash
tb --logout
```

Logs out and deletes stored credentials. Sync is disabled and the client returns to local-only mode.

### Status

```bash
tb --status
```

Shows current sync status, server URL, and whether credentials are saved.

### Migrate Local Data

```bash
tb --migrate
```

Pushes existing local data to the server. Use this after registering to upload your existing tasks.

## Global Options

### Custom Taskbook Directory

```bash
tb --taskbook-dir <path>
```

Use a custom directory for storing taskbook data instead of `~/.taskbook/`.

```bash
tb --taskbook-dir ~/work-tasks --task "Work item"
```

Can also be set via environment variable:

```bash
export TASKBOOK_DIR=~/work-tasks
tb --task "Work item"
```

### CLI Mode

```bash
tb --cli
```

Runs in non-interactive CLI mode (disables TUI features).

### Help

```bash
tb --help
tb -h
```

Displays help message with all available options.

### Version

```bash
tb --version
tb -v
```

Displays the installed version.

## Examples

```bash
# Daily workflow
tb                                    # View boards
tb --task @work "Code review"         # Create task
tb --begin 1                          # Start working
tb --check 1                          # Mark done

# Organize tasks
tb --task @personal "Buy groceries" p:2
tb --star 3
tb --move @3 shopping

# Search and filter
tb --find meeting
tb --list pending starred
tb --timeline

# Cleanup
tb --delete 5 6 7
tb --clear
tb --archive                          # View deleted items
tb --restore 5                        # Oops, restore one

# Server sync
tb --register
tb --migrate
tb --status
```
