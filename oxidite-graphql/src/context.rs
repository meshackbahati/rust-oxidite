use std::collections::HashMap;
#[cfg(feature = "database")]
use oxidite_db::Database as OxiditeDatabase;

/// GraphQL context that provides access to database and other resources
pub struct Context {
    /// Database instance, available when the `database` feature is enabled.
    #[cfg(feature = "database")]
    pub database: Option<Box<dyn OxiditeDatabase>>,
    /// Arbitrary key-value extensions for passing data through the context.
    pub extensions: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Context {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "database")]
            database: None,
            extensions: HashMap::new(),
        }
    }

    /// Attach a database instance to this context.
    #[cfg(feature = "database")]
    pub fn with_database(mut self, db: Box<dyn OxiditeDatabase>) -> Self {
        self.database = Some(db);
        self
    }

    /// Insert an extension value by key.
    pub fn insert_extension<T: 'static + Send + Sync>(&mut self, key: String, value: T) {
        self.extensions.insert(key, Box::new(value));
    }

    /// Get a reference to an extension value by key.
    pub fn get_extension<T: 'static>(&self, key: &str) -> Option<&T> {
        self.extensions.get(key).and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Check if an extension key exists.
    pub fn contains_extension(&self, key: &str) -> bool {
        self.extensions.contains_key(key)
    }

    /// Remove an extension by key.
    pub fn remove_extension(&mut self, key: &str) {
        self.extensions.remove(key);
    }
}

/// Returns a new empty context.
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl juniper::Context for Context {}

#[cfg(test)]
mod tests {
    use super::Context;

    #[test]
    fn extension_lifecycle() {
        let mut ctx = Context::new();
        ctx.insert_extension("request_id".to_string(), "abc-123".to_string());
        assert!(ctx.contains_extension("request_id"));
        assert_eq!(
            ctx.get_extension::<String>("request_id").map(String::as_str),
            Some("abc-123")
        );
        ctx.remove_extension("request_id");
        assert!(!ctx.contains_extension("request_id"));
    }
}
