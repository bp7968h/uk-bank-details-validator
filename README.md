# uk-bank-details-validator

[![crates.io](https://img.shields.io/crates/v/uk-bank-details-validator.svg)](https://crates.io/crates/uk-bank-details-validator)
[![docs.rs](https://img.shields.io/docsrs/uk-bank-details-validator)](https://docs.rs/uk-bank-details-validator)
[![CI](https://github.com/bp7968h/uk-bank-details-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/bp7968h/uk-bank-details-validator/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bp7968h/uk-bank-details-validator/blob/main/LICENSE)

Validate UK bank details (a **sort code** and **account number**) using the
[Vocalink modulus checking specification](https://www.vocalink.com/tools/modulus-checking/).

Modulus checking tells you whether an account number is *possible* for a sort
code. It catches typos and swapped digits before a payment is made, which makes
it useful on payment forms, Direct Debit sign-ups and payout flows.

```rust
use uk_bank_details_validator::{BankDetails, Validation};

let details = BankDetails::new("08-99-99", "66374958")?;

match details.validate() {
    Validation::Valid => println!("details look correct"),
    Validation::Invalid => println!("please check the sort code and account number"),
    Validation::Unchecked => println!("these details can't be checked"),
}
# Ok::<(), uk_bank_details_validator::ParseError>(())
```

## What it does, and doesn't

**Does**

- Implements all three methods: MOD10, MOD11 and DBLAL
- Handles the specification's exceptions 1 to 14, including sort code substitution
- Standardises 6, 7 and 9-digit account numbers
- Accepts `089999`, `08-99-99` and `08 99 99`
- Bundles the Vocalink tables: no network access, no setup
- Works without `std` and without an allocator

**Doesn't**

- Confirm that an account **exists** or is open
- Check the account holder's **name** (that's [Confirmation of Payee](https://www.wearepay.uk/what-we-do/overlay-services/confirmation-of-payee/))
- Guarantee that a payment will succeed

`Valid` means "these details could be real", not "these details are real".

## Installation

```sh
cargo add uk-bank-details-validator
```

## Usage

### Parse, then validate

Creating `BankDetails` checks the *format* and returns a `ParseError` for bad
input. `validate()` then runs the modulus check and always returns a
`Validation`:

```rust
use uk_bank_details_validator::{BankDetails, ParseError};

assert!(matches!(BankDetails::new("08-99", "66374958"), Err(ParseError::SortCode(_))));
assert!(matches!(BankDetails::new("089999", "12AB"), Err(ParseError::AccountNumber(_))));
```

### The three results

| Result | Meaning | What to do |
|---|---|---|
| `Valid` | Passed every required check | Accept |
| `Invalid` | Failed the modulus check | Ask the customer to check their details |
| `Unchecked` | The sort code isn't covered, or it's a foreign currency account (exception 6) | Usually accept: Vocalink says to treat these as valid |

`Unchecked` is separate from `Valid` so you can decide how strict to be. If
you don't need the difference, use `is_acceptable()`:

```rust
use uk_bank_details_validator::BankDetails;

let details = BankDetails::new("08-99-99", "66374958")?;
assert!(details.validate().is_acceptable()); // Valid or Unchecked
# Ok::<(), uk_bank_details_validator::ParseError>(())
```

### Account numbers that aren't 8 digits

| Digits | Rule | Example |
|---|---|---|
| 6 | Prefix with `00` | `123456` → `00123456` |
| 7 | Prefix with `0` | `1234567` → `01234567` |
| 9 | Santander: the first digit replaces the last sort code digit, then the last 8 digits are used | `12-34-56` + `123456789` → `12-34-51` + `23456789` |
| 10 | Not supported yet: returns `ParseError::AmbiguousTenDigitAccount` | |

Ten-digit numbers use the last 8 digits at NatWest but the first 8 at Co-op
Bank and Leeds Building Society, and the specification doesn't say which sort
codes belong to which bank. Rather than guess, and risk calling a wrong
number `Valid`, the crate rejects them.

### Working with the parts

`SortCode` and `AccountNumber` are types of their own, so a value can only
exist if its format is correct:

```rust
use uk_bank_details_validator::{AccountNumber, BankDetails, SortCode};

let sort_code: SortCode = "08-99-99".parse()?;
let account: AccountNumber = "66374958".parse()?;
let details = BankDetails::from_parts(sort_code, account)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Feature flags

| Feature | Default | Description |
|---|---|---|
| `std` | yes | Implements `std::error::Error` for the error types |

Without `std`, the crate is `no_std` and needs no allocator:

```toml
uk-bank-details-validator = { version = "0.1", default-features = false }
```

## Data version

| Crate version | Vocalink specification |
|---|---|
| 0.1.x | 9.10 |

Also available in code as `uk_bank_details_validator::VOCALINK_DATA_VERSION`.
Vocalink updates its tables several times a year; data-only updates are
released as patch versions.

## Minimum supported Rust version

Rust 1.85. Raising it is treated as a minor version change.

## Disclaimer

Not affiliated with, endorsed by or supported by Vocalink, Mastercard or
Pay.UK. "Vocalink" and "Bacs" are trademarks of their respective owners. The
weight and substitution tables are published by Vocalink.

## License

[MIT](https://github.com/bp7968h/uk-bank-details-validator/blob/main/LICENSE)
