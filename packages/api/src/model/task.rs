use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::model::pagination::ListQueryParams;
use crate::util::time_format::format_ts;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_type", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    FullAnalysis,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub dataset_id: Uuid,
    pub asset_id: Uuid,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub configs: Value,
    pub correlation_id: Uuid,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateTaskDTO {
    pub asset_id: Uuid,
    pub task_type: TaskType,
    #[serde(default)]
    pub configs: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadTaskDTO {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub dataset_id: Uuid,
    pub asset_id: Uuid,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub configs: Value,
    pub correlation_id: Uuid,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TaskRow> for ReadTaskDTO {
    fn from(value: TaskRow) -> Self {
        Self {
            id: value.id,
            workspace_id: value.workspace_id,
            dataset_id: value.dataset_id,
            asset_id: value.asset_id,
            task_type: value.task_type,
            status: value.status,
            configs: value.configs,
            correlation_id: value.correlation_id,
            created_by: value.created_by,
            created_by_name: value.created_by_name,
            created_at: format_ts(value.created_at),
            updated_at: format_ts(value.updated_at),
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ListTasksQueryParams {
    #[serde(flatten)]
    pub pagination: ListQueryParams,
    pub asset_id: Option<Uuid>,
    pub status: Option<TaskStatus>,
    pub task_type: Option<TaskType>,
}

#[derive(Debug, Clone)]
pub struct TaskListFilter {
    pub asset_id: Option<Uuid>,
    pub status: Option<TaskStatus>,
    pub task_type: Option<TaskType>,
}
