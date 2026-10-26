//! Vocalink exceptions 1–14 (spec section 2.2.2).
//!
//! Two levels:
//! - [`check_row`]: one check, with the exception's changes to the weights,
//!   digits or pass rule.
//! - [`validate_rows`]: which checks run, and how two results combine.

use crate::{
    SortCode, Validation,
    check::{passes, remainder, total},
    table::{Exception, Method, WeightRow, substitute},
};

// Digit positions in the 14-digit array: u v w x y z | a b c d e f g h
const A: usize = 6;
const B: usize = 7;
const C: usize = 8;
const G: usize = 12;
const H: usize = 13;

/// Exception 2: alternative weights when `a` isn't 0.
const EXCEPTION_2_WEIGHTS: [i16; 14] = [0, 0, 1, 2, 5, 3, 6, 4, 8, 7, 10, 9, 3, 1];
/// Exception 2: alternative weights when `a` isn't 0 and `g` is 9.
const EXCEPTION_2_WEIGHTS_G9: [i16; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 8, 7, 10, 9, 3, 1];
/// Exception 8: sort code used for the calculation.
const EXCEPTION_8_SORT_CODE: [u8; 6] = [0, 9, 0, 1, 2, 6];
/// Exception 9: sort code used for the second check.
const EXCEPTION_9_SORT_CODE: [u8; 6] = [3, 0, 9, 6, 3, 4];
/// Exception 14: weights for the retry.
const EXCEPTION_14_WEIGHTS: [i16; 14] = [0, 0, 0, 0, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1];

/// Run the check(s) for a sort code's rows.
pub(crate) fn validate_rows(digits: [u8; 14], rows: &[WeightRow]) -> Validation {
    match rows {
        [] => Validation::Unchecked,
        [row] => result(check_row(digits, row)),
        [first, second] => two_checks(digits, first, second),
        _ => unreachable!("build.rs allows at most two rows per range"),
    }
}

/// How two checks combine. The default is "both must pass".
fn two_checks(digits: [u8; 14], first: &WeightRow, second: &WeightRow) -> Validation {
    // Exception 6: foreign currency accounts can't be checked.
    if first.exception == Some(Exception::E6) && is_foreign_currency(digits) {
        return Validation::Unchecked;
    }

    let first_passes = check_row(digits, first);

    match (first.exception, second.exception) {
        // Valid if either check passes. The second only runs if the first fails.
        //   2 & 9:   the second retries with sort code 309634.
        //   10 & 11, 12 & 13: two alternative checks.
        (Some(Exception::E2), Some(Exception::E9))
        | (Some(Exception::E10), Some(Exception::E11))
        | (Some(Exception::E12), Some(Exception::E13)) => {
            result(first_passes || check_row(digits, second))
        }
        // 3: if `c` is 6 or 9, skip the second check.
        (_, Some(Exception::E3)) if matches!(digits[C], 6 | 9) => result(first_passes),
        // Default (including 5 and 6): both must pass.
        _ => result(first_passes && check_row(digits, second)),
    }
}

/// One check, with its exception applied.
fn check_row(mut digits: [u8; 14], row: &WeightRow) -> bool {
    let mut weights = row.weights;

    // Before the arithmetic: change the weights or the digits.
    // Every variant is listed (no `_`), so a new exception won't compile
    // until it's been considered here.
    match row.exception {
        Some(Exception::E2) => {
            if digits[A] != 0 {
                weights = if digits[G] == 9 {
                    EXCEPTION_2_WEIGHTS_G9
                } else {
                    EXCEPTION_2_WEIGHTS
                };
            }
        }
        Some(Exception::E5) => {
            let substituted = substitute(sort_code_of(digits));
            set_sort_code(&mut digits, substituted.digits());
        }
        Some(Exception::E7) => {
            if digits[G] == 9 {
                zero_u_to_b(&mut weights);
            }
        }
        Some(Exception::E8) => set_sort_code(&mut digits, EXCEPTION_8_SORT_CODE),
        Some(Exception::E9) => set_sort_code(&mut digits, EXCEPTION_9_SORT_CODE),
        Some(Exception::E10) => {
            if matches!((digits[A], digits[B]), (0 | 9, 9)) && digits[G] == 9 {
                zero_u_to_b(&mut weights);
            }
        }
        Some(
            Exception::E1
            | Exception::E3
            | Exception::E4
            | Exception::E6
            | Exception::E11
            | Exception::E12
            | Exception::E13
            | Exception::E14,
        )
        | None => {}
    }

    let total = total(row.method, digits, weights);

    // After the arithmetic: the pass rule.
    match row.exception {
        // 1: add 27 to the total.
        Some(Exception::E1) => passes(row.method, total + 27),
        // 4: the remainder must equal the two-digit check digit `gh`.
        Some(Exception::E4) => {
            remainder(row.method, total) == i32::from(digits[G]) * 10 + i32::from(digits[H])
        }
        // 5: check digit `g` (MOD11) or `h` (DBLAL).
        Some(Exception::E5) => exception_5(row.method, total, digits),
        // 14: if it fails, retry once with the last digit dropped.
        Some(Exception::E14) => passes(row.method, total) || exception_14_retry(digits),
        Some(
            Exception::E2
            | Exception::E3
            | Exception::E6
            | Exception::E7
            | Exception::E8
            | Exception::E9
            | Exception::E10
            | Exception::E11
            | Exception::E12
            | Exception::E13,
        )
        | None => passes(row.method, total),
    }
}

/// Exception 5's check-digit rules.
fn exception_5(method: Method, total: i32, digits: [u8; 14]) -> bool {
    let rem = remainder(method, total);
    match method {
        // MOD11, check digit g: remainder 0 → g must be 0; remainder 1 → invalid;
        // otherwise 11 − remainder must equal g.
        Method::Mod11 => match rem {
            0 => digits[G] == 0,
            1 => false,
            _ => 11 - rem == i32::from(digits[G]),
        },
        // DBLAL, check digit h: remainder 0 → h must be 0;
        // otherwise 10 − remainder must equal h.
        Method::DblAl => match rem {
            0 => digits[H] == 0,
            _ => 10 - rem == i32::from(digits[H]),
        },
        Method::Mod10 => passes(method, total),
    }
}

/// Exception 14's retry: only if `h` is 0, 1 or 9. Drop `h`, shift the
/// account number right with a leading 0, and run MOD11 again.
fn exception_14_retry(digits: [u8; 14]) -> bool {
    if !matches!(digits[H], 0 | 1 | 9) {
        return false;
    }
    let mut shifted = digits;
    shifted[A] = 0;
    shifted[B..].copy_from_slice(&digits[A..H]);
    passes(
        Method::Mod11,
        total(Method::Mod11, shifted, EXCEPTION_14_WEIGHTS),
    )
}

/// Exception 6: `a` is 4–8 and `g` equals `h`.
fn is_foreign_currency(digits: [u8; 14]) -> bool {
    (4..=8).contains(&digits[A]) && digits[G] == digits[H]
}

fn result(passed: bool) -> Validation {
    if passed {
        Validation::Valid
    } else {
        Validation::Invalid
    }
}

fn zero_u_to_b(weights: &mut [i16; 14]) {
    weights[..=B].fill(0);
}

fn sort_code_of(digits: [u8; 14]) -> SortCode {
    let mut sort = [0u8; 6];
    sort.copy_from_slice(&digits[..6]);
    SortCode::from_digits(sort)
}

fn set_sort_code(digits: &mut [u8; 14], sort_code: [u8; 6]) {
    digits[..6].copy_from_slice(&sort_code);
}
