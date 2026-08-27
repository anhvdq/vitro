use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::pg_database::{PgDatabase, PgDatabaseTrait};
use crate::model::file::FileRow;

pub type FileMetadataRepository = Arc<dyn FileMetadataRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait FileMetadataRepositoryTrait {
    #[allow(clippy::too_many_arguments)]
    async fn insert(
        &self,
        id: Uuid,
        asset_id: Uuid,
        workspace_id: Uuid,
        file_name: String,
        storage_key: String,
        size_bytes: i64,
        content_type: Option<String>,
        uploaded_by: Option<Uuid>,
    ) -> Result<FileRow, SqlxError>;
    async fn list_by_asset(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<FileRow>, SqlxError>;
    async fn get_by_id(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        file_id: Uuid,
    ) -> Result<Option<FileRow>, SqlxError>;
}

pub struct FileMetadataRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl FileMetadataRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        Self { db_conn: conn }
    }
}

#[async_trait]
impl FileMetadataRepositoryTrait for FileMetadataRepositoryImpl {
    async fn insert(
        &self,
        id: Uuid,
        asset_id: Uuid,
        workspace_id: Uuid,
        file_name: String,
        storage_key: String,
        size_bytes: i64,
        content_type: Option<String>,
        uploaded_by: Option<Uuid>,
    ) -> Result<FileRow, SqlxError> {
        sqlx::query_as(
            r#"INSERT INTO files(
                 id, asset_id, workspace_id, file_name, storage_key,
                 size_bytes, content_type, uploaded_by
               )
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING *"#,
        )
        .bind(id)
        .bind(asset_id)
        .bind(workspace_id)
        .bind(file_name)
        .bind(storage_key)
        .bind(size_bytes)
        .bind(content_type)
        .bind(uploaded_by)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn list_by_asset(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<FileRow>, SqlxError> {
        sqlx::query_as(
            r#"SELECT * FROM files
               WHERE workspace_id = $1 AND asset_id = $2
               ORDER BY created_at DESC"#,
        )
        .bind(workspace_id)
        .bind(asset_id)
        .fetch_all(self.db_conn.get_pool())
        .await
    }

    async fn get_by_id(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        file_id: Uuid,
    ) -> Result<Option<FileRow>, SqlxError> {
        sqlx::query_as(
            r#"SELECT * FROM files
               WHERE workspace_id = $1 AND asset_id = $2 AND id = $3"#,
        )
        .bind(workspace_id)
        .bind(asset_id)
        .bind(file_id)
        .fetch_optional(self.db_conn.get_pool())
        .await
    }
}
