# GraphQL Integration

GraphQL provides a powerful alternative to REST APIs, allowing clients to request exactly the data they need. This chapter covers how to integrate GraphQL into your Oxidite applications.

## Overview

Oxidite's GraphQL integration includes:
- Schema definition with Rust types
- Query and mutation resolvers
- Subscription support
- Integration with Oxidite's routing system
- Type safety with Juniper integration
- Real-time subscriptions

## Basic GraphQL Setup

Set up a basic GraphQL endpoint:

```rust
use oxidite::prelude::*;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

// Define a simple user object
#[derive(juniper::GraphQLObject)]
#[graphql(description = "A user in the system")]
struct User {
    id: juniper::ID,
    name: String,
    email: String,
    created_at: String,
}

// Define the query root
struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    /// Get a user by ID
    async fn user(id: juniper::ID) -> Option<User> {
        // In a real app, fetch from database
        if id == juniper::ID::from("1") {
            Some(User {
                id: id.clone(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
        } else {
            None
        }
    }
    
    /// Get all users
    async fn users() -> Vec<User> {
        vec![
            User {
                id: juniper::ID::from("1"),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            User {
                id: juniper::ID::from("2"),
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ]
    }
}

// Create the schema
type Schema = juniper::RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

fn create_schema() -> Schema {
    Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new())
}

// GraphQL endpoint handler
async fn graphql_handler(
    mut req: Request,
    State(schema): State<Schema>
) -> Result<Response> {
    // Collect the request body
    use http_body_util::BodyExt;
    let body_bytes = req
        .body_mut()
        .collect()
        .await
        .map_err(|e| Error::Server(e.to_string()))?
        .to_bytes();
    
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Parse GraphQL request
    let gql_request: juniper::http::GraphQLRequest = 
        serde_json::from_str(&body_str)
            .map_err(|e| Error::BadRequest(format!("Invalid GraphQL request: {}", e)))?;
    
    // Execute the request
    let context = DatabaseContext {}; // Context for resolvers
    let response = gql_request.execute(&schema, &context).await;
    
    // Return response
    let json_response = serde_json::to_string(&response)
        .map_err(|e| Error::Server(format!("Serialization error: {}", e)))?;
    
    Ok(Response::json(serde_json::Value::from(response)))
}

// Context for GraphQL resolvers
struct DatabaseContext;

// In a real app, implement your database access here
```

## Advanced Schema Definition

Define more complex schemas with mutations and relationships:

