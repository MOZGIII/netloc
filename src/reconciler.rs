//! [`Reconciler`] and associated types.

use netloc_core::reporter::{Data, Reporter};
use std::{convert::Infallible, net::IpAddr, time::Duration};
use thiserror::Error;

use crate::{
    ip_resolver,
    state::{self, State},
};

/// Reconciler encapulates the logic to maintain the reconciliation loop.
///
/// In a loop, it obtains the current IP address, and, if it's a new one,
/// reports it to all of the reporters.
pub struct Reconciler<E: std::fmt::Debug> {
    /// The delay between the reconciliation attempts.
    pub delay: Duration,
    /// An IP resolver.
    pub ip_resolver: ip_resolver::http::Resolver,
    /// A list of reporters to use.
    pub reporters: Vec<Box<dyn Reporter<Error = E>>>,
    /// The state to maintain.
    pub current_ip: State<IpAddr>,
}

impl<E: std::fmt::Debug> Reconciler<E> {
    /// Run the reconciliation loop.
    pub async fn run(&mut self) -> Result<Infallible, Error<E>> {
        loop {
            debug!(
                message = "waiting for the delay before reconcillation",
                delay_ms = %self.delay.as_millis(),
            );
            tokio::time::sleep(self.delay).await;
            debug!(message = "reconsiling");
            self.reconcile_once().await?;
        }
    }

    async fn reconcile_once(&mut self) -> Result<(), Error<E>> {
        let obtained_ip = self
            .ip_resolver
            .resolve()
            .await
            .map_err(Error::IpResolution)?;

        let update_effect = self.current_ip.update(obtained_ip);
        if let state::UpdateEffect::Unchanged = update_effect {
            // No changes.
            return Ok(());
        }

        let data = Data {
            ip: obtained_ip.to_string(),
        };
        self.report_all(&data).await.map_err(Error::Reporting)?;
        Ok(())
    }

    async fn report_all(&self, data: &Data) -> Result<(), E> {
        for reporter in &self.reporters {
            reporter.report(data).await?;
        }
        Ok(())
    }
}

/// A reconciler error.
#[derive(Debug, Error)]
pub enum Error<E: std::fmt::Debug> {
    /// An error occured during an IP resolution.
    #[error("IP resolution failed: {0}")]
    IpResolution(anyhow::Error),

    /// An error occured during reporting.
    #[error("reporting state update failed: {0:?}")]
    Reporting(E),
}
