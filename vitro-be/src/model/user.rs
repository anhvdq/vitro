use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: Option<String>,
    pub age: Option<i32>,
    pub email: String,
    pub default_workspace_id: Option<Uuid>,
    pub password_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadUserDTO {
    pub id: Uuid,
    pub name: Option<String>,
    pub age: Option<i32>,
    pub email: String,
    pub default_workspace_id: Option<Uuid>,
}

impl From<User> for ReadUserDTO {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            age: value.age,
            email: value.email,
            default_workspace_id: value.default_workspace_id,
        }
    }
}
