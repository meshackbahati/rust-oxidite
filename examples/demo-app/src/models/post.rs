use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

impl Post {
    pub fn new(user_id: String, title: String, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title,
            content,
            created_at: None,
        }
    }
}