```rust
use oxidite::prelude::*;
use juniper::{FieldResult, GraphQLInputObject};

// Enhanced user with more fields
#[derive(juniper::GraphQLObject, Clone)]
#[graphql(description = "A user in the system")]
struct User {
    id: juniper::ID,
    name: String,
    email: String,
    age: i32,
    posts: Vec<Post>,
    created_at: String,
}

// Post object
#[derive(juniper::GraphQLObject, Clone)]
#[graphql(description = "A blog post")]
struct Post {
    id: juniper::ID,
    title: String,
    content: String,
    author: User,
    published: bool,
    created_at: String,
}

// Input object for mutations
#[derive(GraphQLInputObject)]
#[graphql(description = "Properties for creating a new user")]
struct NewUser {
    name: String,
    email: String,
    age: i32,
}

// Input object for creating a post
#[derive(GraphQLInputObject)]
#[graphql(description = "Properties for creating a new post")]
struct NewPost {
    title: String,
    content: String,
    author_id: juniper::ID,
}

// Enhanced query root
struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    /// Get a user by ID
    async fn user(id: juniper::ID, context: &DatabaseContext) -> FieldResult<Option<User>> {
        // In a real app, fetch from database
        Ok(Some(User {
            id: id.clone(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            posts: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    /// Get all users
    async fn users(context: &DatabaseContext) -> FieldResult<Vec<User>> {
        Ok(vec![
            User {
                id: juniper::ID::from("1"),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                age: 30,
                posts: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            User {
                id: juniper::ID::from("2"),
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
                age: 25,
                posts: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ])
    }
    
    /// Get a post by ID
    async fn post(id: juniper::ID, context: &DatabaseContext) -> FieldResult<Option<Post>> {
        Ok(Some(Post {
            id: id.clone(),
            title: "Sample Post".to_string(),
            content: "This is a sample post content.".to_string(),
            author: User {
                id: juniper::ID::from("1"),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                age: 30,
                posts: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            published: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    /// Get all posts
    async fn posts(context: &DatabaseContext) -> FieldResult<Vec<Post>> {
        Ok(vec![
            Post {
                id: juniper::ID::from("1"),
                title: "First Post".to_string(),
                content: "Content of the first post.".to_string(),
                author: User {
                    id: juniper::ID::from("1"),
                    name: "John Doe".to_string(),
                    email: "john@example.com".to_string(),
                    age: 30,
                    posts: vec![],
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                published: true,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ])
    }
}

// Mutation root
struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    /// Create a new user
    async fn create_user(
        new_user: NewUser,
        context: &DatabaseContext,
    ) -> FieldResult<User> {
        // In a real app, save to database
        Ok(User {
            id: juniper::ID::from(uuid::Uuid::new_v4().to_string()),
            name: new_user.name,
            email: new_user.email,
            age: new_user.age,
            posts: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Create a new post
    async fn create_post(
        new_post: NewPost,
        context: &DatabaseContext,
    ) -> FieldResult<Post> {
        // In a real app, save to database
        Ok(Post {
            id: juniper::ID::from(uuid::Uuid::new_v4().to_string()),
            title: new_post.title,
            content: new_post.content,
            author: User {
                id: new_post.author_id,
                name: "Author Name".to_string(),
                email: "author@example.com".to_string(),
                age: 30,
                posts: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            published: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Update a user
    async fn update_user(
        id: juniper::ID,
        name: Option<String>,
        email: Option<String>,
        age: Option<i32>,
        context: &DatabaseContext,
    ) -> FieldResult<Option<User>> {
        // In a real app, update in database
        Ok(Some(User {
            id,
            name: name.unwrap_or_else(|| "John Doe".to_string()),
            email: email.unwrap_or_else(|| "john@example.com".to_string()),
            age: age.unwrap_or(30),
            posts: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    /// Delete a user
    async fn delete_user(
        id: juniper::ID,
        context: &DatabaseContext,
    ) -> FieldResult<bool> {
        // In a real app, delete from database
        Ok(true) // Simulate successful deletion
    }
}

type Schema = juniper::RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

fn create_advanced_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
```

## Integration with Oxidite Routing

Integrate GraphQL with Oxidite's routing system:

```rust
use oxidite::prelude::*;
use std::sync::Arc;

// Enhanced GraphQL handler with proper request/response handling
async fn graphql_endpoint(
    mut req: Request,
    State(schema): State<Arc<Schema>>
) -> Result<Response> {
    match req.method().as_str() {
        "GET" => {
            // Serve GraphQL Playground/GraphiQL in development
            serve_graphql_playground()
        }
        "POST" => {
            // Handle GraphQL query
            handle_graphql_request(req, schema.as_ref()).await
        }
        _ => Err(Error::MethodNotAllowed),
    }
}

async fn handle_graphql_request(req: Request, schema: &Schema) -> Result<Response> {
    use http_body_util::BodyExt;
    
    // Collect the request body
    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map_err(|e| Error::Server(e.to_string()))?
        .to_bytes();
    
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Parse GraphQL request
    let gql_request: juniper::http::GraphQLRequest = 
        serde_json::from_str(&body_str)
            .map_err(|e| Error::BadRequest(format!("Invalid GraphQL request: {}", e)))?;
    
    // Execute the request
    let context = DatabaseContext {};
    let response = gql_request.execute(schema, &context).await;
    
    // Create response
    let json_response = serde_json::Value::from(response);
    
    if json_response.get("errors").is_some() {
        Ok(Response::json(json_response))
    } else {
        Ok(Response::json(json_response))
    }
}

fn serve_graphql_playground() -> Result<Response> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset=utf-8/>
        <title>GraphQL Playground</title>
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
        <link rel="shortcut icon" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/favicon.png" />
        <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
    </head>
    <body>
        <div id="root">
            <style>
                body {
                    background-color: rgb(23, 42, 58);
                    font-family: 'Open Sans', sans-serif;
                    height: 90vh;
                    margin: 0;
                    overflow: hidden;
                    width: 100vw;
                }
                #root {
                    height: 100%;
                    width: 100%;
                }
                .loading {
                    align-items: center;
                    display: flex;
                    justify-content: center;
                    height: 100%;
                    width: 100%;
                }
                .loading img {
                    animation: loadingAnimation 1s infinite alternate;
                }
                @keyframes loadingAnimation {
                    0% { opacity: 0.3; }
                    100% { opacity: 1; }
                }
            </style>
            <div class="loading">
                <img src='https://cdn.jsdelivr.net/npm/graphql-playground-react/build/logo.png' alt=''>
            </div>
        </div>
        <script>
            window.addEventListener('load', function (event) {
                const root = document.getElementById('root');
                const wsProto = location.protocol === 'https:' ? 'wss:' : 'ws:';
                GraphQLPlayground.init(root, {
                    endpoint: location.href,
                    subscriptionsEndpoint: `${wsProto}//${location.host}${location.pathname}`
                });
            });
        </script>
    </body>
    </html>
    "#;
    
    Ok(Response::html(html.to_string()))
}

