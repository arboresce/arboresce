# Observation and locator v1 contract

**Authority:** normative observation, provenance and locator representation and
admission requirements. **Status:** specified contract with parsed-value
fixtures; no parser, extractor, renderer, transcription service, authorization
boundary or review workflow is qualified by those checks.

The [observation schema](../../contracts/public/v1/observation.schema.json)
provides named definitions and rejects every instance at its library root.
The [validation guide](../../contracts/validation/README.md) supplies the real
standalone fixture consumer. It composes the [common contract](common-contract.md)
and [acquisition contract](acquisition-contract.md) without replacing their
identities, byte limits or command semantics. [Data model](data-model.md) owns
canonical content and state separation; [processing](processing.md) owns
extractor profiles and the trust boundary around model output;
[security and privacy](security-and-privacy.md) owns current authorization;
[resource profiles](resource-profiles.md) own admitted media and runtime budgets.

This family defines records consumed by later domain and internal transport
contracts. It introduces no public mutation, download or review route. A valid
record is neither an authenticated command nor permission to persist, expose,
approve or promote its contents. The later operation owner supplies the complete
closed command and its preconditions before implementing such an operation.

## Definition boundaries

| Definition | Meaning |
| --- | --- |
| `representation_reference` | Exact immutable representation, original, byte identity and processing profile. |
| `utf8_range`, `json_pointer`, `page_region`, `audio_span` | Four closed locator variants; `locator` is their discriminated union. |
| `transcript_segment` | Exact transcript representation and zero-based segment selection used by an audio locator. |
| `capture_reference`, `observation_reference` | Exact capture content revision or immutable observation identity, with ownership scope. |
| `document_source`, `assertion_source`, `inference_source` | Closed provenance descriptors with distinct attribution requirements. |
| `grounding` | A nonempty locator set or an explicit ungrounded, review-required result. |
| `document_observation`, `user_assertion`, `model_inference` | Complete typed records, with the corresponding `kind` and source shape. |
| `observation` | The complete three-kind union used to validate an observation. |
| `observation_response` | Shared response bounds only; not a substitute for a complete variant. |

References, locators, grounding and source descriptors are closed canonical
objects. Unknown fields in those objects fail validation. Outer observation
records follow the bounded additive reader policy in the common contract,
including its maximum 32 members and control-message bounds. Unknown bounded
extensions do not create authority or change the meaning of known fields.
Direct outcome, operation, confidence and review aliases are reserved and
rejected: `state`, `status`, `code`, `problem`, `error`, `failure`,
`failure_code`, `success`, `accepted`, `approved`, `reviewed`, `review_required`,
`confidence`, `result` and `operation`. Confidence, calibration, review decisions
and promotion have their own later contracts; a producer cannot add a flag to
turn an observation into an approved fact.

## Exact identity and admission

`representation_reference` contains `representation_id`, the acquisition
`original_reference`, acquisition `byte_identity` under `bytes`, and the exact
`profile_digest`. Resolve the tuple against the canonical representation record:
the original version, digest, length and profile must all agree. A correct digest
with a different organization, version or profile is not an equivalent reference.
Never follow an artifact's current head, resolve a filename, or repair a mismatch
by substituting another representation with similar text. Mutable availability,
grants, retention and review state remain separate from this immutable identity.

Before reading referenced bytes or admitting an observation, establish current
organization, domain, permitted purpose, source restrictions and authenticated
authority through the owning operation. Apply that authority to every original,
capture, basis observation, audit event and transcript, including indirect
dependencies. Recheck before persistence or disclosure when the operation crosses
an asynchronous boundary. The source's organization must agree with the owning
record; all references must satisfy the domain's permitted source policy.
Opaque IDs and digests confer no access. Foreign or inaccessible references
follow the [API](api.md) non-disclosure policy; errors cannot reveal their real
organization, content or existence through counts or partial results.

Provider and extractor output is untrusted input. The trusted domain boundary
must validate both the parsed schema and all semantic conditions below, derive
or verify provenance from admitted processing work, and reject forged identities
or source-kind changes. Invalid, missing, stale, foreign or digest-mismatched
references cannot be relabeled as harmless ungrounded output. A deliberately
unsupported locator can produce an explicit review-required result only after
its underlying source identity and current access are established.

