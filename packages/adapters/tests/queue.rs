//! PostgreSQL queue invariants from docs/system-design.md "Delivery rules".

use serde_json::json;
use sqlx::PgPool;
use time::Duration;
use vitro_adapters::postgres::{self, PgDeliveryQueue, PgEventSink};
use vitro_core::{
    domain::event::{AppendOptions, Event},
    ports::events::{DeliveryQueue, EventSink},
};

const LEASE: Duration = Duration::minutes(1);

async fn publish(pool: &PgPool, consumers: &[&str], opts: AppendOptions) -> Event {
    let event = Event::new("RawBatchLanded", 1, None, json!({ "stream_id": "s1" }));
    let mut tx = pool.begin().await.unwrap();
    PgEventSink::new(&mut tx)
        .append(&event, consumers, &opts)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    event
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../api/migrations")]
async fn each_subscribed_consumer_gets_its_own_delivery(pool: PgPool) {
    // Fan-out: one event wakes every subscriber independently.
    let event = publish(&pool, &["normalize", "notify"], AppendOptions::default()).await;
    let queue = PgDeliveryQueue::new(pool.clone());

    let normalize = queue.claim("normalize", LEASE, 10).await.unwrap();
    let notify = queue.claim("notify", LEASE, 10).await.unwrap();

    assert_eq!(normalize.len(), 1);
    assert_eq!(notify.len(), 1);
    assert_eq!(
        normalize[0].event, event,
        "the claimed event round-trips unchanged"
    );
}

#[sqlx::test(migrations = "../api/migrations")]
async fn concurrent_claimers_never_get_the_same_delivery(pool: PgPool) {
    // Two workers of one role must not both process an event.
    for _ in 0..20 {
        publish(&pool, &["normalize"], AppendOptions::default()).await;
    }
    let a = PgDeliveryQueue::new(pool.clone());
    let b = PgDeliveryQueue::new(pool.clone());

    let (left, right) = tokio::join!(
        a.claim("normalize", LEASE, 15),
        b.claim("normalize", LEASE, 15)
    );
    let mut ids: Vec<_> = left
        .unwrap()
        .into_iter()
        .chain(right.unwrap())
        .map(|d| d.id)
        .collect();
    let claimed = ids.len();
    ids.sort();
    ids.dedup();

    assert_eq!(claimed, 20, "every delivery is claimed by someone");
    assert_eq!(ids.len(), 20, "no delivery is claimed twice");
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_leased_delivery_is_not_reclaimed_until_the_lease_expires(pool: PgPool) {
    // Crash recovery: a worker that dies mid-handler loses nothing once its lease runs out.
    publish(&pool, &["normalize"], AppendOptions::default()).await;
    let queue = PgDeliveryQueue::new(pool.clone());

    let first = queue
        .claim("normalize", Duration::milliseconds(200), 10)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert!(
        queue
            .claim("normalize", LEASE, 10)
            .await
            .unwrap()
            .is_empty(),
        "lease still held"
    );

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let second = queue.claim("normalize", LEASE, 10).await.unwrap();

    assert_eq!(second.len(), 1, "expired lease is claimable again");
    assert_eq!(second[0].id, first[0].id);
    assert_eq!(second[0].attempts, 2, "the crashed attempt still counts");
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_delayed_delivery_is_not_claimed_before_run_after(pool: PgPool) {
    let opts = AppendOptions {
        delay: Duration::hours(1),
        ..AppendOptions::default()
    };
    publish(&pool, &["planner"], opts).await;

    let claimed = PgDeliveryQueue::new(pool)
        .claim("planner", LEASE, 10)
        .await
        .unwrap();

    assert!(claimed.is_empty());
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_burst_with_one_dedupe_key_collapses_into_one_pending_delivery(pool: PgPool) {
    // Debounce: the planner must run a derivation once per burst, not once per event.
    let opts = AppendOptions {
        dedupe_key: Some("derivation:d1".into()),
        ..AppendOptions::default()
    };
    for _ in 0..3 {
        publish(&pool, &["planner"], opts.clone()).await;
    }

    assert_eq!(
        count(&pool, "SELECT count(*) FROM events").await,
        3,
        "every event is kept for replay"
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM deliveries").await, 1);
}

#[sqlx::test(migrations = "../api/migrations")]
async fn events_roll_back_with_the_state_change_that_caused_them(pool: PgPool) {
    // No outbox: an aborted transaction must leave neither the event nor its deliveries.
    let event = Event::new("RawBatchLanded", 1, None, json!({}));
    let mut tx = pool.begin().await.unwrap();
    PgEventSink::new(&mut tx)
        .append(&event, &["normalize"], &AppendOptions::default())
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    assert_eq!(count(&pool, "SELECT count(*) FROM events").await, 0);
    assert_eq!(count(&pool, "SELECT count(*) FROM deliveries").await, 0);
}

#[sqlx::test(migrations = "../api/migrations")]
async fn a_stale_claim_cannot_complete_or_settle_after_another_worker_reclaims(pool: PgPool) {
    // A slow worker whose lease expired must not mark the newer claim done or dead.
    publish(&pool, &["normalize"], AppendOptions::default()).await;
    let queue = PgDeliveryQueue::new(pool.clone());
    let stale = queue
        .claim("normalize", Duration::milliseconds(100), 10)
        .await
        .unwrap()
        .remove(0);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let current = queue.claim("normalize", LEASE, 10).await.unwrap().remove(0);

    let mut tx = pool.begin().await.unwrap();
    assert!(
        postgres::complete(&mut tx, &stale).await.is_err(),
        "stale completion is refused"
    );
    drop(tx);
    assert!(
        queue.bury(&stale, "late").await.is_err(),
        "stale bury is refused"
    );

    let mut tx = pool.begin().await.unwrap();
    postgres::complete(&mut tx, &current).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM deliveries WHERE status = 'done'"
        )
        .await,
        1
    );
}

#[sqlx::test(migrations = "../api/migrations")]
async fn retry_releases_the_lease_and_delays_the_next_claim(pool: PgPool) {
    publish(&pool, &["normalize"], AppendOptions::default()).await;
    let queue = PgDeliveryQueue::new(pool.clone());
    let delivery = queue.claim("normalize", LEASE, 10).await.unwrap().remove(0);

    queue
        .retry(&delivery, Duration::hours(1), "storage timeout")
        .await
        .unwrap();

    assert!(
        queue
            .claim("normalize", LEASE, 10)
            .await
            .unwrap()
            .is_empty(),
        "not before run_after"
    );
    let error: Option<String> = sqlx::query_scalar("SELECT last_error FROM deliveries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        error.as_deref(),
        Some("storage timeout"),
        "the reason is visible to operators"
    );
}
