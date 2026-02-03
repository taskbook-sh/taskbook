# Installation

## Client (tb)

The `tb` command-line tool is all you need for local task management.

### Pre-built Binaries

Download the latest binary for your platform from [GitHub Releases](https://github.com/alexanderdavidsen/taskbook-rs/releases).

### Build from Source

Requires Rust 1.70 or later.

```bash
# Clone the repository
git clone https://github.com/alexanderdavidsen/taskbook-rs.git
cd taskbook-rs

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
    taskbook-rs.url = "github:alexanderdavidsen/taskbook-rs";
  };

  outputs = { nixpkgs, taskbook-rs, ... }: {
    # For NixOS
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [{
        nixpkgs.overlays = [ taskbook-rs.overlays.default ];
        environment.systemPackages = with pkgs; [ taskbook-rs ];
      }];
    };

    # For nix-darwin (macOS)
    darwinConfigurations.myhost = darwin.lib.darwinSystem {
      modules = [{
        nixpkgs.overlays = [ taskbook-rs.overlays.default ];
        environment.systemPackages = with pkgs; [ taskbook-rs ];
      }];
    };

    # For home-manager
    homeConfigurations.myuser = home-manager.lib.homeManagerConfiguration {
      modules = [{
        nixpkgs.overlays = [ taskbook-rs.overlays.default ];
        home.packages = with pkgs; [ taskbook-rs ];
      }];
    };
  };
}
```

Or run directly:

```bash
nix run github:alexanderdavidsen/taskbook-rs
```

### Cargo Install

```bash
cargo install --git https://github.com/alexanderdavidsen/taskbook-rs
```

## Server (tb-server)

The server is only needed if you want to sync tasks across multiple devices. See [Server Setup](server.md) for details.

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/alexanderdavidsen/taskbook-rs/releases).

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

Simply install taskbook-rs and run `tb` to see your existing tasks.
