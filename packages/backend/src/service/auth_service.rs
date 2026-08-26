use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, password_hash};

use axum_typed_multipart::async_trait;
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};
use sqlx::Error as SqlxError;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::model::auth::AuthLoginResult;
use crate::{
    model::auth::{AppClaims, AuthInfo},
    model::user::User,
    repository::user_repository::UserRepository,
    util::api_response::ServiceError,
};

pub type AuthService = Arc<dyn AuthServiceTrait + Send + Sync>;

#[async_trait]
pub trait AuthServiceTrait {
    async fn signup(
        &self,
        email: &str,
        password: &str,
        name: Option<&str>,
    ) -> Result<AuthLoginResult, ServiceError>;
    async fn login(&self, email: &str, password: &str) -> Result<AuthLoginResult, ServiceError>;
    async fn exchange(&self, provider: &str, token: &str) -> Result<(), ServiceError>;
    async fn extract_auth_info(&self, token: &str) -> Result<AuthInfo, ServiceError>;
    async fn get_me(&self, user_id: Uuid) -> Result<User, ServiceError>;
}

pub struct AuthServiceImpl {
    app_secret: String,
    app_expire_duration: u64,
    user_repository: UserRepository,
}

impl AuthServiceImpl {
    pub fn new(
        user_repository: UserRepository,
        app_secret: String,
        app_expire_duration: u64,
    ) -> Self {
        AuthServiceImpl {
            app_secret,
            app_expire_duration,
            user_repository,
        }
    }

    fn normalize_email(&self, email: &str) -> String {
        email.trim().to_ascii_lowercase()
    }

    fn normalize_name(&self, name: Option<&str>) -> Option<String> {
        name.map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
    }

    fn issue_app_token(&self, user_id: Uuid, email: &str) -> Result<String, ServiceError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Return the epoch time");
        let iat = now.as_secs() as i64;
        let exp = (now + Duration::from_secs(self.app_expire_duration)).as_secs() as i64;
        let claims = AppClaims {
            sub: user_id,
            email: String::from(email),
            iat,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.app_secret.as_ref()),
        )
        .map_err(|e| ServiceError::FailedTokenCreation(e.to_string()))
    }

    fn verify_app_token(&self, token: &str) -> Result<AppClaims, ServiceError> {
        decode::<AppClaims>(
            token,
            &DecodingKey::from_secret(self.app_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => ServiceError::ExpiredAuthToken,
            _ => ServiceError::InvalidAuthToken,
        })
        .map(|t| t.claims)
    }

    fn hash_password(&self, password: &str) -> Result<String, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ServiceError::Unknown(format!("Failed to hash password: {e}")))
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), ServiceError> {
        let parsed = PasswordHash::new(password_hash).map_err(|e| {
            ServiceError::Unknown(format!("Failed to parse stored password hash: {e}"))
        })?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|e| match e {
                password_hash::Error::Password => ServiceError::InvalidAuthInfo,
                _ => ServiceError::Unknown(format!("Failed to verify password: {e}")),
            })
    }
}

#[async_trait]
impl AuthServiceTrait for AuthServiceImpl {
    async fn signup(
        &self,
        email: &str,
        password: &str,
        name: Option<&str>,
    ) -> Result<AuthLoginResult, ServiceError> {
        let email = self.normalize_email(email);
        let name = self.normalize_name(name);
        let password_hash = self.hash_password(password)?;

        let user = self
            .user_repository
            .create_user(&email, &password_hash, name.as_deref())
            .await
            .map_err(|e| match e {
                SqlxError::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                    ServiceError::Conflict(String::from("Email already exists"))
                }
                other => other.into(),
            })?;

        let access_token = self.issue_app_token(user.id, &user.email)?;
        Ok(AuthLoginResult { access_token, user })
    }

    async fn login(&self, email: &str, password: &str) -> Result<AuthLoginResult, ServiceError> {
        let normalized_email = self.normalize_email(email);
        let auth_user = self
            .user_repository
            .get_by_email(&normalized_email)
            .await
            .map_err(|e| match e {
                SqlxError::RowNotFound => ServiceError::InvalidAuthInfo,
                other => other.into(),
            })?;

        let password_hash = auth_user
            .password_hash
            .as_deref()
            .ok_or(ServiceError::InvalidAuthInfo)?;
        self.verify_password(password, password_hash)?;

        let user = auth_user;
        let access_token = self.issue_app_token(user.id, &user.email)?;
        Ok(AuthLoginResult { access_token, user })
    }

    async fn exchange(&self, provider: &str, _token: &str) -> Result<(), ServiceError> {
        // TODO: Support 'Google' provider here
        let _provider = provider.trim().to_ascii_lowercase();
        Err(ServiceError::NotImplemented)
    }

    async fn extract_auth_info(&self, token: &str) -> Result<AuthInfo, ServiceError> {
        match self.verify_app_token(token) {
            Ok(app) => Ok(AuthInfo {
                user_id: app.sub,
                email: app.email,
            }),
            Err(_) => Err(ServiceError::InvalidAuthToken),
        }
    }

    async fn get_me(&self, user_id: Uuid) -> Result<User, ServiceError> {
        self.user_repository
            .get(user_id)
            .await
            .map_err(|e| match e {
                SqlxError::RowNotFound => ServiceError::NotFound(String::from("User not found")),
                other => other.into(),
            })
    }
}
