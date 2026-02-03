use std::net::IpAddr;

/// Server configuration, loaded from environment variables.
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub session_expiry_days: i64,
}

impl ServerConfig {
    pub fn load() -> Self {
        Self {
            host: std::env::var("TB_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string())
                .parse()
                .expect("TB_HOST must be a valid IP address"),
            port: std::env::var("TB_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("TB_PORT must be a valid port number"),
            database_url: std::env::var("TB_DATABASE_URL")
                .expect("TB_DATABASE_URL environment variable is required"),
            session_expiry_days: std::env::var("TB_SESSION_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("TB_SESSION_EXPIRY_DAYS must be a number"),
        }
    }
}
