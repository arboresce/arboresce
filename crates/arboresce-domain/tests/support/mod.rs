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

use arboresce_domain::{
    AudioBounds, AudioSpan, InvalidLocator, Orientation, PageBounds, PageRegion, Utf8Selection,
};

pub fn locator_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/locator-values-v1/cases.json")).unwrap()
}

/// Decode trusted literal hex; malformed fixture encoding is a preparation error.
pub fn decode_hex(input: &str) -> Vec<u8> {
    assert!(input.len().is_multiple_of(2));
    input
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

pub fn encode_hex(input: &[u8]) -> String {
    input.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn utf8_output(source_hex: &str, start: i64, end: i64) -> Result<Value, InvalidLocator> {
    let bytes = decode_hex(source_hex);
    let selection = Utf8Selection::new(&bytes, start, end)?;
    let (start, end) = selection.byte_range();
    Ok(json!({"byte_range":[start,end],"selected_hex":encode_hex(selection.selected().as_bytes())}))
}

pub fn fixture_u32(value: &Value) -> u32 {
    u32::try_from(value.as_u64().unwrap()).unwrap()
}

pub fn pair(value: &Value) -> [u32; 2] {
    [fixture_u32(&value[0]), fixture_u32(&value[1])]
}

pub fn coordinates(value: &Value) -> [i64; 4] {
    [
        value[0].as_i64().unwrap(),
        value[1].as_i64().unwrap(),
        value[2].as_i64().unwrap(),
        value[3].as_i64().unwrap(),
    ]
}

pub fn page_input(input: &Value) -> Result<PageRegion, InvalidLocator> {
    PageRegion::from_upright(
        PageBounds {
            page_count: fixture_u32(&input["bounds"]["page_count"]),
            width: fixture_u32(&input["bounds"]["width"]),
            height: fixture_u32(&input["bounds"]["height"]),
        },
        input["page_index"].as_i64().unwrap(),
        Orientation::from_exif(input["source_orientation"].as_i64().unwrap())?,
        coordinates(&input["coordinates"]),
    )
}

/// Project only fixture inputs; the production API receives no expected answers.
pub fn page_case(case: &Value) -> Result<PageRegion, InvalidLocator> {
    page_input(&json!({"bounds":{"page_count":case["actual_page_count"],
        "width":case["actual_upright_dimensions"][0],"height":case["actual_upright_dimensions"][1]},
        "page_index":case["page_index"],"source_orientation":case["source_orientation"],
        "coordinates":[case["x"],case["y"],case["width"],case["height"]]}))
}

pub fn page_output(value: PageRegion) -> Value {
    let bounds = value.bounds();
    json!({"bounds":{"page_count":bounds.page_count,"width":bounds.width,"height":bounds.height},
        "page_index":value.page_index(),"source_orientation":value.source_orientation().exif(),
        "coordinates":value.coordinates(),"coordinate_space":value.coordinate_space()})
}

pub fn audio_input(input: &Value) -> Result<AudioSpan, InvalidLocator> {
    AudioSpan::new(
        AudioBounds {
            duration_ms: fixture_u32(&input["bounds"]["duration_ms"]),
            channels: fixture_u32(&input["bounds"]["channels"]),
        },
        input["channel_index"].as_i64().unwrap(),
        input["start_ms"].as_i64().unwrap(),
        input["end_ms"].as_i64().unwrap(),
    )
}

pub fn audio_case(case: &Value) -> Result<AudioSpan, InvalidLocator> {
    audio_input(
        &json!({"bounds":{"duration_ms":case["actual_duration_ms"],"channels":case["actual_channels"]},
        "channel_index":case["channel_index"],"start_ms":case["start_ms"],"end_ms":case["end_ms"]}),
    )
}

pub fn audio_output(value: AudioSpan) -> Value {
    let bounds = value.bounds();
    let (start, end) = value.millisecond_range();
    json!({"bounds":{"duration_ms":bounds.duration_ms,"channels":bounds.channels},
        "channel_index":value.channel_index(),"millisecond_range":[start,end],"duration_ms":value.duration_ms()})
}

pub fn locator_traits(case: &Value) -> Value {
    use std::collections::HashSet;
    match case["kind"].as_str().unwrap() {
        "utf8" => {
            let bytes = decode_hex(case["source_hex"].as_str().unwrap());
            let first = Utf8Selection::new(
                &bytes,
                case["start_byte"].as_i64().unwrap(),
                case["end_byte"].as_i64().unwrap(),
            )
            .unwrap();
            let copied = first;
            let debug = format!("{first:?}");
            json!({"copy_preserves_view":std::ptr::eq(first.selected(),copied.selected())
                && first.byte_range()==copied.byte_range(),
                "debug_hides_source":!debug.contains(first.selected())
                    && !debug.contains(std::str::from_utf8(&bytes).unwrap())})
        }
        "page" => {
            let first = page_input(&case["input"][0]).unwrap();
            let second = page_input(&case["input"][1]).unwrap();
            let copied = first;
            let set: HashSet<_> = [first, copied, second].into_iter().collect();
            json!({"copied_equal":copied==first,"distinct_set_size":set.len(),
                "debug_type":format!("{first:?}").split_once(' ').unwrap().0})
        }
        "audio" => {
            let first = audio_input(&case["input"][0]).unwrap();
            let second = audio_input(&case["input"][1]).unwrap();
            let copied = first;
            let set: HashSet<_> = [first, copied, second].into_iter().collect();
            json!({"copied_equal":copied==first,"distinct_set_size":set.len(),
                "debug_type":format!("{first:?}").split_once(' ').unwrap().0})
        }
        "orientation" => {
            let values: Vec<_> = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Orientation::from_exif(v.as_i64().unwrap()).unwrap())
                .collect();
            let copied = values[0];
            let set: HashSet<_> = values.iter().copied().collect();
            json!({"copied_equal":copied==values[0],"distinct_set_size":set.len(),
                "exif_values":values.into_iter().map(Orientation::exif).collect::<Vec<_>>()})
        }
        _ => panic!("unknown prepared locator fixture kind"),
    }
}

use arboresce_domain::{InvalidRevisionWire, WireRevision};

pub fn wire_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/revision-wire-v1/cases.json")).unwrap()
}

