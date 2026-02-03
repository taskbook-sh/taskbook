use std::io::{self, Write};

use base64::Engine;
use colored::Colorize;

use crate::api_client::{ApiClient, LoginRequest, RegisterRequest};
use crate::config::Config;
use crate::credentials::Credentials;
use crate::error::Result;

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_password(message: &str) -> String {
    rpassword::prompt_password(message).unwrap_or_default()
}

/// Register a new account on the server (interactive).
pub fn register(
    server_url: Option<&str>,
    username: Option<&str>,
    email: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    println!("{}", "Register new account".bold());
    println!();

    let server = match server_url {
        Some(s) => s.to_string(),
        None => prompt("Server URL: "),
    };

    let user = match username {
        Some(u) => u.to_string(),
        None => prompt("Username: "),
    };

    let mail = match email {
        Some(e) => e.to_string(),
        None => prompt("Email: "),
    };

    let pass = match password {
        Some(p) => p.to_string(),
        None => {
            let p1 = prompt_password("Password: ");
            let p2 = prompt_password("Confirm password: ");
            if p1 != p2 {
                eprintln!("{}", "Passwords do not match".red());
                std::process::exit(1);
            }
            p1
        }
    };

    let client = ApiClient::new(&server, None);

    let resp = client.register(&RegisterRequest {
        username: user,
        email: mail,
        password: pass,
    })?;

    // Generate encryption key locally
    let key = taskbook_common::encryption::generate_key();
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);

    // Save credentials
    let creds = Credentials {
        server_url: server.clone(),
        token: resp.token,
        encryption_key: key_b64.clone(),
    };
    creds.save()?;

    // Enable sync in config
    let mut config = Config::load().unwrap_or_default();
    config.enable_sync(&server)?;

    println!();
    println!("{}", "Registration successful!".green().bold());
    println!("{}", "Sync is now enabled.".green());
    println!();
    println!(
        "{}",
        "Your encryption key (save this — it cannot be recovered):".yellow()
    );
    println!();
    println!("  {}", key_b64.bright_white().bold());
    println!();

    Ok(())
}

/// Log in to an existing account (interactive).
pub fn login(
    server_url: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    encryption_key: Option<&str>,
) -> Result<()> {
    println!("{}", "Login".bold());
    println!();

    let server = match server_url {
        Some(s) => s.to_string(),
        None => prompt("Server URL: "),
    };

    let user = match username {
        Some(u) => u.to_string(),
        None => prompt("Username: "),
    };

    let pass = match password {
        Some(p) => p.to_string(),
        None => prompt_password("Password: "),
    };

    let key = match encryption_key {
        Some(k) => k.to_string(),
        None => prompt("Encryption key: "),
    };

    let client = ApiClient::new(&server, None);

    let resp = client.login(&LoginRequest {
        username: user,
        password: pass,
    })?;

    let creds = Credentials {
        server_url: server.clone(),
        token: resp.token,
        encryption_key: key,
    };
    creds.save()?;

    // Enable sync in config
    let mut config = Config::load().unwrap_or_default();
    config.enable_sync(&server)?;

    println!();
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
