use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Error, Pool, Postgres};

use super::settings;

pub trait PgDatabaseTrait {
    async fn init() -> Result<Self, Error>
    where
        Self: Sized;
    fn get_pool(&self) -> &Pool<Postgres>;
}

pub struct PgDatabase {
    pool: Pool<Postgres>,
}
impl PgDatabaseTrait for PgDatabase {
    async fn init() -> Result<Self, Error> {
        let host = settings::pg_database_host();
        let port = settings::pg_database_port();
        let db = settings::pg_database_db();
        let schema = settings::pg_database_schema();
        let username = settings::pg_database_username();
        let password = settings::pg_database_password();

        let mut options = PgConnectOptions::new()
            .host(&host)
            .port(port)
            .database(&db)
            .username(&username);
        if let Some(pass) = password {
            options = options.password(&pass);
        }

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |conn, _meta| {
                let schema = schema.clone();
                Box::pin(async move {
                    if let Some(schema) = schema {
                        let quoted_schema = format!("\"{}\"", &schema);
                        let sql = format!("SET search_path TO {}", quoted_schema);
                        sqlx::query(&sql).execute(conn).await?;
                    }
                    Ok(())
                })
            })
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }
    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
