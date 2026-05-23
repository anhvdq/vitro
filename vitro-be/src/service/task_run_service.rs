use axum_typed_multipart::async_trait;

use std::sync::Arc;
use uuid::Uuid;

use crate::model::task_run::TaskRunRow;
use crate::repository::task_run_repository::TaskRunRepository;
use crate::util::api_response::ServiceError;

pub type TaskRunService = Arc<dyn TaskRunServiceTrait + Send + Sync>;

#[async_trait]
pub trait TaskRunServiceTrait {
    async fn list_by_task(
        &self,
        workspace_id: Uuid,
        task_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TaskRunRow>, i64), ServiceError>;
    async fn get_by_id(&self, workspace_id: Uuid, run_id: Uuid)
    -> Result<TaskRunRow, ServiceError>;
}

pub struct TaskRunServiceImpl {
    task_run_repository: TaskRunRepository,
}

impl TaskRunServiceImpl {
    pub fn new(task_run_repository: TaskRunRepository) -> Self {
        Self {
            task_run_repository,
        }
    }
}

#[async_trait]
impl TaskRunServiceTrait for TaskRunServiceImpl {
    async fn list_by_task(
        &self,
        workspace_id: Uuid,
        task_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TaskRunRow>, i64), ServiceError> {
        self.task_run_repository
            .list_by_task(workspace_id, task_id, page, page_size)
            .await
            .map_err(|e| e.into())
    }

    async fn get_by_id(
        &self,
        workspace_id: Uuid,
        run_id: Uuid,
    ) -> Result<TaskRunRow, ServiceError> {
        self.task_run_repository
            .get_by_id(workspace_id, run_id)
            .await
            .map_err(|e| e.into())
    }
}
