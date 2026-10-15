//! Checked immutable wire metadata. These values establish no resource access.

use std::{fmt, str::FromStr};

use crate::{
    Identifier,
    record_json::{self, InvalidLocatorWire, Reader},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnsignedJsonInteger(u64);

impl UnsignedJsonInteger {
    pub fn new(value: u64) -> Result<Self, InvalidLocatorWire> {
        if value <= 9_007_199_254_740_991 {
            Ok(Self(value))
        } else {
            Err(InvalidLocatorWire)
        }
    }

    /// One original numeric token, without surrounding JSON whitespace.
    pub fn from_token(input: &[u8]) -> Result<Self, InvalidLocatorWire> {
        if input.is_empty() || input.len() > 16 || (input.len() > 1 && input[0] == b'0') {
            return Err(InvalidLocatorWire);
        }
        let mut value = 0u64;
        for byte in input {
            if !byte.is_ascii_digit() {
                return Err(InvalidLocatorWire);
            }
            // At most sixteen decimal digits fit without overflowing u64.
            value = value * 10 + u64::from(byte - b'0');
        }
        Self::new(value)
    }
    pub const fn value(self) -> u64 {
        self.0
    }
}
impl FromStr for UnsignedJsonInteger {
    type Err = InvalidLocatorWire;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::from_token(input.as_bytes())
    }
}
impl fmt::Display for UnsignedJsonInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
impl FromStr for Sha256Digest {
    type Err = InvalidLocatorWire;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.len() != 64 {
            return Err(InvalidLocatorWire);
        }
        let mut bytes = [0; 32];
        for (index, pair) in input.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let mut value = 0;
            for byte in pair {
                let nibble = match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    _ => return Err(InvalidLocatorWire),
                };
                value = (value << 4) | nibble;
            }
            bytes[index] = value;
        }
        Ok(Self(bytes))
    }
}
impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SourceFormat {
    PlainText,
    Markdown,
    Json,
    Jpeg,
    Png,
    Wav,
    M4a,
}

impl SourceFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlainText => "plain_text",
            Self::Markdown => "markdown",
            Self::Json => "json",
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Wav => "wav",
            Self::M4a => "m4a",
        }
    }
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::PlainText => "text/plain",
            Self::Markdown => "text/markdown",
            Self::Json => "application/json",
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Wav => "audio/wav",
            Self::M4a => "audio/mp4",
        }
    }
    pub const fn minimum_bytes(self) -> u64 {
        match self {
            Self::PlainText | Self::Markdown => 0,
            _ => 1,
        }
    }
    pub const fn maximum_bytes(self) -> u64 {
        match self {
            Self::PlainText | Self::Markdown | Self::Json => 10_485_760,
            Self::Jpeg | Self::Png => 26_214_400,
            Self::Wav | Self::M4a => 20_971_520,
        }
    }
}
impl FromStr for SourceFormat {
    type Err = InvalidLocatorWire;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(match input {
            "plain_text" => Self::PlainText,
            "markdown" => Self::Markdown,
            "json" => Self::Json,
            "jpeg" => Self::Jpeg,
            "png" => Self::Png,
            "wav" => Self::Wav,
            "m4a" => Self::M4a,
            _ => return Err(InvalidLocatorWire),
        })
    }
}

