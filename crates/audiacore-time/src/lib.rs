//! Deterministic caller-owned deadline and timer semantics.
//!
//! Callers supply absolute timestamps explicitly. This crate does not read a
//! clock, sleep, schedule work, spawn tasks, or register global timers.

use std::{collections::BTreeMap, error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};

const TIMER_ID_EMPTY: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-TIME-001"),
    "Timer identifier must not be empty.",
    "Provide a non-empty timer identifier.",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub const fn from_millis(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deadline(Timestamp);

impl Deadline {
    pub const fn at(timestamp: Timestamp) -> Self {
        Self(timestamp)
    }

    pub const fn timestamp(self) -> Timestamp {
        self.0
    }

    pub const fn is_due(self, now: Timestamp) -> bool {
        now.0 >= self.0.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimerId(String);

impl TimerId {
    pub fn new(value: impl Into<String>) -> Result<Self, TimerIdError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(TimerIdError);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerIdError;

impl CodedError for TimerIdError {
    fn definition(&self) -> &'static ErrorDefinition {
        &TIMER_ID_EMPTY
    }
}

impl fmt::Display for TimerIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("timer identifier must not be empty")
    }
}

impl Error for TimerIdError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerSet {
    timers: BTreeMap<TimerId, Deadline>,
}

#[allow(clippy::new_without_default)]
impl TimerSet {
    pub fn new() -> Self {
        Self {
            timers: BTreeMap::new(),
        }
    }

    pub fn arm(&mut self, id: TimerId, deadline: Deadline) -> Option<Deadline> {
        self.timers.insert(id, deadline)
    }

    pub fn cancel(&mut self, id: &TimerId) -> Option<Deadline> {
        self.timers.remove(id)
    }

    pub fn next_deadline(&self) -> Option<Deadline> {
        self.timers.values().copied().min()
    }

    pub fn drain_due(&mut self, now: Timestamp) -> Vec<TimerId> {
        let mut due = self
            .timers
            .iter()
            .filter(|(_, deadline)| deadline.is_due(now))
            .map(|(id, deadline)| (deadline.timestamp(), id.clone()))
            .collect::<Vec<_>>();
        due.sort();

        for (_, id) in &due {
            self.timers.remove(id);
        }

        due.into_iter().map(|(_, id)| id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadlines_are_evaluated_only_from_caller_supplied_time() {
        let deadline = Deadline::at(Timestamp::from_millis(100));

        assert!(!deadline.is_due(Timestamp::from_millis(99)));
        assert!(deadline.is_due(Timestamp::from_millis(100)));
        assert!(deadline.is_due(Timestamp::from_millis(101)));
    }

    #[test]
    fn timer_set_orders_and_drains_due_timers_deterministically() {
        let mut timers = TimerSet::new();
        timers.arm(
            TimerId::new("later").unwrap(),
            Deadline::at(Timestamp::from_millis(20)),
        );
        timers.arm(
            TimerId::new("first-b").unwrap(),
            Deadline::at(Timestamp::from_millis(10)),
        );
        timers.arm(
            TimerId::new("first-a").unwrap(),
            Deadline::at(Timestamp::from_millis(10)),
        );

        assert_eq!(
            timers.next_deadline(),
            Some(Deadline::at(Timestamp::from_millis(10)))
        );
        assert_eq!(
            timers.drain_due(Timestamp::from_millis(10)),
            vec![
                TimerId::new("first-a").unwrap(),
                TimerId::new("first-b").unwrap(),
            ]
        );
        assert_eq!(
            timers.next_deadline(),
            Some(Deadline::at(Timestamp::from_millis(20)))
        );
        assert!(timers.drain_due(Timestamp::from_millis(19)).is_empty());
        assert_eq!(
            timers.drain_due(Timestamp::from_millis(20)),
            vec![TimerId::new("later").unwrap()]
        );
        assert_eq!(timers.next_deadline(), None);
    }

    #[test]
    fn rearming_and_cancelling_return_the_replaced_deadline() {
        let id = TimerId::new("heartbeat").unwrap();
        let first = Deadline::at(Timestamp::from_millis(10));
        let second = Deadline::at(Timestamp::from_millis(20));
        let mut timers = TimerSet::new();

        assert_eq!(timers.arm(id.clone(), first), None);
        assert_eq!(timers.arm(id.clone(), second), Some(first));
        assert_eq!(timers.cancel(&id), Some(second));
        assert_eq!(timers.next_deadline(), None);
    }

    #[test]
    fn timer_identifier_validation_has_stable_error_identity() {
        let error = TimerId::new(" ").unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-TIME-001");
    }
}
