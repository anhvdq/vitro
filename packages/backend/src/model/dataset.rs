use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::model::pagination::ListQueryParams;
use crate::util::time_format::format_ts;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "dataset_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatasetStatus {
    Active,
    Archived,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DatasetRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    // From User table
    pub owner_user_name: String,
    pub tags: Vec<String>,
    pub status: DatasetStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateDatasetDTO {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateDatasetDTO {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub owner_user_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadDatasetDTO {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub owner_user_name: String,
    pub tags: Vec<String>,
    pub status: DatasetStatus,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

impl From<DatasetRow> for ReadDatasetDTO {
    fn from(value: DatasetRow) -> Self {
        Self {
            id: value.id,
            workspace_id: value.workspace_id,
            name: value.name,
            display_name: value.display_name,
            description: value.description,
            owner_user_id: value.owner_user_id,
            owner_user_name: value.owner_user_name,
            tags: value.tags,
            status: value.status,
            created_at: format_ts(value.created_at),
            updated_at: format_ts(value.updated_at),
            archived_at: value.archived_at.map(format_ts),
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ListDatasetsQueryParams {
    #[serde(flatten)]
    pub pagination: ListQueryParams,
    pub status: Option<DatasetStatus>,
    pub search: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DatasetListFilter {
    pub status: Option<DatasetStatus>,
    pub search: Option<String>,
    pub tags: Vec<String>,
}
