use axum::body::Bytes;
use axum_typed_multipart::{FieldData, async_trait};
use std::io::Error as IoError;
use std::sync::Arc;

use crate::model::file::StoredFile;

pub type FileRepository = Arc<dyn FileRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait FileRepositoryTrait {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, IoError>;
}

pub struct FileRepositoryImpl;
impl FileRepositoryImpl {
    pub fn new() -> Self {
        FileRepositoryImpl {}
    }
}

#[async_trait]
impl FileRepositoryTrait for FileRepositoryImpl {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, IoError> {
        let file_name = file
            .metadata
            .file_name
            .unwrap_or_else(|| "upload".to_string());
        let content_type = file.metadata.content_type.map(|ct| ct.to_string());
        let size_bytes = file.contents.len() as u64;

        let storage_prefix = storage_prefix.trim_matches('/');
        let path = format!("./data/{storage_prefix}/{file_name}");
        let Some(parent) = std::path::Path::new(&path).parent() else {
            return Err(IoError::other("Invalid upload path"));
        };
        tokio::fs::create_dir_all(parent).await?;
        tokio::fs::write(&path, file.contents).await?;
        Ok(StoredFile {
            path,
            file_name,
            size_bytes,
            content_type,
        })
    }
}
