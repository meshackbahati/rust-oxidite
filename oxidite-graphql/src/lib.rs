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
use http_body_util::BodyExt;

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
        let schema = self.schema.clone();
        
        // POST endpoint for GraphQL queries
        router.post("/graphql", move |req: oxidite_core::OxiditeRequest| {
            let schema = schema.clone();
            async move {
                // Read request body
                let body_bytes = req.into_body()
                    .collect()
                    .await
                    .map_err(|e| oxidite_core::Error::BadRequest(format!("Failed to read body: {}", e)))?
                    .to_bytes();
                
                // Parse GraphQL request
                let graphql_request: juniper::http::GraphQLRequest = serde_json::from_slice(&body_bytes)
                    .map_err(|e| oxidite_core::Error::BadRequest(format!("Invalid GraphQL request: {}", e)))?;
                
                // Create context
                let context = Context::new();
                
                // Execute query
                let response = graphql_request.execute_sync(&schema, &context);
                
                // Return JSON response
                Ok(oxidite_core::OxiditeResponse::json(response))
            }
        });
        
        // GET endpoint for GraphQL playground
        let schema_clone = self.schema.clone();
        router.get("/graphql", move |_req: oxidite_core::OxiditeRequest| {
            async move {
                let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GraphQL Playground</title>
    <style>
        body { margin: 0; padding: 0; font-family: Arial, sans-serif; }
        #playground { height: 100vh; }
    </style>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
</head>
<body>
    <div id="playground"></div>
    <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
    <script>
        GraphQLPlayground.init(document.getElementById('playground'), {
            endpoint: '/graphql'
        })
    </script>
</body>
</html>"#;
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

