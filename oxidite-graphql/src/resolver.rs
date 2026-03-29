//! Resolver extension points.

use crate::Context;

/// Trait for resolver middleware/hooks around GraphQL execution.
pub trait ResolverExtension: Send + Sync {
    /// Called before request execution.
    fn before_execute(&self, _query: &str, _context: &Context) {}

    /// Called after request execution.
    fn after_execute(&self, _query: &str, _context: &Context) {}
}

/// Registry for resolver extensions.
#[derive(Default)]
pub struct ResolverRegistry {
    extensions: Vec<Box<dyn ResolverExtension>>,
}

impl ResolverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<E: ResolverExtension + 'static>(&mut self, extension: E) {
        self.extensions.push(Box::new(extension));
    }

    pub fn notify_before(&self, query: &str, context: &Context) {
        for extension in &self.extensions {
            extension.before_execute(query, context);
        }
    }

    pub fn notify_after(&self, query: &str, context: &Context) {
        for extension in &self.extensions {
            extension.after_execute(query, context);
        }
    }

    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolverExtension, ResolverRegistry};
    use crate::Context;

    struct Noop;
    impl ResolverExtension for Noop {}

    #[test]
    fn registry_registers_extensions() {
        let mut registry = ResolverRegistry::new();
        assert!(registry.is_empty());
        registry.register(Noop);
        assert_eq!(registry.len(), 1);

        let ctx = Context::new();
        registry.notify_before("{ healthCheck }", &ctx);
        registry.notify_after("{ healthCheck }", &ctx);
    }
}
