use axum_typed_multipart::async_trait;

use std::sync::Arc;
use uuid::Uuid;

use crate::model::auth::AuthInfo;
use crate::model::dataset::{
    CreateDatasetDTO, DatasetListFilter, DatasetRow, DatasetStatus, UpdateDatasetDTO,
};
use crate::repository::dataset_repository::DatasetRepository;
use crate::util::api_response::ServiceError;

pub type DatasetService = Arc<dyn DatasetServiceTrait + Send + Sync>;

#[async_trait]
pub trait DatasetServiceTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateDatasetDTO,
    ) -> Result<DatasetRow, ServiceError>;
    async fn get(&self, workspace_id: Uuid, dataset_id: Uuid) -> Result<DatasetRow, ServiceError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: DatasetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<DatasetRow>, i64), ServiceError>;
    async fn update(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        dto: UpdateDatasetDTO,
    ) -> Result<DatasetRow, ServiceError>;
    async fn archive(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
    ) -> Result<DatasetRow, ServiceError>;
}

pub struct DatasetServiceImpl {
    dataset_repository: DatasetRepository,
}

impl DatasetServiceImpl {
    pub fn new(dataset_repository: DatasetRepository) -> Self {
        Self { dataset_repository }
    }
}

#[async_trait]
impl DatasetServiceTrait for DatasetServiceImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateDatasetDTO,
    ) -> Result<DatasetRow, ServiceError> {
        self.dataset_repository
            .create(workspace_id, auth.user_id, dto)
            .await
            .map_err(|e| e.into())
    }

    async fn get(&self, workspace_id: Uuid, dataset_id: Uuid) -> Result<DatasetRow, ServiceError> {
        self.dataset_repository
            .get(workspace_id, dataset_id)
            .await
            .map_err(|e| e.into())
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: DatasetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<DatasetRow>, i64), ServiceError> {
        self.dataset_repository
            .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
            .await
            .map_err(|e| e.into())
    }

    async fn update(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        dto: UpdateDatasetDTO,
    ) -> Result<DatasetRow, ServiceError> {
        self.dataset_repository
            .update(workspace_id, dataset_id, dto)
            .await
            .map_err(|e| e.into())
    }

    async fn archive(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
    ) -> Result<DatasetRow, ServiceError> {
        if let Some(row) = self
            .dataset_repository
            .archive_active(workspace_id, dataset_id)
            .await?
        {
            return Ok(row);
        }

        let existing = self
            .dataset_repository
            .get(workspace_id, dataset_id)
            .await?;
        match existing.status {
            DatasetStatus::Archived => Err(ServiceError::Gone("Dataset already archived".into())),
            DatasetStatus::Active => Err(ServiceError::Unknown("Failed to archive dataset".into())),
        }
    }
}
