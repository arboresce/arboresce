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
compare-and-swap or establish revision order from time. Closed wire/raw-JSON
admission, clocks, dates and locators retain their complete implementation gates.

Validation proves format. Resource existence, ownership, authenticated principal
identity, random generation and ordering belong to their actual consumers.
The library has no transport, storage, ambient clock or random source.

The independently reviewed [identity tables](tests/fixtures/primitives-v1/README.md)
provide exact bytes, roundtrips and rejections before implementation. Remaining
revision/clock/date/locator fixtures retain their later implementation gates.
This library does not establish an API service or MVP runtime.

Use Rust 1.99.0 and the repository's workspace format/check/test/Clippy commands.
Source and reusable test support require fresh direct line/region measurement;
assertion bodies do not inflate source coverage and literal declarations are N/A.
