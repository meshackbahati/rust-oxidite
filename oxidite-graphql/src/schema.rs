use juniper::{RootNode, GraphQLObject, GraphQLInputObject, FieldResult};
use crate::context::Context;

// Define basic query root
pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    async fn api_version() -> &str {
        "1.0"
    }

    async fn health_check() -> bool {
        true
    }
}

// Define basic mutation root
pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    async fn add_todo(text: String) -> FieldResult<String> {
        Ok(format!("Added todo: {}", text))
    }
}

// Create the schema
pub fn create_schema() -> RootNode<'static, QueryRoot, MutationRoot, juniper::EmptySubscription<Context>> {
    RootNode::new(
        QueryRoot,
        MutationRoot,
        juniper::EmptySubscription::new(),
    )
}

// Export the schema type
pub type GraphQLSchema = RootNode<'static, QueryRoot, MutationRoot, juniper::EmptySubscription<Context>>;

// Example of how to define a custom object
#[derive(GraphQLObject)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}

// Example of how to define an input object
#[derive(GraphQLInputObject)]
pub struct NewTodo {
    pub text: String,
    pub completed: Option<bool>,
}