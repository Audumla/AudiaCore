//! Explicit sensitive-value handling with deterministic redaction and no I/O.

use std::{error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode};

const EMPTY_KEY: ErrorCode = ErrorCode::new("VAL-SENSITIVE-001");

#[derive(Clone, PartialEq, Eq)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub const fn expose(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sensitive([REDACTED])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SensitiveKey(String);

impl SensitiveKey {
    pub fn new(value: impl Into<String>) -> Result<Self, SensitiveError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SensitiveError::EmptyKey);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensitiveError {
    EmptyKey,
}

impl CodedError for SensitiveError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EmptyKey => EMPTY_KEY,
        }
    }
}

impl fmt::Display for SensitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => f.write_str("sensitive key must not be empty"),
        }
    }
}

impl Error for SensitiveError {}

pub fn redact_text(mut text: String, values: &[Sensitive<String>]) -> String {
    for value in values {
        if !value.expose().is_empty() {
            text = text.replace(value.expose(), "[REDACTED]");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_exposes_sensitive_value() {
        let secret = Sensitive::new("secret-token".to_owned());
        let debug = format!("{secret:?}");
        assert_eq!(debug, "Sensitive([REDACTED])");
        assert!(!debug.contains("secret-token"));
    }

    #[test]
    fn redaction_is_explicit_and_deterministic() {
        let values = [
            Sensitive::new("alpha".to_owned()),
            Sensitive::new("beta".to_owned()),
        ];
        assert_eq!(
            redact_text("alpha then beta then alpha".to_owned(), &values),
            "[REDACTED] then [REDACTED] then [REDACTED]"
        );
    }

    #[test]
    fn key_validation_has_stable_error_identity() {
        let error = SensitiveKey::new(" ").unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-SENSITIVE-001");
    }
}
