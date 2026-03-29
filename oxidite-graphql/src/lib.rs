//! GraphQL integration for Oxidite
//!
//! Provides GraphQL API capabilities with automatic schema generation

pub mod schema;
pub mod resolver;
pub mod context;

pub use schema::GraphQLSchema;
pub use context::Context;
pub use resolver::{ResolverExtension, ResolverRegistry};

use oxidite_core::{Router, Result};
use juniper::RootNode;
use http_body_util::BodyExt;

/// GraphQL handler for Oxidite
pub struct GraphQLHandler {
    schema: std::sync::Arc<RootNode<'static, schema::QueryRoot, schema::MutationRoot, juniper::EmptySubscription<Context>>>,
    context_factory: std::sync::Arc<dyn Fn() -> Context + Send + Sync>,
}

impl GraphQLHandler {
    pub fn new(schema: RootNode<'static, schema::QueryRoot, schema::MutationRoot, juniper::EmptySubscription<Context>>) -> Self {
        Self {
            schema: std::sync::Arc::new(schema),
            context_factory: std::sync::Arc::new(Context::new),
        }
    }

    /// Configure custom context factory used per request.
    pub fn with_context_factory(
        mut self,
        factory: impl Fn() -> Context + Send + Sync + 'static,
    ) -> Self {
        self.context_factory = std::sync::Arc::new(factory);
        self
    }

    /// Mount GraphQL endpoint to router
    pub fn mount(&self, router: &mut Router) -> Result<()> {
        self.mount_at(router, "/graphql")
    }

    /// Mount GraphQL endpoint to router at a custom path.
    pub fn mount_at(&self, router: &mut Router, endpoint: &'static str) -> Result<()> {
        let schema = self.schema.clone();
        let context_factory = self.context_factory.clone();
        
        // POST endpoint for GraphQL queries
        router.post(endpoint, move |req: oxidite_core::OxiditeRequest| {
            let schema = schema.clone();
            let context_factory = context_factory.clone();
            async move {
                // Read request body
                let body_bytes = req.into_body()
                    .collect()
                    .await
                    .map_err(|e| oxidite_core::Error::BadRequest(format!("Failed to read body: {}", e)))?
                    .to_bytes();
                
                // Parse GraphQL request (single or batch)
                let graphql_request: juniper::http::GraphQLBatchRequest = serde_json::from_slice(&body_bytes)
                    .map_err(|e| oxidite_core::Error::BadRequest(format!("Invalid GraphQL request: {}", e)))?;
                
                // Create context
                let context = (context_factory)();
                
                // Execute query
                let response = graphql_request.execute_sync(&schema, &context);
                
                // Return JSON response
                Ok(oxidite_core::OxiditeResponse::json(response))
            }
        });
        
        // GET endpoint for GraphQL playground
        router.get(endpoint, move |_req: oxidite_core::OxiditeRequest| {
            let endpoint = endpoint;
            async move {
                let html = juniper::http::playground::playground_source(endpoint, None);
                Ok(oxidite_core::OxiditeResponse::html(html))
            }
        });
        
        Ok(())
    }
}

/// Create a default GraphQL handler
pub fn create_handler() -> GraphQLHandler {
    GraphQLHandler::new(schema::create_schema())
}

#[cfg(test)]
mod tests {
    use super::{create_handler, schema, Context};

    #[test]
    fn create_handler_initializes() {
        let _handler = create_handler();
    }

    #[test]
    fn handler_accepts_custom_context_factory() {
        let _handler = create_handler().with_context_factory(|| {
            let mut ctx = Context::new();
            ctx.insert_extension("request_id".to_string(), "abc".to_string());
            ctx
        });
    }

    #[test]
    fn graphql_batch_request_executes() {
        let schema = schema::create_schema();
        let context = Context::new();
        let payload = r#"[{"query":"{ apiVersion }"},{"query":"{ healthCheck }"}]"#;
        let batch: juniper::http::GraphQLBatchRequest =
            serde_json::from_str(payload).expect("valid batch payload");
        let response = batch.execute_sync(&schema, &context);
        let json = serde_json::to_value(response).expect("serialize response");
        assert!(json.is_array());
    }
}
