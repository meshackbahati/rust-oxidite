use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Development,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub queue: QueueConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub custom: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub environment: String,
    #[serde(default)]
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default)]
    pub ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub redis_url: String,
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub redis_url: String,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry: u64,
    #[serde(default)]
    pub cors_origins: Vec<String>,
    #[serde(default)]
    pub rate_limit: u32,
}

// Default functions
fn default_app_name() -> String {
    "oxidite-app".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_pool_size() -> u32 {
    10
}

fn default_ttl() -> u64 {
    3600
}

fn default_workers() -> usize {
    4
}

fn default_jwt_expiry() -> u64 {
    900 // 15 minutes
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            debug: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: num_cpus::get(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            pool_size: default_pool_size(),
            ssl: false,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            driver: "memory".to_string(),
            redis_url: String::new(),
            default_ttl: default_ttl(),
        }
    }
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            driver: "memory".to_string(),
            redis_url: String::new(),
            workers: default_workers(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            jwt_expiry: default_jwt_expiry(),
            cors_origins: vec![],
            rate_limit: 0,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            queue: QueueConfig::default(),
            security: SecurityConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        let _ = dotenv::dotenv();

        let env = env::var("OXIDITE_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        // Try to load oxidite.toml
        let mut config = if Path::new("oxidite.toml").exists() {
            let content = fs::read_to_string("oxidite.toml")?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Override with environment variables
        if let Ok(val) = env::var("APP_NAME") {
            config.app.name = val;
        }
        if let Ok(val) = env::var("SERVER_HOST") {
            config.server.host = val;
        }
        if let Ok(val) = env::var("SERVER_PORT") {
            config.server.port = val.parse().unwrap_or(default_port());
        }
        if let Ok(val) = env::var("DATABASE_URL") {
            config.database.url = val;
        }
        if let Ok(val) = env::var("REDIS_URL") {
            config.cache.redis_url = val.clone();
            config.queue.redis_url = val;
        }
        if let Ok(val) = env::var("JWT_SECRET") {
            config.security.jwt_secret = val;
        }

        config.app.environment = env;

        Ok(config)
    }

    /// Get value from custom configuration
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.custom.get(key).and_then(|v| T::deserialize(v.clone()).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_environment_parsing() {
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("PROD"), Environment::Production);
        assert_eq!(Environment::from_str("development"), Environment::Development);
    }
}
