use std::io::{self, Write};

use base64::Engine;
use colored::Colorize;

use crate::api_client::{ApiClient, LoginRequest, RegisterRequest};
use crate::config::Config;
use crate::credentials::Credentials;
use crate::error::{Result, TaskbookError};

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout()
        .flush()
        .map_err(|e| TaskbookError::General(format!("failed to flush stdout: {e}")))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| TaskbookError::General(format!("failed to read input: {e}")))?;
    Ok(input.trim().to_string())
}

fn prompt_password(message: &str) -> Result<String> {
    rpassword::prompt_password(message)
        .map_err(|e| TaskbookError::General(format!("failed to read password: {e}")))
}

/// Compare two server URLs using the same trailing-slash normalization as
/// `ApiClient::new`.
fn same_server_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// Resolve the server URL for auth commands: an explicit `--server` value
/// wins; otherwise fall back to the configured sync server when sync is
/// already enabled.
fn configured_server_url(flag: Option<&str>, config: &Config) -> Option<String> {
    match flag {
        Some(s) => Some(s.to_string()),
        None if config.sync.enabled => Some(config.sync.server_url.clone()),
        None => None,
    }
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

    let mut config = Config::load_or_default();
    let server = match configured_server_url(server_url, &config) {
        Some(s) => {
            if server_url.is_none() {
                println!("Using server: {}", s);
            }
            s
        }
        None => prompt("Server URL: ")?,
    };

    let user = match username {
        Some(u) => u.to_string(),
        None => prompt("Username: ")?,
    };

    let mail = match email {
        Some(e) => e.to_string(),
        None => prompt("Email: ")?,
    };

    let pass = match password {
        Some(p) => p.to_string(),
        None => {
            let p1 = prompt_password("Password: ")?;
            let p2 = prompt_password("Confirm password: ")?;
            if p1 != p2 {
                return Err(TaskbookError::Auth("passwords do not match".to_string()));
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

    let mut config = Config::load_or_default();
    let server = match configured_server_url(server_url, &config) {
        Some(s) => {
            if server_url.is_none() {
                println!("Using server: {}", s);
            }
            s
        }
        None => prompt("Server URL: ")?,
    };

    let user = match username {
        Some(u) => u.to_string(),
        None => prompt("Username: ")?,
    };

    let pass = match password {
        Some(p) => p.to_string(),
        None => prompt_password("Password: ")?,
    };

    let key = match encryption_key {
        Some(k) => k.to_string(),
        None => prompt("Encryption key: ")?,
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
    let mut config = Config::load_or_default();
    config.disable_sync()?;

    println!("{}", "Logged out.".green());
    println!("{}", "Sync disabled, using local storage.".dimmed());

    Ok(())
}

/// Show current sync status.
pub fn status() -> Result<()> {
    let config = Config::load_or_default();

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

            if config.sync.enabled {
                if !same_server_url(&creds.server_url, &config.sync.server_url) {
                    println!(
                        "{}",
                        format!(
                            "Warning: credentials were saved for {} (config points to {})",
                            creds.server_url, config.sync.server_url
                        )
                        .yellow()
                    );
                    // A bearer token issued by one server must not be sent
                    // to another; skip validation and require re-auth.
                    println!("Session:     {}", "not checked".yellow());
                    println!(
                        "{}",
                        "Hint: run `tb --login` to authenticate against the configured server."
                            .dimmed()
                    );
                    return Ok(());
                }
                // Validate the session against the configured server (user's
                // intent), not a potentially stale credentials URL.
                let client = ApiClient::new(&config.sync.server_url, Some(&creds.token));
                match client.get_me() {
                    Ok(me) => {
                        println!("Session:     {}", "valid".green());
                        println!("Logged in as: {} ({})", me.username, me.email);
                    }
                    Err(e) => {
                        println!("Session:     {}", "invalid".red());
                        println!("Error:       {}", e);
                        println!("{}", "Hint: run `tb --login` to re-authenticate.".dimmed());
                    }
                }
            }
        }
        None => {
            println!("Credentials: {}", "none".dimmed());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_sync(enabled: bool, server_url: &str) -> Config {
        let mut config = Config::default();
        config.sync.enabled = enabled;
        config.sync.server_url = server_url.to_string();
        config
    }

    #[test]
    fn flag_wins_over_configured_server() {
        let config = config_with_sync(true, "http://configured.example");
        assert_eq!(
            configured_server_url(Some("http://flag.example"), &config),
            Some("http://flag.example".to_string())
        );
    }

    #[test]
    fn configured_server_used_when_sync_enabled() {
        let config = config_with_sync(true, "http://configured.example");
        assert_eq!(
            configured_server_url(None, &config),
            Some("http://configured.example".to_string())
        );
    }

    #[test]
    fn no_fallback_when_sync_disabled() {
        let config = config_with_sync(false, "http://configured.example");
        assert_eq!(configured_server_url(None, &config), None);
    }

    #[test]
    fn same_server_url_ignores_trailing_slash() {
        assert!(same_server_url("http://a.example", "http://a.example/"));
        assert!(same_server_url("http://a.example/", "http://a.example"));
        assert!(!same_server_url("http://a.example", "http://b.example"));
    }
}
