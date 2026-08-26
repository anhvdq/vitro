use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use sqlx::{Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::pg_database::{PgDatabase, PgDatabaseTrait};
use crate::model::asset::{AssetListFilter, AssetRow, CreateAssetDTO, UpdateAssetDTO};

pub type AssetRepository = Arc<dyn AssetRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait AssetRepositoryTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        owner_user_id: Uuid,
        asset_id: Uuid,
        connector_id: Uuid,
        storage_prefix: String,
        dto: CreateAssetDTO,
    ) -> Result<AssetRow, SqlxError>;
    async fn get(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, SqlxError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: AssetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<AssetRow>, i64), SqlxError>;
    async fn update(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        dto: UpdateAssetDTO,
    ) -> Result<AssetRow, SqlxError>;
    async fn archive_active(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Option<AssetRow>, SqlxError>;
}

pub struct AssetRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl AssetRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        Self { db_conn: conn }
    }
}

#[async_trait]
impl AssetRepositoryTrait for AssetRepositoryImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        owner_user_id: Uuid,
        asset_id: Uuid,
        connector_id: Uuid,
        storage_prefix: String,
        dto: CreateAssetDTO,
    ) -> Result<AssetRow, SqlxError> {
        sqlx::query_as(
            r#"WITH inserted AS (
                 INSERT INTO assets(
                    id, workspace_id, dataset_id, name, display_name, description,
                    owner_user_id, tags,
                    connector_id, connector_type, storage_prefix, allowed_formats
                 )
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'UPLOAD',$10,ARRAY['CSV']::text[])
                 RETURNING *
               )
               SELECT inserted.*,
                      COALESCE(u.name, u.email, inserted.owner_user_id::text) AS owner_user_name
               FROM inserted
               LEFT JOIN users u ON (u.id = inserted.owner_user_id)"#,
        )
        .bind(asset_id)
        .bind(workspace_id)
        .bind(dto.dataset_id)
        .bind(dto.name)
        .bind(dto.display_name)
        .bind(dto.description)
        .bind(owner_user_id)
        .bind(dto.tags)
        .bind(connector_id)
        .bind(storage_prefix)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn get(&self, workspace_id: Uuid, asset_id: Uuid) -> Result<AssetRow, SqlxError> {
        sqlx::query_as(
            r#"SELECT a.*,
                      COALESCE(u.name, u.email, a.owner_user_id::text) AS owner_user_name
               FROM assets a
               LEFT JOIN users u ON (u.id = a.owner_user_id)
               WHERE a.workspace_id = $1 AND a.id = $2"#,
        )
        .bind(workspace_id)
        .bind(asset_id)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: AssetListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<AssetRow>, i64), SqlxError> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::bigint FROM assets a WHERE a.workspace_id = ");
        count_qb.push_bind(workspace_id);
        count_qb
            .push(" AND a.dataset_id = ")
            .push_bind(filter.dataset_id);

        let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#" SELECT
                    a.*,
                    COALESCE(u.name, u.email, a.owner_user_id::text) AS owner_user_name
                FROM assets a
                LEFT JOIN users u ON (u.id = a.owner_user_id)
                WHERE a.workspace_id = "#
                .trim(),
        );
        data_qb.push_bind(workspace_id);
        data_qb
            .push(" AND a.dataset_id = ")
            .push_bind(filter.dataset_id);

        if let Some(status) = filter.status {
            count_qb.push(" AND a.status = ").push_bind(status);
            data_qb.push(" AND a.status = ").push_bind(status);
        }

        if let Some(search) = filter.search {
            let search = search.trim().to_string();
            if !search.is_empty() {
                count_qb
                    .push(" AND (a.name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%' OR a.display_name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%')");
                data_qb
                    .push(" AND (a.name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%' OR a.display_name ILIKE '%' || ")
                    .push_bind(search.clone())
                    .push(" || '%')");
            }
        }

        if !filter.tags.is_empty() {
            count_qb
                .push(" AND a.tags && ")
                .push_bind(&filter.tags)
                .push("::text[]");
            data_qb
                .push(" AND a.tags && ")
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
            .push(format!(" ORDER BY a.{sort_field} {sort_dir}"))
            .push(" LIMIT ")
            .push_bind(page_size as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let total: (i64,) = count_qb
            .build_query_as()
            .fetch_one(self.db_conn.get_pool())
            .await?;
        let rows: Vec<AssetRow> = data_qb
            .build_query_as()
            .fetch_all(self.db_conn.get_pool())
            .await?;
        Ok((rows, total.0))
    }

    async fn update(
        &self,
        workspace_id: Uuid,
        asset_id: Uuid,
        dto: UpdateAssetDTO,
    ) -> Result<AssetRow, SqlxError> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("WITH updated AS (UPDATE assets SET ");
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
            .push_bind(asset_id)
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
        asset_id: Uuid,
    ) -> Result<Option<AssetRow>, SqlxError> {
        sqlx::query_as(
            r#"WITH updated AS (
                 UPDATE assets
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
        .bind(asset_id)
        .fetch_optional(self.db_conn.get_pool())
        .await
    }
}
