use axum_typed_multipart::async_trait;

use std::sync::Arc;
use uuid::Uuid;

use crate::model::auth::AuthInfo;
use crate::model::task::{CreateTaskDTO, TaskListFilter, TaskRow};
use crate::repository::asset_repository::AssetRepository;
use crate::repository::task_repository::TaskRepository;
use crate::util::api_response::ServiceError;

pub type TaskService = Arc<dyn TaskServiceTrait + Send + Sync>;

#[async_trait]
pub trait TaskServiceTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateTaskDTO,
    ) -> Result<TaskRow, ServiceError>;
    async fn get(&self, workspace_id: Uuid, task_id: Uuid) -> Result<TaskRow, ServiceError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: TaskListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<TaskRow>, i64), ServiceError>;
}

pub struct TaskServiceImpl {
    task_repository: TaskRepository,
    asset_repository: AssetRepository,
}

impl TaskServiceImpl {
    pub fn new(task_repository: TaskRepository, asset_repository: AssetRepository) -> Self {
        Self {
            task_repository,
            asset_repository,
        }
    }
}

#[async_trait]
impl TaskServiceTrait for TaskServiceImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        auth: &AuthInfo,
        dto: CreateTaskDTO,
    ) -> Result<TaskRow, ServiceError> {
        let asset = self
            .asset_repository
            .get(workspace_id, dto.asset_id)
            .await?;
        if matches!(asset.status, crate::model::asset::AssetStatus::Archived) {
            return Err(ServiceError::Gone("Asset is archived".into()));
        }

        let correlation_id = Uuid::new_v4();
        self.task_repository
            .create(
                workspace_id,
                asset.dataset_id,
                auth.user_id,
                dto,
                correlation_id,
            )
            .await
            .map_err(|e| e.into())
    }

    async fn get(&self, workspace_id: Uuid, task_id: Uuid) -> Result<TaskRow, ServiceError> {
        self.task_repository
            .get(workspace_id, task_id)
            .await
            .map_err(|e| e.into())
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: TaskListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<TaskRow>, i64), ServiceError> {
        self.task_repository
            .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
            .await
            .map_err(|e| e.into())
    }
}
