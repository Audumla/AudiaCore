//! Deterministic caller-owned workflow state machines.
//!
//! Workflow owns workflow-local lifecycle, optimistic revision checks, and
//! effects-as-data. It does not publish events, persist snapshots, schedule
//! work, execute effects, own clocks, retry failures, or provide a global
//! workflow runtime.

use std::{error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};

const WORKFLOW_ID_EMPTY: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-WORKFLOW-001"),
    "Workflow instance identifier must not be empty.",
    "Provide a non-empty workflow instance identifier.",
);
const WORKFLOW_TERMINAL: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-WORKFLOW-001"),
    "Workflow instance is already terminal.",
    "Do not apply further transitions to a completed or failed workflow instance.",
);
const WORKFLOW_REVISION_CONFLICT: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-WORKFLOW-002"),
    "Workflow revision does not match the expected revision.",
    "Reload the latest workflow state and retry the decision against that revision.",
);
const WORKFLOW_DEFINITION_REJECTED: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-WORKFLOW-003"),
    "Workflow definition rejected the transition.",
    "Inspect the domain-specific transition error and correct the requested event or state.",
);
const WORKFLOW_REVISION_EXHAUSTED: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("RES-WORKFLOW-001"),
    "Workflow revision space is exhausted.",
    "Create a new workflow instance rather than allowing revision identity to wrap.",
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowInstanceId(String);

impl WorkflowInstanceId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowIdError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(WorkflowIdError);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowIdError;

impl CodedError for WorkflowIdError {
    fn definition(&self) -> &'static ErrorDefinition {
        &WORKFLOW_ID_EMPTY
    }
}

impl fmt::Display for WorkflowIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("workflow instance identifier must not be empty")
    }
}

impl Error for WorkflowIdError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
}

impl WorkflowStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowTransition<S, E> {
    Continue { state: S, effects: Vec<E> },
    Complete { state: S, effects: Vec<E> },
    Fail { state: S, effects: Vec<E> },
}

pub trait WorkflowDefinition {
    type State;
    type Event;
    type Effect;
    type Error;

    fn decide(
        &self,
        state: &Self::State,
        event: &Self::Event,
    ) -> Result<WorkflowTransition<Self::State, Self::Effect>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowReceipt<E> {
    revision: u64,
    status: WorkflowStatus,
    effects: Vec<E>,
}

impl<E> WorkflowReceipt<E> {
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn status(&self) -> WorkflowStatus {
        self.status
    }

    pub fn effects(&self) -> &[E] {
        &self.effects
    }

    pub fn into_effects(self) -> Vec<E> {
        self.effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowSnapshot<S> {
    id: WorkflowInstanceId,
    revision: u64,
    status: WorkflowStatus,
    state: S,
}

impl<S> WorkflowSnapshot<S> {
    pub fn id(&self) -> &WorkflowInstanceId {
        &self.id
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn status(&self) -> WorkflowStatus {
        self.status
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowError<E> {
    Terminal(WorkflowStatus),
    RevisionConflict { expected: u64, actual: u64 },
    Definition(E),
    RevisionExhausted,
}

impl<E> CodedError for WorkflowError<E> {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::Terminal(_) => &WORKFLOW_TERMINAL,
            Self::RevisionConflict { .. } => &WORKFLOW_REVISION_CONFLICT,
            Self::Definition(_) => &WORKFLOW_DEFINITION_REJECTED,
            Self::RevisionExhausted => &WORKFLOW_REVISION_EXHAUSTED,
        }
    }
}

impl<E: fmt::Display> fmt::Display for WorkflowError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Terminal(status) => write!(f, "workflow instance is terminal: {status:?}"),
            Self::RevisionConflict { expected, actual } => write!(
                f,
                "workflow revision conflict: expected {expected}, actual {actual}"
            ),
            Self::Definition(error) => write!(f, "workflow definition rejected transition: {error}"),
            Self::RevisionExhausted => f.write_str("workflow revision space is exhausted"),
        }
    }
}

impl<E: Error + 'static> Error for WorkflowError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Definition(error) => Some(error),
            Self::Terminal(_) | Self::RevisionConflict { .. } | Self::RevisionExhausted => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowInstance<S> {
    id: WorkflowInstanceId,
    revision: u64,
    status: WorkflowStatus,
    state: S,
}

impl<S> WorkflowInstance<S> {
    pub fn new(id: WorkflowInstanceId, initial_state: S) -> Self {
        Self {
            id,
            revision: 0,
            status: WorkflowStatus::Running,
            state: initial_state,
        }
    }

