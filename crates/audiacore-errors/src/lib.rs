//! Stable reusable error identity without presentation ownership.
//!
//! Owning crates keep typed Rust errors and dynamic diagnostic context. This
//! crate owns only stable error-code identity and prefix-derived category.
//! Human-facing message templates, kinds, resolutions and catalogue loading
//! belong to the configured presentation layer above this crate.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Validation,
    Constraint,
    Resource,
    Io,
    Network,
    Timeout,
    External,
    Configuration,
    Version,
    Internal,
    Unsupported,
}

impl ErrorCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Constraint => "constraint",
            Self::Resource => "resource",
            Self::Io => "io",
            Self::Network => "network",
            Self::Timeout => "timeout",
            Self::External => "external",
            Self::Configuration => "configuration",
            Self::Version => "version",
            Self::Internal => "internal",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode(&'static str);

impl ErrorCode {
    pub const fn new(code: &'static str) -> Self {
        assert!(Self::is_valid(code), "invalid stable error code");
        Self(code)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub const fn is_valid(code: &str) -> bool {
        valid_code(code)
    }

    pub fn category(self) -> ErrorCategory {
        if self.0.starts_with("VAL-") {
            ErrorCategory::Validation
        } else if self.0.starts_with("CON-") {
            ErrorCategory::Constraint
        } else if self.0.starts_with("RES-") {
            ErrorCategory::Resource
        } else if self.0.starts_with("IO-") {
            ErrorCategory::Io
        } else if self.0.starts_with("NET-") {
            ErrorCategory::Network
        } else if self.0.starts_with("TO-") {
            ErrorCategory::Timeout
        } else if self.0.starts_with("EXT-") {
            ErrorCategory::External
        } else if self.0.starts_with("CFG-") {
            ErrorCategory::Configuration
        } else if self.0.starts_with("VER-") {
            ErrorCategory::Version
        } else if self.0.starts_with("INT-") {
            ErrorCategory::Internal
        } else if self.0.starts_with("UNS-") {
            ErrorCategory::Unsupported
        } else {
            unreachable!("ErrorCode::new validates category prefixes")
        }
    }
}

pub trait CodedError {
    fn code(&self) -> ErrorCode;

    fn category(&self) -> ErrorCategory {
        self.code().category()
    }
}

const fn valid_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    if bytes.len() < 9 {
        return false;
    }

    let mut first_hyphen = bytes.len();
    let mut last_hyphen = 0;
    let mut index = 0;
    let mut previous_hyphen = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'-' {
            if index == 0 || previous_hyphen {
                return false;
            }
            if first_hyphen == bytes.len() {
                first_hyphen = index;
            }
            last_hyphen = index;
            previous_hyphen = true;
        } else {
            if !byte.is_ascii_uppercase() && !byte.is_ascii_digit() {
                return false;
            }
            previous_hyphen = false;
        }
        index += 1;
    }

    if previous_hyphen || first_hyphen == bytes.len() || last_hyphen == first_hyphen {
        return false;
    }
    if !valid_prefix(bytes, first_hyphen) {
        return false;
    }

    if bytes.len() - last_hyphen - 1 != 3 {
        return false;
    }
    let mut suffix = last_hyphen + 1;
    while suffix < bytes.len() {
        if !bytes[suffix].is_ascii_digit() {
            return false;
        }
        suffix += 1;
    }

    let mut segment_start = first_hyphen + 1;
    if segment_start >= last_hyphen {
        return false;
    }
    index = segment_start;
    while index < last_hyphen {
        if index == segment_start {
            if !bytes[index].is_ascii_uppercase() {
                return false;
            }
        } else if bytes[index] == b'-' {
            segment_start = index + 1;
            if segment_start >= last_hyphen {
                return false;
            }
        } else if !bytes[index].is_ascii_uppercase() && !bytes[index].is_ascii_digit() {
            return false;
        }
        index += 1;
    }

    true
}

const fn valid_prefix(bytes: &[u8], length: usize) -> bool {
    prefix_eq(bytes, length, b"VAL")
        || prefix_eq(bytes, length, b"CON")
        || prefix_eq(bytes, length, b"RES")
        || prefix_eq(bytes, length, b"IO")
        || prefix_eq(bytes, length, b"NET")
        || prefix_eq(bytes, length, b"TO")
        || prefix_eq(bytes, length, b"EXT")
        || prefix_eq(bytes, length, b"CFG")
        || prefix_eq(bytes, length, b"VER")
        || prefix_eq(bytes, length, b"INT")
        || prefix_eq(bytes, length, b"UNS")
}

const fn prefix_eq(bytes: &[u8], length: usize, prefix: &[u8]) -> bool {
    if length != prefix.len() {
        return false;
    }

    let mut index = 0;
    while index < length {
        if bytes[index] != prefix[index] {
            return false;
        }
        index += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ExampleError;

    impl CodedError for ExampleError {
        fn code(&self) -> ErrorCode {
            ErrorCode::new("CON-ERRORS-001")
        }
    }

    #[test]
    fn coded_errors_expose_identity_not_presentation() {
        let error = ExampleError;
        assert_eq!(error.code().as_str(), "CON-ERRORS-001");
        assert_eq!(error.category(), ErrorCategory::Constraint);
        assert_eq!(error.category().as_str(), "constraint");
    }

    #[test]
    fn prefix_categories_preserve_original_contract_meaning() {
        assert_eq!(
            ErrorCode::new("RES-EXAMPLE-001").category(),
            ErrorCategory::Resource
        );
        assert_eq!(
            ErrorCode::new("CON-EXAMPLE-001").category(),
            ErrorCategory::Constraint
        );
    }

    #[test]
    fn accepted_code_shape_supports_component_segments() {
        let code = ErrorCode::new("VAL-EXAMPLE-COMPONENT-001");
        assert_eq!(code.as_str(), "VAL-EXAMPLE-COMPONENT-001");
        assert_eq!(code.category(), ErrorCategory::Validation);
    }

    #[test]
    fn dynamic_validation_matches_the_static_constructor_contract() {
        assert!(ErrorCode::is_valid("EXT-GPTAUTO-004"));
        assert!(!ErrorCode::is_valid("VAL-1COMPONENT-001"));
        assert!(!ErrorCode::is_valid("VAL-COMPONENT_-001"));
        assert!(!ErrorCode::is_valid("VAL--001"));
        assert!(!ErrorCode::is_valid("BAD-COMPONENT-001"));
    }

    #[test]
    #[should_panic(expected = "invalid stable error code")]
    fn invalid_code_shape_is_rejected() {
        let _ = ErrorCode::new("bad-code");
    }
}
