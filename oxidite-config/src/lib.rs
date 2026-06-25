//! # Oxidite Config
//!
//! Configuration and environment variable management for Oxidite applications.
//! Loads `oxidite.toml` with support for typed config sections, flat `[env]` tables,
//! namespaced tables, nested overrides, and `.env` file loading.
//!
//! # Usage
//!
//! ```rust
//! use oxidite_config::Config;
//!
//! let config = Config::load()
//!     .map_err(|e| eprintln!("config error: {e}")).unwrap();
//!
//! let host: String = config.get("server.host")
//!     .unwrap_or_else(|| "127.0.0.1".to_string());
//! println!("Server: {host}");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during configuration loading and parsing
///
/// ```rust
/// use oxidite_config::ConfigError;
///
/// let err = ConfigError::MissingKey("server.port".to_string());
/// assert_eq!(format!("{}", err), "missing configuration key: server.port");
/// ```
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
    #[error("Ambiguous namespace prefix '{prefix}' matches multiple config paths: {candidates:?}. Rename one of the conflicting tables or properties in oxidite.toml.")]
    AmbiguousNamespace {
        prefix: String,
        candidates: Vec<String>,
    },
    #[error("Environment variable '{var_name}' matches namespace '{namespace}' but has an empty property key. Table-level overrides are not supported.")]
    EmptyPropertyKey {
        var_name: String,
        namespace: String,
    },
}

/// Application environment mode
///
/// ```rust
/// use oxidite_config::Environment;
///
/// let env = Environment::from_str("production");
/// assert_eq!(env.as_str(), "production");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Environment {
    /// Parse an environment string into an `Environment` variant
    ///
    /// Recognises `"production"` / `"prod"`, `"testing"` / `"test"`;
    /// everything else defaults to `Development`.
    ///
    /// ```rust
    /// use oxidite_config::Environment;
    ///
    /// assert_eq!(Environment::from_str("prod"), Environment::Production);
    /// assert_eq!(Environment::from_str("test"), Environment::Testing);
    /// assert_eq!(Environment::from_str("staging"), Environment::Development);
    /// ```
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Development,
        }
    }

    /// Return the string representation of the environment variant
    ///
    /// ```rust
    /// use oxidite_config::Environment;
    ///
    /// assert_eq!(Environment::Production.as_str(), "production");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Production => "production",
        }
    }
}

/// Root configuration struct representing an `oxidite.toml` file
///
/// Contains typed sections (`app`, `server`, `database`, `cache`, `queue`, `security`),
/// a flat `[env]` table, and any unknown root-level tables captured via `#[serde(flatten)]`
/// for namespaced environment variable injection.
///
/// ```rust
/// use oxidite_config::Config;
///
/// let config = Config::default();
/// assert_eq!(config.server.port, 3000);
/// ```
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
    /// Injected into the process environment at load time.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Any unknown root-level TOML tables are captured here via `#[serde(flatten)]`.
    ///
    /// Enables namespaced environment variables: table `[google]` with `client_id = "abc"`
    /// becomes `GOOGLE_CLIENT_ID=abc`. Nested tables (`[google.oauth]`) flatten recursively.
    #[serde(flatten, default)]
    pub custom: HashMap<String, toml::Value>,
}

/// Application metadata configuration
///
/// ```rust
/// use oxidite_config::AppConfig;
///
/// let app = AppConfig::default();
/// assert_eq!(app.name, "oxidite-app");
/// ```
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

/// HTTP server configuration (host, port, worker count)
///
/// ```rust
/// use oxidite_config::ServerConfig;
///
/// let srv = ServerConfig::default();
/// assert_eq!(srv.host, "127.0.0.1");
/// assert_eq!(srv.port, 3000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub workers: usize,
}

/// Database connection configuration
///
/// ```rust
/// use oxidite_config::DatabaseConfig;
///
/// let db = DatabaseConfig::default();
/// assert_eq!(db.pool_size, 10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default)]
    pub ssl: bool,
}

/// Cache driver configuration (memory or Redis)
///
/// ```rust
/// use oxidite_config::CacheConfig;
///
/// let cache = CacheConfig::default();
/// assert_eq!(cache.driver, "memory");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub redis_url: String,
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,
}

/// Background job queue configuration
///
/// ```rust
/// use oxidite_config::QueueConfig;
///
/// let queue = QueueConfig::default();
/// assert_eq!(queue.driver, "memory");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub redis_url: String,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

