use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("YAML parse error: {0}")]
    YamlDe(#[from] serde_yaml::Error),
    #[error("invalid value for environment variable `{name}`: `{value}`")]
    InvalidEnvValue { name: String, value: String },
    #[error("missing configuration key: {0}")]
    MissingKey(String),
    #[error("invalid type for configuration key: {0}")]
    InvalidType(String),
}

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
    pub cors_methods: Vec<String>,
    #[serde(default)]
    pub cors_headers: Vec<String>,
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
            cors_methods: vec![],
            cors_headers: vec![],
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
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        if let Ok(val) = env::var("APP_NAME") {
            self.app.name = val;
        }
        if let Ok(val) = env::var("SERVER_HOST") {
            self.server.host = val;
        }
        if let Ok(val) = env::var("SERVER_PORT") {
            self.server.port = val
                .parse()
                .map_err(|_| ConfigError::InvalidEnvValue {
                    name: "SERVER_PORT".to_string(),
                    value: val,
                })?;
        }
        if let Ok(val) = env::var("DATABASE_URL") {
            self.database.url = val;
        }
        if let Ok(val) = env::var("REDIS_URL") {
            self.cache.redis_url = val.clone();
            self.queue.redis_url = val;
        }
        if let Ok(val) = env::var("JWT_SECRET") {
            self.security.jwt_secret = val;
        }
        Ok(())
    }

    fn has_key(&self, key: &str) -> bool {
        if self.custom.contains_key(key) {
            return true;
        }
        let root = toml::Value::try_from(self).ok();
        if let Some(root) = root {
            let mut cursor = &root;
            for part in key.split('.') {
                if let Some(next) = cursor.get(part) {
                    cursor = next;
                } else {
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn load() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        let env_val = env::var("OXIDITE_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let mut config = if Path::new("oxidite.toml").exists() {
            let content = fs::read_to_string("oxidite.toml")?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        config.apply_env_overrides()?;
        config.app.environment = env_val;
        Ok(config)
    }

    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        let env_name = env::var("OXIDITE_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let mut config = if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        config.app.environment = env_name;
        config.apply_env_overrides()?;
        Ok(config)
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        if let Some(value) = self.custom.get(key) {
            if let Ok(parsed) = T::deserialize(value.clone()) {
                return Some(parsed);
            }
        }

        let root = toml::Value::try_from(self).ok()?;
        let mut cursor = &root;
        for part in key.split('.') {
            cursor = cursor.get(part)?;
        }

        T::deserialize(cursor.clone()).ok()
    }

    pub fn get_required<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, ConfigError> {
        self.get(key).ok_or_else(|| {
            if self.has_key(key) {
                ConfigError::InvalidType(key.to_string())
            } else {
                ConfigError::MissingKey(key.to_string())
            }
        })
    }

    pub fn get_u16(&self, key: &str) -> Result<u16, ConfigError> {
        self.get_required(key)
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        self.get_required(key)
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
    }

    #[test]
    fn test_get_required_typed_values() {
        let config = Config::default();
        assert_eq!(config.get_u16("server.port").unwrap(), 3000);
    }

    #[test]
    fn test_invalid_server_port_env_returns_error() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let prev = env::var("SERVER_PORT").ok();
        env::set_var("SERVER_PORT", "not-a-port");
        let result = Config::load();
        if let Some(v) = prev { env::set_var("SERVER_PORT", v); } else { env::remove_var("SERVER_PORT"); }
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_applies_env_overrides() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let prev_host = env::var("SERVER_HOST").ok();
        env::set_var("SERVER_HOST", "0.0.0.0");
        let cfg = Config::load_from("non-existent.toml").unwrap();
        if let Some(v) = prev_host { env::set_var("SERVER_HOST", v); } else { env::remove_var("SERVER_HOST"); }
        assert_eq!(cfg.server.host, "0.0.0.0");
    }

    use std::sync::Mutex;
    static SERIAL_TEST: Mutex<()> = Mutex::new(());
}
