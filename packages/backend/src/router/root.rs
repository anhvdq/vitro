use std::sync::Arc;

use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::from_fn_with_state;
use axum::{Router, routing::IntoMakeService};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

use crate::config::settings::{jwt_expire_duration, jwt_secret};
use crate::middleware::auth::authorize;
use crate::repository::file_metadata_repository::{
    FileMetadataRepository, FileMetadataRepositoryImpl,
};
use crate::repository::file_repository::{FileRepository, FileRepositoryImpl};
use crate::repository::user_repository::UserRepository;
use crate::repository::{
    asset_repository::{AssetRepository, AssetRepositoryImpl},
    dataset_repository::{DatasetRepository, DatasetRepositoryImpl},
    task_repository::{TaskRepository, TaskRepositoryImpl},
    task_run_repository::{TaskRunRepository, TaskRunRepositoryImpl},
};
use crate::router::{ASSET_TAG, AUTH_TAG, DATASET_TAG, TASK_RUN_TAG, TASK_TAG};
use crate::service::auth_service::{AuthService, AuthServiceImpl};
use crate::service::file_service::{FileService, FileServiceImpl};
use crate::{
    config::object_store::ObjectStore,
    config::pg_database::PgDatabase,
    repository::user_repository::UserRepositoryImpl,
    service::{
        asset_service::{AssetService, AssetServiceImpl},
        dataset_service::{DatasetService, DatasetServiceImpl},
        task_run_service::{TaskRunService, TaskRunServiceImpl},
        task_service::{TaskService, TaskServiceImpl},
    },
};

use super::{assets, auth, datasets, task_runs, tasks};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = AUTH_TAG, description = "Auth API"),
        (name = DATASET_TAG, description = "Datasets API"),
        (name = ASSET_TAG, description = "Assets API"),
        (name = TASK_TAG, description = "Tasks API"),
        (name = TASK_RUN_TAG, description = "Task Runs API")
    ),
    security(
        ("bearerAuth" = [])
    ),
    modifiers(&BearerSecurity)
)]
struct ApiDoc;

struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // components is guaranteed to be present
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

const SCALAR_CUSTOM_HTML: &str = r#"
    <!doctype html>
    <html>
      <head>
        <title>Scalar</title>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body>
        <div id="app"></div>

        <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
        <script>
          Scalar.createApiReference("\#app", {
            content: $spec,
            "hideClientButton": true,
            "hideModels": true,
            "defaultOpenAllTags": true,
            "expandAllModelSections": false,
            "expandAllResponses": true,
            "showDeveloperTools": "localhost",
            "showToolbar": "localhost",
            "operationTitleSource": "summary",
            "agent": { "disabled": true },
          });
        </script>
      </body>
    </html>
"#;

pub fn routes(db_conn: Arc<PgDatabase>, object_store: Arc<ObjectStore>) -> IntoMakeService<Router> {
    // Initialize user service
    let user_repository: UserRepository = Arc::new(UserRepositoryImpl::new(Arc::clone(&db_conn)));

    let auth_service: AuthService = Arc::new(AuthServiceImpl::new(
        Arc::clone(&user_repository),
        jwt_secret(),
        jwt_expire_duration(),
    ));

    let file_repository: FileRepository =
        Arc::new(FileRepositoryImpl::new(Arc::clone(&object_store)));
    let file_metadata_repository: FileMetadataRepository =
        Arc::new(FileMetadataRepositoryImpl::new(Arc::clone(&db_conn)));
    let file_service: FileService = Arc::new(FileServiceImpl::new(
        Arc::clone(&file_repository),
        Arc::clone(&file_metadata_repository),
    ));

    let dataset_repository: DatasetRepository =
        Arc::new(DatasetRepositoryImpl::new(Arc::clone(&db_conn)));
    let dataset_service: DatasetService =
        Arc::new(DatasetServiceImpl::new(Arc::clone(&dataset_repository)));

    let asset_repository: AssetRepository =
        Arc::new(AssetRepositoryImpl::new(Arc::clone(&db_conn)));
    let asset_service: AssetService = Arc::new(AssetServiceImpl::new(
        Arc::clone(&asset_repository),
        Arc::clone(&dataset_repository),
        Arc::clone(&file_service),
    ));

    let task_repository: TaskRepository = Arc::new(TaskRepositoryImpl::new(Arc::clone(&db_conn)));
    let task_service: TaskService = Arc::new(TaskServiceImpl::new(
        Arc::clone(&task_repository),
        Arc::clone(&asset_repository),
    ));

    let task_run_repository: TaskRunRepository =
        Arc::new(TaskRunRepositoryImpl::new(Arc::clone(&db_conn)));
    let task_run_service: TaskRunService =
        Arc::new(TaskRunServiceImpl::new(Arc::clone(&task_run_repository)));

    let protected = OpenApiRouter::new()
        .merge(auth::protected_routes(Arc::clone(&auth_service)))
        .merge(datasets::routes(Arc::clone(&dataset_service)))
        .merge(assets::routes(Arc::clone(&asset_service)))
        .merge(tasks::routes(Arc::clone(&task_service)))
        .merge(task_runs::routes(Arc::clone(&task_run_service)))
        .route_layer(from_fn_with_state(Arc::clone(&auth_service), authorize));

    let public = OpenApiRouter::new()
        .routes(routes!(health_check))
        .merge(auth::public_routes(Arc::clone(&auth_service)));
    let (app_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(public)
        .merge(protected)
        .split_for_parts();
    let app_router =
        app_router.merge(Scalar::with_url("/scalar", api).custom_html(SCALAR_CUSTOM_HTML));

    app_router
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_origin(["http://localhost:5173".parse::<HeaderValue>().unwrap()])
                .allow_headers([
                    AUTHORIZATION,
                    ACCEPT,
                    CONTENT_TYPE,
                    "X-Workspace-Id".parse::<HeaderName>().unwrap(),
                ]),
        )
        .layer(TraceLayer::new_for_http()) // Enable logging
        .into_make_service()
}

#[utoipa::path(
    get,
    path = "/health-check",
    summary = "Health check",
    tag = "Misc",
    responses(
        (status = 200, description = "Health check", body = ())
    ),
    security(())
)]
async fn health_check() -> String {
    format!("App version: {}", env!("CARGO_PKG_VERSION"))
}
