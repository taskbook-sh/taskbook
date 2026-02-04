use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, KeyEvent};

use crate::error::{Result, TaskbookError};

/// Terminal events
#[derive(Debug)]
#[allow(dead_code)]
pub enum Event {
    /// Keyboard input
    Key(KeyEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Periodic tick for UI updates
    Tick,
}

/// Global flag to pause event polling (used when launching external editor)
static EVENT_POLLING_PAUSED: AtomicBool = AtomicBool::new(false);

/// Pause the event handler (stops polling for keyboard events)
pub fn pause_event_handler() {
    EVENT_POLLING_PAUSED.store(true, Ordering::SeqCst);
    // Give the event loop time to notice the pause
    thread::sleep(Duration::from_millis(50));
}

/// Resume the event handler
pub fn resume_event_handler() {
    EVENT_POLLING_PAUSED.store(false, Ordering::SeqCst);
}

/// Event handler with background thread
pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();

        let handler = thread::spawn(move || loop {
            // Check if we should pause polling
            if EVENT_POLLING_PAUSED.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            if event::poll(tick_rate).unwrap_or(false) {
                // Double-check pause flag after poll returns
                if EVENT_POLLING_PAUSED.load(Ordering::SeqCst) {
                    continue;
                }
                match event::read() {
                    Ok(event::Event::Key(key)) => {
                        if sender.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                    Ok(event::Event::Resize(width, height)) => {
                        if sender.send(Event::Resize(width, height)).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            } else if sender.send(Event::Tick).is_err() {
                break;
            }
        });

        Self { receiver, handler }
    }

    /// Get the next event, blocking until one is available
    pub fn next(&self) -> Result<Event> {
        self.receiver
            .recv()
            .map_err(|e| TaskbookError::Tui(e.to_string()))
    }
}
