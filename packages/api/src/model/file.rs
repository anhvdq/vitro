use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::util::time_format::format_ts;

#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
pub struct UploadFileRequestDTO {
    #[schema(format = "binary")]
    pub file: String,
}

#[derive(TryFromMultipart)]
pub struct FileUploadDTO {
    pub file: FieldData<Bytes>,
}

/// Result of writing a file to object storage.
#[derive(Debug, Clone)]
pub struct StoredFile {
    pub storage_key: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
}

/// Raw DB row shape for the `files` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileRow {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub workspace_id: Uuid,
    pub file_name: String,
    pub storage_key: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

/// API-facing shape for a persisted file.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileResponseDTO {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub workspace_id: Uuid,
    pub file_name: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: String,
}

impl From<FileRow> for FileResponseDTO {
    fn from(value: FileRow) -> Self {
        Self {
            id: value.id,
            asset_id: value.asset_id,
            workspace_id: value.workspace_id,
            file_name: value.file_name,
            size_bytes: value.size_bytes as u64,
            content_type: value.content_type,
            uploaded_by: value.uploaded_by,
            created_at: format_ts(value.created_at),
        }
    }
}
