use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::pg_database::{PgDatabase, PgDatabaseTrait},
    model::user::User,
};

pub type UserRepository = Arc<dyn UserRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait UserRepositoryTrait {
    async fn get(&self, id: Uuid) -> Result<User, SqlxError>;
    async fn get_by_email(&self, email: &str) -> Result<User, SqlxError>;
    #[allow(dead_code)]
    async fn get_by_identity(&self, provider: &str, subject: &str) -> Result<User, SqlxError>;
    #[allow(dead_code)]
    async fn create_identity(
        &self,
        provider: &str,
        subject: &str,
        email: &str,
        user_id: Uuid,
    ) -> Result<(), SqlxError>;
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        name: Option<&str>,
    ) -> Result<User, SqlxError>;
}

pub struct UserRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl UserRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        UserRepositoryImpl { db_conn: conn }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepositoryImpl {
    async fn get(&self, id: Uuid) -> Result<User, SqlxError> {
        sqlx::query_as(
            r#"SELECT id, name, age, email, default_workspace_id, null as password_hash
               FROM users
               WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn get_by_email(&self, email: &str) -> Result<User, SqlxError> {
        sqlx::query_as(
            r#"SELECT id, name, age, email, default_workspace_id, password_hash
               FROM users
               WHERE email = $1"#,
        )
        .bind(email)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn get_by_identity(&self, provider: &str, subject: &str) -> Result<User, SqlxError> {
        sqlx::query_as(
            r#"SELECT u.id, u.name, u.age, u.email, u.default_workspace_id, u.password_hash
               FROM auth_identities ai
               JOIN users u ON (u.id = ai.user_id)
               WHERE ai.provider = $1 AND ai.subject = $2"#,
        )
        .bind(provider)
        .bind(subject)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn create_identity(
        &self,
        provider: &str,
        subject: &str,
        email: &str,
        user_id: Uuid,
    ) -> Result<(), SqlxError> {
        sqlx::query(
            r#"INSERT INTO auth_identities(provider, subject, email, user_id)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (provider, subject)
               DO UPDATE SET email = EXCLUDED.email, user_id = EXCLUDED.user_id"#,
        )
        .bind(provider)
        .bind(subject)
        .bind(email)
        .bind(user_id)
        .execute(self.db_conn.get_pool())
        .await?;
        Ok(())
    }

    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        name: Option<&str>,
    ) -> Result<User, SqlxError> {
        let mut tx = self.db_conn.get_pool().begin().await?;
        let workspace_id: (Uuid,) =
            sqlx::query_as(r#"INSERT INTO workspaces DEFAULT VALUES RETURNING id"#)
                .fetch_one(&mut *tx)
                .await?;

        let user: User = sqlx::query_as(
            r#"INSERT INTO users(name, email, password_hash, default_workspace_id)
                              VALUES ($1, $2, $3, $4)
                              RETURNING id, name, age, email, default_workspace_id, password_hash"#,
        )
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .bind(workspace_id.0)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO users_workspaces(user_id, workspace_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(user.id)
        .bind(workspace_id.0)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.get(user.id).await
    }
}
