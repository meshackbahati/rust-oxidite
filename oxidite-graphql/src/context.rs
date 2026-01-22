use std::collections::HashMap;
use oxidite_db::Database as OxiditeDatabase;

/// GraphQL context that provides access to database and other resources
pub struct Context {
    pub database: Option<Box<dyn OxiditeDatabase>>,
    pub extensions: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            database: None,
            extensions: HashMap::new(),
        }
    }

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
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl juniper::Context for Context {}