//! The three modulus checks: MOD10, MOD11 and DBLAL (spec section 2.2).
//!
//! Plain arithmetic on 14 digits and 14 weights. No tables, no exceptions:
//! exceptions build on these functions in `exceptions.rs`.
use crate::table::Method;

/// The weighted total for one check.
///
/// - MOD10 / MOD11: the sum of `digit × weight`.
/// - DBLAL: the sum of the *individual digits* of each product,
///   so a product of 18 adds 1 + 8.
///
/// Returned separately from [`passes`] because some exceptions adjust the
/// total (exception 1 adds 27) or compare the remainder to a check digit
/// (exception 4).
pub(crate) fn total(method: Method, digits: [u8; 14], weights: [i16; 14]) -> i32 {
    digits
        .iter()
        .zip(weights)
        .map(|(&digit, weight)| {
            let product = i32::from(digit) * i32::from(weight);
            match method {
                Method::Mod10 | Method::Mod11 => product,
                Method::DblAl => digit_sum(product),
            }
        })
        .sum()
}

/// The remainder after dividing the total by the method's modulus.
///
/// Uses `rem_euclid` so the result is never negative: some rows have a
/// weight of -1, which can make the total negative.
pub(crate) fn remainder(method: Method, total: i32) -> i32 {
    total.rem_euclid(modulus(method))
}

/// Whether a total passes: the remainder must be 0.
pub(crate) fn passes(method: Method, total: i32) -> bool {
    remainder(method, total) == 0
}

const fn modulus(method: Method) -> i32 {
    match method {
        Method::Mod10 | Method::DblAl => 10,
        Method::Mod11 => 11,
    }
}

/// The sum of the decimal digits of a number: 18 → 9, 72 → 9, 7 → 7.
fn digit_sum(mut n: i32) -> i32 {
    n = n.abs();
    let mut sum = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Total and pass/fail in one go, for the plain case with no exception.
    fn standard_check(method: Method, digits: [u8; 14], weights: [i16; 14]) -> bool {
        passes(method, total(method, digits, weights))
    }

    /// "089999" + "66374958" → 14 digits.
    fn digits(sort_code: &str, account: &str) -> [u8; 14] {
        let mut out = [0u8; 14];
        for (slot, byte) in out.iter_mut().zip(sort_code.bytes().chain(account.bytes())) {
            *slot = byte - b'0';
        }
        out
    }

    #[test]
    fn digit_sum_adds_each_digit() {
        assert_eq!(digit_sum(0), 0);
        assert_eq!(digit_sum(7), 7);
        assert_eq!(digit_sum(18), 9);
        assert_eq!(digit_sum(72), 9);
        assert_eq!(digit_sum(1152), 9);
    }

    #[test]
    fn dblal_example_from_the_spec_diagram() {
        // 499273 12345678 with weights 2 1 2 1 … → 70
        let weights = [2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1];
        let d = digits("499273", "12345678");
        assert_eq!(total(Method::DblAl, d, weights), 70);
        assert!(standard_check(Method::DblAl, d, weights));
    }

    #[test]
    fn mod10_valid() {
        // valacdos.txt: 089000 089999 MOD10
        let weights = [0, 0, 0, 0, 0, 0, 7, 1, 3, 7, 1, 3, 7, 1];
        let d = digits("089999", "66374958");
        assert_eq!(total(Method::Mod10, d, weights), 180);
        assert!(standard_check(Method::Mod10, d, weights));
    }

    #[test]
    fn mod11_valid() {
        // valacdos.txt: 102400 107999 MOD11 → total 242 = 22 × 11
        let weights = [0, 0, 0, 0, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1];
        let d = digits("107999", "88837491");
        assert_eq!(total(Method::Mod11, d, weights), 242);
        assert!(standard_check(Method::Mod11, d, weights));
    }

    #[test]
    fn changing_one_digit_fails() {
        // A typo in the last account digit: 66374958 → 66374959.
        let weights = [0, 0, 0, 0, 0, 0, 7, 1, 3, 7, 1, 3, 7, 1];
        assert!(!standard_check(
            Method::Mod10,
            digits("089999", "66374959"),
            weights
        ));
    }

    #[test]
    fn negative_total_has_non_negative_remainder() {
        // Only h counts, with weight -1: total -5.
        let mut weights = [0; 14];
        weights[13] = -1;
        let d = digits("000000", "00000005");
        assert_eq!(total(Method::Mod11, d, weights), -5);
        assert_eq!(remainder(Method::Mod11, -5), 6);
        assert!(!passes(Method::Mod11, -5));
    }
}
