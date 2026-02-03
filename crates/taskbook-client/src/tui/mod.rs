mod actions;
mod app;
mod event;
mod input_handler;
mod theme;
mod ui;
pub mod widgets;

use crate::error::{Result, TaskbookError};
pub use app::App;

use std::io;
use std::path::Path;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

/// Run the TUI application
pub fn run(taskbook_dir: Option<&Path>) -> Result<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| TaskbookError::Tui(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| TaskbookError::Tui(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| TaskbookError::Tui(e.to_string()))?;

    // Create app and run
    let mut app = App::new(taskbook_dir)?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode().map_err(|e| TaskbookError::Tui(e.to_string()))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| TaskbookError::Tui(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| TaskbookError::Tui(e.to_string()))?;

    res
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let events = event::EventHandler::new(250);

    while app.running {
        terminal
            .draw(|f| ui::render(f, app))
            .map_err(|e| TaskbookError::Tui(e.to_string()))?;

        match events.next()? {
            event::Event::Key(key) => {
                actions::handle_key_event(app, key)?;
            }
            event::Event::Tick => {
                app.tick();
            }
            event::Event::Resize(_, _) => {}
        }
    }

    Ok(())
}