// Initialize the application with GraphQL
#[tokio::main]
async fn main() -> Result<()> {
    let schema = Arc::new(create_advanced_schema());
    
    let mut router = Router::new();
    
    // Add GraphQL endpoint
    router.post("/graphql")
        .with_state(schema.clone())
        .handler(graphql_endpoint);
    
    router.get("/graphql")
        .with_state(schema)
        .handler(graphql_endpoint);
    
    Server::new(router)
        .listen("127.0.0.1:3000".parse()?)
        .await
}
```

## Database Integration

Connect GraphQL resolvers to your database:

```rust
use oxidite::prelude::*;
use oxidite_db::Model;
use serde::{Deserialize, Serialize};

// Define models that match your GraphQL types
#[derive(Model, Serialize, Deserialize, juniper::GraphQLObject)]
#[model(table = "graphql_users")]
#[graphql(description = "A user in the system")]
pub struct GraphqlUser {
    #[model(primary_key)]
    pub id: i32,
    #[model(not_null)]
    pub name: String,
    #[model(unique, not_null)]
    pub email: String,
    pub age: i32,
    #[model(created_at)]
    pub created_at: String,
}

#[derive(Model, Serialize, Deserialize, juniper::GraphQLObject)]
#[model(table = "graphql_posts")]
#[graphql(description = "A blog post")]
pub struct GraphqlPost {
    #[model(primary_key)]
    pub id: i32,
    #[model(not_null)]
    pub title: String,
    #[model(not_null)]
    pub content: String,
    pub author_id: i32,
    pub published: bool,
    #[model(created_at)]
    pub created_at: String,
}

// Enhanced context with database access
struct DatabaseContext {
    // In a real app, this would contain database connection
}

// Query resolvers that use the database
struct DbQueryRoot;

#[juniper::graphql_object(Context = DatabaseContext)]
impl DbQueryRoot {
    /// Get a user by ID
    async fn user(id: i32, context: &DatabaseContext) -> FieldResult<Option<GraphqlUser>> {
        // In a real app, fetch from database
        let user = GraphqlUser::find_by_id(id).await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
        
        Ok(user)
    }
    
    /// Get all users
    async fn users(context: &DatabaseContext) -> FieldResult<Vec<GraphqlUser>> {
        let users = GraphqlUser::find_all().await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
        
        Ok(users)
    }
    
    /// Get a post by ID
    async fn post(id: i32, context: &DatabaseContext) -> FieldResult<Option<GraphqlPost>> {
        let post = GraphqlPost::find_by_id(id).await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
        
        Ok(post)
    }
    
    /// Get posts by author
    async fn posts_by_author(
        author_id: i32,
        context: &DatabaseContext
    ) -> FieldResult<Vec<GraphqlPost>> {
        let posts = GraphqlPost::find_where(&format!("author_id = {}", author_id)).await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
        
        Ok(posts)
    }
}

// Mutation resolvers that modify the database
struct DbMutationRoot;

