//! Detect IP address and report it.

#![warn(missing_docs)]

use netloc::state::State;
use netloc::{ip_resolver, reconciler::Reconciler};
use netloc_core::reporter::Reporter;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// HTTP URL to get the IP address from.
    #[structopt(short, long, env)]
    pub url: String,

    /// Delay between the reconcillation attempts.
    #[structopt(short, long, env, parse(try_from_str = parse_duration::parse), default_value = "10m")]
    pub delay: std::time::Duration,

    /// Discord webhook URL to invoke to report the updated IP address.
    #[structopt(long, env)]
    pub discord_webhook_url: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let Opt {
        url,
        delay,
        discord_webhook_url,
    } = Opt::from_args();

    let mut reporters: Vec<Box<dyn Reporter<Error = Box<dyn std::error::Error>>>> =
        vec![Box::new(netloc_stdout::Stdout)];

    if let Some(discord_webhook_url) = discord_webhook_url {
        reporters.push(Box::new(netloc_discord::Discord {
            webhook: netloc_discord::Webhook::from_url(&discord_webhook_url),
        }));
    } else {
        eprintln!("Warn: discord webhook URL not set, skipping discord reporting");
    }

    let mut reconciler = Reconciler {
        delay,
        ip_resolver: ip_resolver::Http {
            client: reqwest::Client::new(),
            url,
        },
        reporters,
        current_ip: State::uninitialized(),
    };

    if let Err(err) = reconciler.run().await {
        eprintln!("Reconciliation failed with an error: {}", err);
        std::process::exit(1);
    }

    // No non-error exit condition.
    unreachable!();
}
