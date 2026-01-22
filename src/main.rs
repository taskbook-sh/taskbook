use std::path::PathBuf;
use std::process;

use clap::Parser;

mod commands;
mod config;
mod directory;
mod error;
mod models;
mod render;
mod storage;
mod taskbook;
mod tui;

const HELP_TEXT: &str = r#"
  Usage
    $ tb [<options> ...]

    Options
        none             Display board view
      --archive, -a      Display archived items
      --begin, -b        Start/pause task
      --check, -c        Check/uncheck task
      --clear            Delete all checked items
      --copy, -y         Copy item description
      --delete, -d       Delete item
      --edit, -e         Edit item description
      --find, -f         Search for items
      --help, -h         Display help message
      --list, -l         List items by attributes
      --move, -m         Move item between boards
      --note, -n         Create note
      --priority, -p     Update priority of task
      --restore, -r      Restore items from archive
      --star, -s         Star/unstar item
      --taskbook-dir     Define a custom taskbook directory
      --task, -t         Create task
      --timeline, -i     Display timeline view
      --version, -v      Display installed version

    Examples
      $ tb
      $ tb --archive
      $ tb --begin 2 3
      $ tb --check 1 2
      $ tb --clear
      $ tb --copy 1 2 3
      $ tb --delete 4
      $ tb --edit @3 Merge PR #42
      $ tb --find documentation
      $ tb --list pending coding
      $ tb --move @1 cooking
      $ tb --note @coding Mergesort worse-case O(nlogn)
      $ tb --priority @3 2
      $ tb --restore 4
      $ tb --star 2
      $ tb --task @coding @reviews Review PR #42
      $ tb --task @coding Improve documentation
      $ tb --task Make some buttercream
      $ tb --timeline
"#;

#[derive(Parser)]
#[command(
    name = "tb",
    version = env!("CARGO_PKG_VERSION"),
    about = "Tasks, boards & notes for the command-line habitat",
    after_help = HELP_TEXT
)]
struct Cli {
    /// Input arguments (task description, IDs, search terms, etc.)
    #[arg(trailing_var_arg = true)]
    input: Vec<String>,

    /// Display archived items
    #[arg(short = 'a', long)]
    archive: bool,

    /// Start/pause task
    #[arg(short = 'b', long)]
    begin: bool,

    /// Check/uncheck task
    #[arg(short = 'c', long)]
    check: bool,

    /// Delete all checked items
    #[arg(long)]
    clear: bool,

    /// Copy item description to clipboard
    #[arg(short = 'y', long)]
    copy: bool,

    /// Delete item
    #[arg(short = 'd', long)]
    delete: bool,

    /// Edit item description
    #[arg(short = 'e', long)]
    edit: bool,

    /// Search for items
    #[arg(short = 'f', long)]
    find: bool,

    /// List items by attributes
    #[arg(short = 'l', long)]
    list: bool,

    /// Move item between boards
    #[arg(short = 'm', long)]
    r#move: bool,

    /// Create note
    #[arg(short = 'n', long)]
    note: bool,

    /// Update priority of task
    #[arg(short = 'p', long)]
    priority: bool,

    /// Restore items from archive
    #[arg(short = 'r', long)]
    restore: bool,

    /// Star/unstar item
    #[arg(short = 's', long)]
    star: bool,

    /// Create task
    #[arg(short = 't', long)]
    task: bool,

    /// Display timeline view
    #[arg(short = 'i', long)]
    timeline: bool,

    /// Define a custom taskbook directory
    #[arg(long = "taskbook-dir", value_name = "PATH")]
    taskbook_dir: Option<PathBuf>,

    /// Run in CLI mode (non-interactive)
    #[arg(long)]
    cli: bool,
}

fn main() {
    let cli = Cli::parse();

    // Determine if we should run TUI or CLI mode
    let has_action_flags = cli.archive
        || cli.task
        || cli.note
        || cli.check
        || cli.begin
        || cli.star
        || cli.delete
        || cli.restore
        || cli.edit
        || cli.r#move
        || cli.priority
        || cli.copy
        || cli.find
        || cli.list
        || cli.clear
        || cli.timeline;

    // Run TUI if: no action flags, no CLI flag, and no input
    let run_tui = !cli.cli && !has_action_flags && cli.input.is_empty();

    if run_tui {
        // Run interactive TUI
        if let Err(e) = tui::run(cli.taskbook_dir.as_deref()) {
            eprintln!("TUI error: {}", e);
            process::exit(1);
        }
    } else {
        // Run CLI mode
        let result = commands::run(
            cli.input,
            cli.archive,
            cli.task,
            cli.restore,
            cli.note,
            cli.delete,
            cli.check,
            cli.begin,
            cli.star,
            cli.priority,
            cli.copy,
            cli.timeline,
            cli.find,
            cli.list,
            cli.edit,
            cli.r#move,
            cli.clear,
            cli.taskbook_dir,
        );

        if let Err(e) = result {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
