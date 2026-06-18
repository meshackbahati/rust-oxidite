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
    /// Custom environment variables defined in `[env]` of `oxidite.toml`.
    /// These are injected into the process environment at load time so that
    /// `env::var("WATU_API_KEY")` and similar calls work.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Any unknown root-level TOML tables are captured here via `#[serde(flatten)]`.
    ///
    /// This enables **namespaced environment variables**: a table like `[google]`
    /// with `client_id = "abc"` is injected as `GOOGLE_CLIENT_ID=abc`.
    /// Nested tables (`[google.oauth]`) flatten recursively (`GOOGLE_OAUTH_CLIENT_ID`).
    ///
    /// Also usable for arbitrary custom config via `config.get("table.key")`.
    #[serde(flatten, default)]
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
            env: HashMap::new(),
            custom: HashMap::new(),
        }
    }
}

impl Config {
    /// Inject `[env]` entries and namespaced tables into the process environment.
    ///
    /// **Resolution order** (highest to lowest priority):
    /// 1. Real OS environment variables
    /// 2. `.env` file entries (loaded earlier via `dotenv`)
    /// 3. `[env]` flat table entries
    /// 4. Namespaced tables (`[google]`, `[platform]`, etc.)
    ///
    /// Namespaced tables are flattened recursively:
    /// - `[google]` with `client_id = "x"` → `GOOGLE_CLIENT_ID=x`
    /// - `[google.oauth]` with `client_id = "x"` → `GOOGLE_OAUTH_CLIENT_ID=x`
    ///
    /// A variable is only set if it is not already defined (or is empty) in the
    /// OS environment, so real env vars and `.env` entries always take precedence.
    fn inject_env_vars(&self) {
        // Flat [env] table
        for (key, value) in &self.env {
            let already_set = env::var(key)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            if !already_set {
                env::set_var(key, value);
            }
        }

        // Namespaced tables from flattened custom (unknown root-level TOML tables)
        for (prefix, value) in &self.custom {
            Self::inject_namespaced_env(prefix, value);
        }
    }

    /// Recursively flatten a TOML value into environment variables.
    ///
    /// - A table like `[google]` with `client_id = "abc"` produces `GOOGLE_CLIENT_ID=abc`.
    /// - A nested table like `[google.oauth]` with `client_id = "abc"` produces
    ///   `GOOGLE_OAUTH_CLIENT_ID=abc`.
    /// - Non-table values (strings, integers, booleans) are converted to strings and set.
    /// - Existing (non-empty) OS env vars are never overwritten.
    fn inject_namespaced_env(prefix: &str, value: &toml::Value) {
        let upper_prefix = prefix.to_uppercase();
        match value {
            toml::Value::Table(table) => {
                for (key, val) in table {
                    let env_key = format!("{}_{}", upper_prefix, key.to_uppercase());
                    Self::inject_namespaced_env(&env_key, val);
                }
            }
            _ => {
                let already_set = env::var(prefix)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                if !already_set {
                    let s = match value {
                        toml::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    env::set_var(prefix, s);
                }
            }
        }
    }

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
        // Check flattened custom table first (e.g., "google.client_id", "platform.name")
        {
            let mut parts = key.split('.');
            if let Some(first) = parts.next() {
                if let Some(val) = self.custom.get(first) {
                    let mut cur = val;
                    let mut found = true;
                    for part in parts {
                        if let Some(next) = cur.get(part) {
                            cur = next;
                        } else {
                            found = false;
                            break;
                        }
                    }
                    if found {
                        return true;
                    }
                }
            }
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
        // Load .env only if OXIDITE_SKIP_DOTENV is not set
        if env::var("OXIDITE_SKIP_DOTENV").is_err() {
            let _ = dotenv::dotenv();
        }

        let env_val = env::var("OXIDITE_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let mut config = if Path::new("oxidite.toml").exists() {
            let content = fs::read_to_string("oxidite.toml")?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Inject [env] vars from oxidite.toml into the process environment.
        // Real OS env vars take precedence, so inject_env_vars only sets
        // variables that are not already defined.
        config.inject_env_vars();

        config.apply_env_overrides()?;
        config.app.environment = env_val;
        Ok(config)
    }

    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        // Load .env only if OXIDITE_SKIP_DOTENV is not set
        if env::var("OXIDITE_SKIP_DOTENV").is_err() {
            let _ = dotenv::dotenv();
        }

        let env_name = env::var("OXIDITE_ENV")
            .or_else(|_| env::var("ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let mut config = if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Inject [env] vars from oxidite.toml into the process environment
        config.inject_env_vars();

        config.app.environment = env_name;
        config.apply_env_overrides()?;
        Ok(config)
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        // Check flattened custom table first (e.g., "google.client_id", "platform.name")
        {
            let mut parts = key.split('.');
            if let Some(first) = parts.next() {
                if let Some(val) = self.custom.get(first) {
                    let mut cursor = val;
                    let mut found = true;
                    for part in parts {
                        if let Some(next) = cursor.get(part) {
                            cursor = next;
                        } else {
                            found = false;
                            break;
                        }
                    }
                    if found {
                        if let Ok(parsed) = T::deserialize(cursor.clone()) {
                            return Some(parsed);
                        }
                    }
                }
            }
        }

        // Fall back to known config fields via serialization round-trip
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

    #[test]
    fn test_flat_env_table_injection() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [env]
            FLAT_TEST_VAR = "flat_value"
        "#;
        let prev = env::var("FLAT_TEST_VAR").ok();
        env::remove_var("FLAT_TEST_VAR");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("FLAT_TEST_VAR").unwrap(), "flat_value");

        if let Some(v) = prev { env::set_var("FLAT_TEST_VAR", v); } else { env::remove_var("FLAT_TEST_VAR"); }
    }

    #[test]
    fn test_namespaced_env_injection() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [google]
            client_id = "g-123"
            client_secret = "g-secret"
        "#;
        let prev_id = env::var("GOOGLE_CLIENT_ID").ok();
        let prev_secret = env::var("GOOGLE_CLIENT_SECRET").ok();
        env::remove_var("GOOGLE_CLIENT_ID");
        env::remove_var("GOOGLE_CLIENT_SECRET");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("GOOGLE_CLIENT_ID").unwrap(), "g-123");
        assert_eq!(env::var("GOOGLE_CLIENT_SECRET").unwrap(), "g-secret");

        if let Some(v) = prev_id { env::set_var("GOOGLE_CLIENT_ID", v); } else { env::remove_var("GOOGLE_CLIENT_ID"); }
        if let Some(v) = prev_secret { env::set_var("GOOGLE_CLIENT_SECRET", v); } else { env::remove_var("GOOGLE_CLIENT_SECRET"); }
    }

