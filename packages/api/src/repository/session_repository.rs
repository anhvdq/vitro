use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::pg_database::{PgDatabase, PgDatabaseTrait},
    model::auth::AuthInfo,
};

pub type SessionRepository = Arc<dyn SessionRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait SessionRepositoryTrait {
    /// Stores a session for `user_id` that expires `ttl_secs` from now.
    async fn create(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        ttl_secs: f64,
    ) -> Result<(), SqlxError>;
    /// The session owner for an unexpired session, or `None`.
    async fn find_valid(&self, token_hash: &[u8]) -> Result<Option<AuthInfo>, SqlxError>;
    /// Removes the session with this token hash; a missing session is not an error.
    async fn delete_by_hash(&self, token_hash: &[u8]) -> Result<(), SqlxError>;
    /// Removes one of the user's sessions (logout).
    async fn delete(&self, user_id: Uuid, session_id: Uuid) -> Result<(), SqlxError>;
}

pub struct SessionRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl SessionRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        SessionRepositoryImpl { db_conn: conn }
    }
}

#[async_trait]
impl SessionRepositoryTrait for SessionRepositoryImpl {
    async fn create(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        ttl_secs: f64,
    ) -> Result<(), SqlxError> {
        sqlx::query(
            r#"INSERT INTO sessions (user_id, token_hash, expires_at)
               VALUES ($1, $2, now() + make_interval(secs => $3))"#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(ttl_secs)
        .execute(self.db_conn.get_pool())
        .await
        .map(|_| ())
    }

    async fn find_valid(&self, token_hash: &[u8]) -> Result<Option<AuthInfo>, SqlxError> {
        let row: Option<(Uuid, Uuid, String)> = sqlx::query_as(
            r#"SELECT s.id, u.id, u.email
               FROM sessions s
               JOIN users u ON u.id = s.user_id
               WHERE s.token_hash = $1 AND s.expires_at > now()"#,
        )
        .bind(token_hash)
        .fetch_optional(self.db_conn.get_pool())
        .await?;
        Ok(row.map(|(session_id, user_id, email)| AuthInfo {
            session_id,
            user_id,
            email,
        }))
    }

    async fn delete_by_hash(&self, token_hash: &[u8]) -> Result<(), SqlxError> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(token_hash)
            .execute(self.db_conn.get_pool())
            .await
            .map(|_| ())
    }

    async fn delete(&self, user_id: Uuid, session_id: Uuid) -> Result<(), SqlxError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
            .bind(session_id)
            .bind(user_id)
            .execute(self.db_conn.get_pool())
            .await
            .map(|_| ())
    }
}
