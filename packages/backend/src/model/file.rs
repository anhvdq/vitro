use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileResponseDTO {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
}

impl From<StoredFile> for FileResponseDTO {
    fn from(value: StoredFile) -> Self {
        Self {
            path: value.path,
            file_name: value.file_name,
            size_bytes: value.size_bytes,
            content_type: value.content_type,
        }
    }
}
