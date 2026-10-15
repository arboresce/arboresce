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
