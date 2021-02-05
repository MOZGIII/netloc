//! An HTTP IP resolver.

use reqwest::Client;
use std::net::IpAddr;

/// An IP resolver that issues an HTTP request to a remote server and reads
/// the IP address from the response body.
///
/// The IP address is expected to be returned in plaintext.
pub struct Http {
    /// An HTTP client to use.
    pub client: Client,
    /// The URL to get the IP address from.
    pub url: String,
}

impl Http {
    /// Make an HTTP call to obtain the IP address.
    pub async fn resolve(&self) -> Result<IpAddr, anyhow::Error> {
        let res = self.client.get(&self.url).send().await?;
        let status = res.status();
        if !status.is_success() {
            anyhow::bail!("server returned an error status: {}", status);
        }
        let utf8 = res.text().await?;
        let ip_address = utf8.parse()?;
        Ok(ip_address)
    }
}
