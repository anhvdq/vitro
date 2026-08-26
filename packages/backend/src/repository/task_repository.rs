use axum_typed_multipart::async_trait;
use serde_json::Value;
use sqlx::Error as SqlxError;
use sqlx::{Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::pg_database::{PgDatabase, PgDatabaseTrait};
use crate::model::task::{CreateTaskDTO, TaskListFilter, TaskRow};

pub type TaskRepository = Arc<dyn TaskRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait TaskRepositoryTrait {
    async fn create(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        created_by: Uuid,
        dto: CreateTaskDTO,
        correlation_id: Uuid,
    ) -> Result<TaskRow, SqlxError>;
    async fn get(&self, workspace_id: Uuid, task_id: Uuid) -> Result<TaskRow, SqlxError>;
    async fn list(
        &self,
        workspace_id: Uuid,
        filter: TaskListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<TaskRow>, i64), SqlxError>;
}

pub struct TaskRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl TaskRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        Self { db_conn: conn }
    }
}

#[async_trait]
impl TaskRepositoryTrait for TaskRepositoryImpl {
    async fn create(
        &self,
        workspace_id: Uuid,
        dataset_id: Uuid,
        created_by: Uuid,
        dto: CreateTaskDTO,
        correlation_id: Uuid,
    ) -> Result<TaskRow, SqlxError> {
        let configs: Value = dto.configs;
        sqlx::query_as(
            r#"WITH inserted AS (
                 INSERT INTO tasks(
                    workspace_id, dataset_id, asset_id, task_type, status, configs,
                    correlation_id, created_by
                 )
                 VALUES ($1,$2,$3,$4,'QUEUED',$5,$6,$7)
                 RETURNING *
               )
               SELECT inserted.*,
                      COALESCE(u.name, u.email, inserted.created_by::text) AS created_by_name
               FROM inserted
               LEFT JOIN users u ON (u.id = inserted.created_by)"#,
        )
        .bind(workspace_id)
        .bind(dataset_id)
        .bind(dto.asset_id)
        .bind(dto.task_type)
        .bind(configs)
        .bind(correlation_id)
        .bind(created_by)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn get(&self, workspace_id: Uuid, task_id: Uuid) -> Result<TaskRow, SqlxError> {
        sqlx::query_as(
            r#"SELECT t.*,
                      COALESCE(u.name, u.email, t.created_by::text) AS created_by_name
               FROM tasks t
               LEFT JOIN users u ON (u.id = t.created_by)
               WHERE t.workspace_id = $1 AND t.id = $2"#,
        )
        .bind(workspace_id)
        .bind(task_id)
        .fetch_one(self.db_conn.get_pool())
        .await
    }

    async fn list(
        &self,
        workspace_id: Uuid,
        filter: TaskListFilter,
        page: u32,
        page_size: u32,
        sort_field: String,
        sort_dir: String,
    ) -> Result<(Vec<TaskRow>, i64), SqlxError> {
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::bigint FROM tasks t WHERE t.workspace_id = ");
        count_qb.push_bind(workspace_id);
        let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#" SELECT
                    t.*,
                COALESCE(u.name, u.email, t.created_by::text) AS created_by_name
                FROM tasks t
                LEFT JOIN users u ON (u.id = t.created_by)
                WHERE t.workspace_id = "#
                .trim(),
        );
        data_qb.push_bind(workspace_id);

        if let Some(asset_id) = filter.asset_id {
            count_qb.push(" AND t.asset_id = ").push_bind(asset_id);
            data_qb.push(" AND t.asset_id = ").push_bind(asset_id);
        }
        if let Some(status) = filter.status {
            count_qb.push(" AND t.status = ").push_bind(status);
            data_qb.push(" AND t.status = ").push_bind(status);
        }
        if let Some(task_type) = filter.task_type {
            count_qb.push(" AND t.task_type = ").push_bind(task_type);
            data_qb.push(" AND t.task_type = ").push_bind(task_type);
        }

        let sort_field = match sort_field.as_str() {
            "created_at" | "updated_at" => sort_field,
            _ => "created_at".to_string(),
        };
        let sort_dir = match sort_dir.as_str() {
            "asc" => "asc",
            _ => "desc",
        };

        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        data_qb
            .push(format!(" ORDER BY t.{sort_field} {sort_dir}"))
            .push(" LIMIT ")
            .push_bind(page_size as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let total: (i64,) = count_qb
            .build_query_as()
            .fetch_one(self.db_conn.get_pool())
            .await?;
        let rows: Vec<TaskRow> = data_qb
            .build_query_as()
            .fetch_all(self.db_conn.get_pool())
            .await?;
        Ok((rows, total.0))
    }
}
