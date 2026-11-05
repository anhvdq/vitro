use async_trait::async_trait;
use time::Duration;

use crate::{
    domain::event::{AppendOptions, Delivery, Event},
    error::PlatformError,
};

/// Appends events inside the caller's unit of work, so state and events commit together.
#[async_trait]
pub trait EventSink: Send {
    async fn append(
        &mut self,
        event: &Event,
        consumers: &[&str],
        opts: &AppendOptions,
    ) -> Result<(), PlatformError>;
}

/// Claims and settles deliveries. Each call runs in its own short transaction.
#[async_trait]
pub trait DeliveryQueue: Send + Sync {
    /// Leases up to `limit` due deliveries; a lease that expires makes them claimable again.
    async fn claim(
        &self,
        consumer: &str,
        lease: Duration,
        limit: u32,
    ) -> Result<Vec<Delivery>, PlatformError>;
    /// Releases the lease and makes the delivery claimable after `delay`.
    async fn retry(
        &self,
        delivery: &Delivery,
        delay: Duration,
        reason: &str,
    ) -> Result<(), PlatformError>;
    /// Marks the delivery dead; it stays in the table with its reason.
    async fn bury(&self, delivery: &Delivery, reason: &str) -> Result<(), PlatformError>;
}
