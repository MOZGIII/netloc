//! Report an IP address update via an HTTP request.
//!
//! Currently plain text only, but other formats are planned.

#![warn(missing_docs)]

use async_trait::async_trait;
use netloc_core::reporter::{Data, Reporter};
pub use reqwest::Client;

/// A [`Reporter`] that sends the data to the specified URL.
pub struct HttpRequest {
    /// HTTP client.
    pub client: Client,

    /// The URL to send the request to.
    pub url: String,
}

impl HttpRequest {
    /// Create a new [`HttpRequest`] reporter from a url.
    pub fn from_url(url: impl Into<String>) -> Self {
        let client = Client::new();
        let url = url.into();
        Self { client, url }
    }
}

#[async_trait]
impl Reporter for HttpRequest {
    type Error = Box<dyn std::error::Error>;

    async fn report(&self, data: &Data) -> Result<(), Self::Error> {
        self.client
            .post(&self.url)
            .header(reqwest::header::CONTENT_TYPE, "text/plain")
            .body(data.ip.clone())
            .send()
            .await?;
        Ok(())
    }
}
