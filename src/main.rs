//! Detect IP address and report it.

#![warn(missing_docs)]

#[macro_use]
extern crate tracing;

use std::{str::FromStr, time::Duration};

use hyper_system_resolver::addr_info_hints::AddressFamily;
use netloc::state::State;
use netloc::{ip_resolver, reconciler::Reconciler};
use netloc_core::reporter::Reporter;
use tracing::warn;

use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestedAddressType {
    Any,
    IPv4,
    IPv6,
}

impl FromStr for RequestedAddressType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _s @ "any" => Ok(Self::Any),
            _s @ "IPv4" | _s @ "ipv4" | _s @ "v4" | _s @ "4" => Ok(Self::IPv4),
            _s @ "IPv6" | _s @ "ipv6" | _s @ "v6" | _s @ "6" => Ok(Self::IPv6),
            _ => anyhow::bail!("invalid requested address type: {}", s),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// HTTP URL to get the IP address from.
    #[structopt(short, long, env)]
    pub url: hyper::Uri,

    /// Delay between the reconcillation attempts.
    #[structopt(short, long, env, parse(try_from_str = parse_duration::parse), default_value = "10m")]
    pub delay: std::time::Duration,

    /// Discord webhook URL to invoke to report the updated IP address.
    #[structopt(long, env)]
    pub discord_webhook_url: Option<String>,

    /// HTTP URL to report the updated IP address in plain text.
    #[structopt(long, env)]
    pub http_request_url: Option<String>,

    /// The address type to request.
    /// Currently this controls how the hostname of the HTTP is resolved.
    #[structopt(short = "t", long, env, default_value = "any")]
    pub requested_address_type: RequestedAddressType,

    /// The maximum size of the HTTP response body during IP resolution.
    #[structopt(long, env, default_value = "1024")]
    pub max_body_size: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let Opt {
        url,
        delay,
        discord_webhook_url,
        http_request_url,
        requested_address_type,
        max_body_size,
    } = Opt::from_args();

    let mut reporters: Vec<Box<dyn Reporter<Error = Box<dyn std::error::Error>>>> =
        vec![Box::new(netloc_stdout::Stdout)];

    if let Some(discord_webhook_url) = discord_webhook_url {
        reporters.push(Box::new(netloc_discord::Discord::from_url(
            &discord_webhook_url,
        )));
    } else {
        warn!("discord webhook URL not set, skipping discord reporting");
    }

    if let Some(http_request_url) = http_request_url {
        reporters.push(Box::new(netloc_http_request::HttpRequest::from_url(
            &http_request_url,
        )));
    } else {
        warn!("HTTP URL not set, skipping HTTP request reporting");
    }

    let client = ip_resolver::http::build_client(match requested_address_type {
        RequestedAddressType::Any => AddressFamily::Unspec,
        RequestedAddressType::IPv4 => AddressFamily::Inet,
        RequestedAddressType::IPv6 => AddressFamily::Inet6,
    });

    let mut reconciler = Reconciler {
        delay,
        ip_resolver: ip_resolver::http::Resolver {
            client,
            url,
            max_body_size: Some(max_body_size),
        },
        reporters,
        current_ip: State::uninitialized(),
    };

    loop {
        match reconciler.run().await {
            Err(error) => {
                let restart_delay = Duration::from_secs(10);
                error!(
                    message = "reconciler loop exited with an error, will restart after a delay",
                    %error,
                    restart_delay_ms = %restart_delay.as_millis(),
                );
                tokio::time::sleep(restart_delay).await;
                continue;
            }
            Ok(_) => unreachable!("infallible reached"),
        }
    }
}
