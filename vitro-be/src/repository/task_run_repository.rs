use axum_typed_multipart::async_trait;
use sqlx::Error as SqlxError;
use sqlx::{Postgres, QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::pg_database::{PgDatabase, PgDatabaseTrait};
use crate::model::task_run::TaskRunRow;

pub type TaskRunRepository = Arc<dyn TaskRunRepositoryTrait + Send + Sync>;

#[async_trait]
pub trait TaskRunRepositoryTrait {
    async fn list_by_task(
        &self,
        workspace_id: Uuid,
        task_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TaskRunRow>, i64), SqlxError>;
    async fn get_by_id(&self, workspace_id: Uuid, run_id: Uuid) -> Result<TaskRunRow, SqlxError>;
}

pub struct TaskRunRepositoryImpl {
    db_conn: Arc<PgDatabase>,
}

impl TaskRunRepositoryImpl {
    pub fn new(conn: Arc<PgDatabase>) -> Self {
        Self { db_conn: conn }
    }
}

#[async_trait]
impl TaskRunRepositoryTrait for TaskRunRepositoryImpl {
    async fn list_by_task(
        &self,
        workspace_id: Uuid,
        task_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<TaskRunRow>, i64), SqlxError> {
        let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT COUNT(*)::bigint
               FROM task_runs tr
               JOIN tasks t ON (t.id = tr.task_id)
               WHERE t.workspace_id = "#,
        );
        count_qb.push_bind(workspace_id);
        count_qb.push(" AND tr.task_id = ").push_bind(task_id);

        let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT tr.*
               FROM task_runs tr
               JOIN tasks t ON (t.id = tr.task_id)
               WHERE t.workspace_id = "#,
        );
        data_qb.push_bind(workspace_id);
        data_qb.push(" AND tr.task_id = ").push_bind(task_id);

        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        data_qb
            .push(" ORDER BY tr.created_at DESC")
            .push(" LIMIT ")
            .push_bind(page_size as i64)
            .push(" OFFSET ")
            .push_bind(offset);

        let total: (i64,) = count_qb
            .build_query_as()
            .fetch_one(self.db_conn.get_pool())
            .await?;
        let rows: Vec<TaskRunRow> = data_qb
            .build_query_as()
            .fetch_all(self.db_conn.get_pool())
            .await?;
        Ok((rows, total.0))
    }

    async fn get_by_id(&self, workspace_id: Uuid, run_id: Uuid) -> Result<TaskRunRow, SqlxError> {
        sqlx::query_as(
            r#"SELECT tr.*
               FROM task_runs tr
               JOIN tasks t ON (t.id = tr.task_id)
               WHERE t.workspace_id = $1 AND tr.id = $2"#,
        )
        .bind(workspace_id)
        .bind(run_id)
        .fetch_one(self.db_conn.get_pool())
        .await
    }
}
