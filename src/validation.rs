//! The result of a modulus check.

/// The result of validating bank details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum Validation {
    /// The details passed every required check.
    Valid,
    /// The details failed the modulus check: probably a typo.
    Invalid,
    /// The sort code isn't covered by the Vocalink tables, so there was
    /// nothing to check. Vocalink says to treat these as valid.
    Unchecked,
}

impl Validation {
    /// `true` for [`Valid`](Self::Valid) and [`Unchecked`](Self::Unchecked):
    /// the details can be accepted.
    #[must_use]
    pub const fn is_acceptable(self) -> bool {
        matches!(self, Self::Valid | Self::Unchecked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceptable_results() {
        assert!(Validation::Valid.is_acceptable());
        assert!(Validation::Unchecked.is_acceptable());
        assert!(!Validation::Invalid.is_acceptable());
    }
}
