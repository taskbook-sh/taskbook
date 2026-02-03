use base64::Engine;
use colored::Colorize;

use crate::api_client::{ApiClient, LoginRequest, RegisterRequest};
use crate::config::Config;
use crate::credentials::Credentials;
use crate::error::Result;

/// Register a new account on the server.
pub fn register(server_url: &str, username: &str, email: &str, password: &str) -> Result<()> {
    let client = ApiClient::new(server_url, None);

    let resp = client.register(&RegisterRequest {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
    })?;

    // Generate encryption key locally
    let key = taskbook_common::encryption::generate_key();
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);

    // Save credentials
    let creds = Credentials {
        server_url: server_url.to_string(),
        token: resp.token,
        encryption_key: key_b64.clone(),
    };
    creds.save()?;

    // Enable sync in config
    let mut config = Config::load().unwrap_or_default();
    config.enable_sync(server_url)?;

    println!("{}", "Registration successful!".green().bold());
    println!("{}", "Sync is now enabled.".green());
    println!();
    println!(
        "{}",
        "Your encryption key (save this — it cannot be recovered):".yellow()
    );
    println!("  {}", key_b64.bright_white().bold());

    Ok(())
}

/// Log in to an existing account.
pub fn login(server_url: &str, username: &str, password: &str, encryption_key: &str) -> Result<()> {
    let client = ApiClient::new(server_url, None);

    let resp = client.login(&LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    })?;

    let creds = Credentials {
        server_url: server_url.to_string(),
        token: resp.token,
        encryption_key: encryption_key.to_string(),
    };
    creds.save()?;

    // Enable sync in config
    let mut config = Config::load().unwrap_or_default();
    config.enable_sync(server_url)?;

    println!("{}", "Login successful!".green().bold());
    println!("{}", "Sync is now enabled.".green());

    Ok(())
}

/// Log out and delete credentials.
pub fn logout() -> Result<()> {
    if let Some(creds) = Credentials::load()? {
        let client = ApiClient::new(&creds.server_url, Some(&creds.token));
        // Best-effort server logout
        let _ = client.logout();
    }

    Credentials::delete()?;

    // Disable sync in config
    let mut config = Config::load().unwrap_or_default();
    config.disable_sync()?;

    println!("{}", "Logged out.".green());
    println!("{}", "Sync disabled, using local storage.".dimmed());

    Ok(())
}

/// Show current sync status.
pub fn status() -> Result<()> {
    let config = Config::load().unwrap_or_default();

    if config.sync.enabled {
        println!("Mode:   {}", "remote".green().bold());
        println!("Server: {}", config.sync.server_url);
    } else {
        println!("Mode:   {}", "local".yellow().bold());
    }

    match Credentials::load()? {
        Some(creds) => {
            println!("Credentials: {}", "saved".green());
            println!("Server URL:  {}", creds.server_url);
        }
        None => {
            println!("Credentials: {}", "none".dimmed());
        }
    }

    Ok(())
}
