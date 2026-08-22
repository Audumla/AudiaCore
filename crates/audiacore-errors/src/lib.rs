//! Stable reusable error identity without a global error framework.
//!
//! Owning crates keep their typed Rust errors and dynamic context. This crate
//! provides only stable code/message/resolution metadata for failures that
//! cross reusable capability or application boundaries.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Validation,
    Conflict,
    Resolution,
    Io,
    Network,
    Timeout,
    External,
    Configuration,
    Version,
    Internal,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode(&'static str);

impl ErrorCode {
    pub const fn new(code: &'static str) -> Self {
        assert!(valid_code(code), "invalid stable error code");
        Self(code)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub fn category(self) -> ErrorCategory {
        if self.0.starts_with("VAL-") {
            ErrorCategory::Validation
        } else if self.0.starts_with("CON-") {
            ErrorCategory::Conflict
        } else if self.0.starts_with("RES-") {
            ErrorCategory::Resolution
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorDefinition {
    code: ErrorCode,
    message: &'static str,
    resolution: &'static str,
}

impl ErrorDefinition {
    pub const fn new(
        code: ErrorCode,
        message: &'static str,
        resolution: &'static str,
    ) -> Self {
        assert!(!message.is_empty(), "canonical error message must not be empty");
        assert!(!resolution.is_empty(), "error resolution must not be empty");
        Self {
            code,
            message,
            resolution,
        }
    }

    pub const fn code(self) -> ErrorCode {
        self.code
    }

    pub const fn message(self) -> &'static str {
        self.message
    }

    pub const fn resolution(self) -> &'static str {
        self.resolution
    }

    pub fn category(self) -> ErrorCategory {
        self.code.category()
    }
}

pub trait CodedError {
    fn definition(&self) -> &'static ErrorDefinition;

    fn code(&self) -> ErrorCode {
        self.definition().code()
    }

    fn category(&self) -> ErrorCategory {
        self.definition().category()
    }

    fn canonical_message(&self) -> &'static str {
        self.definition().message()
    }

    fn resolution(&self) -> &'static str {
        self.definition().resolution()
    }
}

const fn valid_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    if bytes.len() < 8 {
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

    valid_prefix(bytes, first_hyphen)
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

    const EXAMPLE: ErrorDefinition = ErrorDefinition::new(
        ErrorCode::new("CON-ERRORS-001"),
        "Example semantic condition occurred.",
        "Correct the example condition and retry.",
    );

    struct ExampleError;

    impl CodedError for ExampleError {
        fn definition(&self) -> &'static ErrorDefinition {
            &EXAMPLE
        }
    }

    #[test]
    fn definitions_keep_one_code_message_resolution_and_category() {
        let error = ExampleError;
        assert_eq!(error.code().as_str(), "CON-ERRORS-001");
        assert_eq!(error.category(), ErrorCategory::Conflict);
        assert_eq!(
            error.canonical_message(),
            "Example semantic condition occurred."
        );
        assert_eq!(
            error.resolution(),
            "Correct the example condition and retry."
        );
    }

    #[test]
    fn accepted_code_shape_supports_component_segments() {
        let code = ErrorCode::new("VAL-EXAMPLE-COMPONENT-001");
        assert_eq!(code.as_str(), "VAL-EXAMPLE-COMPONENT-001");
        assert_eq!(code.category(), ErrorCategory::Validation);
    }

    #[test]
    #[should_panic(expected = "invalid stable error code")]
    fn invalid_code_shape_is_rejected() {
        let _ = ErrorCode::new("bad-code");
    }
}
