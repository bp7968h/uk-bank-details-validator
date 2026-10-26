# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/). Updates to the bundled Vocalink
tables are listed under **Data**.

## [Unreleased]

## [0.1.0] - 2026-10-26

### Added

- `BankDetails` with `new`, `from_parts` and `validate`.
- `SortCode` and `AccountNumber` parsing, with `SortCodeError`,
  `AccountNumberError` and `ParseError`.
- `Validation` (`Valid`, `Invalid`, `Unchecked`) and `is_acceptable`.
- MOD10, MOD11 and DBLAL checks with exceptions 1 to 14.
- Standardisation of 6, 7 and 9-digit account numbers.
- `no_std` support without an allocator.

### Data

- Vocalink modulus checking tables, specification 9.10.

[Unreleased]: https://github.com/bp7968h/uk-bank-details-validator/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bp7968h/uk-bank-details-validator/releases/tag/v0.1.0