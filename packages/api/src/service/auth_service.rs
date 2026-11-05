use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::rand_core::RngCore;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, password_hash};

use axum_typed_multipart::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::auth::AuthLoginResult;
use crate::{
    model::auth::AuthInfo,
    model::user::User,
    repository::{session_repository::SessionRepository, user_repository::UserRepository},
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
    async fn logout(&self, auth: &AuthInfo) -> Result<(), ServiceError>;
    async fn get_me(&self, user_id: Uuid) -> Result<User, ServiceError>;
}

pub struct AuthServiceImpl {
    session_expire_duration: u64,
    user_repository: UserRepository,
    session_repository: SessionRepository,
}

/// The stored form of a session token. The token is 256 random bits, so a plain SHA-256
/// is enough; a slow hash would only add cost to every request.
fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

impl AuthServiceImpl {
    pub fn new(
        user_repository: UserRepository,
        session_repository: SessionRepository,
        session_expire_duration: u64,
    ) -> Self {
        AuthServiceImpl {
            session_expire_duration,
            user_repository,
            session_repository,
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

    /// Starts a session and returns its token, the only copy that ever exists.
    async fn start_session(&self, user_id: Uuid) -> Result<String, ServiceError> {
        let mut bytes = [0u8; 32];
        password_hash::rand_core::OsRng.fill_bytes(&mut bytes);
        let token = URL_SAFE_NO_PAD.encode(bytes);
        self.session_repository
            .create(
                user_id,
                &hash_token(&token),
                self.session_expire_duration as f64,
            )
            .await?;
        Ok(token)
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

        let access_token = self.start_session(user.id).await?;
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
        let access_token = self.start_session(user.id).await?;
        Ok(AuthLoginResult { access_token, user })
    }

    async fn exchange(&self, provider: &str, _token: &str) -> Result<(), ServiceError> {
        // TODO: Support 'Google' provider here
        let _provider = provider.trim().to_ascii_lowercase();
        Err(ServiceError::NotImplemented)
    }

    async fn extract_auth_info(&self, token: &str) -> Result<AuthInfo, ServiceError> {
        let hash = hash_token(token);
        match self.session_repository.find_valid(&hash).await? {
            Some(auth) => Ok(auth),
            None => {
                // Drop the session if it only expired, so dead sessions do not pile up.
                self.session_repository.delete_by_hash(&hash).await?;
                Err(ServiceError::InvalidAuthToken)
            }
        }
    }

    async fn logout(&self, auth: &AuthInfo) -> Result<(), ServiceError> {
        self.session_repository
            .delete(auth.user_id, auth.session_id)
            .await?;
        Ok(())
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
