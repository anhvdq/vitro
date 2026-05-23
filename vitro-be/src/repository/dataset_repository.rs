use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use sqlx::{Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::pg_database::{PgDatabase, PgDatabaseTrait};
use crate::model::dataset::{CreateDatasetDTO, DatasetListFilter, DatasetRow, UpdateDatasetDTO};

pub type DatasetRepository = Arc<dyn DatasetRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait DatasetRepositoryTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        owner_user_id: Uuid,
        dto: CreateDatasetDTO,
    ) -> Result<DatasetRow, SqlxError>;
    async fn get(&self, workspace_id: Uuid, dataset_id: Uuid) -> Result<DatasetRow, SqlxError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: DatasetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<DatasetRow>, i64), SqlxError>;
    async fn update(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        dto: UpdateDatasetDTO,
    ) -> Result<DatasetRow, SqlxError>;
    async fn archive_active(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
    ) -> Result<Option<DatasetRow>, SqlxError>;
}

pub struct DatasetRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl DatasetRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        Self { db_conn: conn }
    }
}

#[async_trait]
impl DatasetRepositoryTrait for DatasetRepositoryImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        owner_user_id: Uuid,
        dto: CreateDatasetDTO,
    ) -> Result<DatasetRow, SqlxError> {
        sqlx::query_as(
            r#"WITH inserted AS (
                 INSERT INTO datasets(workspace_id, name, display_name, description, owner_user_id, tags)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING *
               )
               SELECT inserted.*,
                      COALESCE(u.name, u.email, inserted.owner_user_id::text) AS owner_user_name
               FROM inserted
               LEFT JOIN users u ON (u.id = inserted.owner_user_id)"#,
        )
        .bind(workspace_id)
        .bind(dto.name)
        .bind(dto.display_name)
        .bind(dto.description)
        .bind(owner_user_id)
        .bind(dto.tags)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn get(&self, workspace_id: Uuid, dataset_id: Uuid) -> Result<DatasetRow, SqlxError> {
        sqlx::query_as(
            r#"SELECT d.*,
                      COALESCE(u.name, u.email, d.owner_user_id::text) AS owner_user_name
               FROM datasets d
               LEFT JOIN users u ON (u.id = d.owner_user_id)
               WHERE d.workspace_id = $1 AND d.id = $2"#,
        )
        .bind(workspace_id)
        .bind(dataset_id)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: DatasetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<DatasetRow>, i64), SqlxError> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::bigint FROM datasets d WHERE d.workspace_id = ");
        count_qb.push_bind(workspace_id);

        let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#" SELECT
                    d.*,
                    COALESCE(u.name, u.email, d.owner_user_id::text) AS owner_user_name
                FROM datasets d
                LEFT JOIN users u ON (u.id = d.owner_user_id)
                WHERE d.workspace_id = "#
                .trim(),
        );
        data_qb.push_bind(workspace_id);

        if let Some(status) = filter.status {
            count_qb.push(" AND d.status = ").push_bind(status);
            data_qb.push(" AND d.status = ").push_bind(status);
        }

        if let Some(search) = filter.search {
            let search = search.trim().to_string();
            if !search.is_empty() {
                count_qb
                    .push(" AND (d.name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%' OR d.display_name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%')");
                data_qb
                    .push(" AND (d.name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%' OR d.display_name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%')");
            }
        }

        if !filter.tags.is_empty() {
            count_qb
                .push(" AND d.tags && ")
                .push_bind(&filter.tags)
                .push("::text[]");
            data_qb
                .push(" AND d.tags && ")
                .push_bind(&filter.tags)
                .push("::text[]");
        }

        let sort_field = match sort_field.as_str() {
            "created_at" | "updated_at" | "display_name" => sort_field,
            _ => "created_at".to_string(),
        };
        let sort_dir = match sort_dir.as_str() {
            "asc" => "asc",
            _ => "desc",
        };

        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        data_qb
            .push(format!(" ORDER BY d.{sort_field} {sort_dir}"))
            .push(" LIMIT ")
            .push_bind(page_size as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let total: (i64,) = count_qb
            .build_query_as()
            .fetch_one(self.db_conn.get_pool())
            .await?;
        let rows: Vec<DatasetRow> = data_qb
            .build_query_as()
            .fetch_all(self.db_conn.get_pool())
            .await?;
        Ok((rows, total.0))
    }

    async fn update(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        dto: UpdateDatasetDTO,
    ) -> Result<DatasetRow, SqlxError> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("WITH updated AS (UPDATE datasets SET ");
        let mut separated = qb.separated(", ");

        if let Some(display_name) = dto.display_name {
            separated
                .push("display_name = ")
                .push_bind_unseparated(display_name);
        }
        if let Some(description) = dto.description {
            separated
                .push("description = ")
                .push_bind_unseparated(description);
        }
        if let Some(owner_user_id) = dto.owner_user_id {
            separated
                .push("owner_user_id = ")
                .push_bind_unseparated(owner_user_id);
        }
        if let Some(tags) = dto.tags {
            separated.push("tags = ").push_bind_unseparated(tags);
        }

        separated.push("updated_at = now()");

        let query = qb
            .push(" WHERE workspace_id = ")
            .push_bind(workspace_id)
            .push(" AND id = ")
            .push_bind(dataset_id)
            .push(
                r#" RETURNING *
                )
                SELECT updated.*,
                       COALESCE(u.name, u.email, updated.owner_user_id::text) AS owner_user_name
                FROM updated
                LEFT JOIN users u ON (u.id = updated.owner_user_id)"#,
            )
            .build_query_as();
        query.fetch_one(self.db_conn.get_pool()).await
    }

    async fn archive_active(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
    ) -> Result<Option<DatasetRow>, SqlxError> {
        sqlx::query_as(
            r#"WITH updated AS (
                 UPDATE datasets
                 SET status = 'ARCHIVED', archived_at = now(), updated_at = now()
                 WHERE workspace_id = $1 AND id = $2 AND status = 'ACTIVE'
                 RETURNING *
               )
               SELECT updated.*,
                      COALESCE(u.name, u.email, updated.owner_user_id::text) AS owner_user_name
               FROM updated
               LEFT JOIN users u ON (u.id = updated.owner_user_id)"#,
        )
        .bind(workspace_id)
        .bind(dataset_id)
        .fetch_optional(self.db_conn.get_pool())
        .await
    }
}
