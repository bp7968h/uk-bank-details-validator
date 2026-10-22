#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[path = "../build/parse.rs"]
#[allow(dead_code)]
mod build_parse;

mod account_number;
mod error;
mod sort_code;
mod validation;

pub use account_number::{AccountNumber, AccountNumberError};
pub use error::ParseError;
pub use sort_code::{SortCode, SortCodeError};
pub use validation::Validation;

/// The Vocalink specification version the bundled tables come from.
/// 
/// *Note*
/// Not affiliated with, endorsed by or supported by Vocalink, Mastercard or
/// Pay.UK. "Vocalink" and "Bacs" are trademarks of their respective owners. The
/// weight and substitution tables are published by Vocalink.
pub const VOCALINK_DATA_VERSION: &str = "9.10";