//! Worker loop outcomes from docs/code-design.md "Event handling and the worker loop".

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use sqlx::PgPool;
use time::Duration;
use vitro_adapters::postgres::{PgDeliveryQueue, PgEventSink};
use vitro_core::{
    domain::event::{AppendOptions, Delivery, Event},
    error::PlatformError,
    ports::events::{DeliveryQueue, EventSink},
};
use vitro_worker::claim_loop::{Ctx, Deps, Done, Handler, Registry, process};

const CONSUMER: &str = "normalize";
const KIND: &str = "RawBatchLanded";

/// Emits a follow-up event and completes, as a real handler would.
struct Publishes;

#[async_trait]
impl Handler for Publishes {
    async fn handle(&self, ctx: &Ctx) -> Result<Done, PlatformError> {
        let next = Event::caused_by(
            &ctx.delivery().event,
            "DatasetVersionPublished",
            1,
            json!({}),
        );
        let mut tx = ctx.begin().await?;
        PgEventSink::new(&mut tx)
            .append(&next, &["planner"], &AppendOptions::default())
            .await?;
        ctx.finish(tx).await
    }
}

/// Emits an event, then fails before finishing.
struct PublishesThenFails(fn() -> PlatformError);

#[async_trait]
impl Handler for PublishesThenFails {
    async fn handle(&self, ctx: &Ctx) -> Result<Done, PlatformError> {
        let next = Event::caused_by(
            &ctx.delivery().event,
            "DatasetVersionPublished",
            1,
            json!({}),
        );
        let mut tx = ctx.begin().await?;
        PgEventSink::new(&mut tx)
            .append(&next, &["planner"], &AppendOptions::default())
            .await?;
        Err((self.0)())
    }
}

struct Panics;

#[async_trait]
impl Handler for Panics {
    async fn handle(&self, _ctx: &Ctx) -> Result<Done, PlatformError> {
        panic!("bug in handler")
    }
}

fn registry(handler: Arc<dyn Handler>) -> Registry {
    let mut registry = Registry::default();
    registry.register(CONSUMER, KIND, handler);
    registry
}

async fn claim_one(pool: &PgPool, max_attempts: i32) -> (Deps, Delivery) {
    let event = Event::new(KIND, 1, None, json!({}));
    let mut tx = pool.begin().await.unwrap();
    let opts = AppendOptions {
        max_attempts,
        ..AppendOptions::default()
    };
    PgEventSink::new(&mut tx)
        .append(&event, &[CONSUMER], &opts)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let queue = Arc::new(PgDeliveryQueue::new(pool.clone()));
    let delivery = queue
        .claim(CONSUMER, Duration::minutes(1), 1)
        .await
        .unwrap()
        .remove(0);
    (
        Deps {
            pool: pool.clone(),
            queue,
        },
        delivery,
    )
}

async fn state(pool: &PgPool) -> (String, Option<String>) {
    sqlx::query_as("SELECT status, last_error FROM deliveries WHERE consumer = $1")
        .bind(CONSUMER)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn follow_up_events(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM events WHERE type = 'DatasetVersionPublished'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_successful_handler_commits_its_events_and_completion_together(pool: PgPool) {
    let (deps, delivery) = claim_one(&pool, 5).await;

    process(&deps, &registry(Arc::new(Publishes)), delivery)
        .await
        .unwrap();

    assert_eq!(state(&pool).await.0, "done");
    assert_eq!(follow_up_events(&pool).await, 1, "the cascade continues");
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_retryable_failure_is_retried_later_and_leaves_no_partial_effects(pool: PgPool) {
    let (deps, delivery) = claim_one(&pool, 5).await;
    let handler = PublishesThenFails(|| PlatformError::retryable("storage timeout"));

    process(&deps, &registry(Arc::new(handler)), delivery)
        .await
        .unwrap();

    assert_eq!(
        state(&pool).await,
        ("pending".into(), Some("storage timeout".into()))
    );
    assert_eq!(
        follow_up_events(&pool).await,
        0,
        "the failed attempt's event rolled back"
    );
    assert!(
        deps.queue
            .claim(CONSUMER, Duration::minutes(1), 1)
            .await
            .unwrap()
            .is_empty(),
        "backoff applies"
    );
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_retryable_failure_on_the_last_attempt_is_buried_with_its_reason(pool: PgPool) {
    // Fail loud: a delivery that keeps failing ends as dead, never silently dropped.
    let (deps, delivery) = claim_one(&pool, 1).await;
    let handler = PublishesThenFails(|| PlatformError::retryable("storage timeout"));

    process(&deps, &registry(Arc::new(handler)), delivery)
        .await
        .unwrap();

    let (status, error) = state(&pool).await;
    assert_eq!(status, "dead");
    assert_eq!(
        error.as_deref(),
        Some("gave up after 1 attempts: storage timeout")
    );
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_permanent_failure_is_buried_without_retrying(pool: PgPool) {
    let (deps, delivery) = claim_one(&pool, 5).await;
    let handler = PublishesThenFails(|| PlatformError::permanent("file is not a CSV"));

    process(&deps, &registry(Arc::new(handler)), delivery)
        .await
        .unwrap();

    assert_eq!(
        state(&pool).await,
        ("dead".into(), Some("file is not a CSV".into()))
    );
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_panicking_handler_is_retried_instead_of_losing_the_delivery(pool: PgPool) {
    let (deps, delivery) = claim_one(&pool, 5).await;

    process(&deps, &registry(Arc::new(Panics)), delivery)
        .await
        .unwrap();

    let (status, error) = state(&pool).await;
    assert_eq!(status, "pending");
    assert!(error.unwrap().contains("handler panicked"));
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_delivery_without_a_registered_handler_is_buried(pool: PgPool) {
    // A wiring bug must surface as a dead delivery, not an event nobody handles.
    let (deps, delivery) = claim_one(&pool, 5).await;

    process(&deps, &Registry::default(), delivery)
        .await
        .unwrap();

    assert_eq!(
        state(&pool).await,
        ("dead".into(), Some("no handler registered".into()))
    );
}
