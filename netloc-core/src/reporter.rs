//! [`Reporter`] and associated types.

use async_trait::async_trait;

/// An abstract way to report the [`Data`].
///
/// Implement this to send the IP address updates to a new detination.
#[async_trait]
pub trait Reporter {
    /// The error that this reporter can produce.
    type Error;

    /// Report the passed [`Data`].
    async fn report(&self, data: &Data) -> Result<(), Self::Error>;
}

/// The data associated with an IP address update.
#[derive(Debug)]
pub struct Data {
    /// The IP address.
    pub ip: String,
}
