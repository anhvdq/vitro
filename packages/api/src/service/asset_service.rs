use axum::body::Bytes;
use axum_typed_multipart::{FieldData, async_trait};
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::asset::AssetListFilter;
use crate::model::asset::{AssetRow, AssetStatus, CreateAssetDTO, UpdateAssetDTO};
use crate::model::auth::AuthInfo;
use crate::model::file::FileRow;
use crate::repository::asset_repository::AssetRepository;
use crate::repository::dataset_repository::DatasetRepository;
use crate::service::file_service::FileService;
use crate::util::api_response::ServiceError;

pub type AssetService = Arc<dyn AssetServiceTrait + Send + Sync>;

#[async_trait]
pub trait AssetServiceTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateAssetDTO,
    ) -> Result<AssetRow, ServiceError>;
    async fn get(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, ServiceError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: AssetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<AssetRow>, i64), ServiceError>;
    async fn update(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        dto: UpdateAssetDTO,
    ) -> Result<AssetRow, ServiceError>;
    async fn archive(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, ServiceError>;
    async fn upload_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        auth: &AuthInfo,
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

pub struct AssetServiceImpl {
    asset_repository: AssetRepository,
    dataset_repository: DatasetRepository,
    file_service: FileService,
}

impl AssetServiceImpl {
    pub fn new(
        asset_repository: AssetRepository,
        dataset_repository: DatasetRepository,

        file_service: FileService,
    ) -> Self {
        Self {
            asset_repository,
            dataset_repository,
            file_service,
        }
    }

    fn validate_csv(file_name: &str) -> Result<(), ServiceError> {
        if file_name.to_ascii_lowercase().ends_with(".csv") {
            Ok(())
        } else {
            Err(ServiceError::BadRequest(
                "Only .csv uploads are supported".into(),
            ))
        }
    }
}

#[async_trait]
impl AssetServiceTrait for AssetServiceImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateAssetDTO,
    ) -> Result<AssetRow, ServiceError> {
        // Validate dataset exists in workspace and is active
        let dataset = self
            .dataset_repository
            .get(workspace_id, dto.dataset_id)
            .await
            .map_err(|e| match e {
                SqlxError::RowNotFound => ServiceError::NotFound("Dataset not found".into()),
                other => other.into(),
            })?;
        if matches!(
            dataset.status,
            crate::model::dataset::DatasetStatus::Archived
        ) {
            return Err(ServiceError::Gone("Dataset is archived".into()));
        }

        let connector_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let storage_prefix = format!(
            "ws/{workspace_id}/datasets/{dataset_id}/assets/{asset_id}",
            workspace_id = workspace_id,
            dataset_id = dto.dataset_id,
            asset_id = asset_id
        );

        let row = self
            .asset_repository
            .create(
                workspace_id,
                auth.user_id,
                asset_id,
                connector_id,
                storage_prefix,
                dto,
            )
            .await?;
        Ok(row)
    }

    async fn get(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, ServiceError> {
        self.asset_repository
            .get(workspace_id, asset_id)
            .await
            .map_err(|e| e.into())
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: AssetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<AssetRow>, i64), ServiceError> {
        self.asset_repository
            .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
            .await
            .map_err(|e| e.into())
    }

    async fn update(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        dto: UpdateAssetDTO,
    ) -> Result<AssetRow, ServiceError> {
        self.asset_repository
            .update(workspace_id, asset_id, dto)
            .await
            .map_err(|e| e.into())
    }

    async fn archive(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, ServiceError> {
        if let Some(row) = self
            .asset_repository
            .archive_active(workspace_id, asset_id)
            .await?
        {
            return Ok(row);
        }

        let existing = self.asset_repository.get(workspace_id, asset_id).await?;
        match existing.status {
            AssetStatus::Archived => Err(ServiceError::Gone("Asset already archived".into())),
            AssetStatus::Active => Err(ServiceError::Unknown("Failed to archive asset".into())),
        }
    }

    async fn upload_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        auth: &AuthInfo,
        file: FieldData<Bytes>,
    ) -> Result<FileRow, ServiceError> {
        let asset = self.asset_repository.get(workspace_id, asset_id).await?;
        if matches!(asset.status, AssetStatus::Archived) {
            return Err(ServiceError::Gone("Asset is archived".into()));
        }

        let file_name = file.metadata.file_name.as_deref().unwrap_or("upload.csv");
        Self::validate_csv(file_name)?;
        self.file_service
            .save_file(
                workspace_id,
                asset_id,
                auth.user_id,
                &asset.storage_prefix,
                file,
            )
            .await
    }

    async fn list_files(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<FileRow>, ServiceError> {
        self.file_service.list_files(workspace_id, asset_id).await
    }

    async fn download_file(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        file_id: Uuid,
    ) -> Result<(FileRow, Bytes), ServiceError> {
        self.file_service
            .download_file(workspace_id, asset_id, file_id)
            .await
    }
}
