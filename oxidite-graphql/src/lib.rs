//! GraphQL integration for Oxidite
//!
//! Provides GraphQL API capabilities with automatic schema generation

pub mod schema;
pub mod resolver;
pub mod context;

pub use schema::GraphQLSchema;
pub use context::Context;

use oxidite_core::{Router, Result};
use juniper::RootNode;
use std::sync::Arc;

/// GraphQL handler for Oxidite
pub struct GraphQLHandler {
    schema: std::sync::Arc<RootNode<'static, schema::QueryRoot, schema::MutationRoot, juniper::EmptySubscription<Context>>>,
}

impl GraphQLHandler {
    pub fn new(schema: RootNode<'static, schema::QueryRoot, schema::MutationRoot, juniper::EmptySubscription<Context>>) -> Self {
        Self {
            schema: std::sync::Arc::new(schema),
        }
    }

    /// Mount GraphQL endpoint to router
    pub fn mount(&self, router: &mut Router) -> Result<()> {
        // Add GraphQL endpoint handlers
        router.post("/graphql", |req| async move {
            // Simple response for now
            Ok(oxidite_core::OxiditeResponse::text("GraphQL endpoint"))
        });
        router.get("/graphql", |req| async move {
            // Simple playground response for now
            let html = "<h1>GraphQL Playground</h1>";
            Ok(oxidite_core::OxiditeResponse::html(html))
        });
        
        Ok(())
    }
}

/// Create a default GraphQL handler
pub fn create_handler() -> GraphQLHandler {
    GraphQLHandler::new(schema::create_schema())
}

