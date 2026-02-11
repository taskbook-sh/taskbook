# Installation

## Client (tb)

The `tb` command-line tool is all you need for local task management.

### Pre-built Binaries

Download the latest binary for your platform from [GitHub Releases](https://github.com/taskbook-sh/taskbook/releases).

### Build from Source

Requires Rust 1.70 or later.

```bash
# Clone the repository
git clone https://github.com/taskbook-sh/taskbook.git
cd taskbook

# Build release binary
cargo build --release

# The binary is at target/release/tb
# Copy it to your PATH
cp target/release/tb ~/.local/bin/
```

### Nix Flake

Add to your system flake:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    taskbook.url = "github:taskbook-sh/taskbook";
  };

  outputs = { nixpkgs, taskbook, ... }: {
    # For NixOS
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [{
        nixpkgs.overlays = [ taskbook.overlays.default ];
        environment.systemPackages = with pkgs; [ taskbook ];
      }];
    };

    # For nix-darwin (macOS)
    darwinConfigurations.myhost = darwin.lib.darwinSystem {
      modules = [{
        nixpkgs.overlays = [ taskbook.overlays.default ];
        environment.systemPackages = with pkgs; [ taskbook ];
      }];
    };

    # For home-manager
    homeConfigurations.myuser = home-manager.lib.homeManagerConfiguration {
      modules = [{
        nixpkgs.overlays = [ taskbook.overlays.default ];
        home.packages = with pkgs; [ taskbook ];
      }];
    };
  };
}
```

Or run directly:

```bash
nix run github:taskbook-sh/taskbook
```

### Cargo Install

```bash
cargo install --git https://github.com/taskbook-sh/taskbook
```

## Server (tb-server)

The server is only needed if you want to sync tasks across multiple devices. See [Server Setup](server.md) for details.

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/taskbook-sh/taskbook/releases).

### Build from Source

```bash
cargo build --release -p taskbook-server

# Binary is at target/release/tb-server
```

### Docker

```bash
# Build the server image
docker build -f Dockerfile.server -t taskbook-server .

# Or use Docker Compose for a complete setup
docker compose up -d
```

## Verify Installation

```bash
# Check version
tb --version

# Create your first task
tb --task "Hello, taskbook!"

# View your board
tb
```

## Migrating from Node.js Taskbook

If you're migrating from the original Node.js taskbook, your existing data will work automatically. The data format and directory structure (`~/.taskbook/`) are fully compatible.

Simply install taskbook and run `tb` to see your existing tasks.
