//! Universal application and execution identity plus opaque composition.
//!
//! This crate is the dependency floor. It deliberately owns no capability,
//! policy, authority, lifecycle, diagnostic, I/O, runtime, serialization, or
//! provider semantics.

use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierError {
    kind: &'static str,
}

impl IdentifierError {
    const fn empty(kind: &'static str) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> &'static str {
        self.kind
    }
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} must not be empty", self.kind)
    }
}

impl Error for IdentifierError {}

macro_rules! string_id {
    ($name:ident, $kind:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(IdentifierError::empty($kind));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(ApplicationId, "application id");
string_id!(ExecutionId, "execution id");
string_id!(CorrelationId, "correlation id");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    execution_id: ExecutionId,
    correlation_id: CorrelationId,
}

impl ExecutionContext {
    pub const fn new(execution_id: ExecutionId, correlation_id: CorrelationId) -> Self {
        Self {
            execution_id,
            correlation_id,
        }
    }

    pub const fn execution_id(&self) -> &ExecutionId {
        &self.execution_id
    }

    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application<C> {
    id: ApplicationId,
    composition: C,
}

impl<C> Application<C> {
    pub const fn new(id: ApplicationId, composition: C) -> Self {
        Self { id, composition }
    }

    pub const fn id(&self) -> &ApplicationId {
        &self.id
    }

    pub const fn composition(&self) -> &C {
        &self.composition
    }

    pub fn into_composition(self) -> C {
        self.composition
    }

    pub fn map<D>(self, map: impl FnOnce(C) -> D) -> Application<D> {
        Application {
            id: self.id,
            composition: map(self.composition),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_reject_empty_values() {
        for value in ["", " ", "\t\n"] {
            assert!(ApplicationId::new(value).is_err());
            assert!(ExecutionId::new(value).is_err());
            assert!(CorrelationId::new(value).is_err());
        }
    }

    #[test]
    fn identifiers_preserve_non_empty_values() {
        let id = ApplicationId::new("app-one").unwrap();
        assert_eq!(id.as_str(), "app-one");
        assert_eq!(id.to_string(), "app-one");
    }

    #[test]
    fn execution_context_carries_identity_without_runtime_semantics() {
        let execution = ExecutionContext::new(
            ExecutionId::new("exec-1").unwrap(),
            CorrelationId::new("corr-1").unwrap(),
        );

        assert_eq!(execution.execution_id().as_str(), "exec-1");
        assert_eq!(execution.correlation_id().as_str(), "corr-1");
    }

    #[test]
    fn application_composition_is_opaque_and_replaceable() {
        let application = Application::new(
            ApplicationId::new("demo").unwrap(),
            ("events", 4_u32),
        );

        let mapped = application.map(|(_, retention)| retention as u64);

        assert_eq!(mapped.id().as_str(), "demo");
        assert_eq!(*mapped.composition(), 4_u64);
        assert_eq!(mapped.into_composition(), 4_u64);
    }
}
