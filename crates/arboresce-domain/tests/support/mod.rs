use std::fmt;

pub mod record_support {
    use arboresce_domain::{
        AudioSpanLocator, ByteIdentity, DeclaredOriginal, InvalidLocatorWire, JsonPointer,
        JsonPointerLocator, Orientation, OriginalReference, PageRegionLocator,
        RepresentationReference, Sha256Digest, TranscriptSegment, Utf8RangeLocator, WireLocator,
    };
    use serde_json::{Value, json};
    use std::{collections::HashSet, fmt, fmt::Write};

    pub fn fixture() -> Value {
        serde_json::from_str(include_str!("../fixtures/locator-wire-v1/cases.json")).unwrap()
    }
    pub fn original_cases() -> Value {
        serde_json::from_str(include_str!(
            "../../../../contracts/validation/observation-v1.cases.json"
        ))
        .unwrap()
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub enum Record {
        Bytes(ByteIdentity),
        Declared(DeclaredOriginal),
        Original(OriginalReference),
        Representation(RepresentationReference),
        Transcript(TranscriptSegment),
        Locator(WireLocator),
    }
    impl fmt::Display for Record {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Bytes(v) => v.fmt(f),
                Self::Declared(v) => v.fmt(f),
                Self::Original(v) => v.fmt(f),
                Self::Representation(v) => v.fmt(f),
                Self::Transcript(v) => v.fmt(f),
                Self::Locator(v) => v.fmt(f),
            }
        }
    }

    pub fn read(target: &str, input: &[u8]) -> Result<Record, InvalidLocatorWire> {
        match target {
            "byte_identity" => ByteIdentity::from_json(input).map(Record::Bytes),
            "declared_original" => DeclaredOriginal::from_json(input).map(Record::Declared),
            "original_reference" => OriginalReference::from_json(input).map(Record::Original),
            "representation_reference" => {
                RepresentationReference::from_json(input).map(Record::Representation)
            }
            "transcript_segment" => TranscriptSegment::from_json(input).map(Record::Transcript),
            "utf8_range" => Utf8RangeLocator::from_json(input).map(|v| Record::Locator(v.into())),
            "json_pointer" => {
                JsonPointerLocator::from_json(input).map(|v| Record::Locator(v.into()))
            }
            "page_region" => PageRegionLocator::from_json(input).map(|v| Record::Locator(v.into())),
            "audio_span" => AudioSpanLocator::from_json(input).map(|v| Record::Locator(v.into())),
            "locator" => WireLocator::from_json(input).map(Record::Locator),
            _ => panic!("unknown frozen record target"),
        }
    }

    pub fn source_output(v: DeclaredOriginal) -> Value {
        json!({"format":v.format().as_str(),"media_type":v.media_type(),"sha256":v.sha256().to_string(),"byte_length":v.byte_length()})
    }
    pub fn bytes_output(v: ByteIdentity) -> Value {
        json!({"sha256":v.sha256().to_string(),"byte_length":v.byte_length()})
    }
    pub fn original_output(v: OriginalReference) -> Value {
        json!({"organization_id":v.organization_id().to_string(),"artifact_id":v.artifact_id().to_string(),"version_id":v.version_id().to_string(),"source":source_output(v.source())})
    }
    pub fn representation_output(v: RepresentationReference) -> Value {
        json!({"representation_id":v.representation_id().to_string(),"original":original_output(v.original()),"bytes":bytes_output(v.bytes()),"profile_digest":v.profile_digest().to_string()})
    }
    pub fn transcript_output(v: TranscriptSegment) -> Value {
        json!({"transcript":representation_output(v.transcript()),"segment_index":v.segment_index()})
    }
    pub fn output(value: &Record) -> Value {
        match value {
            Record::Bytes(v) => bytes_output(*v),
            Record::Declared(v) => source_output(*v),
            Record::Original(v) => original_output(*v),
            Record::Representation(v) => representation_output(*v),
            Record::Transcript(v) => transcript_output(*v),
            Record::Locator(v) => {
                let mut out = json!({"kind":v.kind(),"representation":representation_output(v.representation())});
                match v {
                    WireLocator::Utf8Range(v) => {
                        assert_eq!(v.representation(), value_representation(value));
                        out["start_byte"] = v.start_byte().into();
                        out["end_byte"] = v.end_byte().into();
                    }
                    WireLocator::JsonPointer(v) => {
                        assert_eq!(v.representation(), value_representation(value));
                        out["pointer"] = v.pointer().as_str().into();
                        out["canonicalization"] = v.canonicalization().into();
                        out["selected_value_sha256"] = v.selected_value_sha256().to_string().into();
                    }
                    WireLocator::PageRegion(v) => {
                        assert_eq!(v.representation(), value_representation(value));
                        let [x, y, width, height] = v.rectangle();
                        out["page_index"] = v.page_index().into();
                        out["render_sha256"] = v.render_sha256().to_string().into();
                        out["source_orientation"] = v.source_orientation().exif().into();
                        out["coordinate_space"] = v.coordinate_space().into();
                        out["x"] = x.into();
                        out["y"] = y.into();
                        out["width"] = width.into();
                        out["height"] = height.into();
                    }
                    WireLocator::AudioSpan(v) => {
                        assert_eq!(v.representation(), value_representation(value));
                        out["start_ms"] = v.start_ms().into();
                        out["end_ms"] = v.end_ms().into();
                        out["channel_index"] = v.channel_index().into();
                        if let Some(t) = v.transcript_segment() {
                            out["transcript_segment"] = transcript_output(t);
                        }
                    }
                }
                out
            }
        }
    }
    fn value_representation(value: &Record) -> RepresentationReference {
        match value {
            Record::Locator(v) => v.representation(),
            _ => panic!("prepared locator only"),
        }
    }

    fn digest(v: &Value) -> Sha256Digest {
        v.as_str().unwrap().parse().unwrap()
    }
    fn declared(v: &Value) -> Result<DeclaredOriginal, InvalidLocatorWire> {
        DeclaredOriginal::new(
            v["format"].as_str().unwrap().parse()?,
            digest(&v["sha256"]),
            v["byte_length"].as_u64().unwrap(),
        )
    }
    fn bytes(v: &Value) -> Result<ByteIdentity, InvalidLocatorWire> {
        ByteIdentity::new(digest(&v["sha256"]), v["byte_length"].as_u64().unwrap())
    }
    fn original(v: &Value) -> Result<OriginalReference, InvalidLocatorWire> {
        Ok(OriginalReference::new(
            v["organization_id"].as_str().unwrap().parse().unwrap(),
            v["artifact_id"].as_str().unwrap().parse().unwrap(),
            v["version_id"].as_str().unwrap().parse().unwrap(),
            declared(&v["source"])?,
        ))
    }
    fn representation(v: &Value) -> Result<RepresentationReference, InvalidLocatorWire> {
        Ok(RepresentationReference::new(
            v["representation_id"].as_str().unwrap().parse().unwrap(),
            original(&v["original"])?,
            bytes(&v["bytes"])?,
            digest(&v["profile_digest"]),
        ))
    }
    fn transcript(v: &Value) -> Result<TranscriptSegment, InvalidLocatorWire> {
        TranscriptSegment::new(
            representation(&v["transcript"])?,
            v["segment_index"].as_u64().unwrap(),
        )
    }

    /// Numeric construction uses the original input's typed integer, never an expected field.
    pub fn construct(target: &str, v: &Value) -> Result<Record, InvalidLocatorWire> {
        match target {
            "byte_identity" => bytes(v).map(Record::Bytes),
            "declared_original" => declared(v).map(Record::Declared),
            "original_reference" => original(v).map(Record::Original),
            "representation_reference" => representation(v).map(Record::Representation),
            "transcript_segment" => transcript(v).map(Record::Transcript),
            "utf8_range" => Utf8RangeLocator::new(
                representation(&v["representation"])?,
                v["start_byte"].as_u64().unwrap(),
                v["end_byte"].as_u64().unwrap(),
            )
            .map(|x| Record::Locator(x.into())),
            "json_pointer" => Ok(Record::Locator(
                JsonPointerLocator::new(
                    representation(&v["representation"])?,
                    JsonPointer::parse(v["pointer"].as_str().unwrap())?,
                    digest(&v["selected_value_sha256"]),
                )
                .into(),
            )),
            "page_region" => PageRegionLocator::new(
                representation(&v["representation"])?,
                v["page_index"].as_u64().unwrap(),
                digest(&v["render_sha256"]),
                Orientation::from_exif(v["source_orientation"].as_i64().unwrap())
                    .map_err(|_| InvalidLocatorWire)?,
                [
                    v["x"].as_u64().unwrap(),
                    v["y"].as_u64().unwrap(),
                    v["width"].as_u64().unwrap(),
                    v["height"].as_u64().unwrap(),
                ],
            )
            .map(|x| Record::Locator(x.into())),
            "audio_span" => {
                let segment = v.get("transcript_segment").map(transcript).transpose()?;
                AudioSpanLocator::new(
                    representation(&v["representation"])?,
                    v["start_ms"].as_u64().unwrap(),
                    v["end_ms"].as_u64().unwrap(),
                    v["channel_index"].as_u64().unwrap(),
                    segment,
                )
                .map(|x| Record::Locator(x.into()))
            }
            _ => panic!("unknown frozen construction target"),
        }
    }

    pub fn replace(root: &mut Value, path: &[Value], replacement: Value) {
        let mut current = root;
        for part in path {
            current = &mut current[part.as_str().unwrap()];
        }
        *current = replacement;
    }
    pub fn field_input(fixture: &Value, row: &Value) -> Vec<u8> {
        let mut value = fixture["bases"][row["base"].as_str().unwrap()].clone();
        let marker = "$record_original_numeric_token$";
        replace(&mut value, row["path"].as_array().unwrap(), marker.into());
        let serialized = serde_json::to_string(&value).unwrap();
        let quoted = serde_json::to_string(marker).unwrap();
        assert_eq!(serialized.matches(&quoted).count(), 1);
        serialized
            .replacen(&quoted, row["token"].as_str().unwrap(), 1)
            .into_bytes()
    }
    pub fn raw_input(row: &Value) -> Vec<u8> {
        super::wire_bytes(row)
    }
    pub fn pointer_input(row: &Value) -> String {
        if let Some(r) = row.get("recipe") {
            format!(
                "{}{}",
                r["prefix"].as_str().unwrap(),
                r["unit"]
                    .as_str()
                    .unwrap()
                    .repeat(r["count"].as_u64().unwrap().try_into().unwrap())
            )
        } else {
            row["pointer"].as_str().unwrap().to_owned()
        }
    }

    pub fn same_traits(value: &Record) -> (bool, usize, bool) {
        fn copy<T: Copy>(v: T) -> T {
            v
        }
        match value {
            Record::Bytes(v) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Declared(v) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Original(v) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Representation(v) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Transcript(v) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Locator(WireLocator::Utf8Range(v)) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Locator(WireLocator::PageRegion(v)) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Locator(WireLocator::AudioSpan(v)) => {
                assert_eq!(copy(*v), *v);
            }
            Record::Locator(WireLocator::JsonPointer(v)) => {
                assert_eq!(v.clone(), *v);
            }
        }
        let cloned = value.clone();
        let set: HashSet<_> = [value.clone(), cloned.clone()].into_iter().collect();
        let mut rejected = super::RejectedOutput;
        (
            cloned == *value,
            set.len(),
            write!(&mut rejected, "{value}").is_err(),
        )
    }
}

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
