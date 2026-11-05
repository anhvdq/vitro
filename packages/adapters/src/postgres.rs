//! PostgreSQL event queue (docs/system-design.md "Delivery rules").

use async_trait::async_trait;
use sqlx::{FromRow, PgConnection, PgPool, Postgres, Transaction};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use vitro_core::{
    domain::event::{AppendOptions, Delivery, Event},
    error::PlatformError,
    ports::events::{DeliveryQueue, EventSink},
};

fn db_error(e: sqlx::Error) -> PlatformError {
    // Connection-level failures are transient; anything else is a query or schema bug.
    match e {
        sqlx::Error::Io(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::Tls(_) => PlatformError::retryable(format!("database unavailable: {e}")),
        _ => PlatformError::programming(format!("database error: {e}")),
    }
}

/// Appends events within one open transaction.
pub struct PgEventSink<'t> {
    conn: &'t mut PgConnection,
}

impl<'t> PgEventSink<'t> {
    pub fn new(tx: &'t mut Transaction<'_, Postgres>) -> Self {
        Self { conn: &mut **tx }
    }
}

#[async_trait]
impl EventSink for PgEventSink<'_> {
    async fn append(
        &mut self,
        event: &Event,
        consumers: &[&str],
        opts: &AppendOptions,
    ) -> Result<(), PlatformError> {
        sqlx::query(
            "INSERT INTO events (id, type, version, tenant_id, occurred_at, correlation_id, causation_id, data)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(event.id)
        .bind(&event.kind)
        .bind(event.version)
        .bind(event.tenant_id)
        .bind(event.occurred_at)
        .bind(event.correlation_id)
        .bind(event.causation_id)
        .bind(&event.data)
        .execute(&mut *self.conn)
        .await
        .map_err(db_error)?;

        for consumer in consumers {
            // A pending delivery with the same dedupe key already covers this event.
            sqlx::query(
                "INSERT INTO deliveries (event_id, consumer, max_attempts, run_after, dedupe_key)
                 VALUES ($1, $2, $3, now() + make_interval(secs => $4), $5)
                 ON CONFLICT (consumer, dedupe_key) WHERE status = 'pending' AND dedupe_key IS NOT NULL
                 DO NOTHING",
            )
            .bind(event.id)
            .bind(consumer)
            .bind(opts.max_attempts)
            .bind(opts.delay.as_seconds_f64())
            .bind(&opts.dedupe_key)
            .execute(&mut *self.conn)
            .await
            .map_err(db_error)?;
        }
        Ok(())
    }
}

/// Marks `delivery` done inside the handler's transaction, so its effects and its
/// completion commit together. Fails if this claim's lease was lost to another worker,
/// which rolls the handler's effects back.
pub async fn complete(
    tx: &mut Transaction<'_, Postgres>,
    delivery: &Delivery,
) -> Result<(), PlatformError> {
    let done = sqlx::query(
        "UPDATE deliveries SET status = 'done', locked_until = NULL, updated_at = now()
         WHERE id = $1 AND attempts = $2 AND status = 'pending'",
    )
    .bind(delivery.id)
    .bind(delivery.attempts)
    .execute(&mut **tx)
    .await
    .map_err(db_error)?;
    if done.rows_affected() == 0 {
        return Err(PlatformError::retryable(format!(
            "delivery {} lost its lease before completing",
            delivery.id
        )));
    }
    Ok(())
}

#[derive(FromRow)]
struct ClaimedRow {
    id: Uuid,
    consumer: String,
    attempts: i32,
    max_attempts: i32,
    event_id: Uuid,
    #[sqlx(rename = "type")]
    kind: String,
    version: i32,
    tenant_id: Option<Uuid>,
    occurred_at: OffsetDateTime,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
    data: serde_json::Value,
}

pub struct PgDeliveryQueue {
    pool: PgPool,
}

impl PgDeliveryQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeliveryQueue for PgDeliveryQueue {
    async fn claim(
        &self,
        consumer: &str,
        lease: Duration,
        limit: u32,
    ) -> Result<Vec<Delivery>, PlatformError> {
        let rows: Vec<ClaimedRow> = sqlx::query_as(
            "WITH due AS (
               SELECT id FROM deliveries
               WHERE consumer = $1
                 AND status = 'pending'
                 AND run_after <= now()
                 AND (locked_until IS NULL OR locked_until < now())
               ORDER BY run_after, id
               FOR UPDATE SKIP LOCKED
               LIMIT $3
             ), claimed AS (
               UPDATE deliveries d
               SET locked_until = now() + make_interval(secs => $2),
                   attempts = d.attempts + 1,
                   updated_at = now()
               FROM due
               WHERE d.id = due.id
               RETURNING d.id, d.consumer, d.attempts, d.max_attempts, d.event_id
             )
             SELECT c.id, c.consumer, c.attempts, c.max_attempts, c.event_id,
                    e.type, e.version, e.tenant_id, e.occurred_at, e.correlation_id, e.causation_id, e.data
             FROM claimed c JOIN events e ON e.id = c.event_id
             ORDER BY e.occurred_at, c.id",
        )
        .bind(consumer)
        .bind(lease.as_seconds_f64())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(rows
            .into_iter()
            .map(|r| Delivery {
                id: r.id,
                consumer: r.consumer,
                attempts: r.attempts,
                max_attempts: r.max_attempts,
                event: Event {
                    id: r.event_id,
                    kind: r.kind,
                    version: r.version,
                    tenant_id: r.tenant_id,
                    occurred_at: r.occurred_at,
                    correlation_id: r.correlation_id,
                    causation_id: r.causation_id,
                    data: r.data,
                },
            })
            .collect())
    }

    async fn retry(
        &self,
        delivery: &Delivery,
        delay: Duration,
        reason: &str,
    ) -> Result<(), PlatformError> {
        let query = sqlx::query(
            "UPDATE deliveries
             SET locked_until = NULL, run_after = now() + make_interval(secs => $3), last_error = $4, updated_at = now()
             WHERE id = $1 AND attempts = $2 AND status = 'pending'",
        )
        .bind(delivery.id)
        .bind(delivery.attempts)
        .bind(delay.as_seconds_f64())
        .bind(reason);
        settle(&self.pool, delivery, query).await
    }

    async fn bury(&self, delivery: &Delivery, reason: &str) -> Result<(), PlatformError> {
        let query = sqlx::query(
            "UPDATE deliveries
             SET status = 'dead', locked_until = NULL, last_error = $3, updated_at = now()
             WHERE id = $1 AND attempts = $2 AND status = 'pending'",
        )
        .bind(delivery.id)
        .bind(delivery.attempts)
        .bind(reason);
        settle(&self.pool, delivery, query).await
    }
}

/// Runs a retry or bury guarded by the claim's attempt number, so a stale claim (its
/// lease expired and another worker reclaimed it) cannot overwrite the newer claim.
async fn settle(
    pool: &PgPool,
    delivery: &Delivery,
    query: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>,
) -> Result<(), PlatformError> {
    let result = query.execute(pool).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(PlatformError::retryable(format!(
            "delivery {} was not settled: claim {} is no longer current",
            delivery.id, delivery.attempts
        )));
    }
    Ok(())
}
