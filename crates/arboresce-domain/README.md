# Arboresce domain values

The current library implements checked immutable UUIDv4 identifiers. It accepts
only the canonical 36-byte lower-case spelling or 16 bytes with version 4 and
the RFC variant. Display produces that exact canonical spelling. Equality and
hashing compare the validated bytes. Errors retain no invalid input and emit a
fixed diagnostic.

`ContentRevision`, `ControlRevision` and `Generation` are distinct immutable
integer types. `new` checks an already typed `u64`; `FromStr` checks only the
canonical decimal value payload. Content/control admit 1 through
999999999999999999, and generation also admits zero. `value` returns the exact
integer, and display emits canonical decimal bytes without floating-point
conversion. `checked_next` returns an advanced copy or a fixed exhaustion error
while preserving the original value. The
[revision value fixtures](tests/fixtures/revision-values-v1/README.md) own the
independently reviewed scalar expectations.

These scalar APIs do not admit a complete tagged wire object, perform a stored
compare-and-swap or establish revision order from time.

`WireRevision::from_json` admits complete closed content/control/generation
objects from bounded raw bytes. It retains each distinct checked scalar variant;
each scalar's explicit `from_wire_json` also enforces its own tag. JSON member
order, the four JSON whitespace characters and equivalent ASCII escapes may
vary. Duplicate decoded keys, extra members, malformed input, wrong types, tags
or decimal strings fail. The one-MiB control envelope is checked before traversal,
and decoded fields have fixed small bounds. No numeric value passes through
binary64. `WireRevision` display emits canonical complete kind/value objects;
existing scalar display and `FromStr` remain decimal payloads. Rejections retain
no input, and formatter failures propagate. The independently reviewed
[wire fixtures](tests/fixtures/revision-wire-v1/README.md) preserve all 48 original
revision cases and add raw byte, size, escape and typed-value expectations.
This specialized parser does not admit general JSON or a transport envelope.

`CalendarDate` checks real Gregorian dates in years 0001 through 9999 and
roundtrips exact `YYYY-MM-DD` text. It never infers an instant or timezone.
`UtcTimestamp` checks exact UTC millisecond text and explicit date/time components,
rejecting leap seconds, local ambiguity, alternate offsets and precision. Its
signed integer milliseconds preserve pre-epoch instants exactly. Both types are
immutable and distinct. `Clock` is a fallible injected interface with a caller-owned
error type and no ambient implementation. Successful repeats, backward movement
and failures remain the supplier's exact outcomes; time confers no revision order.
The [time value fixtures](tests/fixtures/time-values-v1/README.md) retain independent
date, instant, clock, construction and formatter expectations.

`Utf8Selection` borrows a checked, nonempty byte range from exact valid UTF-8
input. It rejects invalid whole input and non-boundary offsets without replacement
or normalization; debug output omits the text. `PageRegion::from_upright` checks
provided page and pixel bounds without another orientation transform.
`PageRegion::from_raw` first checks the physical raw bounds, maps all four pixel-edge
corners using one of the eight checked Exif orientations, then applies the wire
component limits to the upright rectangle. `AudioSpan`
checks a half-open integer-millisecond range against the supplied duration,
channel count and wire capacities. The
[locator fixtures](tests/fixtures/locator-values-v1/README.md) retain independent
byte selections, bounds, full-page edge transforms and failures.

The supplied numeric page/audio bounds do not establish a qualified decoder,
render identity, source reference, codec, sample conversion or authority. Large
synthetic bounds exercise arithmetic capacities only.

`DeclaredOriginal`, `OriginalReference`, `ByteIdentity` and
`RepresentationReference` preserve complete checked immutable metadata. The seven
source formats retain exact MIME and length bounds; representation byte identity
is independently bounded. `Sha256Digest` is a checked digest value and computes
no hash. `UnsignedJsonInteger` checks original decimal tokens before coercion;
each consuming field retains its own narrower range.

`Utf8RangeLocator`, `JsonPointerLocator`, `PageRegionLocator` and `AudioSpanLocator`
are complete closed wire shapes, combined by `WireLocator`. Typed entry points
enforce their own kind, retain the exact reference tuple and optional transcript,
and emit complete escaped JSON. These wire values can retain equal or reversed
intervals; the separate semantic values above check actual contextual ranges.
`JsonPointer` retains its original string and once-decoded tokens, with 4096-scalar
and 64-token capacities. It preserves controls, Unicode and empty-key meaning
without normalization or percent decoding; debug output omits pointer content.
The specialized known-record reader checks one MiB before UTF-8/traversal,
rejects decoded duplicate or unknown fields and supports only its bounded acyclic
record graph. Complete outputs stay below 32768 bytes. The independent
[record fixtures](tests/fixtures/locator-wire-v1/README.md) retain all 89 original
parsed-wire cases and add original-token, constructor, Unicode, closed-shape and
complete identity expectations before consumption.

Wire metadata and pointer syntax do not resolve a canonical representation,
verify referenced bytes, select source JSON, compute JCS, admit media or grant
access. General source JSON/JCS and aggregate/selected processing budgets retain
their owning gates.