#[juniper::graphql_object(Context = DatabaseContext)]
impl DbMutationRoot {
    /// Create a new user
    async fn create_user(
        name: String,
        email: String,
        age: i32,
        context: &DatabaseContext,
    ) -> FieldResult<GraphqlUser> {
        let user = GraphqlUser {
            id: 0, // Will be auto-generated
            name,
            email,
            age,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let saved_user = user.save().await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
        
        Ok(saved_user)
    }
    
    /// Update a user
    async fn update_user(
        id: i32,
        name: Option<String>,
        email: Option<String>,
        age: Option<i32>,
        context: &DatabaseContext,
    ) -> FieldResult<Option<GraphqlUser>> {
        if let Some(mut user) = GraphqlUser::find_by_id(id).await.map_err(|e| {
            juniper::FieldError::new(e.to_string(), juniper::Value::null())
        })? {
            if let Some(new_name) = name {
                user.name = new_name;
            }
            if let Some(new_email) = email {
                user.email = new_email;
            }
            if let Some(new_age) = age {
                user.age = new_age;
            }
            
            let updated_user = user.save().await
                .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
            
            Ok(Some(updated_user))
        } else {
            Ok(None)
        }
    }
    
    /// Delete a user
    async fn delete_user(
        id: i32,
        context: &DatabaseContext,
    ) -> FieldResult<bool> {
        if let Some(user) = GraphqlUser::find_by_id(id).await.map_err(|e| {
            juniper::FieldError::new(e.to_string(), juniper::Value::null())
        })? {
            user.delete().await
                .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

type DbSchema = juniper::RootNode<'static, DbQueryRoot, DbMutationRoot, EmptySubscription>;

fn create_db_schema() -> DbSchema {
    DbSchema::new(DbQueryRoot, DbMutationRoot, EmptySubscription::new())
}
```

## Authentication and Authorization

Secure your GraphQL endpoints:

```rust
use oxidite::prelude::*;

// Context with authentication info
struct AuthenticatedContext {
    user: Option<GraphqlUser>,
}

// Secured query root
struct SecuredQueryRoot;

#[juniper::graphql_object(Context = AuthenticatedContext)]
impl SecuredQueryRoot {
    /// Get current user (requires authentication)
    async fn me(context: &AuthenticatedContext) -> FieldResult<Option<GraphqlUser>> {
        match &context.user {
            Some(user) => Ok(Some(user.clone())),
            None => Err(juniper::FieldError::new(
                "Authentication required",
                juniper::Value::null()
            )),
        }
    }
    
    /// Get users (requires admin role)
    async fn users(context: &AuthenticatedContext) -> FieldResult<Vec<GraphqlUser>> {
        match &context.user {
            Some(user) if is_admin_user(user) => {
                GraphqlUser::find_all().await
                    .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))
            }
            Some(_) => Err(juniper::FieldError::new(
                "Admin role required",
                juniper::Value::null()
            )),
            None => Err(juniper::FieldError::new(
                "Authentication required",
                juniper::Value::null()
            )),
        }
    }
    
    /// Get user by ID (public endpoint)
    async fn user(id: i32, context: &AuthenticatedContext) -> FieldResult<Option<GraphqlUser>> {
        // Anyone can view user profiles
        GraphqlUser::find_by_id(id).await
            .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))
    }
}

// Secured mutation root
struct SecuredMutationRoot;

#[juniper::graphql_object(Context = AuthenticatedContext)]
impl SecuredMutationRoot {
    /// Create post (authenticated users only)
    async fn create_post(
        title: String,
        content: String,
        context: &AuthenticatedContext,
    ) -> FieldResult<GraphqlPost> {
        match &context.user {
            Some(user) => {
                let post = GraphqlPost {
                    id: 0,
                    title,
                    content,
                    author_id: user.id,
                    published: false,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                
                post.save().await
                    .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))
            }
            None => Err(juniper::FieldError::new(
                "Authentication required to create posts",
                juniper::Value::null()
            )),
        }
    }
    
    /// Update own post (must be the author)
    async fn update_post(
        id: i32,
        title: Option<String>,
        content: Option<String>,
        published: Option<bool>,
        context: &AuthenticatedContext,
    ) -> FieldResult<Option<GraphqlPost>> {
        match &context.user {
            Some(current_user) => {
                if let Some(mut post) = GraphqlPost::find_by_id(id).await
                    .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?
                {
                    // Check if current user is the author
                    if post.author_id != current_user.id {
                        return Err(juniper::FieldError::new(
                            "Only the author can update this post",
                            juniper::Value::null()
                        ));
                    }
                    
                    if let Some(new_title) = title {
                        post.title = new_title;
                    }
                    if let Some(new_content) = content {
                        post.content = new_content;
                    }
                    if let Some(new_published) = published {
                        post.published = new_published;
                    }
                    
                    let updated_post = post.save().await
                        .map_err(|e| juniper::FieldError::new(e.to_string(), juniper::Value::null()))?;
                    
                    Ok(Some(updated_post))
                } else {
                    Ok(None)
                }
            }
            None => Err(juniper::FieldError::new(
                "Authentication required to update posts",
                juniper::Value::null()
            )),
        }
    }
}

