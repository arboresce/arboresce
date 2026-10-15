use std::fmt;

/// A prepared formatter sink that rejects output without touching user state.
pub struct RejectedOutput;

impl fmt::Write for RejectedOutput {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

use arboresce_domain::{ContentRevision, ControlRevision, Generation, RevisionError};
use serde_json::{Value, json};

/// Decode owned, frozen test data; this is not user JSON admission.
pub fn revision_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/revision-values-v1/cases.json")).unwrap()
}

pub fn revision_payload(kind: &str, input: &str) -> Result<(u64, String), RevisionError> {
    match kind {
        "content" => input
            .parse::<ContentRevision>()
            .map(|v| (v.value(), v.to_string())),
        "control" => input
            .parse::<ControlRevision>()
            .map(|v| (v.value(), v.to_string())),
        "generation" => input
            .parse::<Generation>()
            .map(|v| (v.value(), v.to_string())),
        _ => panic!("unknown prepared revision fixture kind"),
    }
}

pub fn revision_numeric(kind: &str, input: u64) -> Result<(u64, String), RevisionError> {
    match kind {
        "content" => ContentRevision::new(input).map(|v| (v.value(), v.to_string())),
        "control" => ControlRevision::new(input).map(|v| (v.value(), v.to_string())),
        "generation" => Generation::new(input).map(|v| (v.value(), v.to_string())),
        _ => panic!("unknown prepared revision fixture kind"),
    }
}

pub fn revision_advance(kind: &str, input: &str) -> (Value, Option<RevisionError>) {
    macro_rules! advance {
        ($value:ty) => {{
            let before = input.parse::<$value>().unwrap();
            let next = before.checked_next();
            (
                json!({"accepted":next.is_ok(),"after":next.as_ref().ok().map(ToString::to_string),
                    "original_after_attempt":before.to_string()}),
                next.err(),
            )
        }};
    }
    match kind {
        "content" => advance!(ContentRevision),
        "control" => advance!(ControlRevision),
        "generation" => advance!(Generation),
        _ => panic!("unknown prepared revision fixture kind"),
    }
}

pub fn revision_traits(kind: &str, values: &[Value]) -> Value {
    use std::{collections::HashSet, fmt::Write};
    macro_rules! traits {
        ($value:ty) => {{
            let first = <$value>::new(values[0].as_u64().unwrap()).unwrap();
            let second = <$value>::new(values[1].as_u64().unwrap()).unwrap();
            let copied = first;
            let set: HashSet<_> = [first, copied, second].into_iter().collect();
            let mut output = RejectedOutput;
            json!({"copied_equal":copied==first,"ordered":first<second,"distinct_set_size":set.len(),
                "debug_first":format!("{first:?}"),"canonical_first":first.to_string(),
                "canonical_second":second.to_string(),"formatter_error_propagates":write!(&mut output,"{first}").is_err()})
        }};
    }
    match kind {
        "content" => traits!(ContentRevision),
        "control" => traits!(ControlRevision),
        "generation" => traits!(Generation),
        _ => panic!("unknown prepared revision fixture kind"),
    }
}

/// Read independent original date/time/clock vectors without changing them.
pub fn primitive_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/primitives-v1/semantics.json")).unwrap()
}

pub fn time_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/time-values-v1/cases.json")).unwrap()
}

use arboresce_domain::{CalendarDate, Clock, UtcTimestamp};
use std::collections::VecDeque;

/// Prepared test supplier; failures are supplied alongside the successful values.
pub struct SuppliedClock {
    outcomes: VecDeque<Result<UtcTimestamp, String>>,
    pub calls: usize,
}

impl SuppliedClock {
    pub fn new(outcomes: &[Value]) -> Self {
        Self {
            outcomes: outcomes
                .iter()
                .map(|row| match row.get("ok") {
                    Some(value) => Ok(value.as_str().unwrap().parse().unwrap()),
                    None => Err(row["error"].as_str().unwrap().to_owned()),
                })
                .collect(),
            calls: 0,
        }
    }
}

impl Clock for SuppliedClock {
    type Error = String;

    fn now(&mut self) -> Result<UtcTimestamp, Self::Error> {
        self.calls += 1;
        self.outcomes.pop_front().expect("prepared clock exhausted")
    }
}

/// Exercise object-safe clock injection while preserving exact caller outcomes.
pub fn read_clock(clock: &mut dyn Clock<Error = String>, count: usize) -> Value {
    Value::Array(
        (0..count)
            .map(|_| match clock.now() {
                Ok(value) => json!({"ok":value.to_string()}),
                Err(error) => json!({"error":error}),
            })
            .collect(),
    )
}

pub fn time_traits(kind: &str, values: &[Value]) -> Value {
    use std::{collections::HashSet, fmt::Write};
    macro_rules! traits {
        ($value:ty, $prefix:literal) => {{
            let first = values[0].as_str().unwrap().parse::<$value>().unwrap();
            let second = values[1].as_str().unwrap().parse::<$value>().unwrap();
            let copied = first;
            let set: HashSet<_> = [first, copied, second].into_iter().collect();
            let mut output = RejectedOutput;
            json!({"copied_equal":copied==first,"ordered":first<second,"distinct_set_size":set.len(),
                "debug_prefix":if format!("{first:?}").starts_with($prefix) {$prefix} else {"invalid"},
                "formatter_error_propagates":write!(&mut output,"{first}").is_err()})
        }};
    }
    match kind {
        "date" => traits!(CalendarDate, "CalendarDate {"),
        "timestamp" => traits!(UtcTimestamp, "UtcTimestamp {"),
        _ => panic!("unknown prepared time fixture kind"),
    }
}
