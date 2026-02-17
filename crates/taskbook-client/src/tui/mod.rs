mod actions;
mod app;
mod event;
mod input_handler;
mod theme;
mod ui;
pub mod widgets;

use crate::config::Config;
use crate::credentials::Credentials;
use crate::error::{Result, TaskbookError};
pub use app::App;

use std::io::{self, Write};
use std::path::Path;

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

/// Temporarily suspend the TUI to run an external command (like an editor).
/// Returns a guard that restores the terminal when dropped.
pub fn suspend_tui() -> Result<TuiSuspendGuard> {
    // First, pause the event handler thread to stop it from consuming input
    event::pause_event_handler();

    let mut stdout = io::stdout();

    // Disable mouse capture first (while still in raw mode)
    execute!(stdout, DisableMouseCapture).map_err(|e| TaskbookError::Tui(e.to_string()))?;

    // Leave alternate screen
    execute!(stdout, LeaveAlternateScreen).map_err(|e| TaskbookError::Tui(e.to_string()))?;

    // Disable raw mode
    disable_raw_mode().map_err(|e| TaskbookError::Tui(e.to_string()))?;

    // Show cursor and reset terminal state
    execute!(
        stdout,
        cursor::Show,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .map_err(|e| TaskbookError::Tui(e.to_string()))?;

    stdout.flush().ok();
    Ok(TuiSuspendGuard { _private: () })
}

/// Guard that restores TUI state when dropped
pub struct TuiSuspendGuard {
    _private: (),
}

impl TuiSuspendGuard {
    /// Explicitly resume the TUI (called automatically on drop)
    pub fn resume(self) -> Result<()> {
        self.do_resume()
    }

    fn do_resume(&self) -> Result<()> {
        let mut stdout = io::stdout();
        enable_raw_mode().map_err(|e| TaskbookError::Tui(e.to_string()))?;
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )
        .map_err(|e| TaskbookError::Tui(e.to_string()))?;
        stdout.flush().ok();

        // Drain any stale keyboard events buffered while the editor was running
        event::drain_input_buffer();

        // Resume the event handler thread
        event::resume_event_handler();

        Ok(())
    }
}

impl Drop for TuiSuspendGuard {
    fn drop(&mut self) {
        // Best effort to restore terminal on drop
        let _ = self.do_resume();
    }
}

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
    let events = create_event_handler(&app.config);

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
            event::Event::DataChanged { archived } => {
                use app::ViewMode;
                match (app.view, archived) {
                    (ViewMode::Archive, true) => {
                        app.items = app.taskbook.get_all_archive_items()?;
                        app.update_display_order();
                    }
                    (ViewMode::Board | ViewMode::Timeline | ViewMode::Journal, false) => {
                        app.refresh_items()?;
                    }
                    _ => {} // Data will be loaded when user switches views
                }
            }
        }
    }

    Ok(())
}

/// Create the appropriate event handler based on sync configuration.
fn create_event_handler(config: &Config) -> event::EventHandler {
    if config.sync.enabled {
        if let Ok(Some(creds)) = Credentials::load() {
            return event::EventHandler::new_with_sse(250, creds.server_url, creds.token);
        }
    }
    event::EventHandler::new(250)
}
