//! Spec Test: Tests agains all the test cases given in the vocalink specification

use uk_bank_details_validator::{
    BankDetails,
    Validation::{self, *},
};

// (sort code, account, expected, description)
const CASES: &[(&str, &str, Validation, &str)] = &[
    ("089999", "66374958", Valid, "pass modulus 10 check."),
    ("107999", "88837491", Valid, "pass modulus 11 check."),
    (
        "202959",
        "63748472",
        Valid,
        "pass modulus 11 and double alternate checks.",
    ),
    (
        "871427",
        "46238510",
        Valid,
        "Exception 10 & 11 where first check passes and second
check fails.",
    ),
    (
        "872427",
        "46238510",
        Valid,
        "Exception 10 & 11 where first check fails and second check
passes.",
    ),
    (
        "871427",
        "09123496",
        Valid,
        "Exception 10 where in the account number ab=09 and the
g=9. The first check passes and second check fails.",
    ),
    (
        "871427",
        "99123496",
        Valid,
        "Exception 10 where in the account number ab=99 and the
g=9. The first check passes and the second check fails.",
    ),
    (
        "820000",
        "73688637",
        Valid,
        "Exception 3, and the sorting code is the start of a range. As
c=6 the second check should be ignored.",
    ),
    (
        "827999",
        "73988638",
        Valid,
        "Exception 3, and the sorting code is the end of a range. As
c=9 the second check should be ignored.",
    ),
    (
        "827101",
        "28748352",
        Valid,
        "Exception 3. As c<>6 or 9 perform both checks pass.",
    ),
    (
        "134020",
        "63849203",
        Valid,
        "Exception 4 where the remainder is equal to the checkdigit.",
    ),
    (
        "118765",
        "64371389",
        Valid,
        "Exception 1 – ensures that 27 has been added to the
accumulated total and passes double alternate modulus
check.",
    ),
    (
        "200915",
        "41011166",
        Unchecked,
        "Exception 6 where the account fails standard check but is a
foreign currency account.",
    ),
    (
        "938611",
        "07806039",
        Valid,
        "Exception 5 where the check passes.",
    ),
    (
        "938600",
        "42368003",
        Valid,
        "Exception 5 where the check passes with substitution. ",
    ),
    (
        "938063",
        "55065200",
        Valid,
        "Exception 5 where both checks produce a remainder of 0
and pass.",
    ),
    (
        "772798",
        "99345694",
        Valid,
        "Exception 7 where passes but would fail the standard check. ",
    ),
    (
        "086090",
        "06774744",
        Valid,
        "Exception 8 where the check passes. ",
    ),
    (
        "309070",
        "02355688",
        Valid,
        "Exception 2 & 9 where the first check passes. ",
    ),
    (
        "309070",
        "12345668",
        Valid,
        "Exception 2 & 9 where the first check fails and second check
passes with substitution.",
    ),
    (
        "309070",
        "12345677",
        Valid,
        "Exception 2 & 9 where a!=0 and g!=9 and passes. ",
    ),
    (
        "309070",
        "99345694",
        Valid,
        "Exception 2 & 9 where a!=0 and g=9 and passes. ",
    ),
    (
        "938063",
        "15764273",
        Invalid,
        "Exception 5 where the first checkdigit is correct and the
second incorrect.",
    ),
    (
        "938063",
        "15764264",
        Invalid,
        "Exception 5 where the first checkdigit is incorrect and the
second correct.",
    ),
    (
        "938063",
        "15763217",
        Invalid,
        "Exception 5 where the first checkdigit is incorrect with a
remainder of 1.",
    ),
    (
        "118765",
        "64371388",
        Invalid,
        "Exception 1 where it fails double alternate check.",
    ),
    (
        "203099",
        "66831036",
        Invalid,
        "Pass modulus 11 check and fail double alternate check.",
    ),
    (
        "203099",
        "58716970",
        Invalid,
        "Fail modulus 11 check and pass double alternate check.",
    ),
    ("089999", "66374959", Invalid, "Fail modulus 10 check."),
    ("107999", "88837493", Invalid, "Fail modulus 11 check."),
    (
        "074456",
        "12345112",
        Valid,
        "Exception 12/13 where passes modulus 11 check (in this
example, modulus 10 check fails, however, there is no need
for it to be performed as the first check passed).",
    ),
    (
        "070116",
        "34012583",
        Valid,
        "Exception 12/13 where passes the modulus 11check (in this
example, modulus 10 check passes as well, however, there is
no need for it to be performed as the first check passed).",
    ),
    (
        "074456",
        "11104102",
        Valid,
        "Exception 12/13 where fails the modulus 11 check, but
passes the modulus 10 check.",
    ),
    (
        "180002",
        "00000190",
        Valid,
        "Exception 14 where the first check fails and the second
check passes.",
    ),
];

#[test]
fn spec_cases() {
    let mut failures = Vec::new();
    for &(sort_code, account, expected, description) in CASES {
        let got = BankDetails::new(sort_code, account).unwrap().validate();
        if got != expected {
            failures.push(format!(
                "{sort_code} {account}: expected {expected:?}, got {got:?} ({description})"
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
