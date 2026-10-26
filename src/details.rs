//! A sort code and account number together, standardised to 6 + 8 digits.
use crate::{AccountNumber, ParseError, SortCode, Validation, exceptions, table};

/// UK bank details: a sort code and an eight-digit account number.
///
/// Account numbers that aren't eight digits are standardised when the
/// details are created, following the Vocalink specification.
///
/// ```
/// # use uk_bank_details_validator::BankDetails;
/// let details = BankDetails::new("08-99-99", "66374958")?;
/// assert_eq!(details.sort_code().to_string(), "08-99-99");
///
/// // Seven digits are padded with a leading zero.
/// let short = BankDetails::new("08-99-99", "1234567")?;
/// assert_eq!(short.account_digits(), [0, 1, 2, 3, 4, 5, 6, 7]);
/// # Ok::<(), uk_bank_details_validator::ParseError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BankDetails {
    sort_code: SortCode,
    account: [u8; 8],
}

impl BankDetails {
    /// Parse and standardise a sort code and account number.
    ///
    /// # Errors
    ///
    /// - [`ParseError::SortCode`] or [`ParseError::AccountNumber`] if either
    ///   value is badly formatted.
    /// - [`ParseError::AmbiguousTenDigitAccount`] for a ten-digit account
    ///   number, because the rule depends on the bank.
    pub fn new(sort_code: &str, account_number: &str) -> Result<Self, ParseError> {
        Self::from_parts(sort_code.parse()?, account_number.parse()?)
    }

    /// Run the Vocalink modulus check(s) for these details.
    ///
    /// ```
    /// # use uk_bank_details_validator::{BankDetails, Validation};
    /// let details = BankDetails::new("08-99-99", "66374958")?;
    /// assert_eq!(details.validate(), Validation::Valid);
    /// # Ok::<(), uk_bank_details_validator::ParseError>(())
    /// ```
    pub fn validate(&self) -> Validation {
        exceptions::validate_rows(self.digits(), table::lookup(self.sort_code))
    }

    /// Combine an already-parsed sort code and account number.
    ///
    /// # Errors
    ///
    /// [`ParseError::AmbiguousTenDigitAccount`] for a ten-digit account number.
    pub fn from_parts(
        sort_code: SortCode,
        account_number: AccountNumber,
    ) -> Result<Self, ParseError> {
        let entered = account_number.digits();
        let mut sort_code = sort_code;
        let mut account = [0u8; 8];

        match entered.len() {
            // 6, 7 or 8 digits: right-align, so the gap on the left stays 0.
            //   123456   → 00123456
            //   1234567  → 01234567
            //   12345678 → 12345678
            6..=8 => account[8 - entered.len()..].copy_from_slice(entered),
            // 9 digits (Santander): the first account digit replaces the last
            // sort code digit, then keep the last eight account digits.
            //   12-34-56 + 123456789 → 12-34-51 + 23456789
            9 => {
                let mut sort_digits = sort_code.digits();
                sort_digits[5] = entered[0];
                sort_code = SortCode::from_digits(sort_digits);
                account.copy_from_slice(&entered[1..]);
            }
            // 10 digits: NatWest keeps the last eight, Co-op and Leeds BS the
            // first eight, and we can't tell the bank from the sort code.
            10 => return Err(ParseError::AmbiguousTenDigitAccount),
            _ => unreachable!("AccountNumber guarantees 6 to 10 digits"),
        }
        Ok(Self { sort_code, account })
    }

    /// The sort code used for checking. For nine-digit Santander numbers this
    /// differs from the one entered.
    #[must_use]
    pub const fn sort_code(&self) -> SortCode {
        self.sort_code
    }

    /// The standardised eight-digit account number.
    #[must_use]
    pub const fn account_digits(&self) -> [u8; 8] {
        self.account
    }

    /// All 14 digits in spec order, `u v w x y z a b c d e f g h`:
    /// the sort code followed by the account number.
    pub(crate) fn digits(&self) -> [u8; 14] {
        let mut all = [0u8; 14];
        all[..6].copy_from_slice(&self.sort_code.digits());
        all[6..].copy_from_slice(&self.account);
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details(sort_code: &str, account: &str) -> Result<BankDetails, ParseError> {
        BankDetails::new(sort_code, account)
    }

    #[test]
    fn eight_digits_unchanged() {
        let d = details("08-99-99", "66374958").unwrap();
        assert_eq!(d.account_digits(), [6, 6, 3, 7, 4, 9, 5, 8]);
        assert_eq!(d.digits(), [0, 8, 9, 9, 9, 9, 6, 6, 3, 7, 4, 9, 5, 8]);
    }

    #[test]
    fn six_and_seven_digits_padded_with_zeros() {
        assert_eq!(
            details("089999", "123456").unwrap().account_digits(),
            [0, 0, 1, 2, 3, 4, 5, 6]
        );
        assert_eq!(
            details("089999", "1234567").unwrap().account_digits(),
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn nine_digits_santander_moves_first_digit_into_sort_code() {
        let d = details("12-34-56", "123456789").unwrap();
        assert_eq!(d.sort_code().to_string(), "12-34-51");
        assert_eq!(d.account_digits(), [2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn ten_digits_are_ambiguous() {
        assert_eq!(
            details("08-60-01", "1234567890"),
            Err(ParseError::AmbiguousTenDigitAccount)
        );
    }

    #[test]
    fn format_errors_are_wrapped() {
        assert!(matches!(
            details("08-99", "66374958"),
            Err(ParseError::SortCode(_))
        ));
        assert!(matches!(
            details("089999", "12AB"),
            Err(ParseError::AccountNumber(_))
        ));
    }

    fn validate(sort_code: &str, account: &str) -> Validation {
        details(sort_code, account).unwrap().validate()
    }

    #[test]
    fn valid_single_check() {
        assert_eq!(validate("08-99-99", "66374958"), Validation::Valid); // MOD10
        assert_eq!(validate("10-79-99", "88837491"), Validation::Valid); // MOD11
    }

    #[test]
    fn typo_is_invalid() {
        assert_eq!(validate("08-99-99", "66374959"), Validation::Invalid);
        assert_eq!(validate("08-99-99", "66734958"), Validation::Invalid); // swapped digits
    }

    #[test]
    fn sort_code_not_in_table_is_unchecked() {
        assert_eq!(validate("01-67-16", "12345678"), Validation::Unchecked);
        assert_eq!(validate("99-99-99", "12345678"), Validation::Unchecked);
    }

    #[test]
    fn two_checks_both_must_pass() {
        // 040010-040014 has a MOD11 row and a DBLAL row, no exception.
        assert_eq!(validate("04-00-10", "10000051"), Validation::Valid); // both pass
        assert_eq!(validate("04-00-10", "10000000"), Validation::Invalid); // only MOD11 passes
    }
}