All integer fields in this family use the common raw unsigned-integer token
profile. Schema `integer` checks mathematical parsed values, so it does not
establish the required lexical admission. Implementations must use checked
arithmetic for offsets, index conversion, sums and geometry. The following
ceilings are chosen v1 wire capacities, not claims that a decoder, database or
profile supports every representable value.

## UTF-8 byte ranges

`utf8_range` contains the exact representation and explicit `start_byte` and
`end_byte`. Its half-open interval must satisfy
`0 <= start_byte < end_byte <= representation byte length`. The schema allows
start 0 through 9007199254740991 and end 1 through 9007199254740991, matching the
acquisition byte-identity arithmetic ceiling. Ordering and actual bounds are
mandatory domain checks, including when both endpoints pass the schema.

The representation must be valid UTF-8 in a qualified text profile. Both
endpoints must be code-point boundaries; offsets count bytes, not characters,
UTF-16 units, graphemes or display columns. Empty or reversed intervals fail.
Normalization, newline conversion, OCR, transcription or other rewriting creates
a different representation. A range from original CRLF bytes cannot be applied
to normalized LF text, even when the displayed words match. Decode errors are
not repaired with replacement characters while retaining the original digest.

## JSON Pointer and selected-value identity

`json_pointer` holds a JSON string using [RFC 6901](https://www.rfc-editor.org/info/rfc6901/)
pointer syntax, limited here to 4096 Unicode characters and 64 slash-prefixed
tokens. Empty string selects the root; `/` selects an empty object key. Decode
`~1` before `~0`, once, so `~01` selects a literal `~1` key. Preserve Unicode
exactly. Control characters, including NUL, are permitted in tokens and must be
escaped in JSON transport. No URI-fragment or percent decoding is implicit.

Object selection uses exact member names. `/01` and `/-` are valid object-key
pointers; array selection requires a zero-based decimal index without leading
zeros and an existing element. `-` never selects an existing array element.
Missing members, descent through a scalar, duplicate source members and
out-of-range indices fail selection. A selected `null`, `false`, zero or empty
string is a value, not a missing result. Syntax validation alone cannot establish
any of those selection outcomes.

`canonicalization` is exactly `jcs_selection_v1`.
`selected_value_sha256` is SHA-256 over the selected value's complete canonical
UTF-8 encoding using [RFC 8785 JCS](https://www.rfc-editor.org/info/rfc8785/).
It is separate from the digest of the representation's original JSON bytes.
The canonical result has no added newline or byte-order mark. Object sorting
uses JCS UTF-16 ordering; array order and string contents are preserved. Do not
hash a display string, reserialized enclosing document or missing-value marker.

The selected source value is not a common control-response object. Its keys may
be empty, contain escaped controls or contain other valid Unicode characters.
Do not apply the common `json_key` minimum length or its control-key restriction
to source JSON, its selected subtree or the root selected value. The original
bytes remain immutable even when selection is unsupported.

The v1 selection profile makes these additional, deliberate choices:

- Selected values may be scalars, arrays or objects. Maximum selected-value
  depth is 64, counting its root as 1. Maximum total value nodes is 65536,
  counting each scalar and container once; member names are not separate nodes.
  Maximum canonical output is 10485760 bytes. These ceilings bound traversal,
  hashing and retained canonical work; they do not declare decoder performance
  or reduce the acquisition limit for retaining a valid original.
- JSON source parsing has a separately declared, qualified processing profile
  for aggregate depth, members, allocation and execution budgets. Check that
  profile before traversal; a short pointer is not permission to parse an
  unbounded document. Stream canonical output through a checked byte budget
  and reject overflow without publishing a partial digest.
- Reject duplicate member names, invalid UTF-8, unpaired surrogates, non-finite
  numbers and values outside the profile. Do not normalize Unicode or repair
  unsupported values. Strings and keys remain subject to the total node/output
  budgets even though the control-object key policy does not apply.
- Numbers use finite binary64 JCS semantics within -9007199254740991 through
  9007199254740991. Fractional source numbers are allowed. Retain raw numeric
  lexemes until admission can verify that their mathematical decimal value
  equals the value denoted by the shortest JCS serialization after binary64
  parsing. Exponent and trailing-zero spellings may therefore agree; excess
  precision, overflow and underflow that change the value fail. JCS deliberately
  canonicalizes either sign of zero as `0`. Exact values outside this profile
  need a separately specified representation, such as an explicitly typed
  string; do not silently round or rewrite an original.

Thus `0.1`, `1.0` and `1e0` can be source numbers under this profile, while a
precision-changing decimal or `1e-400` cannot silently become another value.
This source-number rule is separate from the raw unsigned integer tokens used
for locator offsets and dimensions. Later Rust and Python consumers must agree
on selection, decimal admission, UTF-16 key ordering and canonical bytes before
they share a selected-value identity.

## Upright page geometry

`page_region` names the exact representation, zero-based `page_index`, exact
page `render_sha256`, `source_orientation`, and `coordinate_space` exactly
`upright_pixels_v1`. It records integer `x`, `y`, `width` and `height` in upright
pixel-edge coordinates, with origin at the top left, x rightwards and y
downwards. The rectangle is `[x, x+width) × [y, y+height)`. Width and height must
be positive. Its checked right and bottom edges must lie within the actual
upright render dimensions.

The schema bounds page index to 0..65535, each origin coordinate to
0..2147483647 and each dimension to 1..2147483647. These capacities do not admit
65536 pages, oversized images or a new document format. A qualified rendering
profile must define actual page count, dimensions, density, color handling and
page-to-render digest association. A single-image profile has page index 0.
The render digest must identify the exact bytes used to obtain the geometry;
for a representation consisting of that single render, it agrees with the
representation byte digest. Any qualified multi-page representation must retain
its exact page/render mapping. A newly rendered page cannot reuse an old locator.

Record orientation 1 for an absent orientation tag under the qualified profile;
reject malformed or unsupported orientation metadata rather than guessing.
All eight [Exif orientations (CIPA DC-008-2012, Figure 12)](https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf)
must have defined handling, including mirrored cases. For raw dimensions W×H,
the following raw-to-upright transforms apply to pixel **edges**:

| Orientation | Upright edge (x′, y′) | Upright dimensions |
| --- | --- | --- |
| 1 | (x, y) | W×H |
| 2 | (W−x, y) | W×H |
| 3 | (W−x, H−y) | W×H |
| 4 | (x, H−y) | W×H |
| 5 | (y, x) | H×W |
| 6 | (H−y, x) | H×W |
| 7 | (H−y, W−x) | H×W |
| 8 | (y, W−x) | H×W |

Transform all four rectangle corners, then take the upright bounds. These are
not pixel-index formulas using W−1 or H−1. Orientation normalization occurs
exactly once; the profile must record any additional crop, scale or render
transform. Reject a locator if the stored transform cannot reproduce the
declared coordinate frame. Display zoom never changes canonical coordinates.

## Audio and transcript spans

`audio_span` identifies the exact decoded or normalized audio representation,
zero-based `channel_index`, `start_ms` and `end_ms`. The half-open interval must
satisfy `0 <= start_ms < end_ms <= actual duration`. The v1 range is start
0..299999 and end 1..300000 milliseconds, preserving the accepted five-minute
audio profile. Channel index 0..65535 is a wire ceiling; the actual decoded
channel count and selected codec profile impose the supported bound. Reject
an absent channel, empty/reversed interval or duration overflow.

Use the representation's time origin and qualified sample/time conversion,
not wall-clock time or an offset in the compressed file. The processing profile
must define sample-rate conversion, millisecond boundary rounding and exact
mapping for trimming, channel selection, resampling and normalization. A span
cannot be copied between representations without validating that mapping.
Keep the original and normalized identities and timing provenance distinct.

When a claim uses a transcript segment, include `transcript_segment` containing
the exact transcript `representation_reference` and zero-based `segment_index`
0..65535. The actual segment must exist in that immutable transcript. Its
profile and retained alignment must associate it with the same immutable
original and the cited audio representation, channel and time span. A segment
index in a newer transcript or differently segmented output is not equivalent.
The optional field permits direct audio observations; it cannot be omitted to
conceal transcript provenance when a transcript was used. Speaker/diarization
labels are not identities and this locator supplies no speaker identity field.

## Observation kind, attribution and grounding

Each complete observation has a server-issued `observation_id`,
`organization_id`, canonical `recorded_at`, exact `kind`, nonempty `statement`
of at most 65536 characters, the appropriate `source`, and `grounding`.
The timestamp records canonical admission, not the claimed event's occurrence.
An observation is immutable; corrections and disagreements create separately
identified records with later domain relationships. `observation_reference`
therefore identifies an exact immutable record without following a mutable head.

`document_observation` records what an admitted document supports. Its source
contains the exact acquisition original reference and the extraction profile
digest. Every locator must resolve to that original through its own exact
representation. The extraction profile is the profile used to create the
observation; a representation's profile may be a different recorded conversion.
Extracted text saying an event happened is not proof that the event happened.
The distinction concerns evidential meaning, not the implementation technique:
a model may assist OCR or faithful extraction without making the extracted
document statement an inferred fact. Derived conclusions remain model inferences.

`user_assertion` preserves attributable supplied context. Its source is either:

- `capture_context`: the exact organization/domain/capture/content revision and
  a server audit event identifying the authenticated context supplier and
  canonical recorded time. The captured `user_context` stays in that immutable
  revision. No synthetic representation or fabricated byte locator is created.
- `supplied_representation`: that exact capture reference, the actual supplied
  representation and its attribution audit event. The accepted attachment and
  sealed binding must associate its immutable original with that capture
  revision. The trusted boundary must establish the user-supplied assertion
  role; merely uploading an arbitrary document does not make its claims the
  supplier's assertions. A voice note's supplier is not automatically a speaker
  or the person discussed in it.

The audit event is a server-owned immutable association, not client-selected
proof of authorship. Verify its capture, principal, source and recorded-time
relationships. Later identity and public audit representations own any exposed
actor details. Source text or a model's statement of who spoke cannot replace
authenticated attribution.

`model_inference` records a model-derived claim without relabeling it as an
observation or assertion. Its source has the exact model/extraction
`profile_digest` and an ordered `basis` list of 0..64 distinct immutable
observation references. An empty basis explicitly names no prior observations;
it cannot imply hidden reviewed evidence. Preserve the actual admitted inputs
in the processing provenance, including directly cited representations. Reject
self-reference, cycles, foreign basis records and substituted profiles.
A located inference remains an inference; a model does not gain review authority
by emitting a valid source descriptor.

`grounding.kind=located` requires 1..16 distinct locators. This is a bounded set
of exact citations, not a confidence score, corroboration claim or approval.
All locators must pass semantic validation; do not retain a valid subset while
silently dropping a failed citation. Exceeding the bound requires a separately
specified representation or deliberate subdivision, never silent truncation.

`grounding.kind=ungrounded` requires `review_required=true` and one reason:
`not_provided`, `unsupported_format`, `ambiguous` or `context_only`. It cannot
contain locators. Inline `capture_context` is always `context_only`; the other
source forms cannot use that reason. The known capture/audit attribution is
retained, while the assertion has no independent representation locator. The
other reasons describe an admitted source without usable grounding and require
review before downstream promotion. They cannot excuse forged or unauthorized
references. Located records may also require review under their later operation;
absence of the ungrounded flag is never an approval shortcut.

## Parsed fixtures and pending semantic qualification

The finite independently authored manifest checks the named shapes, bounds,
closed descriptors, source-kind branches and reserved response aliases. Its
fixed declaration order is part of the reviewed manifest; later runtime suites
may reorder execution by stable scenario IDs. Parsed shape acceptance of an
equal or reversed interval is deliberate: JSON Schema does not compare sibling
numeric values. The domain check must still reject that interval.

The following scenarios are required future tests, **not executed results**.
Their owning typed-domain, processing, storage, service and CLI suites must use
independently retained exact inputs and expected outputs under the
[testing and coverage contract](testing-and-coverage.md). The later shared
fixture/runner work promotes them into executable fixtures; that work must not
infer a service pass from the present parsed-value lane.

| ID | Independent scenario and required semantic result |
| --- | --- |
| OBS-SEM-01 | Exact reference with changed organization, original version, byte digest/length or profile fails; matching digest in a foreign source does not disclose content. A current-head replacement cannot satisfy an old reference. |
| OBS-SEM-02 | UTF-8 `Aé🙂\n` has 8 bytes and boundaries 0,1,3,7,8. `[1,3)` selects `é`; `[3,7)` selects `🙂`. Endpoints 2 or 6, equal/reversed endpoints and end beyond 8 fail. |
| OBS-SEM-03 | CRLF original and LF-derived text keep distinct identities and ranges. Invalid UTF-8, replacement-character repair and a range applied to the wrong normalization fail. |
| OBS-SEM-04 | For `{"":17,"branch/key":{"~slot":["zero",false]},"01":"named","-":"literal","~1":null}`, `/` selects 17, `/branch~1key/~0slot/1` selects false, `/01` and `/-` select object keys, `/~01` selects null and empty pointer selects the complete root. |
| OBS-SEM-05 | Against arrays, leading-zero index, `-`, missing/out-of-range index and descent through scalar fail. Empty/control/Unicode keys remain distinct; duplicate source keys fail. Missing selection never hashes as null. |
| OBS-SEM-06 | Independent JCS vectors cover selected null/false/zero/string/array/object, key permutation, UTF-16 versus scalar sort order, escaping, signed zero and decimal/exponent spellings. Recompute exact canonical bytes/digest; representation-byte and selected-value digests remain separate. |
| OBS-SEM-07 | Selection depth/node/output at each exact ceiling is accepted when otherwise valid; one beyond fails without partial identity. Reject excess-precision numeric rounding, underflow, non-finite values and invalid Unicode. Retain an otherwise valid original when its selection profile is unsupported. |
| OBS-SEM-08 | Raw W=7,H=5, rectangle edges (1,1)–(4,2), orientations 1..8 produce respectively [1,1,4,2], [3,1,6,2], [3,3,6,4], [1,3,4,4], [1,1,2,4], [3,1,4,4], [3,3,4,6], [1,3,2,6]. Dimensions are 7×5 for 1..4 and 5×7 for 5..8. |
| OBS-SEM-09 | Last valid page/full-boundary rectangle passes with its exact render; missing page, wrong render digest, zero/overflow geometry, double orientation, wrong crop/scale and pixel-index/edge confusion fail. No unsupported multi-page format is implied. |
| OBS-SEM-10 | Audio exact start/end/duration and valid channel pass; reversed/empty/too-long spans, absent channel and millisecond/sample-boundary errors fail. Validate independent trim/resample/channel mapping against the exact normalized representation. |
| OBS-SEM-11 | Exact transcript segment and retained same-original audio alignment pass; another transcript revision, invalid segment, wrong channel/original/profile or omitted used-transcript provenance fail. Diarization never establishes speaker identity. |
| OBS-SEM-12 | Document observation, supplied assertion and model inference preserve distinct kinds for the same words. Forged source-kind, changed extractor/model profile and invented attribution fail at the trusted boundary. |
| OBS-SEM-13 | Inline context resolves its exact immutable capture revision and server audit event while remaining context-only, ungrounded and review-required. New capture head, forged audit event or unrelated attachment cannot replace that attribution. |
| OBS-SEM-14 | Located output validates every citation; one invalid citation fails the record. Unsupported or ambiguous admitted grounding remains explicit and review-required; a forged/foreign reference cannot be downgraded to ungrounded. |
| OBS-SEM-15 | Basis references resolve immutable admitted records in order. Empty basis does not invent support; duplicate IDs, self-reference, cycles and hidden cross-domain dependencies fail. Concurrent changes cannot substitute new heads. |
| OBS-SEM-16 | Revocation between source lookup and persistence/exposure prevents the later action; no partial output, counts or provenance leaks. Repeat with indirect transcript, audit and inference dependencies. |
| OBS-SEM-17 | Raw integer field `1` may pass; `1.0`, `1e0`, `-0`, boolean and a decimal rounded to an integer fail the unsigned token profile even if a generic parsed schema would accept a mathematical integer. Source JCS numbers retain their separate rule. |
| OBS-SEM-18 | Located or high-quality extraction never bypasses review, source policy or promotion. Conflicting document/user/model claims remain separately attributable; provider output cannot manufacture approval or an executable operation. |

No actual decode, Unicode range selection, JCS hashing, EXIF transform, audio
alignment, current authorization, cross-language agreement or production
qualification is claimed until its owning implementation produces fresh
evidence. An unavailable real profile or service input blocks that qualification
gate while independent contract work may continue.
