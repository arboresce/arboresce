//! Closed locator shapes. Contextual source selection has separate checked types.

use std::{fmt, str::FromStr};

use crate::{
    MAX_BYTE_OFFSET, MAX_PIXEL_COMPONENT, Orientation,
    record_json::{self, InvalidLocatorWire, Reader},
    references::{RepresentationReference, Sha256Digest, TranscriptSegment, UnsignedJsonInteger},
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct JsonPointer {
    original: String,
    tokens: Vec<String>,
}
impl JsonPointer {
    pub fn parse(input: &str) -> Result<Self, InvalidLocatorWire> {
        if input.len() > 16_384
            || input.chars().count() > 4096
            || input.matches('/').count() > 64
            || (!input.is_empty() && !input.starts_with('/'))
        {
            return Err(InvalidLocatorWire);
        }
        let mut tokens = Vec::new();
        for raw in input.split('/').skip(1) {
            let mut chars = raw.chars();
            let mut token = String::new();
            while let Some(c) = chars.next() {
                token.push(if c == '~' {
                    match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err(InvalidLocatorWire),
                    }
                } else {
                    c
                });
            }
            tokens.push(token);
        }
        Ok(Self {
            original: input.to_owned(),
            tokens,
        })
    }
    pub fn as_str(&self) -> &str {
        &self.original
    }
    pub fn tokens(&self) -> impl ExactSizeIterator<Item = &str> {
        self.tokens.iter().map(String::as_str)
    }
}
impl FromStr for JsonPointer {
    type Err = InvalidLocatorWire;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}
impl fmt::Debug for JsonPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JsonPointer(<redacted>)")
    }
}

