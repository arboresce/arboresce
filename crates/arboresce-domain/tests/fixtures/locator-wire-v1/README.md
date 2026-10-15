# Independent locator and reference wire fixtures

These original public synthetic fixtures use MIT OR Apache-2.0. The pinned
common, acquisition and observation contracts own their meanings. The existing
89 selected parsed-wire cases retain their IDs, values, validity flags and
parsed-value roundtrips. Wire-valid equal/reversed ranges and capacity-only
metadata remain representable; actual selection, media, exact resource admission
and current authority retain their separate gates.

The production graph remains standard-library-only. Three measured modules own
private known-record JSON handling, immutable reference/value types, and the four
locator wire variants. The accepted revision parser remains unchanged. No general
JSON parser, source/JCS profile or transport stream is admitted by these cases.

The selected public API is as follows. Every fallible operation returns the
input-independent unit InvalidLocatorWire. Checked metadata is Copy/Clone/Eq/Hash
with immutable getters. Pointer-containing values are Clone/Eq/Hash and have
pointer-free Debug. There is no unchecked numeric or textual constructor.

- UnsignedJsonInteger exposes new(u64), from_token(&[u8]), FromStr, value() and
  canonical decimal Display. It admits 0 through 9007199254740991, inspecting
  original tokens before coercion; consuming fields enforce their tighter bounds.
- Sha256Digest exposes from_bytes([u8;32]), as_bytes(), FromStr and lower-case
  64-hex Display. It identifies digest bytes; it implements no hash algorithm.
- SourceFormat has seven checked FromStr variants, as_str(), media_type(),
  minimum_bytes() and maximum_bytes(). DeclaredOriginal::new(format,digest,length)
  checks those exact byte bounds. Its format(), media_type(), sha256() and
  byte_length() preserve the complete declared source. Empty text/Markdown is
  allowed; JSON/images/audio require at least one byte. MIME strings are exact.
- ByteIdentity::new(digest,length) exposes sha256() and byte_length() and permits
  zero through 2^53-1 independently of declared-original format limits.
- OriginalReference::new(organization_id,artifact_id,version_id,source) exposes
  each exact checked component through those named getters.
- RepresentationReference::new(representation_id,original,bytes,profile_digest)
  exposes each component. TranscriptSegment::new(transcript,segment_index)
  exposes the exact transcript representation and an index 0 through 65535.
- JsonPointer::parse(&str) and FromStr retain as_str() exactly and expose tokens()
  as an exact-size iterator of once-decoded strings. The 4096 Unicode-scalar and
  64-slash-token limits apply to the original pointer before tilde decoding.
  Empty root and empty-key pointers differ; no percent/fragment/Unicode repair
  applies. Control characters, including NUL, remain valid pointer content.
- Utf8RangeLocator::new(representation,start_byte,end_byte) checks wire capacities
  only. JsonPointerLocator::new(representation,pointer,selected_value_sha256)
  preserves canonicalization() exactly jcs_selection_v1. PageRegionLocator::new
  (representation,page_index,render_sha256,source_orientation,[x,y,width,height])
  checks each wire component and exposes rectangle() in that order, page_index(),
  render_sha256(), source_orientation() and coordinate_space() upright_pixels_v1.
  AudioSpanLocator::new(representation,start_ms,end_ms,channel_index,transcript_segment)
  preserves its optional exact transcript. All variants expose representation()
  and their remaining named field getters; no constructor infers contextual order,
  geometry, duration, identity equivalence or access from those wire capacities.
- All five reference records and four locator structs expose complete closed
  from_json(&[u8]) admission and complete JSON Display/to_string(). WireLocator
  is the four-variant union, has From conversions, from_json(), kind() and
  representation(), and emits its exact chosen record. A target-specific parser
  cannot admit another variant. Null never means an absent transcript segment;
  only an audio locator may carry that member.

The crate-private reader checks at most 1048576 bytes before UTF-8 or traversal.
Each parsed record has at most ten members; its acyclic known-record graph has
at most five nested containers, within the common 16/128 envelope. A union may
recognize names across all variants but cannot consume an eleventh member. Decoded keys are capped
at 21 scalars; other decoded fields have their exact known capacities (UUID36,
digest64, format10, MIME16, pointer4096). It retains no eager general Value tree,
no recursive unknown record, and no input-sized allocation. All standard JSON
escapes and valid surrogate pairs preserve exact Unicode. Decoded duplicate keys,
unknown/missing members, wrong types, invalid UTF-8/surrogates, BOM, malformed
syntax and trailing input fail. Original integer tokens never become floats.

Complete emission uses fixed named fields and JSON string escaping, not a general
JCS claim. At most 4096 pointer scalars require at most 24576 escaped bytes; its
fixed metadata/other variants keep every complete output below the declared
32768-byte bound. The maximum-control-pointer case exercises that worst string
expansion. Formatter errors propagate; a failed write supplies no successful
complete frame. No partial digest or source-selection result is produced.

The fields group uses fixed base records, explicit paths and original raw tokens;
its expected integer never supplies production input. Each of 30 exposed nested
integer paths tests zero/one/maximum/overflow and lexical/wrong-type distinctions.
The values group exercises constructors and wire/decimal/digest APIs. Pointer
recipes describe literal repetitions, not an oracle. Objects/text groups contain
exact raw inputs, including otherwise-valid input at/over the byte limit. Traits
exercise every getter, complete roundtrip, one-field identity distinctions,
copy/clone/hash, redacted pointer debugging and formatter/static error behavior.
All prior independent domain and verification cases remain required. These pure
wire fixtures supply no actual media, human, live-model, recovery or product pass.

The additive identity pairs retain all earlier expected rows. They explicitly
expect unequal values and a two-member set after an isolated valid numeric tuple
component, pointer or optional-presence change, or a valid format/MIME change.
Both complete parsed records must roundtrip. Nested source/representation lengths
and transcript indices propagate through the owning immutable value. Four union
variants with the same representation remain pairwise distinct in a four-member
set. Getter or serializer success alone does not prove equality semantics.
