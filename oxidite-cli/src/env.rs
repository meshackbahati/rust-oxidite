use std::path::{Path, PathBuf};
use oxidite_config::Config;

/// Load .env file from the project root (where oxidite.toml or Cargo.toml exists)
pub fn load_env() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(project_root) = find_project_root() {
        let env_path = project_root.join(".env");
        if env_path.exists() {
            dotenvy::from_path(env_path).ok();
        } else {
            dotenvy::dotenv().ok();
        }
    } else {
        dotenvy::dotenv().ok();
    }
    Ok(())
}

/// Find the project root (directory containing oxidite.toml or Cargo.toml)
pub fn find_project_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current_dir = std::env::current_dir()?;

    loop {
        if current_dir.join("oxidite.toml").exists() || current_dir.join("Cargo.toml").exists() {
            return Ok(current_dir);
        }

        if !current_dir.pop() {
            return Err("Could not find project root (oxidite.toml or Cargo.toml)".into());
        }
    }
}

/// Get DATABASE_URL with proper priority:
/// 1. Environment variable
/// 2. oxidite.toml [database].url
pub fn get_database_url() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Ok(url);
        }
    }

    if let Ok(config) = Config::load() {
        if !config.database.url.trim().is_empty() {
            return Ok(config.database.url.clone());
        }
    }

    Err("DATABASE_URL not set in .env or oxidite.toml".into())
}

/// Validate PostgreSQL URL
pub fn validate_postgres_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(())
    } else {
        Err(format!(
            "DATABASE_URL must be a PostgreSQL connection string (postgres:// or postgresql://)\nGot: {}",
            url
        ).into())
    }
}