    #[test]
    fn test_nested_namespaced_env_injection() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [google.oauth]
            client_id = "nested-123"
            client_secret = "nested-secret"
        "#;
        let prev_id = env::var("GOOGLE_OAUTH_CLIENT_ID").ok();
        let prev_secret = env::var("GOOGLE_OAUTH_CLIENT_SECRET").ok();
        env::remove_var("GOOGLE_OAUTH_CLIENT_ID");
        env::remove_var("GOOGLE_OAUTH_CLIENT_SECRET");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap(), "nested-123");
        assert_eq!(env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap(), "nested-secret");

        if let Some(v) = prev_id { env::set_var("GOOGLE_OAUTH_CLIENT_ID", v); } else { env::remove_var("GOOGLE_OAUTH_CLIENT_ID"); }
        if let Some(v) = prev_secret { env::set_var("GOOGLE_OAUTH_CLIENT_SECRET", v); } else { env::remove_var("GOOGLE_OAUTH_CLIENT_SECRET"); }
    }

    #[test]
    fn test_single_name_var_in_namespace() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [platform]
            name = "myapp"
        "#;
        let prev = env::var("PLATFORM_NAME").ok();
        env::remove_var("PLATFORM_NAME");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("PLATFORM_NAME").unwrap(), "myapp");

        if let Some(v) = prev { env::set_var("PLATFORM_NAME", v); } else { env::remove_var("PLATFORM_NAME"); }
    }

    #[test]
    fn test_os_env_takes_precedence_over_namespaced() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [google]
            client_id = "toml-value"
        "#;
        let prev = env::var("GOOGLE_CLIENT_ID").ok();
        env::set_var("GOOGLE_CLIENT_ID", "os-value");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("GOOGLE_CLIENT_ID").unwrap(), "os-value");

        if let Some(v) = prev { env::set_var("GOOGLE_CLIENT_ID", v); } else { env::remove_var("GOOGLE_CLIENT_ID"); }
    }

    #[test]
    fn test_env_table_takes_precedence_over_namespace() {
        let _lock = SERIAL_TEST.lock().unwrap();
        // Define the same var in both [env] and a namespace table.
        // [env] is processed first, so its value wins.
        let toml_str = r#"
            [env]
            GOOGLE_CLIENT_ID = "from-env-table"

            [google]
            client_id = "from-namespace"
        "#;
        let prev = env::var("GOOGLE_CLIENT_ID").ok();
        env::remove_var("GOOGLE_CLIENT_ID");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("GOOGLE_CLIENT_ID").unwrap(), "from-env-table");

        if let Some(v) = prev { env::set_var("GOOGLE_CLIENT_ID", v); } else { env::remove_var("GOOGLE_CLIENT_ID"); }
    }

    #[test]
    fn test_get_namespaced_custom_value() {
        let toml_str = r#"
            [google]
            client_id = "abc"

            [google.oauth]
            redirect_url = "http://localhost/callback"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.get::<String>("google.client_id").unwrap(), "abc");
        assert_eq!(
            config.get::<String>("google.oauth.redirect_url").unwrap(),
            "http://localhost/callback"
        );
    }

    #[test]
    fn test_has_key_namespaced() {
        let toml_str = r#"
            [platform]
            name = "test"

            [platform.api]
            key = "secret"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.has_key("platform.name"));
        assert!(config.has_key("platform.api.key"));
        assert!(!config.has_key("platform.missing"));
    }

    #[test]
    fn test_non_string_namespaced_values() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let toml_str = r#"
            [myapp]
            port = 8080
            debug = true
        "#;
        let prev_port = env::var("MYAPP_PORT").ok();
        let prev_debug = env::var("MYAPP_DEBUG").ok();
        env::remove_var("MYAPP_PORT");
        env::remove_var("MYAPP_DEBUG");

        let config: Config = toml::from_str(toml_str).unwrap();
        config.inject_env_vars();

        assert_eq!(env::var("MYAPP_PORT").unwrap(), "8080");
        assert_eq!(env::var("MYAPP_DEBUG").unwrap(), "true");

        if let Some(v) = prev_port { env::set_var("MYAPP_PORT", v); } else { env::remove_var("MYAPP_PORT"); }
        if let Some(v) = prev_debug { env::set_var("MYAPP_DEBUG", v); } else { env::remove_var("MYAPP_DEBUG"); }
    }

    use std::sync::Mutex;
    static SERIAL_TEST: Mutex<()> = Mutex::new(());
}
