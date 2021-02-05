//! Print the IP address to stdout.

#![warn(missing_docs)]

use async_trait::async_trait;
use netloc_core::reporter::{Data, Reporter};
use std::error::Error;

/// A [`Reporter`] that prints the data to stdout.
pub struct Stdout;

#[async_trait]
impl Reporter for Stdout {
    type Error = Box<dyn Error>;

    async fn report(&self, data: &Data) -> Result<(), Self::Error> {
        println!("{}", data.ip);
        Ok(())
    }
}