`RecordedTime` is distinct from optional attributed `ObservedTime`. The latter
retains an `ObservedValue::Date` or `Instant` and a complete original,
representation or asserted organization/principal reference. The pure
`ObservedValue::from_text` factory accepts exact precision names `date` and `instant` and delegates to the
existing checked primitives. Unsupported names, local/offset times and invalid
text fail with an input-independent error. They never become successful absent
observed time. Date-only precision never acquires midnight or an inferred zone.

`EventTimes` preserves both meanings in an immutable pair, including future event
times and earlier receipt times. Direct typed construction does not read an actual
clock. `record` invokes a supplied fallible `Clock` exactly once and propagates
its exact caller-owned error. Repeats and backwards values impose no revision
order. Constructing a later correction preserves the earlier pair; actual history
persistence and applicability periods remain with their owning consumers.

`UtcCalendarPolicy` requires explicit organization, policy ID, content revision
and the exact foundation profile `utc_calendar_v1`, whose declared calendar is
UTC. `ObservedTime::reporting_date` rejects missing policy or a mismatch with the
attribution's owning organization. Its `ReportingDate` retains the complete
observed value/attribution, policy identity/revision/profile and derived date.
A date preserves itself; an instant uses its explicit UTC date. A projected date
does not assign an instant or source timezone to an original date-only assertion.

These checked references establish no organization policy membership, source
truth, authenticated authorship or authority. This foundation has no default,
ambient timezone or IANA resolver. Later relative dates, review periods/deadlines
and financial reports still require their actual selected IANA rules, policy
versions, membership and ambiguity qualification. The independent
[event-time fixtures](tests/fixtures/event-times-v1/README.md) retain complete
pairs, errors, clock call counts, precision and policy expectations before this
consumer. Whole source/native qualification and all actual clock/service/product
gates remain required.

`SourceJson::parse` checks complete raw source bytes under the separate
[source processing profile](../../docs/requirements/resource-profiles.md#bounded-source-json-processing).
It rejects invalid UTF-8, grammar, Unicode, decoded duplicate keys and unsupported
original numbers throughout the source before any path selection. Ownership is
flat: child indices do not recursively own descendants. The caller's bytes are
never modified. The supplied complete source has its own byte identity.

`select` resolves a checked `JsonPointer`, preserves exact object names and
requires canonical unsigned array index spelling. It preflights selected depth,
nodes and complete escaped size, then uses the selected vetted JCS adapter and
bounded in-memory writer. `CanonicalJsonSelection` exposes immutable `as_bytes`,
`byte_length` and `sha256`; it returns no partial output or digest. Its equality
and hash include complete canonical bytes. Debug and fixed error diagnostics
retain no source or selected content. The
[source fixtures](tests/fixtures/source-json-v1/README.md) preserve independent
literal, boundary, property, mutation, trait and original pointer expectations.
These APIs do not establish acquisition retention, shared Python identity,
signature validity or an operational processing service.

Validation proves format. Resource existence, ownership, authenticated principal
identity, random generation and ordering belong to their actual consumers.
The library has no transport, storage, ambient clock or random source.

`CanonicalJsonNumber::from_token` and `FromStr` admit one complete JSON number
token up to 4096 UTF-8 bytes. The separate `canonical-json-number-v1` scalar
profile supports signed/fractional finite binary64 values within the common safe
range. It preserves the original mathematical decimal until comparison with the
shortest vetted JCS decimal. Unsupported precision, overflow, nonzero underflow,
whitespace, trailing data and malformed grammar fail before any value or digest
is returned. Long insignificant zeros and padded exponents remain supported;
complete lexical zero may ignore even a huge exponent only after grammar checks.

`as_bytes`, `byte_length` and `sha256` return complete canonical output, its length
and SHA-256. Output has at most 32 ASCII bytes, without a newline or BOM. Immutable
equality/hashing compare canonical admitted values, so equivalent spellings and
both signs of zero agree. Debug redacts content; display emits canonical bytes.
Static errors retain no token, and formatter failures propagate. The
[number literals](tests/fixtures/canonical-number-v1/README.md) independently
freeze decimal decisions, exact bytes/digests, traits and published dependency
vectors before consumption.

The explicit [dependency profile](../../docs/requirements/dependency-qualification.md#selected-scalar-canonical-number-profile)
uses only the canonicalizer's finite primitive-f64 `to_vec` path and software
SHA-256. The reviewed subset includes its 128-byte vector, boxed writer reference
and 132-byte formatting stack buffer; arbitrary serialization and object buffers
are outside this admission. This computes a scalar canonical digest, not an
original's representation digest, money value, signature or authority. Full JSON
parsing, UTF-8/Unicode/duplicate validation, UTF16 key ordering, source and selected
budgets, actual pointer selection and cross-language identity remain required.

The independently reviewed [identity tables](tests/fixtures/primitives-v1/README.md)
provide exact bytes, roundtrips and rejections before implementation. Remaining
source JSON Pointer and whole primitive fixtures retain their complete implementation gates.
This library does not establish an API service or MVP runtime.

Use Rust 1.99.0 and the repository's workspace format/check/test/Clippy commands.
Source and reusable test support require fresh direct line/region measurement;
assertion bodies do not inflate source coverage and literal declarations are N/A.