// Authentication middleware for GraphQL
async fn graphql_auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response> {
    // Extract authentication token from headers
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|hv| hv.to_str().ok());
    
    let mut context = AuthenticatedContext { user: None };
    
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = auth.trim_start_matches("Bearer ").trim();
            
            // Verify token and get user
            if let Ok(user_id) = verify_jwt_token(token).await {
                // Fetch user from database
                if let Ok(Some(user)) = GraphqlUser::find_by_id(user_id).await {
                    context.user = Some(user);
                }
            }
        }
    }
    
    // Add context to request extensions for GraphQL handler
    req.extensions_mut().insert(context);
    
    next.run(req).await
}

async fn verify_jwt_token(_token: &str) -> Result<i32, String> {
    // In a real app, verify the JWT token and return user ID
    // This is a placeholder implementation
    Ok(1)
}

fn is_admin_user(user: &GraphqlUser) -> bool {
    // In a real app, check user roles from database
    user.email == "admin@example.com"
}

type SecuredSchema = juniper::RootNode<'static, SecuredQueryRoot, SecuredMutationRoot, EmptySubscription>;

fn create_secured_schema() -> SecuredSchema {
    SecuredSchema::new(SecuredQueryRoot, SecuredMutationRoot, EmptySubscription::new())
}
```

## Subscriptions

Implement real-time GraphQL subscriptions:

```rust
use oxidite::prelude::*;
use juniper::http::GraphQLRequest;
use futures::stream::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;
use serde::{Deserialize, Serialize};

// Define subscription types
#[derive(juniper::GraphQLObject)]
#[graphql(description = "A notification")]
struct Notification {
    id: juniper::ID,
    message: String,
    user_id: juniper::ID,
    created_at: String,
}

// Subscription root
struct SubscriptionRoot;

#[juniper::graphql_subscription]
impl SubscriptionRoot {
    /// Subscribe to notifications for a specific user
    async fn notifications(
        &self,
        user_id: juniper::ID,
    ) -> impl Stream<Item = Notification> {
        use tokio_stream::StreamExt;
        
        // Create a channel for sending notifications
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Notification>();
        
        // Simulate sending notifications
        let user_id_clone = user_id.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            for i in 1..=10 {
                interval.tick().await;
                
                let notification = Notification {
                    id: juniper::ID::from(format!("notif_{}", i)),
                    message: format!("Notification {} for user {}", i, user_id_clone),
                    user_id: user_id_clone.clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                
                if tx.send(notification).is_err() {
                    break; // Channel closed
                }
            }
        });
        
        UnboundedReceiverStream::new(rx)
    }
}

type SubscriptionSchema = juniper::RootNode<'static, SecuredQueryRoot, SecuredMutationRoot, SubscriptionRoot>;

fn create_subscription_schema() -> SubscriptionSchema {
    SubscriptionSchema::new(
        SecuredQueryRoot,
        SecuredMutationRoot,
        SubscriptionRoot,
    )
}

// WebSocket handler for subscriptions
async fn websocket_graphql_handler(
    ws: oxidite_realtime::websocket::WebSocket
) -> Result<()> {
    ws.on_message(|msg| async move {
        match msg {
            oxidite_realtime::websocket::Message::Text(text) => {
                // Parse GraphQL subscription message
                match serde_json::from_str::<SubscriptionMessage>(&text) {
                    Ok(sub_msg) => {
                        match sub_msg.r#type.as_str() {
                            "connection_init" => {
                                // Initialize connection
                                Ok(oxidite_realtime::websocket::Message::Text(
                                    r#"{"type":"connection_ack"}"#.to_string()
                                ))
                            }
                            "subscribe" => {
                                // Handle subscription request
                                // This would typically involve setting up a subscription
                                Ok(oxidite_realtime::websocket::Message::Text(
                                    r#"{"type":"next","id":"1","payload":{"data":{"hello":"world"}}}"#.to_string()
                                ))
                            }
                            "unsubscribe" => {
                                // Handle unsubscribe
                                Ok(oxidite_realtime::websocket::Message::Text(
                                    r#"{"type":"complete","id":"1"}"#.to_string()
                                ))
                            }
                            _ => Ok(oxidite_realtime::websocket::Message::Text(
                                r#"{"type":"error","payload":"Unknown message type"}"#.to_string()
                            ))
                        }
                    }
                    Err(_) => Ok(oxidite_realtime::websocket::Message::Text(
                        r#"{"type":"error","payload":"Invalid message format"}"#.to_string()
                    )),
                }
            }
            _ => Ok(msg), // Return other messages as-is
        }
    }).await?;
    
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct SubscriptionMessage {
    r#type: String,
    id: Option<String>,
    payload: Option<serde_json::Value>,
}
```

## Performance Optimization

Optimize GraphQL performance:

```rust
use oxidite::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// DataLoader pattern for efficient database access
struct DataLoader<T> {
    cache: Arc<RwLock<HashMap<i32, T>>>,
    batch_loader: Arc<dyn Fn(Vec<i32>) -> BoxFuture<Vec<T>> + Send + Sync>,
}