/// Security configuration (JWT, CORS, rate limiting)
///
/// ```rust
/// use oxidite_config::SecurityConfig;
///
/// let sec = SecurityConfig::default();
/// assert_eq!(sec.jwt_expiry, 900);
/// ```
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
    900
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

/// A registry entry mapping an uppercase env prefix to a config path within `custom`.
struct NamespaceEntry {
    /// Uppercase prefix with trailing underscore, e.g. `"DEMO_SERVICE_"`.
    env_prefix: String,
    /// Segments of the config path, e.g. `["demo", "service"]`.
    config_path: Vec<String>,
}

/// Coerce a raw environment variable string into the most specific `toml::Value` variant.
///
/// Order of precedence: Boolean → Integer → Float → String.
fn coerce_env_value(raw: &str) -> toml::Value {
    let trimmed = raw.trim();

    // Stage 1 — Boolean (exact case-insensitive match only)
    match trimmed.to_lowercase().as_str() {
        "true" => return toml::Value::Boolean(true),
        "false" => return toml::Value::Boolean(false),
        _ => {}
    }

    // Stage 2 — Integer (base-10 digits, optional leading minus)
    if let Ok(n) = trimmed.parse::<i64>() {
        return toml::Value::Integer(n);
    }

    // Stage 3 — Float (must contain `.`, `e`, `E`, `inf`, or `nan`)
    if trimmed.contains('.')
        || trimmed.contains('e')
        || trimmed.contains('E')
        || trimmed.eq_ignore_ascii_case("inf")
        || trimmed.eq_ignore_ascii_case("-inf")
        || trimmed.eq_ignore_ascii_case("nan")
    {
        if let Ok(f) = trimmed.parse::<f64>() {
            return toml::Value::Float(f);
        }
    }

    // Stage 4 — String fallthrough (preserve original casing)
    toml::Value::String(raw.to_string())
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
    /// Inject `[env]` entries, known config sections, and namespaced tables into the
    /// process environment.
    ///
    /// Every TOML section becomes an uppercase env prefix: `[server] host = "x"` produces
    /// `SERVER_HOST=x`. This means `oxidite.toml` and `.env` are equivalent — a value can
    /// be defined in either place and both `config.get("section.key")` and
    /// `std::env::var("SECTION_KEY")` work.
    ///
    /// **Resolution order** (highest to lowest priority):
    /// 1. Real OS environment variables
    /// 2. `.env` file entries (loaded earlier via `dotenv`)
    /// 3. `[env]` flat table entries
    /// 4. Known sections (`[server]`, `[app]`, `[database]`, etc.)
    /// 5. Custom namespaced tables (`[google]`, `[platform]`, etc.)
    ///
    /// A variable is only set if it is not already defined (or is empty) in the
    /// OS environment, so real env vars and `.env` entries always take precedence.
    fn inject_env_vars(&self) {
        for (key, value) in &self.env {
            let already_set = env::var(key)
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            if !already_set {
                env::set_var(key, value);
            }
        }

        // Serialize self to TOML and inject every top-level table as namespaced env vars.
        // Because `custom` uses #[serde(flatten)], both known sections (server, app, …)
        // and custom tables (google, platform, …) appear at the top level — so they all
        // produce the same env-var pattern: e.g. `[server] host = "x"` → `SERVER_HOST=x`.
        if let Ok(root) = toml::Value::try_from(self) {
            if let toml::Value::Table(table) = root {
                for (key, value) in table {
                    if key == "env" {
                        continue; // already handled above
                    }
                    Self::inject_namespaced_env(&key, &value);
                }
            }
        }
    }

    /// Recursively flatten a TOML value into environment variables.
    ///
    /// - Table `[google]` with `client_id = "abc"` produces `GOOGLE_CLIENT_ID=abc`.
    /// - Nested `[google.oauth]` with `client_id = "abc"` produces `GOOGLE_OAUTH_CLIENT_ID=abc`.
    /// - Non-string values (integers, booleans) are converted to strings.
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

    /// Apply well-known environment variable overrides to typed config fields
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

    /// Collect environment variable overrides and inject them into `self.custom`.
    ///
    /// Scans all current `std::env::vars()`, matches them against the existing
    /// namespace registry (built from `self.custom` keys), and injects typed values
    /// into the correct nested path within the `custom` HashMap.
    ///
    /// `pre_dotenv_keys` is a snapshot of env var names taken **before** `.env`
    /// was loaded, so real OS-level variables (PATH, HOME, XDG_*) are excluded.
    fn collect_env_overrides(&mut self, pre_dotenv_keys: &HashSet<String>) -> Result<(), ConfigError> {
        let registry = self.build_namespace_registry()?;

        for (env_name, raw_value) in env::vars() {
            if pre_dotenv_keys.contains(&env_name) {
                continue;
            }

            let Some(entry) = registry.iter().find(|e| {
                env_name.starts_with(&e.env_prefix)
                    || env_name == e.env_prefix.trim_end_matches('_')
            }) else {
                continue;
            };

            let remaining = if env_name.starts_with(&entry.env_prefix) {
                &env_name[entry.env_prefix.len()..]
            } else {
                ""
            };

            if remaining.is_empty() {
                continue;
            }
            let field_key = remaining.to_lowercase();
            let value = coerce_env_value(&raw_value);

            Self::inject_env_override(&mut self.custom, &entry.config_path, &field_key, value);
        }

        Ok(())
    }

    /// Build a sorted namespace registry from known sections and custom HashMap keys.
    ///
    /// Each entry maps an uppercase env prefix (e.g., `DEMO_SERVICE_`) to a config
    /// path (e.g., `["demo", "service"]`). The registry is sorted longest-prefix-first
    /// so that `DEMO_SERVICE_` is matched before `DEMO_`.
    ///
    /// If two different config paths produce the same env prefix, a
    /// `ConfigError::AmbiguousNamespace` is returned.
    fn build_namespace_registry(&self) -> Result<Vec<NamespaceEntry>, ConfigError> {
        let mut registry: Vec<NamespaceEntry> = Vec::new();

        // Recursively collect paths from custom HashMap
        for (key, value) in &self.custom {
            Self::collect_custom_paths(key, value, &[], &mut registry);
        }

        // Check for ambiguous prefixes
        let mut seen: HashMap<String, Vec<String>> = HashMap::new();
        for entry in &registry {
            seen.entry(entry.env_prefix.clone())
                .or_default()
                .push(entry.config_path.join("."));
        }

        for (prefix, paths) in &seen {
            let unique: HashSet<&str> = paths.iter().map(|s| s.as_str()).collect();
            if unique.len() > 1 {
                return Err(ConfigError::AmbiguousNamespace {
                    prefix: prefix.clone(),
                    candidates: unique.into_iter().map(|s| s.to_string()).collect(),
                });
            }
        }

        // Sort longest prefix first for longest-match resolution
        registry.sort_by(|a, b| b.env_prefix.len().cmp(&a.env_prefix.len()));
        Ok(registry)
    }

    /// Recursively walk a TOML value tree and register every table path as a namespace entry.
    fn collect_custom_paths(
        key: &str,
        value: &toml::Value,
        ancestors: &[String],
        registry: &mut Vec<NamespaceEntry>,
    ) {
        if let toml::Value::Table(table) = value {
            let mut path: Vec<String> = ancestors.to_vec();
            // If the key itself contains dots (from serde flatten of [demo.service]),
            // split into individual segments
            for segment in key.split('.') {
                path.push(segment.to_string());
            }

            let prefix = path.iter()
                .map(|s| s.to_uppercase())
                .collect::<Vec<_>>()
                .join("_")
                + "_";

            registry.push(NamespaceEntry {
                env_prefix: prefix,
                config_path: path.clone(),
            });

            // Recurse into sub-tables for dotted sub-table detection
            for (sub_key, sub_val) in table {
                Self::collect_custom_paths(sub_key, sub_val, &path, registry);
            }
        }
    }

    /// Inject a typed value into `self.custom` at the path described by `config_path`
    /// with the leaf key `field_key`.
    ///
    /// For a flat namespace `["demo"]` with field_key `"url"`, this writes
    /// `custom["demo"]["url"] = value`.
    ///
    /// For a nested path `["demo", "service"]` with field_key `"url"`, this writes
    /// `custom["demo"]["service"]["url"] = value`.
    fn inject_env_override(
        custom: &mut HashMap<String, toml::Value>,
        config_path: &[String],
        field_key: &str,
        value: toml::Value,
    ) {
        if config_path.is_empty() {
            return;
        }

        let namespace = &config_path[0];
        let Some(toml::Value::Table(ref mut top_table)) = custom.get_mut(namespace) else {
            return;
        };

        if config_path.len() == 1 {
            top_table.insert(field_key.to_string(), value);
            return;
        }

        let mut current = top_table;
        for segment in &config_path[1..] {
            match current.get_mut(segment) {
                Some(toml::Value::Table(ref mut next)) => current = next,
                _ => return,
            }
        }
        current.insert(field_key.to_string(), value);
    }

    /// Check if a given dotted key exists in the configuration
    ///
    /// Checks custom namespaced tables first, then falls back to known config fields.
    ///
    /// ```rust
    /// use oxidite_config::Config;
    ///
    /// let config = Config::default();
    /// assert!(config.has_key("server.port"));
    /// assert!(!config.has_key("nonexistent.key"));
    /// ```
    pub fn has_key(&self, key: &str) -> bool {
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

    /// Load configuration from `oxidite.toml` in the current directory
    ///
    /// Falls back to `Config::default()` if the file does not exist.
    /// Loads `.env` file first (unless `OXIDITE_SKIP_DOTENV` is set),
    /// then injects config env vars, then applies known env overrides.
    ///
    /// ```rust
    /// use oxidite_config::Config;
    ///
    /// let config = Config::load()
    ///     .map_err(|e| eprintln!("config error: {e}")).unwrap_or_default();
    /// println!("App: {}", config.app.name);
    /// ```
    pub fn load() -> Result<Self, ConfigError> {
        // Snapshot OS env vars BEFORE loading .env (so we can distinguish
        // host-level vars like PATH, HOME from user-override vars in .env)
        let pre_dotenv_keys: HashSet<String> = env::vars()
            .map(|(k, _)| k)
            .collect();

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

        // NEW: Pull env overrides into custom namespaces
        config.collect_env_overrides(&pre_dotenv_keys)?;

        config.inject_env_vars();
        config.apply_env_overrides()?;
        config.app.environment = env_val;
        Ok(config)
    }

    /// Load configuration from a custom file path
    ///
    /// Same as `load()` but reads from an explicit path instead of `oxidite.toml`.
    ///
    /// ```rust
    /// use oxidite_config::Config;
    ///
    /// let config = Config::load_from("/etc/myapp/config.toml")
    ///     .map_err(|e| eprintln!("config error: {e}")).unwrap_or_default();
    /// ```
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let pre_dotenv_keys: HashSet<String> = env::vars()
            .map(|(k, _)| k)
            .collect();

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

        config.collect_env_overrides(&pre_dotenv_keys)?;
        config.inject_env_vars();
        config.app.environment = env_name;
        config.apply_env_overrides()?;
        Ok(config)
    }

    /// Get a typed configuration value by dotted key path
    ///
    /// Checks custom namespaced tables first, then known config fields.
    /// Returns `None` if the key is missing or the type cannot be deserialized.
    ///
    /// ```rust
    /// use oxidite_config::Config;
    ///
    /// let config = Config::default();
    /// let port: Option<u16> = config.get("server.port");
    /// assert_eq!(port, Some(3000));
    /// ```
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
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

        let root = toml::Value::try_from(self).ok()?;
        let mut cursor = &root;
        for part in key.split('.') {
            cursor = cursor.get(part)?;
        }

        T::deserialize(cursor.clone()).ok()
    }

    /// Get a required typed configuration value, returning a `ConfigError` on failure
    ///
    /// ```rust
    /// use oxidite_config::Config;
    ///
    /// let config = Config::default();
    /// let port = config.get_required::<u16>("server.port")
    ///     .map_err(|e| eprintln!("missing config: {e}")).unwrap();
    /// assert_eq!(port, 3000);
    /// ```
    pub fn get_required<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, ConfigError> {
        self.get(key).ok_or_else(|| {
            if self.has_key(key) {
                ConfigError::InvalidType(key.to_string())
            } else {
                ConfigError::MissingKey(key.to_string())
            }
        })
    }

    /// Convenience method: get a `u16` value or return a `ConfigError`
    pub fn get_u16(&self, key: &str) -> Result<u16, ConfigError> {
        self.get_required(key)
    }

    /// Convenience method: get a `bool` value or return a `ConfigError`
    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        self.get_required(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ──────────────────────────────────────────────────────────

    /// Parse TOML and run the full load pipeline (collect_env_overrides,
    /// inject_env_vars, apply_env_overrides) with isolation.
    ///
    /// Takes a **pre-set** snapshot of env vars so any OS-level or leaked
    /// env vars are blacklisted. Only the env vars passed in `env_vars`
    /// (which are set AFTER the snapshot) pass through.
    fn config_from_toml_with_env(
        toml_str: &str,
        env_vars: &[(&str, &str)],
    ) -> Result<Config, ConfigError> {
        let _lock = SERIAL_TEST.lock().unwrap();
        // Purge any env vars leaked from a previous test that panicked
        for &(k, _) in env_vars {
            let _ = env::remove_var(k);
        }
        // Snapshot — captures any leaked env vars from previous tests
        let pre_set_keys: HashSet<String> = env::vars().map(|(k, _)| k).collect();

        let mut backups: Vec<(String, Option<String>)> = Vec::new();
        for &(k, v) in env_vars {
            backups.push((k.to_string(), env::var(k).ok()));
            env::set_var(k, v);
        }

        let mut config: Config = toml::from_str(toml_str).unwrap();
        // pre_set_keys blacklists anything that existed before we set our test vars
        config.collect_env_overrides(&pre_set_keys)?;
        config.inject_env_vars();
        config.apply_env_overrides()?;

        for (k, prev) in backups {
            match prev {
                Some(v) => env::set_var(&k, v),
                None => env::remove_var(&k),
            }
        }
        Ok(config)
    }

    // ── Existing tests (preserved) ──────────────────────────────────────

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
        if let Some(v) = prev {
            env::set_var("SERVER_PORT", v);
        } else {
            env::remove_var("SERVER_PORT");
        }
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_applies_env_overrides() {
        let _lock = SERIAL_TEST.lock().unwrap();
        let prev_host = env::var("SERVER_HOST").ok();
        env::set_var("SERVER_HOST", "0.0.0.0");
        let cfg = Config::load_from("non-existent.toml").unwrap();
        if let Some(v) = prev_host {
            env::set_var("SERVER_HOST", v);
        } else {
            env::remove_var("SERVER_HOST");
        }
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

        if let Some(v) = prev {
            env::set_var("FLAT_TEST_VAR", v);
        } else {
            env::remove_var("FLAT_TEST_VAR");
        }
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

        if let Some(v) = prev_id {
            env::set_var("GOOGLE_CLIENT_ID", v);
        } else {
            env::remove_var("GOOGLE_CLIENT_ID");
        }
        if let Some(v) = prev_secret {
            env::set_var("GOOGLE_CLIENT_SECRET", v);
        } else {
            env::remove_var("GOOGLE_CLIENT_SECRET");
        }
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

        if let Some(v) = prev_id {
            env::set_var("GOOGLE_OAUTH_CLIENT_ID", v);
        } else {
            env::remove_var("GOOGLE_OAUTH_CLIENT_ID");
        }
        if let Some(v) = prev_secret {
            env::set_var("GOOGLE_OAUTH_CLIENT_SECRET", v);
        } else {
            env::remove_var("GOOGLE_OAUTH_CLIENT_SECRET");
        }
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

        if let Some(v) = prev {
            env::set_var("PLATFORM_NAME", v);
        } else {
            env::remove_var("PLATFORM_NAME");
        }
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

        if let Some(v) = prev {
            env::set_var("GOOGLE_CLIENT_ID", v);
        } else {
            env::remove_var("GOOGLE_CLIENT_ID");
        }
    }

    #[test]
    fn test_env_table_takes_precedence_over_namespace() {
        let _lock = SERIAL_TEST.lock().unwrap();
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

        if let Some(v) = prev {
            env::set_var("GOOGLE_CLIENT_ID", v);
        } else {
            env::remove_var("GOOGLE_CLIENT_ID");
        }
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

        if let Some(v) = prev_port {
            env::set_var("MYAPP_PORT", v);
        } else {
            env::remove_var("MYAPP_PORT");
        }
        if let Some(v) = prev_debug {
            env::set_var("MYAPP_DEBUG", v);
        } else {
            env::remove_var("MYAPP_DEBUG");
        }
    }

    // ── New: Scenario Tests ─────────────────────────────────────────────

    // Test 1: Flat custom namespace with string override
    #[test]
    fn test_flat_custom_string_override() {
        let toml = r#"[demo]
            url = "from-toml""#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_URL", "from-dotenv")],
        )
        .unwrap();
        assert_eq!(
            config.get::<String>("demo.url").unwrap(),
            "from-dotenv"
        );
    }

    // Test 2: Flat custom namespace with integer override
    #[test]
    fn test_flat_custom_integer_override() {
        let toml = r#"[demo]
            timeout = 10"#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_TIMEOUT", "30")],
        )
        .unwrap();
        assert_eq!(config.get::<i64>("demo.timeout").unwrap(), 30);
    }

    // Test 3: Flat custom namespace with boolean override
    #[test]
    fn test_flat_custom_boolean_override() {
        let toml = r#"[demo]
            debug = false"#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_DEBUG", "true")],
        )
        .unwrap();
        assert_eq!(config.get::<bool>("demo.debug").unwrap(), true);
    }

    // Test 4: Dotted sub-table namespace with string override
    #[test]
    fn test_dotted_subtable_override() {
        let toml = r#"[demo.service]
            url = "from-toml-nested""#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_SERVICE_URL", "from-dotenv-nested")],
        )
        .unwrap();
        assert_eq!(
            config.get::<String>("demo.service.url").unwrap(),
            "from-dotenv-nested"
        );
    }

    // Test 5: Multiple custom namespaces
    #[test]
    fn test_multiple_custom_namespace_overrides() {
        let toml = r#"
            [demo]
            url = "demo-toml"

            [google]
            client_id = "google-toml"
        "#;
        let config = config_from_toml_with_env(
            toml,
            &[
                ("DEMO_URL", "demo-dotenv"),
                ("GOOGLE_CLIENT_ID", "google-dotenv"),
            ],
        )
        .unwrap();
        assert_eq!(
            config.get::<String>("demo.url").unwrap(),
            "demo-dotenv"
        );
        assert_eq!(
            config.get::<String>("google.client_id").unwrap(),
            "google-dotenv"
        );
    }

    // Test 6: Known section (SERVER_PORT) still works
    #[test]
    fn test_known_section_server_port_still_works() {
        let toml = r#"[server]
            port = 3000"#;
        let config = config_from_toml_with_env(
            toml,
            &[("SERVER_PORT", "9000")],
        )
        .unwrap();
        assert_eq!(config.server.port, 9000);
        assert_eq!(
            config.get::<u16>("server.port").unwrap(),
            9000
        );
    }

    // Test 7: No env override — TOML value preserved
    #[test]
    fn test_no_env_override_preserves_toml() {
        let toml = r#"[demo]
            url = "from-toml"
            timeout = 10"#;
        let config = config_from_toml_with_env(toml, &[]).unwrap();
        assert_eq!(
            config.get::<String>("demo.url").unwrap(),
            "from-toml"
        );
        assert_eq!(config.get::<i64>("demo.timeout").unwrap(), 10);
    }

    // Test 8: Partial override — some properties overridden, others not
    #[test]
    fn test_partial_override() {
        let toml = r#"[demo]
            url = "toml-url"
            timeout = 10"#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_URL", "env-url")],
        )
        .unwrap();
        assert_eq!(
            config.get::<String>("demo.url").unwrap(),
            "env-url"
        );
        assert_eq!(config.get::<i64>("demo.timeout").unwrap(), 10);
    }

    // Test 9: OS env vars are ignored (pre_dotenv_keys guard)
    #[test]
    fn test_os_env_vars_ignored() {
        // Use the real load_path which takes a real pre_dotenv_keys snapshot.
        // We create a minimal TOML with no custom namespaces to avoid collision.
        let _lock = SERIAL_TEST.lock().unwrap();
        let prev_path = env::var("PATH").ok();
        let prev_home = env::var("HOME").ok();
        env::set_var("PATH", "/usr/bin:/bin");
        env::set_var("HOME", "/root");
        // load_from takes a snapshot internally and will blacklist PATH/HOME
        let config = Config::load_from("non-existent.toml").unwrap();
        assert!(config.get::<String>("path").is_none());
        assert!(config.get::<String>("home").is_none());
        if let Some(v) = prev_path {
            env::set_var("PATH", v);
        } else {
            env::remove_var("PATH");
        }
        if let Some(v) = prev_home {
            env::set_var("HOME", v);
        } else {
            env::remove_var("HOME");
        }
    }

    // Test 10: Ambiguous namespace detection — dotted vs flat collision
    #[test]
    fn test_ambiguous_namespace_dotted_vs_flat() {
        let toml = r#"
            [demo_service]
            url = "flat"

            [demo.service]
            url = "dotted"
        "#;
        let result = config_from_toml_with_env(toml, &[]);
        assert!(result.is_err());
        match result {
            Err(ConfigError::AmbiguousNamespace { prefix, candidates }) => {
                assert_eq!(prefix, "DEMO_SERVICE_");
                assert!(candidates.contains(&"demo_service".to_string()));
                assert!(candidates.contains(&"demo.service".to_string()));
            }
            _ => panic!("expected AmbiguousNamespace error"),
        }
    }

    // Test 11: Case-sensitive collision detection
    #[test]
    fn test_case_sensitive_collision() {
        let toml = r#"
            [demo]
            url = "lower"

            [DEMO]
            url = "upper"
        "#;
        let result = config_from_toml_with_env(toml, &[]);
        assert!(result.is_err());
        match result {
            Err(ConfigError::AmbiguousNamespace { prefix, .. }) => {
                assert_eq!(prefix, "DEMO_");
            }
            _ => panic!("expected AmbiguousNamespace error"),
        }
    }

    // Test 12: Env var with no matching namespace is ignored
    #[test]
    fn test_unknown_env_var_ignored() {
        let toml = r#"[demo]
            url = "ok""#;
        let config = config_from_toml_with_env(
            toml,
            &[("UNKNOWN_KEY", "somevalue")],
        )
        .unwrap();
        // Unknown key does not appear in custom
        assert!(config.custom.get("unknown").is_none());
        // Known namespace still works
        assert_eq!(
            config.get::<String>("demo.url").unwrap(),
            "ok"
        );
    }

    // Test 13: Table-level override is silently skipped
    #[test]
    fn test_table_level_override_skipped() {
        let toml = r#"[demo.service]
            url = "nested""#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_SERVICE", "not-a-property")],
        )
        .unwrap();
        // The table-level env var (no remaining key) is skipped
        // The nested value from TOML is preserved
        assert_eq!(
            config.get::<String>("demo.service.url").unwrap(),
            "nested"
        );
    }

    // Test 14: Float coercion
    #[test]
    fn test_float_coercion() {
        let toml = r#"[demo]
            threshold = 0.5"#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_THRESHOLD", "0.75")],
        )
        .unwrap();
        let val: f64 = config.get("demo.threshold").unwrap();
        assert!((val - 0.75).abs() < 1e-10);
    }

    // Test 15: String that looks numeric — coerced to typed Value
    // "2.0" contains '.' → Float(2.0). The type in custom is Float,
    // so reading as f64 works correctly. Reading as String would fail
    // because serde's String visitor does not accept visit_f64.
    #[test]
    fn test_numeric_like_string_override() {
        let toml = r#"[demo]
            version = "1.0""#;
        let config = config_from_toml_with_env(
            toml,
            &[("DEMO_VERSION", "2.0")],
        )
        .unwrap();
        let val: f64 = config.get("demo.version").unwrap();
        assert!((val - 2.0).abs() < 1e-10);
    }

    // Test: coerce_env_value boolean
    #[test]
    fn test_coerce_boolean() {
        assert_eq!(coerce_env_value("true"), toml::Value::Boolean(true));
        assert_eq!(coerce_env_value("TRUE"), toml::Value::Boolean(true));
        assert_eq!(coerce_env_value("false"), toml::Value::Boolean(false));
        assert_eq!(coerce_env_value("FALSE"), toml::Value::Boolean(false));
    }

    // Test: coerce_env_value integer
    #[test]
    fn test_coerce_integer() {
        assert_eq!(coerce_env_value("30"), toml::Value::Integer(30));
        assert_eq!(coerce_env_value("-5"), toml::Value::Integer(-5));
        assert_eq!(coerce_env_value("0"), toml::Value::Integer(0));
        // Not an integer (has decimal)
        match coerce_env_value("3.14") {
            toml::Value::Float(_) => {}
            _ => panic!("expected Float"),
        }
    }

    // Test: coerce_env_value float
    #[test]
    fn test_coerce_float() {
        match coerce_env_value("3.14") {
            toml::Value::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            _ => panic!("expected Float"),
        }
    }

    // Test: coerce_env_value string fallthrough
    #[test]
    fn test_coerce_string() {
        assert_eq!(
            coerce_env_value("hello"),
            toml::Value::String("hello".to_string())
        );
        assert_eq!(
            coerce_env_value("abc123"),
            toml::Value::String("abc123".to_string())
        );
    }

    use std::sync::Mutex;
    static SERIAL_TEST: Mutex<()> = Mutex::new(());
}
