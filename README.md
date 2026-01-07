# taskbook-rs

A Rust port of [taskbook](https://github.com/klaussinani/taskbook) - tasks, boards & notes for the command-line habitat.

## Installation

Download the latest binary from [releases](https://github.com/alexanderdavidsen/taskbook-rs/releases) or build from source:

```bash
cargo install --path .
```

### Nix Flake

Add to your system flake:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    taskbook-rs.url = "github:alexanderdavidsen/taskbook-rs";
  };

  outputs = { nixpkgs, taskbook-rs, ... }: {
    darwinConfigurations.myhost = darwin.lib.darwinSystem {
      modules = [{
        nixpkgs.overlays = [ taskbook-rs.overlays.default ];
        environment.systemPackages = with pkgs; [ taskbook-rs ];
      }];
    };
  };
}
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

## Configuration

Configuration is stored in `~/.taskbook.json`. Example:

```json
{
  "taskbookDirectory": "~",
  "displayCompleteTasks": true,
  "displayProgressOverview": true,
  "theme": "catppuccin-macchiato"
}
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `taskbookDirectory` | Directory for storing taskbook data | `~` (creates `~/.taskbook/`) |
| `displayCompleteTasks` | Show completed tasks in board view | `true` |
| `displayProgressOverview` | Show progress statistics | `true` |
| `theme` | Color theme (preset name or custom colors) | `"default"` |

### Themes

Available preset themes:

- `default` - Neutral gray, works on most terminals
- `catppuccin-macchiato` - [Catppuccin](https://catppuccin.com/) Macchiato variant
- `catppuccin-mocha` - Catppuccin Mocha (darkest)
- `catppuccin-frappe` - Catppuccin Frappé (medium)
- `catppuccin-latte` - Catppuccin Latte (light)
- `high-contrast` - High contrast for accessibility

#### Custom Theme

You can define custom RGB colors for each element:

```json
{
  "theme": {
    "muted": { "r": 165, "g": 173, "b": 203 },
    "success": { "r": 166, "g": 218, "b": 149 },
    "warning": { "r": 238, "g": 212, "b": 159 },
    "error": { "r": 237, "g": 135, "b": 150 },
    "info": { "r": 138, "g": 173, "b": 244 },
    "pending": { "r": 198, "g": 160, "b": 246 },
    "starred": { "r": 238, "g": 212, "b": 159 }
  }
}
```

| Color | Used For |
|-------|----------|
| `muted` | Secondary text (IDs, labels, completed tasks) |
| `success` | Checkmarks, completed counts, normal priority |
| `warning` | In-progress indicators, medium priority |
| `error` | Error messages, high priority |
| `info` | Notes, in-progress counts |
| `pending` | Pending task icons and counts |
| `starred` | Star indicators |

## Data Compatibility

This implementation uses the same data format and directory (`~/.taskbook/`) as the original Node.js version, allowing seamless migration.

## License

MIT - see [LICENSE](LICENSE)

## Credits

Original project by [Klaus Sinani](https://github.com/klaussinani/taskbook)
