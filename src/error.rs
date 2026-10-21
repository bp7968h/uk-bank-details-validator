//! Errors from creating bank details.

use core::fmt;

use crate::{AccountNumberError, SortCodeError};

/// Why a sort code and account number couldn't be turned into bank details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The sort code was badly formatted.
    SortCode(SortCodeError),
    /// The account number was badly formatted.
    AccountNumber(AccountNumberError),
    /// A ten-digit account number: the crate can't tell whether to use the
    /// first or last eight digits.
    AmbiguousTenDigitAccount,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SortCode(e) => write!(f, "invalid sort code: {e}"),
            Self::AccountNumber(e) => write!(f, "invalid account number: {e}"),
            Self::AmbiguousTenDigitAccount => f.write_str(
                "ten-digit account number: can't tell whether to use the first or last eight digits",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SortCode(e) => Some(e),
            Self::AccountNumber(e) => Some(e),
            Self::AmbiguousTenDigitAccount => None,
        }
    }
}

impl From<SortCodeError> for ParseError {
    fn from(e: SortCodeError) -> Self {
        Self::SortCode(e)
    }
}

impl From<AccountNumberError> for ParseError {
    fn from(e: AccountNumberError) -> Self {
        Self::AccountNumber(e)
    }
}
