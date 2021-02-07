//! Report as IP address update to Discord.

#![warn(missing_docs)]

use async_trait::async_trait;
use netloc_core::reporter::{Data, Reporter};
pub use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Message {
    content: String,
}

/// A [`Reporter`] that sends the data to the Discord via a Webhook.
pub struct Discord {
    /// HTTP client.
    pub client: Client,

    /// Discord Webhook to use.
    pub webhook_url: String,
}

impl Discord {
    /// Create a new [`Discord`] reporter from a webhook url.
    pub fn from_url(webhook_url: impl Into<String>) -> Self {
        let client = Client::new();
        let webhook_url = webhook_url.into();
        Self {
            client,
            webhook_url,
        }
    }
}

#[async_trait]
impl Reporter for Discord {
    type Error = Box<dyn std::error::Error>;

    async fn report(&self, data: &Data) -> Result<(), Self::Error> {
        let message = Message {
            content: data.ip.clone(),
        };
        self.client
            .post(&self.webhook_url)
            .json(&message)
            .send()
            .await?;
        Ok(())
    }
}
