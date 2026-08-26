use std::sync::Arc;

use axum::body::Bytes;
use axum_typed_multipart::{FieldData, async_trait};

use crate::{
    model::file::StoredFile, repository::file_repository::FileRepository,
    util::api_response::ServiceError,
};

pub type FileService = Arc<dyn FileServiceTrait + Send + Sync>;

#[async_trait]
pub trait FileServiceTrait {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, ServiceError>;
}

pub struct FileServiceImpl {
    file_repository: FileRepository,
}

impl FileServiceImpl {
    pub fn new(file_repository: FileRepository) -> Self {
        FileServiceImpl { file_repository }
    }
}

#[async_trait]
impl FileServiceTrait for FileServiceImpl {
    async fn save_file(
        &self,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<StoredFile, ServiceError> {
        self.file_repository
            .save_file(storage_prefix, file)
            .await
            .map_err(|err| ServiceError::Unknown(err.to_string()))
    }
}