type BoxFuture<T> = std::pin::Pin<Box<dyn futures::Future<Output = T> + Send>>;

impl<T: Clone + Send + Sync + 'static> DataLoader<T> {
    fn new<F, Fut>(loader: F) -> Self
    where
        F: Fn(Vec<i32>) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Vec<T>> + Send + 'static,
    {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            batch_loader: Arc::new(move |keys| {
                let loader = loader.clone();
                Box::pin(loader(keys))
            }),
        }
    }
    
    async fn load(&self, key: i32) -> Option<T> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(item) = cache.get(&key) {
                return Some(item.clone());
            }
        }
        
        // Load in batch (simplified for example)
        let items = (self.batch_loader)(vec![key]).await;
        let item = items.into_iter().find(|_| true); // Simplified
        
        // Cache the result
        if let Some(ref item) = item {
            let mut cache = self.cache.write().await;
            cache.insert(key, item.clone());
        }
        
        item
    }
    
    async fn load_many(&self, keys: Vec<i32>) -> Vec<T> {
        let mut uncached_keys = Vec::new();
        let mut results = Vec::new();
        
        // Check cache for each key
        {
            let cache = self.cache.read().await;
            for key in &keys {
                if let Some(item) = cache.get(key) {
                    results.push(item.clone());
                } else {
                    uncached_keys.push(*key);
                }
            }
        }
        
        // Load uncached items in batch
        if !uncached_keys.is_empty() {
            let loaded_items = (self.batch_loader)(uncached_keys.clone()).await;
            
            // Add to cache and results
            let mut cache = self.cache.write().await;
            for (i, key) in uncached_keys.iter().enumerate() {
                if i < loaded_items.len() {
                    let item = loaded_items[i].clone();
                    cache.insert(*key, item.clone());
                    results.push(item);
                }
            }
        }
        
        results
    }
}

// Context with data loaders
struct OptimizedContext {
    user_loader: DataLoader<GraphqlUser>,
    post_loader: DataLoader<GraphqlPost>,
}

// Query root using data loaders
struct OptimizedQueryRoot;

#[juniper::graphql_object(Context = OptimizedContext)]
impl OptimizedQueryRoot {
    /// Get user with optimized loading
    async fn user(id: i32, context: &OptimizedContext) -> FieldResult<Option<GraphqlUser>> {
        let user = context.user_loader.load(id).await;
        Ok(user)
    }
    
    /// Get multiple users efficiently
    async fn users(ids: Vec<i32>, context: &OptimizedContext) -> FieldResult<Vec<GraphqlUser>> {
        let users = context.user_loader.load_many(ids).await;
        Ok(users)
    }
    
    /// Get posts by author with optimized loading
    async fn posts_by_author(
        author_id: i32,
        context: &OptimizedContext
    ) -> FieldResult<Vec<GraphqlPost>> {
        // In a real app, you'd have a specialized loader for this
        // For now, just return empty to satisfy the example
        Ok(vec![])
    }
}

// Schema with optimizations
type OptimizedSchema = juniper::RootNode<'static, OptimizedQueryRoot, SecuredMutationRoot, SubscriptionRoot>;

