use axum::{Extension, Json, extract::State};
use axum_extra::extract::WithRejection;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::model::auth::{
    AuthExchangeRequestDTO, AuthInfo, AuthLoginRequestDTO, AuthLoginResponseDTO,
    AuthSignupRequestDTO,
};
use crate::model::user::ReadUserDTO;
use crate::router::AUTH_TAG;
use crate::service::auth_service::AuthService;
use crate::util::api_request::ValidatedJsonOrForm;
use crate::util::api_response::{ApiError, ApiSuccess};

pub fn public_routes(service: AuthService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(login))
        .routes(routes!(exchange))
        .with_state(service)
}

pub fn protected_routes(service: AuthService) -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(me)).with_state(service)
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    summary = "Sign up with email and password",
    tag = AUTH_TAG,
    request_body = AuthSignupRequestDTO,
    security(()),
    responses(
        (status = 201, description = "Signup succeeded", body = ApiSuccess<AuthLoginResponseDTO>),
        (status = 400, description = "Invalid request body"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn signup(
    State(service): State<AuthService>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<AuthSignupRequestDTO>,
        ApiError,
    >,
) -> Result<
    (
        axum::http::StatusCode,
        Json<ApiSuccess<AuthLoginResponseDTO>>,
    ),
    ApiError,
> {
    let signed_up = service
        .signup(&dto.email, &dto.password, dto.name.as_deref())
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(ApiSuccess::new(signed_up.into())),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    summary = "Login with email and password",
    tag = AUTH_TAG,
    request_body = AuthLoginRequestDTO,
    security(()),
    responses(
        (status = 200, description = "Login succeeded", body = ApiSuccess<AuthLoginResponseDTO>),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn login(
    State(service): State<AuthService>,
    WithRejection(ValidatedJsonOrForm(dto), _): WithRejection<
        ValidatedJsonOrForm<AuthLoginRequestDTO>,
        ApiError,
    >,
) -> Result<Json<ApiSuccess<AuthLoginResponseDTO>>, ApiError> {
    let logged_in = service.login(&dto.email, &dto.password).await?;
    Ok(Json(ApiSuccess::new(logged_in.into())))
}

#[utoipa::path(
    post,
    path = "/auth/exchange",
    summary = "Provider login exchange",
    tag = AUTH_TAG,
    request_body = AuthExchangeRequestDTO,
    security(()),
    responses(
        (status = 501, description = "Provider not implemented"),
        (status = 400, description = "Unsupported provider or invalid request"),
    )
)]
async fn exchange(
    State(service): State<AuthService>,
    WithRejection(Json(body), _): WithRejection<Json<AuthExchangeRequestDTO>, ApiError>,
) -> Result<(), ApiError> {
    service
        .exchange(&body.provider, &body.token)
        .await
        .map_err(|e| e.into())
}

#[utoipa::path(
    get,
    path = "/auth/me",
    summary = "Get current user",
    tag = AUTH_TAG,
    responses(
        (status = 200, description = "Fetched successfully", body = ApiSuccess<ReadUserDTO>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
async fn me(
    State(service): State<AuthService>,
    Extension(auth): Extension<AuthInfo>,
) -> Result<Json<ApiSuccess<ReadUserDTO>>, ApiError> {
    let user = service.get_me(auth.user_id).await?;
    Ok(Json(ApiSuccess::new(user.into())))
}
