//! UK sort codes.

use core::{fmt, str::FromStr};

/// A UK sort code: six digits, usually written `08-99-99`.
///
/// Create one by parsing a string:
///
/// ```
/// use uk_bank_details_validator::SortCode;
/// let sort_code: SortCode = "08-99-99".parse().unwrap();
/// assert_eq!(sort_code.to_string(), "08-99-99");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortCode([u8; 6]);

impl SortCode {
    /// Internal constructor for the generated tables. Panics (at compile time
    /// when used in a `static`) if any digit is above 9.
    pub(crate) const fn from_digits(digits: [u8; 6]) -> Self {
        let mut i = 0;
        while i < 6 {
            assert!(digits[i] <= 9, "sort code digit out of range");
            i += 1;
        }
        Self(digits)
    }

    /// The six digits, e.g. `08-99-99` → `[0, 8, 9, 9, 9, 9]`.
    #[must_use]
    pub const fn digits(self) -> [u8; 6] {
        self.0
    }
}

/// Why a string couldn't be parsed as a [`SortCode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortCodeError {
    /// The input was empty or only whitespace.
    Empty,
    /// The input contained something other than digits, `-` or spaces.
    InvalidCharacter {
        /// The offending character.
        character: char,
        /// Its position in the input (0-based, counting characters).
        position: usize,
    },
    /// The input didn't contain exactly six digits.
    WrongDigitCount(usize),
    /// Six digits, but separators in the wrong places or mixed,
    /// e.g. `0899-99` or `08-99 99`.
    InvalidFormat,
}

impl fmt::Display for SortCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("sort code is empty"),
            Self::InvalidCharacter {
                character,
                position,
            } => write!(
                f,
                "sort code contains invalid character {character:?} at position {position}"
            ),
            Self::WrongDigitCount(count) => {
                write!(f, "sort code must have 6 digits, found {count}")
            }
            Self::InvalidFormat => {
                f.write_str("sort code must be written as 123456, 12-34-56 or 12 34 56")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SortCodeError {}

impl FromStr for SortCode {
    type Err = SortCodeError;

    /// Accepts `089999`, `08-99-99` or `08 99 99`, with optional whitespace
    /// around the whole value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(SortCodeError::Empty);
        }

        // 1. Only digits and separators allowed. After this, `s` is ASCII,
        //    so byte positions and character positions are the same.
        if let Some((position, character)) = s
            .chars()
            .enumerate()
            .find(|&(_, c)| !c.is_ascii_digit() && c != '-' && c != ' ')
        {
            return Err(SortCodeError::InvalidCharacter {
                character,
                position,
            });
        }

        // 2. Exactly six digits.
        let digit_count = s.bytes().filter(u8::is_ascii_digit).count();
        if digit_count != 6 {
            return Err(SortCodeError::WrongDigitCount(digit_count));
        }

        // 3. Separators only between pairs, and the same one both times.
        let bytes = s.as_bytes();
        let well_formed = match bytes.len() {
            6 => true, // six digits, no separators
            8 => bytes[2] == bytes[5] && !bytes[2].is_ascii_digit(),
            _ => false,
        };
        if !well_formed {
            return Err(SortCodeError::InvalidFormat);
        }

        // 4. Collect the digits, skipping separators.
        let mut digits = [0u8; 6];
        for (slot, byte) in digits
            .iter_mut()
            .zip(bytes.iter().filter(|b| b.is_ascii_digit()))
        {
            *slot = byte - b'0';
        }
        Ok(Self(digits))
    }
}

impl fmt::Display for SortCode {
    /// Formats as `08-99-99`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.0;
        write!(f, "{}{}-{}{}-{}{}", d[0], d[1], d[2], d[3], d[4], d[5])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<SortCode, SortCodeError> {
        s.parse()
    }

    #[test]
    fn accepts_the_three_common_formats() {
        let expected = SortCode([0, 8, 9, 9, 9, 9]);
        assert_eq!(parse("089999"), Ok(expected));
        assert_eq!(parse("08-99-99"), Ok(expected));
        assert_eq!(parse("08 99 99"), Ok(expected));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(parse("  08-99-99\n"), Ok(SortCode([0, 8, 9, 9, 9, 9])));
    }

    #[test]
    fn displays_with_hyphens_and_leading_zero() {
        assert_eq!(parse("089999").unwrap().to_string(), "08-99-99");
        assert_eq!(parse("010004").unwrap().to_string(), "01-00-04");
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse(""), Err(SortCodeError::Empty));
        assert_eq!(parse("   "), Err(SortCodeError::Empty));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(
            parse("08x999"),
            Err(SortCodeError::InvalidCharacter {
                character: 'x',
                position: 2
            })
        );
        assert_eq!(
            parse("08.99.99"),
            Err(SortCodeError::InvalidCharacter {
                character: '.',
                position: 2
            })
        );
        // non-ASCII: position counts characters, not bytes
        assert_eq!(
            parse("£08-99-99"),
            Err(SortCodeError::InvalidCharacter {
                character: '£',
                position: 0
            })
        );
    }

    #[test]
    fn rejects_wrong_digit_count() {
        assert_eq!(parse("08-99"), Err(SortCodeError::WrongDigitCount(4)));
        assert_eq!(parse("0899999"), Err(SortCodeError::WrongDigitCount(7)));
    }

    #[test]
    fn rejects_misplaced_or_mixed_separators() {
        assert_eq!(parse("0899-99"), Err(SortCodeError::InvalidFormat));
        assert_eq!(parse("08-99 99"), Err(SortCodeError::InvalidFormat));
        assert_eq!(parse("0-8-9-9-9-9"), Err(SortCodeError::InvalidFormat));
        assert_eq!(parse("08--9999"), Err(SortCodeError::InvalidFormat));
        assert_eq!(parse("08  99  99"), Err(SortCodeError::InvalidFormat));
    }

    #[test]
    fn orders_numerically() {
        assert!(parse("01-00-04").unwrap() < parse("04-00-03").unwrap());
        assert!(parse("09-99-99").unwrap() < parse("10-00-00").unwrap());
    }
}
