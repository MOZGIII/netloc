//! An HTTP IP resolver.

use std::net::IpAddr;

use bytes::BytesMut;
use hyper::body::HttpBody;
use hyper_system_resolver::addr_info_hints::AddressFamily;

#[cfg(unix)]
use libc::AI_ALL;

#[cfg(windows)]
use winapi::shared::ws2def::AI_ALL;

/// Effective hyper DNS resolver type.
pub type DnsResolver = hyper_system_resolver::system::Resolver;
/// Effective hyper connector type.
pub type Connector = hyper_tls::HttpsConnector<hyper::client::HttpConnector<DnsResolver>>;
/// Effective hyper client type.
pub type Client = hyper::client::Client<Connector>;

/// Build a [`Client`] with opinionated settings.
pub fn build_client(resolver_address_family: AddressFamily) -> Client {
    let mut addr_info_hints: dns_lookup::AddrInfoHints = hyper_system_resolver::AddrInfoHints {
        address_family: resolver_address_family,
    }
    .into();
    addr_info_hints.flags = AI_ALL;

    let dns_resovler =
        hyper_system_resolver::system::Resolver::new(hyper_system_resolver::system::System {
            addr_info_hints: Some(addr_info_hints),
            service: None,
        });

    let mut http_connector = hyper::client::HttpConnector::new_with_resolver(dns_resovler);
    http_connector.set_happy_eyeballs_timeout(None);
    http_connector.enforce_http(false);

    let mut https_connector = hyper_tls::HttpsConnector::new_with_connector(http_connector);
    https_connector.https_only(false);

    hyper::client::Client::builder().build(https_connector)
}

/// An IP resolver that issues an HTTP request to a remote server and reads
/// the IP address from the response body.
///
/// The IP address is expected to be returned in plaintext.
pub struct Resolver {
    /// An HTTP client to use.
    pub client: Client,
    /// The URL to get the IP address from.
    pub url: hyper::Uri,
    /// The maximum amount of bytes to read from of the HTTP response body.
    pub max_body_size: Option<usize>,
}

impl Resolver {
    /// Make an HTTP call to obtain the IP address.
    pub async fn resolve(&self) -> Result<IpAddr, anyhow::Error> {
        let res = self.client.get(self.url.clone()).await?;
        let (hyper::http::response::Parts { status, .. }, mut body) = res.into_parts();
        if !status.is_success() {
            anyhow::bail!("server returned an error status: {}", status);
        }
        let mut buf = BytesMut::with_capacity(64);
        while let Some(chunk) = body.data().await {
            let chunk = chunk?;
            if let Some(max_body_size) = self.max_body_size {
                let new_body_size = buf.len() + chunk.len();
                if new_body_size > max_body_size {
                    anyhow::bail!(
                        "the body size exceede the limit: {} (limit is {})",
                        new_body_size,
                        max_body_size
                    );
                }
            }
            buf.extend_from_slice(&chunk);
        }
        let utf8 = std::str::from_utf8(&buf)?;
        let trimmed = utf8.trim();
        debug!(message = "parsing IP address", ip_address = ?trimmed);
        let ip_address = trimmed.parse()?;
        Ok(ip_address)
    }
}
