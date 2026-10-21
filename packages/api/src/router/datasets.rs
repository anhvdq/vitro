use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::model::auth::AuthInfo;
use crate::model::dataset::{
    CreateDatasetDTO, DatasetListFilter, DatasetStatus, ListDatasetsQueryParams, ReadDatasetDTO,
    UpdateDatasetDTO,
};
use crate::model::pagination::PaginatedResponseDTO;
use crate::router::DATASET_TAG;
use crate::service::dataset_service::DatasetService;
use crate::util::api_request::ValidatedJsonOrForm;
use crate::util::api_response::{ApiError, ApiSuccess};
use crate::util::pagination::parse_sort;
use crate::util::workspace::WorkspaceId;

pub fn routes(state: DatasetService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_dataset))
        .routes(routes!(list_datasets))
        .routes(routes!(get_dataset))
        .routes(routes!(update_dataset))
        .routes(routes!(archive_dataset))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/datasets",
    summary = "Create dataset",
    tag = DATASET_TAG,
    request_body = CreateDatasetDTO,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id")
    ),
    responses(
        (status = 201, description = "Dataset created", body = ApiSuccess<ReadDatasetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn create_dataset(
    State(service): State<DatasetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Extension(auth): Extension<AuthInfo>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<CreateDatasetDTO>,
        ApiError,
    >,
) -> Result<(axum::http::StatusCode, Json<ApiSuccess<ReadDatasetDTO>>), ApiError> {
    let created = service.create(workspace_id, &auth, dto).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiSuccess::new(created.into())),
    ))
}

#[utoipa::path(
    get,
    path = "/datasets",
    summary = "List datasets",
    tag = DATASET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Page size (default 20, max 100)"),
        ("sort" = Option<String>, Query, description = "Sort, e.g. created_at:desc"),
        ("status" = Option<DatasetStatus>, Query, description = "Filter by status"),
        ("search" = Option<String>, Query, description = "Search in name/display_name"),
        ("tags" = Option<String>, Query, description = "Comma-separated tags; matches any")
    ),
    responses(
        (status = 200, description = "Datasets fetched", body = PaginatedResponseDTO<ReadDatasetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn list_datasets(
    State(service): State<DatasetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Query(q): Query<ListDatasetsQueryParams>,
) -> Result<Json<PaginatedResponseDTO<ReadDatasetDTO>>, ApiError> {
    let page = q.pagination.page();
    let page_size = q.pagination.page_size();
    let (sort_field, sort_dir) = parse_sort(
        q.pagination.sort,
        "created_at",
        &["created_at", "updated_at", "display_name"],
    );
    let tags = q
        .tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let filter = DatasetListFilter {
        status: q.status,
        search: q.search,
        tags,
    };
    let (data, total) = service
        .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
        .await?;
    let data = data.into_iter().map(ReadDatasetDTO::from).collect();
    Ok(Json(PaginatedResponseDTO {
        data,
        page,
        page_size,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/datasets/{dataset_id}",
    summary = "Get dataset by id",
    tag = DATASET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("dataset_id" = Uuid, Path, description = "Dataset id")
    ),
    responses(
        (status = 200, description = "Dataset fetched", body = ApiSuccess<ReadDatasetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_dataset(
    State(service): State<DatasetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(dataset_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadDatasetDTO>>, ApiError> {
    service
        .get(workspace_id, dataset_id)
        .await
        .map_err(|e| e.into())
        .map(ReadDatasetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}

#[utoipa::path(
    patch,
    path = "/datasets/{dataset_id}",
    summary = "Update dataset",
    tag = DATASET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("dataset_id" = Uuid, Path, description = "Dataset id")
    ),
    request_body = UpdateDatasetDTO,
    responses(
        (status = 200, description = "Dataset updated", body = ApiSuccess<ReadDatasetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn update_dataset(
    State(service): State<DatasetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(dataset_id), _): WithRejection<Path<Uuid>, ApiError>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<UpdateDatasetDTO>,
        ApiError,
    >,
) -> Result<Json<ApiSuccess<ReadDatasetDTO>>, ApiError> {
    service
        .update(workspace_id, dataset_id, dto)
        .await
        .map_err(|e| e.into())
        .map(ReadDatasetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}

#[utoipa::path(
    delete,
    path = "/datasets/{dataset_id}",
    summary = "Archive dataset",
    tag = DATASET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("dataset_id" = Uuid, Path, description = "Dataset id")
    ),
    responses(
        (status = 200, description = "Dataset archived", body = ApiSuccess<ReadDatasetDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 410, description = "Already archived"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn archive_dataset(
    State(service): State<DatasetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(dataset_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadDatasetDTO>>, ApiError> {
    service
        .archive(workspace_id, dataset_id)
        .await
        .map_err(|e| e.into())
        .map(ReadDatasetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}
