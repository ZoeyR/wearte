#![cfg_attr(yarte_nightly, feature(specialization))]

#[macro_use]
extern crate cfg_if;

pub use std::fmt::Error;
pub type Result<I> = ::std::result::Result<I, Error>;
pub mod helpers;
