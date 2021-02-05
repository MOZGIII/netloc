//! Report as IP address update to Discord.

#![warn(missing_docs)]

use async_trait::async_trait;
use netloc_core::reporter::{Data, Reporter};
pub use webhook::Webhook;

/// A [`Reporter`] that sends the data to the Discord via a Webhook.
pub struct Discord {
    /// Discord Webhook to use.
    pub webhook: Webhook,
}

#[async_trait]
impl Reporter for Discord {
    type Error = Box<dyn std::error::Error>;

    async fn report(&self, data: &Data) -> Result<(), Self::Error> {
        self.webhook
            .send(|message| message.embed(|embed| embed.field("IP", &data.ip, true)))
            .await?;
        Ok(())
    }
}
