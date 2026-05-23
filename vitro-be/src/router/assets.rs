use axum::{
    Extension, Json,
    extract::{DefaultBodyLimit, Path, Query, State},
};
use axum_extra::extract::WithRejection;
use axum_typed_multipart::TypedMultipart;
use uuid::Uuid;

use utoipa_axum::{router::OpenApiRouter, router::UtoipaMethodRouterExt, routes};

use crate::model::asset::{
    AssetListFilter, AssetStatus, CreateAssetDTO, ListAssetsQueryParams, ReadAssetDTO,
    UpdateAssetDTO,
};
use crate::model::auth::AuthInfo;
use crate::model::file::{FileResponseDTO, FileUploadDTO, UploadFileRequestDTO};
use crate::model::pagination::PaginatedResponseDTO;
use crate::router::ASSET_TAG;
use crate::service::asset_service::AssetService;
use crate::util::api_request::ValidatedJsonOrForm;
use crate::util::api_response::{ApiError, ApiSuccess};
use crate::util::pagination::parse_sort;
use crate::util::workspace::WorkspaceId;

pub fn routes(state: AssetService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_asset))
        .routes(routes!(list_assets))
        .routes(routes!(get_asset))
        .routes(routes!(update_asset))
        .routes(routes!(archive_asset))
        .routes(routes!(upload_asset_file).layer(DefaultBodyLimit::max(25 * 1024 * 1024)))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/assets",
    summary = "Create asset",
    tag = ASSET_TAG,
    request_body = CreateAssetDTO,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id")
    ),
    responses(
        (status = 201, description = "Asset created", body = ApiSuccess<ReadAssetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn create_asset(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Extension(auth): Extension<AuthInfo>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<CreateAssetDTO>,
        ApiError,
    >,
) -> Result<(axum::http::StatusCode, Json<ApiSuccess<ReadAssetDTO>>), ApiError> {
    let created = service.create(workspace_id, &auth, dto).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiSuccess::new(created.into())),
    ))
}

#[utoipa::path(
    get,
    path = "/assets",
    summary = "List assets",
    tag = ASSET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("dataset_id" = Uuid, Query, description = "Dataset id (required)"),
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Page size (default 20, max 100)"),
        ("sort" = Option<String>, Query, description = "Sort, e.g. created_at:desc"),
        ("status" = Option<AssetStatus>, Query, description = "Filter by status"),
        ("search" = Option<String>, Query, description = "Search in name/display_name"),
        ("tags" = Option<String>, Query, description = "Comma-separated tags; matches any")
    ),
    responses(
        (status = 200, description = "Assets fetched", body = PaginatedResponseDTO<ReadAssetDTO>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn list_assets(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    Query(q): Query<ListAssetsQueryParams>,
) -> Result<Json<PaginatedResponseDTO<ReadAssetDTO>>, ApiError> {
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
    let filter = AssetListFilter {
        dataset_id: q.dataset_id,
        status: q.status,
        search: q.search,
        tags,
    };
    let (data, total) = service
        .list(workspace_id, filter, page, page_size, sort_field, sort_dir)
        .await?;
    let data = data.into_iter().map(ReadAssetDTO::from).collect();
    Ok(Json(PaginatedResponseDTO {
        data,
        page,
        page_size,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/assets/{asset_id}",
    summary = "Get asset by id",
    tag = ASSET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("asset_id" = Uuid, Path, description = "Asset id")
    ),
    responses(
        (status = 200, description = "Asset fetched", body = ApiSuccess<ReadAssetDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn get_asset(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(asset_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadAssetDTO>>, ApiError> {
    service
        .get(workspace_id, asset_id)
        .await
        .map_err(|e| e.into())
        .map(ReadAssetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}

#[utoipa::path(
    patch,
    path = "/assets/{asset_id}",
    summary = "Update asset",
    tag = ASSET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("asset_id" = Uuid, Path, description = "Asset id")
    ),
    request_body = UpdateAssetDTO,
    responses(
        (status = 200, description = "Asset updated", body = ApiSuccess<ReadAssetDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn update_asset(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(asset_id), _): WithRejection<Path<Uuid>, ApiError>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<UpdateAssetDTO>,
        ApiError,
    >,
) -> Result<Json<ApiSuccess<ReadAssetDTO>>, ApiError> {
    service
        .update(workspace_id, asset_id, dto)
        .await
        .map_err(|e| e.into())
        .map(ReadAssetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}

#[utoipa::path(
    delete,
    path = "/assets/{asset_id}",
    summary = "Archive asset",
    tag = ASSET_TAG,
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("asset_id" = Uuid, Path, description = "Asset id")
    ),
    responses(
        (status = 200, description = "Asset archived", body = ApiSuccess<ReadAssetDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 410, description = "Already archived"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn archive_asset(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(asset_id), _): WithRejection<Path<Uuid>, ApiError>,
) -> Result<Json<ApiSuccess<ReadAssetDTO>>, ApiError> {
    service
        .archive(workspace_id, asset_id)
        .await
        .map_err(|e| e.into())
        .map(ReadAssetDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/assets/{asset_id}/upload",
    summary = "Upload a file for an asset",
    tag = ASSET_TAG,
    request_body(content = UploadFileRequestDTO, content_type = "multipart/form-data"),
    params(
        ("X-Workspace-Id" = String, Header, description = "Workspace id"),
        ("asset_id" = Uuid, Path, description = "Asset id")
    ),
    responses(
        (status = 200, description = "File uploaded", body = ApiSuccess<FileResponseDTO>),
        (status = 400, description = "Invalid upload"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 410, description = "Asset archived"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn upload_asset_file(
    State(service): State<AssetService>,
    WorkspaceId(workspace_id): WorkspaceId,
    WithRejection(Path(asset_id), _): WithRejection<Path<Uuid>, ApiError>,
    WithRejection(TypedMultipart(dto), _): WithRejection<TypedMultipart<FileUploadDTO>, ApiError>,
) -> Result<Json<ApiSuccess<FileResponseDTO>>, ApiError> {
    service
        .upload_file(workspace_id, asset_id, dto.file)
        .await
        .map_err(|e| e.into())
        .map(FileResponseDTO::from)
        .map(ApiSuccess::new)
        .map(Json)
}