    pub fn restore(snapshot: WorkflowSnapshot<S>) -> Self {
        Self {
            id: snapshot.id,
            revision: snapshot.revision,
            status: snapshot.status,
            state: snapshot.state,
        }
    }

    pub fn id(&self) -> &WorkflowInstanceId {
        &self.id
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn status(&self) -> WorkflowStatus {
        self.status
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn snapshot(&self) -> WorkflowSnapshot<S>
    where
        S: Clone,
    {
        WorkflowSnapshot {
            id: self.id.clone(),
            revision: self.revision,
            status: self.status,
            state: self.state.clone(),
        }
    }

    pub fn apply_at<D>(
        &mut self,
        definition: &D,
        expected_revision: u64,
        event: &D::Event,
    ) -> Result<WorkflowReceipt<D::Effect>, WorkflowError<D::Error>>
    where
        D: WorkflowDefinition<State = S>,
    {
        if expected_revision != self.revision {
            return Err(WorkflowError::RevisionConflict {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        if self.status.is_terminal() {
            return Err(WorkflowError::Terminal(self.status));
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(WorkflowError::RevisionExhausted)?;

        let transition = definition
            .decide(&self.state, event)
            .map_err(WorkflowError::Definition)?;
        let (state, status, effects) = match transition {
            WorkflowTransition::Continue { state, effects } => {
                (state, WorkflowStatus::Running, effects)
            }
            WorkflowTransition::Complete { state, effects } => {
                (state, WorkflowStatus::Completed, effects)
            }
            WorkflowTransition::Fail { state, effects } => {
                (state, WorkflowStatus::Failed, effects)
            }
        };

        self.state = state;
        self.status = status;
        self.revision = next_revision;

        Ok(WorkflowReceipt {
            revision: self.revision,
            status: self.status,
            effects,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, convert::Infallible};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct State {
        steps: u8,
    }

    #[derive(Debug, Clone, Copy)]
    enum Event {
        Step,
        Complete,
        Fail,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Effect {
        Progress(u8),
        Finished,
        Failed,
    }

    struct Definition {
        calls: Cell<usize>,
    }

    impl Definition {
        fn new() -> Self {
            Self {
                calls: Cell::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.get()
        }
    }

    impl WorkflowDefinition for Definition {
        type State = State;
        type Event = Event;
        type Effect = Effect;
        type Error = Infallible;

        fn decide(
            &self,
            state: &Self::State,
            event: &Self::Event,
        ) -> Result<WorkflowTransition<Self::State, Self::Effect>, Self::Error> {
            self.calls.set(self.calls.get() + 1);
            Ok(match event {
                Event::Step => {
                    let steps = state.steps + 1;
                    WorkflowTransition::Continue {
                        state: State { steps },
                        effects: vec![Effect::Progress(steps)],
                    }
                }
                Event::Complete => WorkflowTransition::Complete {
                    state: state.clone(),
                    effects: vec![Effect::Finished],
                },
                Event::Fail => WorkflowTransition::Fail {
                    state: state.clone(),
                    effects: vec![Effect::Failed],
                },
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DomainError;

    impl fmt::Display for DomainError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("transition rejected")
        }
    }

    impl Error for DomainError {}

    struct RejectingDefinition;

    impl WorkflowDefinition for RejectingDefinition {
        type State = State;
        type Event = Event;
        type Effect = Effect;
        type Error = DomainError;

        fn decide(
            &self,
            _state: &Self::State,
            _event: &Self::Event,
        ) -> Result<WorkflowTransition<Self::State, Self::Effect>, Self::Error> {
            Err(DomainError)
        }
    }

    fn instance() -> WorkflowInstance<State> {
        WorkflowInstance::new(
            WorkflowInstanceId::new("workflow-1").unwrap(),
            State { steps: 0 },
        )
    }

    #[test]
    fn workflow_assigns_revision_status_and_effects_deterministically() {
        let definition = Definition::new();
        let mut workflow = instance();

        let first = workflow.apply_at(&definition, 0, &Event::Step).unwrap();
        assert_eq!(first.revision(), 1);
        assert_eq!(first.status(), WorkflowStatus::Running);
        assert_eq!(first.effects(), &[Effect::Progress(1)]);
        assert_eq!(workflow.state(), &State { steps: 1 });

        let completed = workflow.apply_at(&definition, 1, &Event::Complete).unwrap();
        assert_eq!(completed.revision(), 2);
        assert_eq!(completed.status(), WorkflowStatus::Completed);
        assert_eq!(completed.effects(), &[Effect::Finished]);
    }

    #[test]
    fn revision_conflict_and_terminal_state_reject_before_domain_logic() {
        let definition = Definition::new();
        let mut workflow = instance();

        let conflict = workflow.apply_at(&definition, 1, &Event::Step).unwrap_err();
        assert_eq!(conflict.code().as_str(), "CON-WORKFLOW-002");
        assert_eq!(definition.calls(), 0);
        assert_eq!(workflow.revision(), 0);

        workflow
            .apply_at(&definition, 0, &Event::Complete)
            .unwrap();
        let calls = definition.calls();
        let terminal = workflow.apply_at(&definition, 1, &Event::Step).unwrap_err();
        assert_eq!(terminal.code().as_str(), "CON-WORKFLOW-001");
        assert_eq!(definition.calls(), calls);
        assert_eq!(workflow.revision(), 1);
    }

    #[test]
    fn revision_exhaustion_rejects_before_domain_logic_or_mutation() {
        let definition = Definition::new();
        let mut workflow = instance();
        workflow.revision = u64::MAX;
        let before = workflow.state.clone();

        let error = workflow
            .apply_at(&definition, u64::MAX, &Event::Step)
            .unwrap_err();

        assert_eq!(error.code().as_str(), "RES-WORKFLOW-001");
        assert_eq!(definition.calls(), 0);
        assert_eq!(workflow.revision(), u64::MAX);
        assert_eq!(workflow.state, before);
    }

    #[test]
    fn definition_rejection_has_stable_error_identity_and_does_not_mutate() {
        let mut workflow = instance();
        let before = workflow.clone();

        let error = workflow
            .apply_at(&RejectingDefinition, 0, &Event::Step)
            .unwrap_err();

        assert_eq!(error.code().as_str(), "CON-WORKFLOW-003");
        assert_eq!(error.source().unwrap().to_string(), "transition rejected");
        assert_eq!(workflow, before);
    }

    #[test]
    fn snapshot_is_an_owned_restorable_consistency_checkpoint() {
        let definition = Definition::new();
        let mut workflow = instance();
        workflow.apply_at(&definition, 0, &Event::Step).unwrap();

        let snapshot = workflow.snapshot();
        workflow.apply_at(&definition, 1, &Event::Fail).unwrap();

        assert_eq!(snapshot.id().as_str(), "workflow-1");
        assert_eq!(snapshot.revision(), 1);
        assert_eq!(snapshot.status(), WorkflowStatus::Running);
        assert_eq!(snapshot.state(), &State { steps: 1 });
        assert_eq!(workflow.status(), WorkflowStatus::Failed);

        let restored = WorkflowInstance::restore(snapshot);
        assert_eq!(restored.id().as_str(), "workflow-1");
        assert_eq!(restored.revision(), 1);
        assert_eq!(restored.status(), WorkflowStatus::Running);
        assert_eq!(restored.state(), &State { steps: 1 });
    }

    #[test]
    fn identifier_validation_has_stable_error_identity() {
        assert_eq!(
            WorkflowInstanceId::new(" ").unwrap_err().code().as_str(),
            "VAL-WORKFLOW-001"
        );
    }
}
