use axum::{
    extract::FromRequestParts,
    http::{HeaderName, request::Parts},
};
use std::future;
use uuid::Uuid;

use crate::util::api_response::{ApiError, ServiceError};

#[derive(Clone, Copy, Debug)]
pub struct WorkspaceId(pub Uuid);

impl<S> FromRequestParts<S> for WorkspaceId
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = (|| {
            let header = HeaderName::from_static("x-workspace-id");
            let raw = parts.headers.get(header).ok_or_else(|| {
                ServiceError::BadRequest("X-Workspace-Id header is required".into())
            })?;
            let raw = raw.to_str().map_err(|_| {
                ServiceError::BadRequest("X-Workspace-Id must be a valid string".into())
            })?;
            let id = Uuid::parse_str(raw)
                .map_err(|_| ServiceError::BadRequest("X-Workspace-Id must be a UUID".into()))?;
            Ok(Self(id))
        })();
        future::ready(result)
    }
}
