//! Caller-owned ordered domain-event streams with explicit retention policy.
//!
//! This capability owns event identity, correlation/causation metadata,
//! monotonic sequence assignment, bounded in-memory retention, and cursor
//! paging. It is not a bus, broker, publisher, subscription registry, durable
//! store, retry engine, or transport.

use std::{collections::VecDeque, error::Error, fmt, num::NonZeroUsize};

use audiacore_core::CorrelationId;
use audiacore_errors::{CodedError, ErrorCode, ErrorDefinition};

const EVENT_ID_EMPTY: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-EVENT-001"),
    "Event identifier must not be empty.",
    "Provide a non-empty event, stream, or causation identifier.",
);
const ZERO_RETENTION_LIMIT: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-EVENT-002"),
    "Event retention limit must be greater than zero.",
    "Configure a positive retention limit or use an unbounded event policy.",
);
const ZERO_PAGE_LIMIT: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("VAL-EVENT-003"),
    "Event page limit must be greater than zero.",
    "Request at least one event per page.",
);
const CURSOR_EXPIRED: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-EVENT-001"),
    "Event cursor has expired.",
    "Restart from an available cursor or use durable storage when replay is required.",
);
const CURSOR_AHEAD: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("CON-EVENT-002"),
    "Event cursor is ahead of the stream.",
    "Use a cursor at or before the latest available event sequence.",
);
const SEQUENCE_EXHAUSTED: ErrorDefinition = ErrorDefinition::new(
    ErrorCode::new("RES-EVENT-001"),
    "Event sequence space is exhausted.",
    "Start a new event stream rather than allowing sequence identity to wrap.",
);

macro_rules! event_id {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EventIdError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(EventIdError($label));
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

event_id!(EventId, "event id");
event_id!(EventStreamId, "event stream id");
event_id!(CausationId, "causation id");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventIdError(&'static str);

impl CodedError for EventIdError {
    fn definition(&self) -> &'static ErrorDefinition {
        &EVENT_ID_EMPTY
    }
}

impl fmt::Display for EventIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} must not be empty", self.0)
    }
}

impl Error for EventIdError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventSequence(u64);

impl EventSequence {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventCursor(u64);

impl EventCursor {
    pub const fn start() -> Self {
        Self(0)
    }

    pub const fn new(last_seen_sequence: u64) -> Self {
        Self(last_seen_sequence)
    }

    pub const fn from_sequence(sequence: EventSequence) -> Self {
        Self(sequence.get())
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventPolicy {
    retention_limit: Option<NonZeroUsize>,
}

impl EventPolicy {
    pub const fn unbounded() -> Self {
        Self {
            retention_limit: None,
        }
    }

    pub fn bounded(retention_limit: usize) -> Result<Self, EventStreamError> {
        let retention_limit =
            NonZeroUsize::new(retention_limit).ok_or(EventStreamError::ZeroRetentionLimit)?;
        Ok(Self {
            retention_limit: Some(retention_limit),
        })
    }

    pub fn retention_limit(self) -> Option<usize> {
        self.retention_limit.map(NonZeroUsize::get)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope<E> {
    event_id: EventId,
    stream_id: EventStreamId,
    sequence: EventSequence,
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
    payload: E,
}

impl<E> EventEnvelope<E> {
    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn stream_id(&self) -> &EventStreamId {
        &self.stream_id
    }

    pub const fn sequence(&self) -> EventSequence {
        self.sequence
    }

    pub fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    pub fn causation_id(&self) -> Option<&CausationId> {
        self.causation_id.as_ref()
    }

    pub fn payload(&self) -> &E {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventStreamError {
    ZeroRetentionLimit,
    ZeroPageLimit,
    CursorExpired {
        cursor: EventCursor,
        oldest_available: EventSequence,
    },
    CursorAhead {
        cursor: EventCursor,
        latest_available: EventSequence,
    },
    SequenceExhausted,
}

impl CodedError for EventStreamError {
    fn definition(&self) -> &'static ErrorDefinition {
        match self {
            Self::ZeroRetentionLimit => &ZERO_RETENTION_LIMIT,
            Self::ZeroPageLimit => &ZERO_PAGE_LIMIT,
            Self::CursorExpired { .. } => &CURSOR_EXPIRED,
            Self::CursorAhead { .. } => &CURSOR_AHEAD,
            Self::SequenceExhausted => &SEQUENCE_EXHAUSTED,
        }
    }
}

impl fmt::Display for EventStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRetentionLimit => {
                f.write_str("event retention limit must be greater than zero")
            }
            Self::ZeroPageLimit => f.write_str("event page limit must be greater than zero"),
            Self::CursorExpired {
                cursor,
                oldest_available,
            } => write!(
                f,
                "event cursor {} expired; oldest available sequence is {}",
                cursor.get(),
                oldest_available.get()
            ),
            Self::CursorAhead {
                cursor,
                latest_available,
            } => write!(
                f,
                "event cursor {} is ahead of latest available sequence {}",
                cursor.get(),
                latest_available.get()
            ),
            Self::SequenceExhausted => f.write_str("event sequence space is exhausted"),
        }
    }
}

impl Error for EventStreamError {}

#[derive(Debug)]
pub struct EventPage<'a, E> {
    events: Vec<&'a EventEnvelope<E>>,
    next_cursor: EventCursor,
    has_more: bool,
}

impl<'a, E> EventPage<'a, E> {
    pub fn events(&self) -> &[&'a EventEnvelope<E>] {
        &self.events
    }

