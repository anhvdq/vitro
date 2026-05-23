use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::model::auth::AuthInfo;
use crate::model::pagination::PaginatedResponseDTO;
use crate::model::task::{
    CreateTaskDTO, ListTasksQueryParams, ReadTaskDTO, TaskListFilter, TaskStatus, TaskType,
};
use crate::router::TASK_TAG;
use crate::service::task_service::TaskService;
use crate::util::api_request::ValidatedJsonOrForm;
use crate::util::api_response::{ApiError, ApiSuccess};
use crate::util::pagination::parse_sort;
use crate::util::workspace::WorkspaceId;

pub fn routes(state: TaskService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_task))
        .routes(routes!(list_tasks))
        .routes(routes!(get_task))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/tasks",
    summary = "Create task",
    tag = TASK_TAG,
    request_body = CreateTaskDTO,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id")
    ),
    responses(
        (status = 201, description = "Task created", body = ApiSuccess<ReadTaskDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 410, description = "Asset archived"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn create_task(
    State(service): State<TaskService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Extension(auth): Extension<AuthInfo>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<CreateTaskDTO>,
        ApiError,
    >,
) -> Result<(axum::http::StatusCode, Json<ApiSuccess<ReadTaskDTO>>), ApiError> {
    let created = service.create(workspace_id, &auth, dto).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiSuccess::new(created.into())),
    ))
}

#[utoipa::path(
    get,
    path = "/tasks",
    summary = "List tasks",
    tag = TASK_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Page size (default 20, max 100)"),
        ("sort" = Option<String>, Query, description = "Sort, e.g. created_at:desc"),
        ("asset_id" = Option<Uuid>, Query, description = "Filter by asset id"),
        ("status" = Option<TaskStatus>, Query, description = "Filter by status"),
        ("task_type" = Option<TaskType>, Query, description = "Filter by task type")
    ),
    responses(
        (status = 200, description = "Tasks fetched", body = PaginatedResponseDTO<ReadTaskDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn list_tasks(
    State(service): State<TaskService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Query(q): Query<ListTasksQueryParams>,
) -> Result<Json<PaginatedResponseDTO<ReadTaskDTO>>, ApiError> {
    let page = q.pagination.page();
    let page_size = q.pagination.page_size();
    let (sort_field, sort_dir) = parse_sort(
        q.pagination.sort,
        "created_at",
        &["created_at", "updated_at"],
    );
    let filter = TaskListFilter {
        asset_id: q.asset_id,
        status: q.status,
        task_type: q.task_type,
    };
    let (data, total) = service
        .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
        .await?;
    let data = data.into_iter().map(ReadTaskDTO::from).collect();
    Ok(Json(PaginatedResponseDTO {
        data,
        page,
        page_size,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/tasks/{task_id}",
    summary = "Get task by id",
    tag = TASK_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("task_id" = Uuid, Path, description = "Task id")
    ),
    responses(
        (status = 200, description = "Task fetched", body = ApiSuccess<ReadTaskDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_task(
    State(service): State<TaskService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(task_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadTaskDTO>>, ApiError> {
    service
        .get(workspace_id, task_id)
        .await
        .map_err(|e| e.into())
        .map(ReadTaskDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}
