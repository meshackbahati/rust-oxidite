use juniper::{RootNode, GraphQLObject, GraphQLInputObject, FieldResult};
use crate::context::Context;

/// GraphQL query root.
pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn api_version() -> &str {
        "1.0"
    }

    fn health_check() -> bool {
        true
    }
}

/// GraphQL mutation root.
pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    fn add_todo(text: String) -> FieldResult<String> {
        Ok(format!("Added todo: {}", text))
    }
}

/// Create the default GraphQL schema with query and mutation roots.
pub fn create_schema() -> RootNode<'static, QueryRoot, MutationRoot, juniper::EmptySubscription<Context>> {
    RootNode::new(
        QueryRoot,
        MutationRoot,
        juniper::EmptySubscription::new(),
    )
}

/// Convenience type alias for the generated GraphQL schema.
pub type GraphQLSchema = RootNode<'static, QueryRoot, MutationRoot, juniper::EmptySubscription<Context>>;

/// A to-do item.
#[derive(GraphQLObject)]
pub struct Todo {
    /// Unique identifier.
    pub id: i32,
    /// To-do item text.
    pub text: String,
    /// Whether the to-do is completed.
    pub completed: bool,
}

/// Input for creating a new to-do item.
#[derive(GraphQLInputObject)]
pub struct NewTodo {
    /// To-do item text.
    pub text: String,
    /// Optional completion status.
    pub completed: Option<bool>,
}