    pub const fn next_cursor(&self) -> EventCursor {
        self.next_cursor
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }
}

#[derive(Debug, Clone)]
pub struct EventStream<E> {
    stream_id: EventStreamId,
    events: VecDeque<EventEnvelope<E>>,
    next_sequence: u64,
    policy: EventPolicy,
}

impl<E> EventStream<E> {
    pub fn new(stream_id: EventStreamId, policy: EventPolicy) -> Self {
        let events = policy
            .retention_limit()
            .map(VecDeque::with_capacity)
            .unwrap_or_default();
        Self {
            stream_id,
            events,
            next_sequence: 1,
            policy,
        }
    }

    pub fn stream_id(&self) -> &EventStreamId {
        &self.stream_id
    }

    pub const fn policy(&self) -> EventPolicy {
        self.policy
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn oldest_sequence(&self) -> Option<EventSequence> {
        self.events.front().map(EventEnvelope::sequence)
    }

    pub fn last_sequence(&self) -> Option<EventSequence> {
        self.events.back().map(EventEnvelope::sequence)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EventEnvelope<E>> {
        self.events.iter()
    }

    pub fn append(
        &mut self,
        event_id: EventId,
        correlation_id: CorrelationId,
        causation_id: Option<CausationId>,
        payload: E,
    ) -> Result<EventSequence, EventStreamError> {
        let sequence = EventSequence::new(self.next_sequence);
        let next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(EventStreamError::SequenceExhausted)?;

        self.events.push_back(EventEnvelope {
            event_id,
            stream_id: self.stream_id.clone(),
            sequence,
            correlation_id,
            causation_id,
            payload,
        });
        self.next_sequence = next_sequence;

        if let Some(limit) = self.policy.retention_limit() {
            while self.events.len() > limit {
                self.events.pop_front();
            }
        }

        Ok(sequence)
    }

    pub fn page_after(
        &self,
        cursor: EventCursor,
        limit: usize,
    ) -> Result<EventPage<'_, E>, EventStreamError> {
        if limit == 0 {
            return Err(EventStreamError::ZeroPageLimit);
        }

        let Some(oldest) = self.oldest_sequence() else {
            if cursor.get() > 0 {
                return Err(EventStreamError::CursorAhead {
                    cursor,
                    latest_available: EventSequence::new(0),
                });
            }
            return Ok(EventPage {
                events: Vec::new(),
                next_cursor: cursor,
                has_more: false,
            });
        };
        let latest = self.last_sequence().unwrap_or(oldest);

        if cursor.get().saturating_add(1) < oldest.get() {
            return Err(EventStreamError::CursorExpired {
                cursor,
                oldest_available: oldest,
            });
        }
        if cursor.get() > latest.get() {
            return Err(EventStreamError::CursorAhead {
                cursor,
                latest_available: latest,
            });
        }

        let events = self
            .events
            .iter()
            .filter(|event| event.sequence().get() > cursor.get())
            .take(limit)
            .collect::<Vec<_>>();
        let next_cursor = events
            .last()
            .map(|event| EventCursor::from_sequence(event.sequence()))
            .unwrap_or(cursor);
        let has_more = self
            .events
            .iter()
            .any(|event| event.sequence().get() > next_cursor.get());

        Ok(EventPage {
            events,
            next_cursor,
            has_more,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum JobEvent {
        Started,
        Progress(u8),
        Completed,
    }

    fn stream(policy: EventPolicy) -> EventStream<JobEvent> {
        EventStream::new(EventStreamId::new("job-42").unwrap(), policy)
    }

    fn append(stream: &mut EventStream<JobEvent>, id: u64, payload: JobEvent) {
        stream
            .append(
                EventId::new(format!("event-{id}")).unwrap(),
                CorrelationId::new("corr-42").unwrap(),
                None,
                payload,
            )
            .unwrap();
    }

    #[test]
    fn policy_is_explicit_and_stream_assigns_monotonic_order() {
        let policy = EventPolicy::unbounded();
        let mut events = stream(policy);
        append(&mut events, 1, JobEvent::Started);
        append(&mut events, 2, JobEvent::Completed);

        assert_eq!(events.policy(), policy);
        assert_eq!(events.len(), 2);
        assert_eq!(events.last_sequence(), Some(EventSequence::new(2)));
        assert_eq!(events.iter().nth(1).unwrap().payload(), &JobEvent::Completed);
    }

    #[test]
    fn bounded_retention_expires_old_cursor_and_pages_retained_evidence() {
        let mut events = stream(EventPolicy::bounded(3).unwrap());
        append(&mut events, 1, JobEvent::Started);
        append(&mut events, 2, JobEvent::Progress(1));
        append(&mut events, 3, JobEvent::Progress(2));
        append(&mut events, 4, JobEvent::Completed);

        assert_eq!(events.oldest_sequence(), Some(EventSequence::new(2)));
        let expired = events.page_after(EventCursor::start(), 2).unwrap_err();
        assert_eq!(expired.code().as_str(), "CON-EVENT-001");

        let first = events.page_after(EventCursor::new(1), 2).unwrap();
        assert_eq!(first.events().len(), 2);
        assert_eq!(first.next_cursor(), EventCursor::new(3));
        assert!(first.has_more());

        let second = events.page_after(first.next_cursor(), 2).unwrap();
        assert_eq!(second.events().len(), 1);
        assert_eq!(second.next_cursor(), EventCursor::new(4));
        assert!(!second.has_more());
    }

    #[test]
    fn ahead_empty_and_zero_limit_have_distinct_semantics() {
        let events = stream(EventPolicy::unbounded());
        assert!(events.page_after(EventCursor::start(), 1).unwrap().events().is_empty());

        let ahead = events.page_after(EventCursor::new(1), 1).unwrap_err();
        assert_eq!(ahead.code().as_str(), "CON-EVENT-002");

        let zero = events.page_after(EventCursor::start(), 0).unwrap_err();
        assert_eq!(zero.code().as_str(), "VAL-EVENT-003");
    }

    #[test]
    fn invalid_policy_and_identifier_have_stable_codes() {
        assert_eq!(
            EventPolicy::bounded(0).unwrap_err().code().as_str(),
            "VAL-EVENT-002"
        );
        assert_eq!(EventId::new(" ").unwrap_err().code().as_str(), "VAL-EVENT-001");
    }

    #[test]
    fn sequence_exhaustion_never_wraps_or_appends() {
        let mut events = stream(EventPolicy::unbounded());
        events.next_sequence = u64::MAX;

        let error = events
            .append(
                EventId::new("last").unwrap(),
                CorrelationId::new("corr").unwrap(),
                None,
                JobEvent::Completed,
            )
            .unwrap_err();

        assert_eq!(error.code().as_str(), "RES-EVENT-001");
        assert!(events.is_empty());
        assert_eq!(events.next_sequence, u64::MAX);
    }
}