/// Prepare only frozen input bytes; production receives no expected result.
pub fn wire_bytes(case: &Value) -> Vec<u8> {
    let mut bytes = match case.get("input_hex") {
        Some(value) => decode_hex(value.as_str().unwrap()),
        None => case["input_utf8"].as_str().unwrap().as_bytes().to_vec(),
    };
    if let Some(padding) = case.get("padding_spaces") {
        let padding = usize::try_from(padding.as_u64().unwrap()).unwrap();
        let size = bytes.len().checked_add(padding).unwrap();
        assert!(
            size <= 1_048_577,
            "prepared wire recipe exceeds its fixture bound"
        );
        bytes.resize(size, b' ');
    }
    bytes
}

pub fn wire_typed(target: &str, bytes: &[u8]) -> Result<WireRevision, InvalidRevisionWire> {
    match target {
        "any" => WireRevision::from_json(bytes),
        "content" => ContentRevision::from_wire_json(bytes).map(WireRevision::from),
        "control" => ControlRevision::from_wire_json(bytes).map(WireRevision::from),
        "generation" => Generation::from_wire_json(bytes).map(WireRevision::from),
        _ => panic!("unknown prepared wire fixture target"),
    }
}

pub fn wire_output(value: WireRevision) -> Value {
    json!({"kind":value.kind(),"value":value.value(),"canonical_utf8":value.to_string()})
}

fn wire_value(kind: &str, value: u64) -> WireRevision {
    match kind {
        "content" => ContentRevision::new(value).unwrap().into(),
        "control" => ControlRevision::new(value).unwrap().into(),
        "generation" => Generation::new(value).unwrap().into(),
        _ => panic!("unknown prepared wire value kind"),
    }
}

pub fn wire_traits(case: &Value) -> Value {
    use std::{collections::HashSet, fmt::Write};
    let kind = case["kind"].as_str().unwrap();
    let values = case["values"].as_array().unwrap();
    if kind == "distinct" {
        let values: Vec<_> = values
            .iter()
            .map(|v| wire_value(v["kind"].as_str().unwrap(), v["value"].as_u64().unwrap()))
            .collect();
        let set: HashSet<_> = values.iter().copied().collect();
        return json!({"distinct_set_size":set.len(),
            "pairwise_equal":[values[0]==values[1],values[0]==values[2],values[1]==values[2]]});
    }
    let first = wire_value(kind, values[0].as_u64().unwrap());
    let second = wire_value(kind, values[1].as_u64().unwrap());
    let copied = first;
    let set: HashSet<_> = [first, copied, second].into_iter().collect();
    let mut rejected = RejectedOutput;
    json!({"copied_equal":copied==first,"distinct_set_size":set.len(),"kind":first.kind(),
        "value":first.value(),"debug_first":format!("{first:?}"),
        "canonical_first":first.to_string(),"canonical_second":second.to_string(),
        "formatter_error_propagates":write!(&mut rejected,"{first}").is_err()})
}
