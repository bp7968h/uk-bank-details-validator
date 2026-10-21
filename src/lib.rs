#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[path = "../build/parse.rs"]
#[allow(dead_code)]
mod build_parse;

mod account_number;
mod sort_code;

pub use account_number::{AccountNumber, AccountNumberError};
pub use sort_code::{SortCode, SortCodeError};
