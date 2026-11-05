//! Generic handling of claimed deliveries (docs/code-design.md "Event handling and the worker loop").

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use time::Duration;
use uuid::Uuid;
use vitro_adapters::postgres;
use vitro_core::{
    domain::event::Delivery,
    error::{ErrorClass, PlatformError},
    ports::events::DeliveryQueue,
};

const BACKOFF_BASE: Duration = Duration::seconds(1);
const BACKOFF_CAP: Duration = Duration::minutes(5);

/// Proof that a handler completed its delivery. Only [`Ctx::finish`] can build it, so a
/// handler that forgets to complete its delivery does not compile.
pub struct Done(());

/// What a handler sees while handling one delivery.
pub struct Ctx {
    pool: PgPool,
    delivery: Delivery,
}

impl Ctx {
    pub fn delivery(&self) -> &Delivery {
        &self.delivery
    }

    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>, PlatformError> {
        self.pool
            .begin()
            .await
            .map_err(|e| PlatformError::retryable(format!("cannot begin transaction: {e}")))
    }

    /// Marks the delivery done in `tx` and commits, so the handler's effects and its
    /// completion are one atomic step.
    pub async fn finish(
        &self,
        mut tx: Transaction<'static, Postgres>,
    ) -> Result<Done, PlatformError> {
        postgres::complete(&mut tx, &self.delivery).await?;
        tx.commit()
            .await
            .map_err(|e| PlatformError::retryable(format!("cannot commit: {e}")))?;
        Ok(Done(()))
    }
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, ctx: &Ctx) -> Result<Done, PlatformError>;
}

/// Handlers keyed by consumer and event type.
#[derive(Default)]
pub struct Registry {
    handlers: HashMap<(String, String), Arc<dyn Handler>>,
}

impl Registry {
    pub fn register(&mut self, consumer: &str, kind: &str, handler: Arc<dyn Handler>) {
        self.handlers
            .insert((consumer.to_owned(), kind.to_owned()), handler);
    }

    fn get(&self, delivery: &Delivery) -> Option<Arc<dyn Handler>> {
        self.handlers
            .get(&(delivery.consumer.clone(), delivery.event.kind.clone()))
            .cloned()
    }
}

pub struct Deps {
    pub pool: PgPool,
    pub queue: Arc<dyn DeliveryQueue>,
}

/// Runs the registered handler for one claimed delivery and settles the outcome:
/// done, retried with backoff, or buried. An error here means settling itself failed.
pub async fn process(
    deps: &Deps,
    registry: &Registry,
    delivery: Delivery,
) -> Result<(), PlatformError> {
    let Some(handler) = registry.get(&delivery) else {
        // A wiring bug: fail loud instead of dropping the event.
        return deps.queue.bury(&delivery, "no handler registered").await;
    };
    let ctx = Ctx {
        pool: deps.pool.clone(),
        delivery: delivery.clone(),
    };
    // Spawned so a panicking handler becomes an error value instead of killing the role.
    let outcome = tokio::spawn(async move { handler.handle(&ctx).await.map(|_| ()) }).await;
    match outcome {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => match e.class() {
            ErrorClass::Retryable => retry_or_bury(deps, &delivery, &e.to_string()).await,
            ErrorClass::Permanent | ErrorClass::Programming => {
                deps.queue.bury(&delivery, &e.to_string()).await
            }
        },
        Err(join_error) => {
            retry_or_bury(deps, &delivery, &format!("handler panicked: {join_error}")).await
        }
    }
}

async fn retry_or_bury(
    deps: &Deps,
    delivery: &Delivery,
    reason: &str,
) -> Result<(), PlatformError> {
    if delivery.attempts >= delivery.max_attempts {
        let reason = format!("gave up after {} attempts: {reason}", delivery.attempts);
        deps.queue.bury(delivery, &reason).await
    } else {
        deps.queue
            .retry(delivery, backoff(delivery.attempts), reason)
            .await
    }
}

/// Exponential backoff from one second, capped at five minutes, plus up to 25% jitter
/// so failed deliveries do not all come back at once.
fn backoff(attempts: i32) -> Duration {
    let exponent = u32::try_from(attempts.saturating_sub(1))
        .unwrap_or(0)
        .min(16);
    let delay = (BACKOFF_BASE * 2_i32.pow(exponent)).min(BACKOFF_CAP);
    let jitter_fraction = f64::from(Uuid::new_v4().as_u128() as u16) / f64::from(u16::MAX);
    delay + delay * (0.25 * jitter_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_then_caps_at_five_minutes_plus_jitter() {
        // Each retry waits at least twice as long as the last, so a struggling dependency
        // is not hammered, and never more than the cap plus jitter.
        for attempts in 1..=5 {
            let d = backoff(attempts);
            let base = Duration::seconds(1 << (attempts - 1));
            assert!(d >= base && d <= base * 1.25, "attempt {attempts}: {d}");
        }
        let capped = backoff(30);
        assert!(
            capped >= BACKOFF_CAP && capped <= BACKOFF_CAP * 1.25,
            "{capped}"
        );
    }
}
