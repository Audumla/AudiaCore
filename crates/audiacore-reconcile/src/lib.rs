//! Pure desired-versus-observed reconciliation planning.
//!
//! This crate produces effect intent as data. It owns no resource identity,
//! ownership, authority, host access, persistence, or presentation semantics.

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
}
