# Checked revision value fixtures

These original public synthetic fixtures use this repository's MIT OR Apache-2.0
terms. The [common contract](../../../../../docs/requirements/common-contract.md)
owns the distinct content/control/generation ranges and exact decimal semantics.
`cases.json` freezes the public scalar API outcomes before their implementation.

Payload cases exercise the exact decimal value string. Numeric cases exercise
an already typed unsigned integer constructor. Content/control admit one through
999999999999999999; generation also admits zero. Successful formatting emits
the same canonical decimal bytes. No floating-point conversion is permitted.
The nine advancement rows copy the exact previously accepted
[primitive semantics](../primitives-v1/semantics.json), retain their `source_id`
and immutable original-value expectations, and add only a unique event prefix.
Both source files are pinned in the fixture. Trait/formatter and input-independent
error strings are explicit selected library API expectations.

The domain runtime remains standard-library-only. Tests decode this literal
fixture using the separately pinned development-only parser; that decoder does
not admit user input. Checked scalar values alone do not qualify the complete
closed tagged JSON wire object, original numeric tokens, source JSON parsing,
JCS, clocks, dates, source selection or locators. Those remain mandatory in the
owning revision/clock/date/locator checkpoint. Existing identity tables, their
meaning and their complete test inventory remain unchanged.