macro_rules! getters {
    ($($name:ident: $ty:ty),+ $(,)?) => {$(pub const fn $name(&self) -> $ty { self.$name })+};
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Utf8RangeLocator {
    representation: RepresentationReference,
    start_byte: u64,
    end_byte: u64,
}
impl Utf8RangeLocator {
    pub fn new(
        representation: RepresentationReference,
        start_byte: u64,
        end_byte: u64,
    ) -> Result<Self, InvalidLocatorWire> {
        if start_byte > MAX_BYTE_OFFSET || end_byte == 0 || end_byte > MAX_BYTE_OFFSET {
            return Err(InvalidLocatorWire);
        }
        Ok(Self {
            representation,
            start_byte,
            end_byte,
        })
    }
    getters!(representation: RepresentationReference, start_byte: u64, end_byte: u64);
}
impl fmt::Display for Utf8RangeLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"kind\":\"utf8_range\",\"representation\":{},\"start_byte\":{},\"end_byte\":{}}}",
            self.representation, self.start_byte, self.end_byte
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsonPointerLocator {
    representation: RepresentationReference,
    pointer: JsonPointer,
    selected_value_sha256: Sha256Digest,
}
impl JsonPointerLocator {
    pub fn new(
        representation: RepresentationReference,
        pointer: JsonPointer,
        selected_value_sha256: Sha256Digest,
    ) -> Self {
        Self {
            representation,
            pointer,
            selected_value_sha256,
        }
    }
    getters!(representation: RepresentationReference, selected_value_sha256: Sha256Digest);
    pub fn pointer(&self) -> &JsonPointer {
        &self.pointer
    }
    pub const fn canonicalization(&self) -> &'static str {
        "jcs_selection_v1"
    }
}
impl fmt::Display for JsonPointerLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"kind\":\"json_pointer\",\"representation\":{},\"pointer\":",
            self.representation
        )?;
        record_json::quoted(f, self.pointer.as_str())?;
        write!(
            f,
            ",\"canonicalization\":\"{}\",\"selected_value_sha256\":\"{}\"}}",
            self.canonicalization(),
            self.selected_value_sha256
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PageRegionLocator {
    representation: RepresentationReference,
    page_index: u16,
    render_sha256: Sha256Digest,
    source_orientation: Orientation,
    rectangle: [u32; 4],
}
impl PageRegionLocator {
    pub fn new(
        representation: RepresentationReference,
        page_index: u64,
        render_sha256: Sha256Digest,
        source_orientation: Orientation,
        rectangle: [u64; 4],
    ) -> Result<Self, InvalidLocatorWire> {
        if rectangle
            .iter()
            .any(|v| *v > u64::from(MAX_PIXEL_COMPONENT))
            || rectangle[2] == 0
            || rectangle[3] == 0
        {
            return Err(InvalidLocatorWire);
        }
        Ok(Self {
            representation,
            page_index: u16::try_from(page_index).map_err(|_| InvalidLocatorWire)?,
            render_sha256,
            source_orientation,
            rectangle: rectangle.map(|v| v as u32),
        })
    }
    getters!(representation: RepresentationReference, page_index: u16, render_sha256: Sha256Digest, source_orientation: Orientation, rectangle: [u32; 4]);
    pub const fn coordinate_space(&self) -> &'static str {
        "upright_pixels_v1"
    }
}
impl fmt::Display for PageRegionLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [x, y, width, height] = self.rectangle;
        write!(
            f,
            "{{\"kind\":\"page_region\",\"representation\":{},\"page_index\":{},\"render_sha256\":\"{}\",\"coordinate_space\":\"{}\",\"source_orientation\":{},\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}",
            self.representation,
            self.page_index,
            self.render_sha256,
            self.coordinate_space(),
            self.source_orientation.exif(),
            x,
            y,
            width,
            height
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioSpanLocator {
    representation: RepresentationReference,
    start_ms: u32,
    end_ms: u32,
    channel_index: u16,
    transcript_segment: Option<TranscriptSegment>,
}
impl AudioSpanLocator {
    pub fn new(
        representation: RepresentationReference,
        start_ms: u64,
        end_ms: u64,
        channel_index: u64,
        transcript_segment: Option<TranscriptSegment>,
    ) -> Result<Self, InvalidLocatorWire> {
        if start_ms > 299_999 || end_ms == 0 || end_ms > 300_000 {
            return Err(InvalidLocatorWire);
        }
        Ok(Self {
            representation,
            start_ms: start_ms as u32,
            end_ms: end_ms as u32,
            channel_index: u16::try_from(channel_index).map_err(|_| InvalidLocatorWire)?,
            transcript_segment,
        })
    }
    getters!(representation: RepresentationReference, start_ms: u32, end_ms: u32, channel_index: u16, transcript_segment: Option<TranscriptSegment>);
}
impl fmt::Display for AudioSpanLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"kind\":\"audio_span\",\"representation\":{},\"start_ms\":{},\"end_ms\":{},\"channel_index\":{}",
            self.representation, self.start_ms, self.end_ms, self.channel_index
        )?;
        if let Some(segment) = self.transcript_segment {
            write!(f, ",\"transcript_segment\":{segment}")?;
        }
        f.write_str("}")
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum WireLocator {
    Utf8Range(Utf8RangeLocator),
    JsonPointer(JsonPointerLocator),
    PageRegion(PageRegionLocator),
    AudioSpan(AudioSpanLocator),
}
impl WireLocator {
    pub fn from_json(input: &[u8]) -> Result<Self, InvalidLocatorWire> {
        record_json::parse(input, Self::read)
    }
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Utf8Range(_) => "utf8_range",
            Self::JsonPointer(_) => "json_pointer",
            Self::PageRegion(_) => "page_region",
            Self::AudioSpan(_) => "audio_span",
        }
    }
    pub const fn representation(&self) -> RepresentationReference {
        match self {
            Self::Utf8Range(v) => v.representation,
            Self::JsonPointer(v) => v.representation,
            Self::PageRegion(v) => v.representation,
            Self::AudioSpan(v) => v.representation,
        }
    }
    fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        let (mut kind, mut representation) = (None, None);
        let (
            mut start_byte,
            mut end_byte,
            mut pointer,
            mut canonicalization,
            mut selected_value_sha256,
        ) = (None, None, None, None, None);
        let (mut page_index, mut render_sha256, mut coordinate_space, mut source_orientation) =
            (None, None, None, None);
        let (mut x, mut y, mut width, mut height) = (None, None, None, None);
        let (mut start_ms, mut end_ms, mut channel_index, mut transcript_segment) =
            (None, None, None, None);
        let mut members = 0;
        reader.object(
            &[
                "kind",
                "representation",
                "start_byte",
                "end_byte",
                "pointer",
                "canonicalization",
                "selected_value_sha256",
                "page_index",
                "render_sha256",
                "coordinate_space",
                "source_orientation",
                "x",
                "y",
                "width",
                "height",
                "start_ms",
                "end_ms",
                "channel_index",
                "transcript_segment",
            ],
            |r, key| {
                members += 1;
                match key {
                    "kind" => kind = Some(r.string(12)?),
                    "representation" => representation = Some(RepresentationReference::read(r)?),
                    "start_byte" => start_byte = Some(r.unsigned()?),
                    "end_byte" => end_byte = Some(r.unsigned()?),
                    "pointer" => pointer = Some(JsonPointer::parse(&r.string(4096)?)?),
                    "canonicalization" => canonicalization = Some(r.string(16)?),
                    "selected_value_sha256" => selected_value_sha256 = Some(r.string(64)?.parse()?),
                    "page_index" => page_index = Some(r.unsigned()?),
                    "render_sha256" => render_sha256 = Some(r.string(64)?.parse()?),
                    "coordinate_space" => coordinate_space = Some(r.string(17)?),
                    "source_orientation" => {
                        let value = UnsignedJsonInteger::new(r.unsigned()?)?.value();
                        // The admitted safe-unsigned range is strictly within i64.
                        source_orientation = Some(
                            Orientation::from_exif(value as i64).map_err(|_| InvalidLocatorWire)?,
                        );
                    }
                    "x" => x = Some(r.unsigned()?),
                    "y" => y = Some(r.unsigned()?),
                    "width" => width = Some(r.unsigned()?),
                    "height" => height = Some(r.unsigned()?),
                    "start_ms" => start_ms = Some(r.unsigned()?),
                    "end_ms" => end_ms = Some(r.unsigned()?),
                    "channel_index" => channel_index = Some(r.unsigned()?),
                    "transcript_segment" => transcript_segment = Some(TranscriptSegment::read(r)?),
                    _ => return Err(InvalidLocatorWire),
                }
                Ok(())
            },
        )?;
        let representation = representation.ok_or(InvalidLocatorWire)?;
        Ok(match kind.as_deref().ok_or(InvalidLocatorWire)? {
            "utf8_range" if members == 4 => Self::Utf8Range(Utf8RangeLocator::new(
                representation,
                start_byte.ok_or(InvalidLocatorWire)?,
                end_byte.ok_or(InvalidLocatorWire)?,
            )?),
            "json_pointer"
                if members == 5 && canonicalization.as_deref() == Some("jcs_selection_v1") =>
            {
                Self::JsonPointer(JsonPointerLocator::new(
                    representation,
                    pointer.ok_or(InvalidLocatorWire)?,
                    selected_value_sha256.ok_or(InvalidLocatorWire)?,
                ))
            }
            "page_region"
                if members == 10 && coordinate_space.as_deref() == Some("upright_pixels_v1") =>
            {
                Self::PageRegion(PageRegionLocator::new(
                    representation,
                    page_index.ok_or(InvalidLocatorWire)?,
                    render_sha256.ok_or(InvalidLocatorWire)?,
                    source_orientation.ok_or(InvalidLocatorWire)?,
                    [
                        x.ok_or(InvalidLocatorWire)?,
                        y.ok_or(InvalidLocatorWire)?,
                        width.ok_or(InvalidLocatorWire)?,
                        height.ok_or(InvalidLocatorWire)?,
                    ],
                )?)
            }
            "audio_span" if members == 5 + usize::from(transcript_segment.is_some()) => {
                Self::AudioSpan(AudioSpanLocator::new(
                    representation,
                    start_ms.ok_or(InvalidLocatorWire)?,
                    end_ms.ok_or(InvalidLocatorWire)?,
                    channel_index.ok_or(InvalidLocatorWire)?,
                    transcript_segment,
                )?)
            }
            _ => return Err(InvalidLocatorWire),
        })
    }
}
impl fmt::Display for WireLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Range(v) => v.fmt(f),
            Self::JsonPointer(v) => v.fmt(f),
            Self::PageRegion(v) => v.fmt(f),
            Self::AudioSpan(v) => v.fmt(f),
        }
    }
}
macro_rules! variant_api {
    ($($ty:ident: $variant:ident),+ $(,)?) => {$(
        impl From<$ty> for WireLocator { fn from(value: $ty) -> Self { Self::$variant(value) } }
        impl $ty {
            pub fn from_json(input: &[u8]) -> Result<Self, InvalidLocatorWire> {
                match WireLocator::from_json(input)? { WireLocator::$variant(value)=>Ok(value),_=>Err(InvalidLocatorWire) }
            }
        }
    )+};
}
variant_api!(Utf8RangeLocator:Utf8Range,JsonPointerLocator:JsonPointer,PageRegionLocator:PageRegion,AudioSpanLocator:AudioSpan);