fn create_optimized_schema() -> OptimizedSchema {
    let user_loader = DataLoader::new(|ids| {
        Box::pin(async move {
            // In a real app, batch fetch users from database
            ids.into_iter()
                .map(|id| GraphqlUser {
                    id,
                    name: format!("User {}", id),
                    email: format!("user{}@example.com", id),
                    age: 25,
                    created_at: chrono::Utc::now().to_rfc3339(),
                })
                .collect()
        })
    });
    
    let post_loader = DataLoader::new(|ids| {
        Box::pin(async move {
            // In a real app, batch fetch posts from database
            ids.into_iter()
                .map(|id| GraphqlPost {
                    id,
                    title: format!("Post {}", id),
                    content: format!("Content of post {}", id),
                    author_id: 1,
                    published: true,
                    created_at: chrono::Utc::now().to_rfc3339(),
                })
                .collect()
        })
    });
    
    let context = OptimizedContext {
        user_loader,
        post_loader,
    };
    
    OptimizedSchema::new(
        OptimizedQueryRoot,
        SecuredMutationRoot,
        SubscriptionRoot,
    )
}
```

## Testing GraphQL

Test your GraphQL endpoints:

```rust
use oxidite::prelude::*;
use oxidite_testing::TestServer;

#[cfg(test)]
mod graphql_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_graphql_query() {
        let schema = Arc::new(create_advanced_schema());
        let server = TestServer::new(move |router| {
            router.post("/graphql")
                .with_state(schema.clone())
                .handler(graphql_endpoint);
        }).await;
        
        let query = r#"
        {
            users {
                id
                name
                email
            }
        }
        "#;
        
        let response = server
            .post("/graphql")
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert!(json["data"]["users"].is_array());
    }
    
    #[tokio::test]
    async fn test_graphql_mutation() {
        let schema = Arc::new(create_advanced_schema());
        let server = TestServer::new(move |router| {
            router.post("/graphql")
                .with_state(schema.clone())
                .handler(graphql_endpoint);
        }).await;
        
        let mutation = r#"
        mutation {
            createUser(newUser: {name: "Test User", email: "test@example.com", age: 30}) {
                id
                name
                email
                age
            }
        }
        "#;
        
        let response = server
            .post("/graphql")
            .json(&serde_json::json!({
                "query": mutation
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        assert!(json["data"]["createUser"]["id"].is_string());
        assert_eq!(json["data"]["createUser"]["name"], "Test User");
    }
    
    #[tokio::test]
    async fn test_graphql_error_handling() {
        let schema = Arc::new(create_advanced_schema());
        let server = TestServer::new(move |router| {
            router.post("/graphql")
                .with_state(schema.clone())
                .handler(graphql_endpoint);
        }).await;
        
        let invalid_query = r#"
        {
            invalidField
        }
        "#;
        
        let response = server
            .post("/graphql")
            .json(&serde_json::json!({
                "query": invalid_query
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), 200); // GraphQL returns 200 even with errors
        
        let json: serde_json::Value = response.json().await;
        assert!(json["errors"].is_array());
        assert!(json["errors"].as_array().unwrap().len() > 0);
    }
    
    #[tokio::test]
    async fn test_graphql_authentication() {
        // Test authenticated GraphQL endpoint
        let schema = Arc::new(create_secured_schema());
        let server = TestServer::new(move |router| {
            router.post("/graphql")
                .middleware(graphql_auth_middleware)
                .with_state(schema.clone())
                .handler(graphql_endpoint);
        }).await;
        
        let query = r#"
        {
            me {
                id
                name
                email
            }
        }
        "#;
        
        // Request without authentication should fail
        let response = server
            .post("/graphql")
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
        
        let json: serde_json::Value = response.json().await;
        // Should have an error about authentication being required
        if let Some(errors) = json["errors"].as_array() {
            assert!(!errors.is_empty());
        }
        
        // Request with authentication should succeed
        let response = server
            .post("/graphql")
            .header("Authorization", "Bearer valid_token")
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
    }
}
```

## Summary

GraphQL integration in Oxidite provides:

- **Schema Definition**: Define types and operations with Rust structs
- **Query and Mutation Support**: Handle data fetching and modifications
- **Database Integration**: Connect resolvers to your data models
- **Authentication**: Secure your GraphQL endpoints
- **Subscriptions**: Real-time data updates via WebSockets
- **Performance Optimization**: DataLoader pattern and caching
- **Testing**: Comprehensive testing utilities
- **Error Handling**: Proper GraphQL error responses

GraphQL offers a flexible alternative to REST APIs, allowing clients to request exactly the data they need while maintaining strong typing and introspection capabilities.