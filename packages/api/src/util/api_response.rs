use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::FormRejection;
use axum_typed_multipart::TypedMultipartError;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Error as SqlxError;
use uuid::Uuid;
use validator::ValidationErrors;

///
/// Wrapper struct for success responses
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub struct ApiSuccess<T: Serialize + utoipa::ToSchema> {
    data: T,
}

impl<T: Serialize + utoipa::ToSchema> ApiSuccess<T>
where
    T: Serialize,
{
    pub(crate) fn new(data: T) -> Self {
        ApiSuccess { data }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, utoipa::ToSchema)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub correlation_id: Uuid,
}

///
/// Wrapper struct for error responses
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub correlation_id: Uuid,
}

impl ApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        ApiError {
            status,
            code: code.into(),
            message: message.into(),
            details: None,
            correlation_id: Uuid::new_v4(),
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let header_name = HeaderName::from_static("x-correlation-id");
        let header_value = HeaderValue::from_str(&self.correlation_id.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("invalid"));
        (
            self.status,
            [(header_name, header_value)],
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                    details: self.details,
                    correlation_id: self.correlation_id,
                },
            }),
        )
            .into_response()
    }
}

///
/// Error conversion
///
impl From<PathRejection> for ApiError {
    fn from(value: PathRejection) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", value.body_text())
            .with_details(json!({"source":"path","reason": value.body_text()}))
    }
}

impl From<JsonRejection> for ApiError {
    fn from(value: JsonRejection) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", value.body_text())
            .with_details(json!({"source":"json","reason": value.body_text()}))
    }
}

impl From<FormRejection> for ApiError {
    fn from(value: FormRejection) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", value.to_string())
            .with_details(json!({"source":"form","reason": value.to_string()}))
    }
}

impl From<TypedMultipartError> for ApiError {
    fn from(value: TypedMultipartError) -> Self {
        ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", value.to_string())
            .with_details(json!({"source":"multipart","reason": value.to_string()}))
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(value: ValidationErrors) -> Self {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            value.to_string(),
        )
        .with_details(json!({"errors": value.to_string()}))
    }
}

#[allow(dead_code)]
pub enum ServiceError {
    Database(String),
    NotFound(String),
    Conflict(String),
    Gone(String),
    BadRequest(String),
    FailedTokenCreation(String),
    InvalidAuthToken,
    InvalidAuthInfo,
    MissingAuthToken,
    MissingRequiredPermission(String),
    ExpiredAuthToken,
    NotImplemented,
    Unknown(String),
}

impl From<SqlxError> for ServiceError {
    fn from(val: SqlxError) -> Self {
        match val {
            SqlxError::RowNotFound => ServiceError::NotFound("Resource not found".into()),
            SqlxError::Database(db_err) => {
                if db_err.code().as_deref() == Some("23505") {
                    ServiceError::Conflict(db_err.to_string())
                } else {
                    ServiceError::Database(db_err.to_string())
                }
            }
            other => ServiceError::Unknown(other.to_string()),
        }
    }
}

impl From<ServiceError> for ApiError {
    fn from(val: ServiceError) -> Self {
        match val {
            ServiceError::Database(msg) => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                msg,
            ),
            ServiceError::NotFound(msg) => ApiError::new(StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ServiceError::Conflict(msg) => ApiError::new(StatusCode::CONFLICT, "CONFLICT", msg),
            ServiceError::Gone(msg) => ApiError::new(StatusCode::GONE, "GONE", msg),
            ServiceError::BadRequest(msg) => {
                ApiError::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", msg)
            }
            ServiceError::FailedTokenCreation(msg) => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                msg,
            ),
            ServiceError::InvalidAuthToken => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid authorization token",
            ),
            ServiceError::InvalidAuthInfo => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Invalid username or password",
            ),
            ServiceError::MissingAuthToken => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Missing authorization token",
            ),
            ServiceError::MissingRequiredPermission(permission) => ApiError::new(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                format!("Missing required permission: {permission}"),
            ),
            ServiceError::ExpiredAuthToken => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Expired authorization token",
            ),
            ServiceError::NotImplemented => ApiError::new(
                StatusCode::NOT_IMPLEMENTED,
                "NOT_IMPLEMENTED",
                "Feature is not yet implemented",
            ),
            ServiceError::Unknown(msg) => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                format!("Undefined error: {msg}"),
            ),
        }
    }
}
