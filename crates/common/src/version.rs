// crates/common/src/version.rs
//! # Semantic Versioning z Anti-Downgrade Protection
//!
//! Implementacja wersjonowania semantycznego (MAJOR.MINOR.PATCH)
//! z wbudowaną ochroną przed atakami downgrade.
//!
//! ## Anti-Downgrade
//! System wymaga, aby nowa wersja była ŚCIŚLE WIĘKSZA od obecnej.
//! Zapobiega to atakom polegającym na wymuszeniu instalacji
//! starszej, podatnej wersji oprogramowania.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Wersja semantyczna: MAJOR.MINOR.PATCH
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parsuje string "X.Y.Z" na SemanticVersion
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: '{}' (expected X.Y.Z)", s));
        }
        let major = parts[0]
            .parse()
            .map_err(|_| format!("Invalid major version: '{}'", parts[0]))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| format!("Invalid minor version: '{}'", parts[1]))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| format!("Invalid patch version: '{}'", parts[2]))?;
        Ok(Self::new(major, minor, patch))
    }

    /// Sprawdza czy `self` jest ściśle nowsza niż `other`.
    /// Używane do ochrony przed atakami downgrade.
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self > other
    }

    /// Sprawdza czy aktualizacja z `from` na `to` jest bezpieczna
    /// (monotonically increasing).
    pub fn is_safe_upgrade(from: &Self, to: &Self) -> bool {
        to.is_newer_than(from)
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        let v3 = SemanticVersion::new(1, 1, 0);
        let v4 = SemanticVersion::new(2, 0, 0);

        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(v4.is_newer_than(&v3));
        assert!(!v1.is_newer_than(&v2));
        assert!(!v1.is_newer_than(&v1)); // Same version is NOT newer
    }

    #[test]
    fn test_safe_upgrade() {
        let from = SemanticVersion::new(1, 0, 0);
        let to = SemanticVersion::new(1, 0, 1);
        assert!(SemanticVersion::is_safe_upgrade(&from, &to));

        // Downgrade attempt
        assert!(!SemanticVersion::is_safe_upgrade(&to, &from));

        // Same version
        assert!(!SemanticVersion::is_safe_upgrade(&from, &from));
    }

    #[test]
    fn test_parse() {
        let v = SemanticVersion::parse("2.1.3").unwrap();
        assert_eq!(v, SemanticVersion::new(2, 1, 3));

        assert!(SemanticVersion::parse("invalid").is_err());
        assert!(SemanticVersion::parse("1.2").is_err());
        assert!(SemanticVersion::parse("1.2.3.4").is_err());
    }
}