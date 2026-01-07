# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains all Rust source. Key files: `main.rs` (CLI entry), `commands.rs` (flag routing), `taskbook.rs` (core logic), `storage.rs` (JSON persistence), `render.rs` (terminal output), and `models/` for task/note types.
- `scripts/` holds automation utilities (see release script).
- `overlay.nix`, `flake.nix`, `devenv.nix`, and `devenv.yaml` define Nix packaging and the dev environment.
- `target/` is build output; do not edit or commit artifacts.

## Build, Test, and Development Commands
Always use the Nix dev shell for Rust tooling; do not install Rust tooling globally.
- `devenv shell` opens the pinned dev environment.
- Inside `devenv shell`, run `cargo build` to build a debug binary (`target/debug/tb`).
- Inside `devenv shell`, run `cargo build --release` to build an optimized binary (`target/release/tb`).
- Inside `devenv shell`, run `cargo run -- --help` to run the CLI from source.
- Inside `devenv shell`, run `cargo test` to run the test suite (if present).
- Inside `devenv shell`, run `cargo clippy` to run lint checks.

## Coding Style & Naming Conventions
- Rust formatting uses `rustfmt`; run `cargo fmt` before pushing changes.
- Indentation is 4 spaces in Rust code.
- Naming follows standard Rust conventions: `snake_case` for functions/vars, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Run `cargo fmt` inside `devenv shell` only; avoid installing any Rust tooling globally.

## Testing Guidelines
- Tests use Rust’s built-in harness (`cargo test`).
- The app reads/writes `~/.taskbook/` for data during runtime; be cautious when running integration-style tests locally.
- When adding tests, keep module-level `#[cfg(test)]` units close to the code they verify.

## Commit & Pull Request Guidelines
- Commit messages are short, imperative, and sentence case (e.g., “Add release script”, “Bump taskbook-rs version to 0.1.1”).
- Release commits follow `Release vX.Y.Z` and are tagged `vX.Y.Z`.
- Always work on feature branches; avoid committing directly to `master`.
- PRs should describe behavior changes, include reproduction steps or CLI examples, and note any data format changes.

## Release Process
- Use `scripts/release.sh <version> [remote]` to bump `overlay.nix` and `Cargo.toml`, commit, tag, and push.
- Example: `scripts/release.sh 0.1.2` (creates tag `v0.1.2`).
- If you need to run `cargo` as part of a release check, do it inside `devenv shell` only.
