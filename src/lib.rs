#![deny(clippy::all, clippy::perf, clippy::complexity, clippy::pedantic)]
#![doc = include_str!("../README.md")]

mod client;
mod errors;
mod location;
mod response;

#[macro_use]
mod api_param_enum;

pub mod air_quality;
pub mod forecast;
pub mod geocoding;

pub use client::*;
pub use errors::*;
pub use jiff;
pub use location::*;
pub use response::{Current, Meta, Series};
