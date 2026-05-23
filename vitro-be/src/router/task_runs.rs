use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::model::pagination::PaginatedResponseDTO;
use crate::model::task_run::{ListTaskRunsQueryParams, ReadTaskRunDTO};
use crate::router::TASK_RUN_TAG;
use crate::service::task_run_service::TaskRunService;
use crate::util::api_response::{ApiError, ApiSuccess};
use crate::util::workspace::WorkspaceId;

pub fn routes(state: TaskRunService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(list_task_runs))
        .routes(routes!(get_task_run))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/tasks/{task_id}/runs",
    summary = "List task runs by task",
    tag = TASK_RUN_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("task_id" = Uuid, Path, description = "Task id"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Page size (default 20, max 100)")
    ),
    responses(
        (status = 200, description = "Task runs fetched", body = PaginatedResponseDTO<ReadTaskRunDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn list_task_runs(
    State(service): State<TaskRunService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(task_id), _): WithRejection<Path<Uuid>, ApiError>,
    Query(q): Query<ListTaskRunsQueryParams>,
) -> Result<Json<PaginatedResponseDTO<ReadTaskRunDTO>>, ApiError> {
    let page = q.pagination.page();
    let page_size = q.pagination.page_size();
    let (data, total) = service
        .list_by_task(workspace_id, task_id, page, page_size)
        .await?;
    let data = data.into_iter().map(ReadTaskRunDTO::from).collect();
    Ok(Json(PaginatedResponseDTO {
        data,
        page,
        page_size,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/task-runs/{run_id}",
    summary = "Get task run by id",
    tag = TASK_RUN_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("run_id" = Uuid, Path, description = "Run id")
    ),
    responses(
        (status = 200, description = "Task run fetched", body = ApiSuccess<ReadTaskRunDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_task_run(
    State(service): State<TaskRunService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(run_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadTaskRunDTO>>, ApiError> {
    service
        .get_by_id(workspace_id, run_id)
        .await
        .map_err(|e| e.into())
        .map(ReadTaskRunDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}
