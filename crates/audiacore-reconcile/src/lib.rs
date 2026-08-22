//! Pure desired-versus-observed reconciliation planning.
//!
//! This crate produces effect intent as data. It never applies effects.

use std::{error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode};

const EMPTY_RESOURCE_ID: ErrorCode = ErrorCode::new("VAL-RECONCILE-001");
const EMPTY_OWNER_ID: ErrorCode = ErrorCode::new("VAL-RECONCILE-002");

macro_rules! string_id {
    ($name:ident, $variant:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ReconcileError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ReconcileError::$variant);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(ResourceId, EmptyResourceId);
string_id!(OwnerId, EmptyOwnerId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileError {
    EmptyResourceId,
    EmptyOwnerId,
}

impl CodedError for ReconcileError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EmptyResourceId => EMPTY_RESOURCE_ID,
            Self::EmptyOwnerId => EMPTY_OWNER_ID,
        }
    }
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyResourceId => f.write_str("resource identifier must not be empty"),
            Self::EmptyOwnerId => f.write_str("owner identifier must not be empty"),
        }
    }
}

impl Error for ReconcileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileAction<T> {
    Noop,
    Create(T),
    Replace(T),
    Delete,
}

pub fn plan<T>(desired: Option<&T>, observed: Option<&T>) -> ReconcileAction<T>
where
    T: Clone + Eq,
{
    match (desired, observed) {
        (None, None) => ReconcileAction::Noop,
        (Some(desired), None) => ReconcileAction::Create(desired.clone()),
        (None, Some(_)) => ReconcileAction::Delete,
        (Some(desired), Some(observed)) if desired == observed => ReconcileAction::Noop,
        (Some(desired), Some(_)) => ReconcileAction::Replace(desired.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_state_produces_no_effect() {
        assert_eq!(plan(Some(&"same"), Some(&"same")), ReconcileAction::Noop);
    }

    #[test]
    fn presence_planning_distinguishes_create_replace_and_delete() {
        assert_eq!(plan(Some(&"new"), None), ReconcileAction::Create("new"));
        assert_eq!(
            plan(Some(&"new"), Some(&"old")),
            ReconcileAction::Replace("new")
        );
        assert_eq!(plan::<&str>(None, Some(&"old")), ReconcileAction::Delete);
    }

    #[test]
    fn identifiers_have_distinct_stable_error_identity() {
        assert_eq!(
            ResourceId::new(" ").unwrap_err().code().as_str(),
            "VAL-RECONCILE-001"
        );
        assert_eq!(
            OwnerId::new("").unwrap_err().code().as_str(),
            "VAL-RECONCILE-002"
        );
    }
}
