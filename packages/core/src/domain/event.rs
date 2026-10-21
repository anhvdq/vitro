use serde_json::Value;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// A domain event (docs/system-design.md "Event model"). `data` holds references only.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub id: Uuid,
    pub kind: String,
    pub version: i32,
    pub tenant_id: Option<Uuid>,
    pub occurred_at: OffsetDateTime,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub data: Value,
}

impl Event {
    /// Starts a new chain: the event is its own correlation root.
    pub fn new(
        kind: impl Into<String>,
        version: i32,
        tenant_id: Option<Uuid>,
        data: Value,
    ) -> Self {
        let id = Uuid::now_v7();
        // PostgreSQL stores microseconds; truncate so a stored event equals the one emitted.
        let now = OffsetDateTime::now_utc();
        let occurred_at = now
            .replace_nanosecond(now.microsecond() * 1_000)
            .unwrap_or(now);
        Self {
            id,
            kind: kind.into(),
            version,
            tenant_id,
            occurred_at,
            correlation_id: id,
            causation_id: None,
            data,
        }
    }

    /// An event emitted while handling `cause`: same tenant and correlation chain.
    pub fn caused_by(cause: &Event, kind: impl Into<String>, version: i32, data: Value) -> Self {
        Self {
            correlation_id: cause.correlation_id,
            causation_id: Some(cause.id),
            ..Self::new(kind, version, cause.tenant_id, data)
        }
    }
}

/// How an event is delivered to its consumers.
#[derive(Clone, Debug)]
pub struct AppendOptions {
    /// Collapses a burst: a consumer holds at most one pending delivery per key.
    pub dedupe_key: Option<String>,
    /// Delay before the delivery becomes claimable (used for debouncing).
    pub delay: Duration,
    /// Attempts before the delivery is buried as dead.
    pub max_attempts: i32,
}

impl Default for AppendOptions {
    fn default() -> Self {
        Self {
            dedupe_key: None,
            delay: Duration::ZERO,
            max_attempts: 5,
        }
    }
}

/// One consumer's claimed copy of an event.
#[derive(Clone, Debug)]
pub struct Delivery {
    pub id: Uuid,
    pub consumer: String,
    /// Attempts including the current one; also the lease token for this claim.
    pub attempts: i32,
    pub max_attempts: i32,
    pub event: Event,
}
