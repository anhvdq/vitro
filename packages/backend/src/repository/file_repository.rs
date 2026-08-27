use axum::body::Bytes;
use axum_typed_multipart::{FieldData, async_trait};
use object_store::ObjectStore as ObjectStoreClientTrait;
use object_store::PutPayload;
use object_store::path::Path as ObjectStorePath;
use std::io::Error as IoError;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::object_store::{ObjectStore, ObjectStoreTrait};
use crate::model::file::StoredFile;

pub type FileRepository = Arc<dyn FileRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait FileRepositoryTrait {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file_id: Uuid,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, IoError>;
    async fn get_file_bytes(&self, storage_key: &str) -> Result<Bytes, IoError>;
    async fn delete_file(&self, storage_key: &str) -> Result<(), IoError>;
}

pub struct FileRepositoryImpl {
    object_store: Arc<ObjectStore>,
}

impl FileRepositoryImpl {
    pub fn new(object_store: Arc<ObjectStore>) -> Self {
        FileRepositoryImpl { object_store }
    }
}

#[async_trait]
impl FileRepositoryTrait for FileRepositoryImpl {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file_id: Uuid,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, IoError> {
        let file_name = file
            .metadata
            .file_name
            .unwrap_or_else(|| "upload".to_string());
        let content_type = file.metadata.content_type.map(|ct| ct.to_string());
        let size_bytes = file.contents.len() as u64;

        let storage_prefix = storage_prefix.trim_matches('/');
        // Key by file id (not just file name) so re-uploading the same
        // filename to an asset never overwrites a previous upload's object.
        let storage_key = format!("{storage_prefix}/{file_id}/{file_name}");

        let path = ObjectStorePath::from(storage_key.as_str());
        let payload = PutPayload::from(file.contents);
        self.object_store
            .get_client()
            .put(&path, payload)
            .await
            .map_err(|err| IoError::other(err.to_string()))?;

        Ok(StoredFile {
            storage_key,
            file_name,
            size_bytes,
            content_type,
        })
    }

    async fn get_file_bytes(&self, storage_key: &str) -> Result<Bytes, IoError> {
        let path = ObjectStorePath::from(storage_key);
        let result = self
            .object_store
            .get_client()
            .get(&path)
            .await
            .map_err(|err| IoError::other(err.to_string()))?;
        result
            .bytes()
            .await
            .map_err(|err| IoError::other(err.to_string()))
    }

    async fn delete_file(&self, storage_key: &str) -> Result<(), IoError> {
        let path = ObjectStorePath::from(storage_key);
        self.object_store
            .get_client()
            .delete(&path)
            .await
            .map_err(|err| IoError::other(err.to_string()))
    }
}
