use std::net::IpAddr;

/// Server configuration, loaded from environment variables.
///
/// Database connection is built from individual variables:
/// - `TB_DB_HOST` (required) - Database hostname
/// - `TB_DB_PORT` (optional, default: 5432) - Database port
/// - `TB_DB_NAME` (required) - Database name
/// - `TB_DB_USER` (required) - Database username
/// - `TB_DB_PASSWORD` (required) - Database password
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub session_expiry_days: i64,
}

impl ServerConfig {
    pub fn load() -> Self {
        let db_host =
            std::env::var("TB_DB_HOST").expect("TB_DB_HOST environment variable is required");
        let db_port = std::env::var("TB_DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let db_name =
            std::env::var("TB_DB_NAME").expect("TB_DB_NAME environment variable is required");
        let db_user =
            std::env::var("TB_DB_USER").expect("TB_DB_USER environment variable is required");
        let db_password = std::env::var("TB_DB_PASSWORD")
            .expect("TB_DB_PASSWORD environment variable is required");

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        Self {
            host: std::env::var("TB_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string())
                .parse()
                .expect("TB_HOST must be a valid IP address"),
            port: std::env::var("TB_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("TB_PORT must be a valid port number"),
            database_url,
            session_expiry_days: std::env::var("TB_SESSION_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("TB_SESSION_EXPIRY_DAYS must be a number"),
        }
    }
}
