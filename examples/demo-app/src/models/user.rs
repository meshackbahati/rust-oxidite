use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

impl User {
    pub fn new(email: String, name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            name,
            created_at: None,
        }
    }
}
