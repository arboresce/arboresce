# Independent primitive fixtures

These are original public synthetic contract fixtures under this repository's
MIT OR Apache-2.0 terms. They contain no customer records, private originals or
credentials. They define expected values before the domain implementation.
They are fixture data, not a second normative specification.

[Common contract](../../../../../docs/requirements/common-contract.md),
[data model](../../../../../docs/requirements/data-model.md) and
[observation contract](../../../../../docs/requirements/observation-contract.md)
own the semantics. [sources.json](sources.json) pins their exact inputs and the
selected existing public wire cases. A changed consumed input needs reconciliation
and affected independent qualification before reuse.

- [expected.rs](expected.rs) contains declaration-only literal text/byte UUIDv4
  and checked binary-construction cases. `Some` means the exact accepted bytes
  or text; `None` means rejection. Accepted text roundtrips unchanged, including
  all four RFC variant nibbles. Low-entropy-looking valid identifiers test format
  only; they do not prove random generation, resource existence or authority.
- [semantics.json](semantics.json) has nine independently authored groups:
  tagged revision parsing, checked immutable advancement, real Gregorian dates,
  exact UTC millisecond timestamps, injected-clock behavior, UTF-8 ranges, JSON
  Pointer selection, upright page bounds and audio/channel bounds.
- `selected_wire_case_ids` in [sources.json](sources.json) names existing
  complete reference/locator wire fixtures in this checkout. Their original
  validity flag means parsed wire shape only. A structurally valid record
  roundtrips its original parsed value. Range ordering, code-point boundaries,
  actual selection, exact referenced bytes and current authority still require
  their separate checks; a wire-valid reversed range must fail semantic admission.

Revision values are distinct tagged integers, not interchangeable counters.
`original_after_attempt` describes the original immutable value, not stored state
after a command. An advancing operation returns a new value, or fails at the
maximum without wrapping. Only generation admits zero. The clock sequence is
supplied explicitly and may repeat or go backwards; it provides no revision order
and calls no ambient clock. Date-only values never imply an instant or timezone.
Timestamp expectations use exact integer milliseconds, including before the Unix
epoch, and reject leap seconds and alternate precision/offset forms.

The UTF-8 source `Aé🙂\n` is exactly eight bytes. Locator selections use half-open
intervals, actual code-point boundaries and explicit synthetic representation
metadata. JSON Pointer cases distinguish missing values from selected null,
false, zero and empty strings. Selected-value canonical bytes and SHA-256 are
independent literals; their digest is distinct from the original JSON digest.
Discriminator cases reject duplicate source members, unpaired surrogates,
precision-changing decimals, underflow, overflow and out-of-profile integers.
They distinguish UTF-16 object ordering from scalar ordering and normalize both
signs of zero to canonical `0`; equivalent decimal/exponent spellings may agree.
Geometry uses upright pixel edges; audio uses the exact declared duration and
channel count. These fixtures do not qualify real decoding, rendering, resampling,
representation admission or authorization.

The first domain checkpoint consumes only the identity tables. The following
revision/clock/date/locator checkpoint consumes its named groups and selected
wire cases after independent acceptance. Every consumer must execute its complete
declared cases and qualify its actual source; these fixtures alone prove no Rust
runtime, service, focused corpus, provider, restore or product behavior.
The named vectors do not replace the consuming checkpoint's complete raw-token,
source-parser, canonicalization, depth/node/output budget and actual process/source
qualification. Those remain required at the actual revision/clock/date/locator
consumer, with exact supported profiles and no partial-digest success.
No fixture generator or parser helper is introduced here. The Rust table has no
functions or executable regions, so its source role is proven metric-specific
N/A rather than invented 100% coverage.
