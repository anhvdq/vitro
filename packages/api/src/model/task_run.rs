use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::pagination::ListQueryParams;
use crate::util::time_format::format_ts;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_run_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskRunStatus {
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TaskRunRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub attempt: i32,
    pub status: TaskRunStatus,
    pub error_message: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub metrics: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadTaskRunDTO {
    pub id: Uuid,
    pub task_id: Uuid,
    pub attempt: i32,
    pub status: TaskRunStatus,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub metrics: Value,
    pub created_at: String,
}

impl From<TaskRunRow> for ReadTaskRunDTO {
    fn from(value: TaskRunRow) -> Self {
        Self {
            id: value.id,
            task_id: value.task_id,
            attempt: value.attempt,
            status: value.status,
            error_message: value.error_message,
            started_at: value.started_at.map(format_ts),
            finished_at: value.finished_at.map(format_ts),
            metrics: value.metrics,
            created_at: format_ts(value.created_at),
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ListTaskRunsQueryParams {
    #[serde(flatten)]
    pub pagination: ListQueryParams,
}
