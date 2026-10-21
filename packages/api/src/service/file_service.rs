use std::sync::Arc;

use axum::body::Bytes;
use axum_typed_multipart::{FieldData, async_trait};
use uuid::Uuid;

use crate::{
    model::file::FileRow, repository::file_metadata_repository::FileMetadataRepository,
    repository::file_repository::FileRepository, util::api_response::ServiceError,
};

pub type FileService = Arc<dyn FileServiceTrait + Send + Sync>;

#[async_trait]
pub trait FileServiceTrait {
    async fn save_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        uploaded_by: Uuid,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<FileRow, ServiceError>;
    async fn list_files(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<FileRow>, ServiceError>;
    async fn download_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        file_id: Uuid,
    ) -> Result<(FileRow, Bytes), ServiceError>;
}

pub struct FileServiceImpl {
    file_repository: FileRepository,
    file_metadata_repository: FileMetadataRepository,
}

impl FileServiceImpl {
    pub fn new(
        file_repository: FileRepository,
        file_metadata_repository: FileMetadataRepository,
    ) -> Self {
        FileServiceImpl {
            file_repository,
            file_metadata_repository,
        }
    }
}

#[async_trait]
impl FileServiceTrait for FileServiceImpl {
    async fn save_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        uploaded_by: Uuid,
        storage_prefix: &str,
        file: FieldData<Bytes>,
    ) -> Result<FileRow, ServiceError> {
        let file_id = Uuid::new_v4();
        let stored = self
            .file_repository
            .save_file(storage_prefix, file_id, file)
            .await
            .map_err(|err| ServiceError::Unknown(err.to_string()))?;

        let storage_key = stored.storage_key.clone();

        match self
            .file_metadata_repository
            .insert(
                file_id,
                asset_id,
                workspace_id,
                stored.file_name,
                stored.storage_key,
                stored.size_bytes as i64,
                stored.content_type,
                Some(uploaded_by),
            )
            .await
        {
            Ok(row) => Ok(row),
            Err(err) => {
                // Metadata insert failed after the object was already written
                // to storage. Best-effort compensating delete to avoid
                // leaving an orphaned object that no API path can ever list
                // or download. The original insert error is still returned
                // to the caller either way.
                if let Err(cleanup_err) = self.file_repository.delete_file(&storage_key).await {
                    tracing::error!(
                        "failed to clean up orphaned storage object '{storage_key}' after metadata insert failure: {cleanup_err}"
                    );
                }
                Err(err.into())
            }
        }
    }

    async fn list_files(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<FileRow>, ServiceError> {
        self.file_metadata_repository
            .list_by_asset(workspace_id, asset_id)
            .await
            .map_err(|err| err.into())
    }

    async fn download_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        file_id: Uuid,
    ) -> Result<(FileRow, Bytes), ServiceError> {
        let row = self
            .file_metadata_repository
            .get_by_id(workspace_id, asset_id, file_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("File not found".into()))?;

        let bytes = self
            .file_repository
            .get_file_bytes(&row.storage_key)
            .await
            .map_err(|err| ServiceError::Unknown(err.to_string()))?;

        Ok((row, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use axum_typed_multipart::FieldMetadata;
    use sqlx::Error as SqlxError;

    use crate::repository::file_metadata_repository::FileMetadataRepositoryTrait;
    use crate::repository::file_repository::FileRepositoryTrait;

    /// Object-storage mock: `save_file` always succeeds (returning a fixed
    /// `StoredFile`), and records whether/how many times `delete_file` was
    /// called so tests can assert the compensating cleanup ran.
    struct MockFileRepository {
        delete_calls: AtomicUsize,
        fail_delete: bool,
    }

    #[async_trait]
    impl FileRepositoryTrait for MockFileRepository {
        async fn save_file(
            &self,
            storage_prefix: &str,
            file_id: Uuid,
            file: FieldData<Bytes>,
        ) -> Result<crate::model::file::StoredFile, std::io::Error> {
            Ok(crate::model::file::StoredFile {
                storage_key: format!("{storage_prefix}/{file_id}/{}", file.contents.len()),
                file_name: "test.csv".to_string(),
                size_bytes: file.contents.len() as u64,
                content_type: Some("text/csv".to_string()),
            })
        }

        async fn get_file_bytes(&self, _storage_key: &str) -> Result<Bytes, std::io::Error> {
            unreachable!("not exercised by these tests")
        }

        async fn delete_file(&self, _storage_key: &str) -> Result<(), std::io::Error> {
            self.delete_calls.fetch_add(1, Ordering::SeqCst);
            if self.fail_delete {
                Err(std::io::Error::other("delete failed"))
            } else {
                Ok(())
            }
        }
    }

    /// Metadata-DB mock: `insert` always fails, simulating a DB hiccup after
    /// the object has already been written to storage.
    struct FailingMetadataRepository {
        insert_called: AtomicBool,
    }

    #[async_trait]
    impl FileMetadataRepositoryTrait for FailingMetadataRepository {
        async fn insert(
            &self,
            _id: Uuid,
            _asset_id: Uuid,
            _workspace_id: Uuid,
            _file_name: String,
            _storage_key: String,
            _size_bytes: i64,
            _content_type: Option<String>,
            _uploaded_by: Option<Uuid>,
        ) -> Result<FileRow, SqlxError> {
            self.insert_called.store(true, Ordering::SeqCst);
            Err(SqlxError::RowNotFound)
        }

        async fn list_by_asset(
            &self,
            _workspace_id: Uuid,
            _asset_id: Uuid,
        ) -> Result<Vec<FileRow>, SqlxError> {
            unreachable!("not exercised by these tests")
        }

        async fn get_by_id(
            &self,
            _workspace_id: Uuid,
            _asset_id: Uuid,
            _file_id: Uuid,
        ) -> Result<Option<FileRow>, SqlxError> {
            unreachable!("not exercised by these tests")
        }
    }

    fn dummy_file() -> FieldData<Bytes> {
        FieldData {
            metadata: FieldMetadata::default(),
            contents: Bytes::from_static(b"a,b,c\n1,2,3\n"),
        }
    }

    /// If the metadata insert fails after the object write already
    /// succeeded, `save_file` must (a) still return the original insert
    /// error to the caller (the upload should fail, not silently succeed),
    /// and (b) attempt a compensating delete of the orphaned object so it
    /// isn't left in storage with no corresponding `files` row (which would
    /// make it permanently invisible to list/download, since both go
    /// through the `files` table).
    #[tokio::test]
    async fn save_file_compensates_and_propagates_error_on_metadata_insert_failure() {
        let file_repository = Arc::new(MockFileRepository {
            delete_calls: AtomicUsize::new(0),
            fail_delete: false,
        });
        let metadata_repository = Arc::new(FailingMetadataRepository {
            insert_called: AtomicBool::new(false),
        });

        let service = FileServiceImpl::new(file_repository.clone(), metadata_repository.clone());

        let result = service
            .save_file(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                "ws/test/assets/a",
                dummy_file(),
            )
            .await;

        assert!(
            matches!(result, Err(ServiceError::NotFound(_))),
            "the original metadata-insert error must be propagated to the caller"
        );
        assert!(metadata_repository.insert_called.load(Ordering::SeqCst));
        assert_eq!(
            file_repository.delete_calls.load(Ordering::SeqCst),
            1,
            "a compensating delete must be attempted exactly once"
        );
    }

    /// If the compensating delete itself also fails (double failure), the
    /// original insert error must still be the one returned to the caller —
    /// the cleanup failure must not panic or replace/swallow it.
    #[tokio::test]
    async fn save_file_propagates_original_error_even_if_compensating_delete_fails() {
        let file_repository = Arc::new(MockFileRepository {
            delete_calls: AtomicUsize::new(0),
            fail_delete: true,
        });
        let metadata_repository = Arc::new(FailingMetadataRepository {
            insert_called: AtomicBool::new(false),
        });

        let service = FileServiceImpl::new(file_repository.clone(), metadata_repository.clone());

        let result = service
            .save_file(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                "ws/test/assets/a",
                dummy_file(),
            )
            .await;

        assert!(
            matches!(result, Err(ServiceError::NotFound(_))),
            "the original metadata-insert error must win even when cleanup also fails"
        );
        assert_eq!(file_repository.delete_calls.load(Ordering::SeqCst), 1);
    }
}
