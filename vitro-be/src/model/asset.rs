use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::model::pagination::ListQueryParams;
use crate::util::time_format::format_ts;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "asset_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectorType {
    Upload,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileFormat {
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConnectorDTO {
    pub id: Uuid,
    pub r#type: ConnectorType,
    pub storage_prefix: String,
    pub allowed_formats: Vec<FileFormat>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AssetRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    // From User table
    pub owner_user_name: String,
    pub tags: Vec<String>,
    pub connector_id: Uuid,
    pub connector_type: String,
    pub storage_prefix: String,
    pub allowed_formats: Vec<String>,
    pub status: AssetStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateAssetDTO {
    pub dataset_id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateAssetDTO {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub owner_user_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadAssetDTO {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub owner_user_name: String,
    pub tags: Vec<String>,
    pub connector: ConnectorDTO,
    pub status: AssetStatus,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

impl From<AssetRow> for ReadAssetDTO {
    fn from(value: AssetRow) -> Self {
        let allowed_formats = value
            .allowed_formats
            .into_iter()
            .filter_map(|f| match f.to_ascii_uppercase().as_str() {
                "CSV" => Some(FileFormat::Csv),
                _ => None,
            })
            .collect::<Vec<_>>();
        let connector_type = match value.connector_type.to_ascii_uppercase().as_str() {
            "UPLOAD" => ConnectorType::Upload,
            _ => ConnectorType::Upload,
        };

        Self {
            id: value.id,
            workspace_id: value.workspace_id,
            dataset_id: value.dataset_id,
            name: value.name,
            display_name: value.display_name,
            description: value.description,
            owner_user_id: value.owner_user_id,
            owner_user_name: value.owner_user_name,
            tags: value.tags,
            connector: ConnectorDTO {
                id: value.connector_id,
                r#type: connector_type,
                storage_prefix: value.storage_prefix,
                allowed_formats,
            },
            status: value.status,
            created_at: format_ts(value.created_at),
            updated_at: format_ts(value.updated_at),
            archived_at: value.archived_at.map(format_ts),
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ListAssetsQueryParams {
    #[serde(flatten)]
    pub pagination: ListQueryParams,
    pub dataset_id: Uuid,
    pub status: Option<AssetStatus>,
    pub search: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetListFilter {
    pub dataset_id: Uuid,
    pub status: Option<AssetStatus>,
    pub search: Option<String>,
    pub tags: Vec<String>,
}
