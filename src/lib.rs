//! The utilities for the `netloc` main binary.

#![warn(missing_docs)]

#[macro_use]
extern crate tracing;

pub mod ip_resolver;
pub mod reconciler;
pub mod state;
