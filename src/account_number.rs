//! UK bank account numbers, as entered (6 to 10 digits).

use core::{fmt, str::FromStr};

/// A bank account number exactly as the customer gave it: 6 to 10 digits.
///
/// Numbers that aren't 8 digits are standardised later, when combined with
/// a sort code in `BankDetails`, because some rules depend on the bank.
///
/// ```
/// # use uk_bank_details_validator::AccountNumber;
/// let account: AccountNumber = "66374958".parse().unwrap();
/// assert_eq!(account.digits(), &[6, 6, 3, 7, 4, 9, 5, 8]);
///
/// // NatWest's ten-digit format with a hyphen is accepted too.
/// let natwest: AccountNumber = "01-23456789".parse().unwrap();
/// assert_eq!(natwest.len(), 10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountNumber {
    /// Digits in positions `0..len`; the rest are always 0, so derived
    /// equality and hashing stay correct.
    digits: [u8; 10],
    len: u8,
}

impl AccountNumber {
    /// The smallest accepted number of digits.
    pub const MIN_DIGITS: usize = 6;
    /// The largest accepted number of digits.
    pub const MAX_DIGITS: usize = 10;

    /// The digits as entered, e.g. `"1234567"` → `[1, 2, 3, 4, 5, 6, 7]`.
    #[must_use]
    pub fn digits(&self) -> &[u8] {
        &self.digits[..self.len()]
    }

    /// How many digits were entered (6 to 10).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Always `false`: an account number has at least six digits.
    /// (Provided because clippy expects `is_empty` next to `len`.)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

/// Why a string couldn't be parsed as an [`AccountNumber`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccountNumberError {
    /// The input was empty or only whitespace.
    Empty,
    /// The input contained something other than digits or a `-`.
    InvalidCharacter {
        /// The offending character.
        character: char,
        /// Its position in the input (0-based, counting characters).
        position: usize,
    },
    /// The input didn't contain 6 to 10 digits.
    WrongDigitCount(usize),
    /// A hyphen somewhere other than NatWest's `01-23456789` format.
    InvalidFormat,
}

impl fmt::Display for AccountNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("account number is empty"),
            Self::InvalidCharacter {
                character,
                position,
            } => write!(
                f,
                "account number contains invalid character {character:?} at position {position}"
            ),
            Self::WrongDigitCount(count) => {
                write!(f, "account number must have 6 to 10 digits, found {count}")
            }
            Self::InvalidFormat => f.write_str(
                "a hyphen is only allowed after the second digit of a ten-digit account number",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AccountNumberError {}

impl FromStr for AccountNumber {
    type Err = AccountNumberError;

    /// Accepts 6 to 10 digits, or NatWest's `01-23456789`, with optional
    /// whitespace around the whole value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AccountNumberError::Empty);
        }

        // 1. Only digits and hyphens. After this, `s` is ASCII.
        if let Some((position, character)) = s
            .chars()
            .enumerate()
            .find(|&(_, c)| !c.is_ascii_digit() && c != '-')
        {
            return Err(AccountNumberError::InvalidCharacter {
                character,
                position,
            });
        }

        // 2. Between 6 and 10 digits.
        let bytes = s.as_bytes();
        let digit_count = bytes.iter().filter(|b| b.is_ascii_digit()).count();
        if !(Self::MIN_DIGITS..=Self::MAX_DIGITS).contains(&digit_count) {
            return Err(AccountNumberError::WrongDigitCount(digit_count));
        }

        // 3. Hyphens: none, or exactly one in NatWest's position (NN-NNNNNNNN).
        let hyphens = bytes.len() - digit_count;
        let well_formed = match hyphens {
            0 => true,
            1 => digit_count == 10 && bytes[2] == b'-',
            _ => false,
        };
        if !well_formed {
            return Err(AccountNumberError::InvalidFormat);
        }

        // 4. Collect the digits, skipping the hyphen.
        let mut digits = [0u8; 10];
        for (slot, byte) in digits
            .iter_mut()
            .zip(bytes.iter().filter(|b| b.is_ascii_digit()))
        {
            *slot = byte - b'0';
        }

        Ok(Self {
            digits,
            // At most 10, checked above.
            #[allow(clippy::cast_possible_truncation)]
            len: digit_count as u8,
        })
    }
}

impl fmt::Display for AccountNumber {
    /// Formats the digits as entered, without any hyphen.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for digit in self.digits() {
            write!(f, "{digit}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<AccountNumber, AccountNumberError> {
        s.parse()
    }

    #[test]
    fn accepts_six_to_ten_digits() {
        for input in ["123456", "1234567", "12345678", "123456789", "1234567890"] {
            let account = parse(input).unwrap();
            assert_eq!(account.len(), input.len());
            assert_eq!(account.to_string(), input);
        }
    }

    #[test]
    fn keeps_leading_zeros() {
        assert_eq!(
            parse("00123456").unwrap().digits(),
            &[0, 0, 1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn accepts_natwest_hyphen_format() {
        let account = parse("01-23456789").unwrap();
        assert_eq!(account, parse("0123456789").unwrap());
        assert_eq!(account.to_string(), "0123456789");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(parse(" 66374958\n"), parse("66374958"));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse(""), Err(AccountNumberError::Empty));
        assert_eq!(parse("  "), Err(AccountNumberError::Empty));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(
            parse("1234AB78"),
            Err(AccountNumberError::InvalidCharacter {
                character: 'A',
                position: 4
            })
        );
        assert_eq!(
            parse("1234 5678"),
            Err(AccountNumberError::InvalidCharacter {
                character: ' ',
                position: 4
            })
        );
    }

    #[test]
    fn rejects_wrong_digit_count() {
        assert_eq!(parse("12345"), Err(AccountNumberError::WrongDigitCount(5)));
        assert_eq!(
            parse("12345678901"),
            Err(AccountNumberError::WrongDigitCount(11))
        );
        assert_eq!(parse("--"), Err(AccountNumberError::WrongDigitCount(0)));
    }

    #[test]
    fn rejects_hyphen_outside_natwest_format() {
        assert_eq!(parse("1234-5678"), Err(AccountNumberError::InvalidFormat));
        assert_eq!(parse("012-3456789"), Err(AccountNumberError::InvalidFormat));
        assert_eq!(parse("01-2345678"), Err(AccountNumberError::InvalidFormat)); // 9 digits
        assert_eq!(
            parse("01--23456789"),
            Err(AccountNumberError::InvalidFormat)
        );
    }
}
