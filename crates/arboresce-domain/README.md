# Arboresce domain values

The current library implements checked immutable UUIDv4 identifiers. It accepts
only the canonical 36-byte lower-case spelling or 16 bytes with version 4 and
the RFC variant. Display produces that exact canonical spelling. Equality and
hashing compare the validated bytes. Errors retain no invalid input and emit a
fixed diagnostic.

Validation proves format. Resource existence, ownership, authenticated principal
identity, random generation and ordering belong to their actual consumers.
The library has no transport, storage, ambient clock or random source.

The independently reviewed [identity tables](tests/fixtures/primitives-v1/README.md)
provide exact bytes, roundtrips and rejections before implementation. Additional
revision/clock/date/locator fixtures retain their later implementation gates.
This library does not establish an API service or MVP runtime.

Use Rust 1.99.0 and the repository's workspace format/check/test/Clippy commands.
Source and reusable test support require fresh direct line/region measurement;
assertion bodies do not inflate source coverage and literal declarations are N/A.