macro_rules! getters {
    ($($name:ident: $ty:ty),+ $(,)?) => {$(pub const fn $name(&self) -> $ty { self.$name })+};
}
// The callback identifier is supplied so each expression binds to that reader.
macro_rules! fields {
    ($reader:ident, $r:ident; $($name:ident: $read:expr),+ $(,)?) => {
        $(let mut $name = None;)+
        $reader.object(&[$(stringify!($name)),+], |$r, key| {
            match key { $(stringify!($name) => $name = Some($read),)+ _ => return Err(InvalidLocatorWire) }
            Ok(())
        })?;
        $(let $name = $name.ok_or(InvalidLocatorWire)?;)+
    };
}
macro_rules! json_api {
    ($($ty:ident),+ $(,)?) => {$(impl $ty {
        pub fn from_json(input: &[u8]) -> Result<Self, InvalidLocatorWire> { record_json::parse(input, Self::read) }
    })+};
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredOriginal {
    format: SourceFormat,
    sha256: Sha256Digest,
    byte_length: u64,
}
impl DeclaredOriginal {
    pub fn new(
        format: SourceFormat,
        sha256: Sha256Digest,
        byte_length: u64,
    ) -> Result<Self, InvalidLocatorWire> {
        if byte_length < format.minimum_bytes() || byte_length > format.maximum_bytes() {
            return Err(InvalidLocatorWire);
        }
        Ok(Self {
            format,
            sha256,
            byte_length,
        })
    }
    getters!(format: SourceFormat, sha256: Sha256Digest, byte_length: u64);
    pub const fn media_type(&self) -> &'static str {
        self.format.media_type()
    }
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        fields!(reader, r; format: r.string(10)?.parse::<SourceFormat>()?, media_type: r.string(16)?, sha256: r.string(64)?.parse()?, byte_length: r.unsigned()?);
        if media_type != format.media_type() {
            return Err(InvalidLocatorWire);
        }
        Self::new(format, sha256, byte_length)
    }
}
impl fmt::Display for DeclaredOriginal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"format\":\"{}\",\"media_type\":\"{}\",\"sha256\":\"{}\",\"byte_length\":{}}}",
            self.format.as_str(),
            self.media_type(),
            self.sha256,
            self.byte_length
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteIdentity {
    sha256: Sha256Digest,
    byte_length: UnsignedJsonInteger,
}
impl ByteIdentity {
    pub fn new(sha256: Sha256Digest, byte_length: u64) -> Result<Self, InvalidLocatorWire> {
        Ok(Self {
            sha256,
            byte_length: UnsignedJsonInteger::new(byte_length)?,
        })
    }
    getters!(sha256: Sha256Digest);
    pub const fn byte_length(&self) -> u64 {
        self.byte_length.value()
    }
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        fields!(reader, r; sha256: r.string(64)?.parse()?, byte_length: r.unsigned()?);
        Self::new(sha256, byte_length)
    }
}
impl fmt::Display for ByteIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"sha256\":\"{}\",\"byte_length\":{}}}",
            self.sha256, self.byte_length
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OriginalReference {
    organization_id: Identifier,
    artifact_id: Identifier,
    version_id: Identifier,
    source: DeclaredOriginal,
}
impl OriginalReference {
    pub const fn new(
        organization_id: Identifier,
        artifact_id: Identifier,
        version_id: Identifier,
        source: DeclaredOriginal,
    ) -> Self {
        Self {
            organization_id,
            artifact_id,
            version_id,
            source,
        }
    }
    getters!(organization_id: Identifier, artifact_id: Identifier, version_id: Identifier, source: DeclaredOriginal);
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        fields!(reader, r; organization_id: r.string(36)?.parse().map_err(|_| InvalidLocatorWire)?, artifact_id: r.string(36)?.parse().map_err(|_| InvalidLocatorWire)?, version_id: r.string(36)?.parse().map_err(|_| InvalidLocatorWire)?, source: DeclaredOriginal::read(r)?);
        Ok(Self::new(organization_id, artifact_id, version_id, source))
    }
}
impl fmt::Display for OriginalReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"organization_id\":\"{}\",\"artifact_id\":\"{}\",\"version_id\":\"{}\",\"source\":{}}}",
            self.organization_id, self.artifact_id, self.version_id, self.source
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RepresentationReference {
    representation_id: Identifier,
    original: OriginalReference,
    bytes: ByteIdentity,
    profile_digest: Sha256Digest,
}
impl RepresentationReference {
    pub const fn new(
        representation_id: Identifier,
        original: OriginalReference,
        bytes: ByteIdentity,
        profile_digest: Sha256Digest,
    ) -> Self {
        Self {
            representation_id,
            original,
            bytes,
            profile_digest,
        }
    }
    getters!(representation_id: Identifier, original: OriginalReference, bytes: ByteIdentity, profile_digest: Sha256Digest);
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        fields!(reader, r; representation_id: r.string(36)?.parse().map_err(|_| InvalidLocatorWire)?, original: OriginalReference::read(r)?, bytes: ByteIdentity::read(r)?, profile_digest: r.string(64)?.parse()?);
        Ok(Self::new(
            representation_id,
            original,
            bytes,
            profile_digest,
        ))
    }
}
impl fmt::Display for RepresentationReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"representation_id\":\"{}\",\"original\":{},\"bytes\":{},\"profile_digest\":\"{}\"}}",
            self.representation_id, self.original, self.bytes, self.profile_digest
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TranscriptSegment {
    transcript: RepresentationReference,
    segment_index: u16,
}
impl TranscriptSegment {
    pub fn new(
        transcript: RepresentationReference,
        segment_index: u64,
    ) -> Result<Self, InvalidLocatorWire> {
        Ok(Self {
            transcript,
            segment_index: u16::try_from(segment_index).map_err(|_| InvalidLocatorWire)?,
        })
    }
    getters!(transcript: RepresentationReference, segment_index: u16);
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, InvalidLocatorWire> {
        fields!(reader, r; transcript: RepresentationReference::read(r)?, segment_index: r.unsigned()?);
        Self::new(transcript, segment_index)
    }
}
impl fmt::Display for TranscriptSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"transcript\":{},\"segment_index\":{}}}",
            self.transcript, self.segment_index
        )
    }
}
json_api!(
    DeclaredOriginal,
    ByteIdentity,
    OriginalReference,
    RepresentationReference,
    TranscriptSegment
);
