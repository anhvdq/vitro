use uuid::Uuid;
use validator::Validate;

use crate::model::user::{ReadUserDTO, User};

#[derive(Clone)]
pub struct AuthInfo {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub email: String,
}

#[derive(Debug)]
pub struct AuthLoginResult {
    pub access_token: String,
    pub user: User,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate, utoipa::ToSchema)]
pub struct AuthSignupRequestDTO {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate, utoipa::ToSchema)]
pub struct AuthLoginRequestDTO {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthLoginResponseDTO {
    pub access_token: String,
    pub user: ReadUserDTO,
}

impl From<AuthLoginResult> for AuthLoginResponseDTO {
    fn from(result: AuthLoginResult) -> Self {
        AuthLoginResponseDTO {
            access_token: result.access_token,
            user: result.user.into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate, utoipa::ToSchema)]
pub struct AuthExchangeRequestDTO {
    /// OAuth provider identifier.
    pub provider: String,
    /// Provider-issued token.
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppClaims {
    pub sub: Uuid,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}
