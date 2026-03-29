use std::collections::HashMap;
#[cfg(feature = "database")]
use oxidite_db::Database as OxiditeDatabase;

/// GraphQL context that provides access to database and other resources
pub struct Context {
    #[cfg(feature = "database")]
    pub database: Option<Box<dyn OxiditeDatabase>>,
    pub extensions: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "database")]
            database: None,
            extensions: HashMap::new(),
        }
    }

    #[cfg(feature = "database")]
    pub fn with_database(mut self, db: Box<dyn OxiditeDatabase>) -> Self {
        self.database = Some(db);
        self
    }

    pub fn insert_extension<T: 'static + Send + Sync>(&mut self, key: String, value: T) {
        self.extensions.insert(key, Box::new(value));
    }

    pub fn get_extension<T: 'static>(&self, key: &str) -> Option<&T> {
        self.extensions.get(key).and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn contains_extension(&self, key: &str) -> bool {
        self.extensions.contains_key(key)
    }

    pub fn remove_extension(&mut self, key: &str) {
        self.extensions.remove(key);
    }
}

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
