use crate::SortCode;

/// How the weighted digits are combined (spec section 2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    /// Sum of products, remainder after dividing by 10 must be 0.
    Mod10,
    /// Sum of products, remainder after dividing by 11 must be 0.
    Mod11,
    /// Sum of the individual digits of each product, remainder after
    /// dividing by 10 must be 0.
    DblAl,
}

/// Vocalink exception codes 1–14 (spec section 2.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Exception {
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    E9,
    E10,
    E11,
    E12,
    E13,
    E14,
}

/// One row of `valacdos.txt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WeightRow {
    pub(crate) start: SortCode,
    pub(crate) end: SortCode,
    pub(crate) method: Method,
    /// Weights for digits `u v w x y z a b c d e f g h`.
    pub(crate) weights: [i16; 14],
    pub(crate) exception: Option<Exception>,
}

impl WeightRow {
    fn contains(&self, sort_code: SortCode) -> bool {
        self.start <= sort_code && sort_code <= self.end
    }
}

// Defines `WEIGHTS: &[WeightRow]` and `SUBSTITUTIONS: &[(SortCode, SortCode)]`.
include!(concat!(env!("OUT_DIR"), "/tables.rs"));

/// The rows that apply to a sort code: none (it can't be checked), one
/// (one check) or two (two checks, in order).
pub(crate) fn lookup(sort_code: SortCode) -> &'static [WeightRow] {
    lookup_in(WEIGHTS, sort_code)
}

/// [`lookup`] over any sorted table, so tests can use their own rows.
///
/// Relies on what `build/main.rs` checks: rows are sorted by `start`, ranges
/// don't overlap, and a range appears at most twice, in adjacent rows.
fn lookup_in(rows: &[WeightRow], sort_code: SortCode) -> &[WeightRow] {
    // Rows before `end` start at or before this sort code.
    let end = rows.partition_point(|row| row.start <= sort_code);
    // Walk back over the (at most two) rows that actually contain it.
    let mut start = end;
    while start > 0 && rows[start - 1].contains(sort_code) {
        start -= 1;
    }
    &rows[start..end]
}

/// The sort code to use in the calculation for exception 5 rows
/// (`scsubtab.txt`). Returns the input if there's no substitution.
pub(crate) fn substitute(sort_code: SortCode) -> SortCode {
    match SUBSTITUTIONS.binary_search_by_key(&sort_code, |&(original, _)| original) {
        Ok(index) => SUBSTITUTIONS[index].1,
        Err(_) => sort_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc(s: &str) -> SortCode {
        s.parse().unwrap()
    }

    #[test]
    fn table_is_sorted() {
        assert!(WEIGHTS.windows(2).all(|w| w[0].start <= w[1].start));
        assert!(SUBSTITUTIONS.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn finds_one_row_at_range_start_and_end() {
        // 010004 016715 MOD11 — the first row of the table.
        for code in ["010004", "012345", "016715"] {
            let rows = lookup(sc(code));
            assert_eq!(rows.len(), 1, "{code}");
            assert_eq!(rows[0].method, Method::Mod11);
        }
    }

    #[test]
    fn finds_two_rows_in_order() {
        // 070116 has a MOD11 row (exception 12) then a MOD10 row (exception 13).
        let rows = lookup(sc("070116"));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].exception, Some(Exception::E12));
        assert_eq!(rows[1].exception, Some(Exception::E13));
    }

    #[test]
    fn finds_nothing_outside_the_table() {
        assert!(lookup(sc("000000")).is_empty()); // before the first row
        assert!(lookup(sc("016716")).is_empty()); // just after a range
        assert!(lookup(sc("999999")).is_empty()); // after the last row
    }

    #[test]
    fn substitutes_listed_sort_codes_only() {
        assert_eq!(substitute(sc("938173")), sc("938017"));
        assert_eq!(substitute(sc("089999")), sc("089999"));
    }
}
