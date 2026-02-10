use std::io::BufRead;
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
    /// Remote data changed (received via SSE)
    DataChanged { archived: bool },
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
    #[allow(dead_code)]
    sse_handler: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = spawn_input_thread(sender, tick_rate);

        Self {
            receiver,
            handler,
            sse_handler: None,
        }
    }

    /// Create an event handler that also listens for SSE sync notifications.
    pub fn new_with_sse(tick_rate: u64, server_url: String, token: String) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = spawn_input_thread(sender.clone(), tick_rate);
        let sse_handler = spawn_sse_thread(sender, server_url, token);

        Self {
            receiver,
            handler,
            sse_handler: Some(sse_handler),
        }
    }

    /// Get the next event, blocking until one is available
    pub fn next(&self) -> Result<Event> {
        self.receiver
            .recv()
            .map_err(|e| TaskbookError::Tui(e.to_string()))
    }
}

fn spawn_input_thread(sender: mpsc::Sender<Event>, tick_rate: u64) -> thread::JoinHandle<()> {
    let tick_rate = Duration::from_millis(tick_rate);

    thread::spawn(move || loop {
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
    })
}

fn spawn_sse_thread(
    sender: mpsc::Sender<Event>,
    server_url: String,
    token: String,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/api/v1/events", server_url.trim_end_matches('/'));

        loop {
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "text/event-stream")
                .send();

            match resp {
                Ok(response) if response.status().is_success() => {
                    let reader = std::io::BufReader::new(response);
                    let mut current_event = String::new();
                    let mut current_data = String::new();

                    for line in reader.lines() {
                        let line = match line {
                            Ok(l) => l,
                            Err(_) => break, // Connection lost
                        };

                        if let Some(val) = line.strip_prefix("event:") {
                            current_event = val.trim().to_string();
                        } else if let Some(val) = line.strip_prefix("data:") {
                            current_data = val.trim().to_string();
                        } else if line.is_empty() && !current_event.is_empty() {
                            // End of SSE frame — dispatch event
                            if current_event == "data_changed" {
                                let archived = current_data == "archive";
                                if sender
                                    .send(Event::DataChanged { archived })
                                    .is_err()
                                {
                                    return; // TUI closed
                                }
                            }
                            current_event.clear();
                            current_data.clear();
                        }
                    }
                }
                _ => {} // Connection failed or non-success status
            }

            // Reconnect after delay; exit if TUI has closed (sender dropped)
            thread::sleep(Duration::from_secs(5));
            if sender.send(Event::Tick).is_err() {
                return;
            }
        }
    })
}
