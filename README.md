# taskbook-rs

A Rust port of [taskbook](https://github.com/klaussinani/taskbook) - tasks, boards & notes for the command-line habitat.

## Installation

Download the latest binary from [releases](https://github.com/alexanderdavidsen/taskbook-rs/releases) or build from source:

```bash
cargo install --path .
```

## Usage

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

## Data Compatibility

This implementation uses the same data format and directory (`~/.taskbook/`) as the original Node.js version, allowing seamless migration.

## License

MIT - see [LICENSE](LICENSE)

## Credits

Original project by [Klaus Sinani](https://github.com/klaussinani/taskbook)
