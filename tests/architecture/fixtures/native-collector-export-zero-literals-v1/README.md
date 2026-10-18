# Exact zero in coverage export totals

These original public synthetic literals use MIT OR Apache-2.0. They add 23
assembly cases to the existing [complete export contract](../native-collector-export-v1/README.md)
and [lossless numeric literals](../native-collector-export-literals-v1/README.md).
Every previous case and acceptance requirement remains unchanged.

The second capture has no file records. Its six metric objects contain exactly
21 numeric fields, and EVERY value must be mathematically zero. A decimal
coefficient with any nonzero digit remains nonzero even when its exponent makes
conversion to binary64 underflow to zero. Consequently, neither parsing a
number as floating point nor comparing the rounded result with zero establishes
this contract. Integer zero, signed zero, fractional zero and exponent forms of
zero are accepted; exponent digits do not make a zero coefficient nonzero.

The first 21 cases place one explicit nonzero underflow literal in each required
metric field while every other field remains zero. Each must reject with the
existing finite assembly error. The next case supplies valid zero spellings
across all 21 fields. The last preserves nonzero tiny values in BOTH first-part
file data and second-part function data. Underflow is not a reason to reject or
change values outside the second capture's zero-file totals.

Only each case.input reaches assembly. The ordered parts contain exact utf8 byte
strings. Negative expected objects and complete positive expected_utf8 strings
remain in thin assertions. Positive comparisons use ENTIRE output bytes, without
rounding expected or actual numbers through another representation. Expected
meanings are frozen independently before consumer changes; no consumer-generated
output supplies the oracle.

Existing strict duplicate-key, type, shape, nonfinite and trailing-data rejection
still applies. Preserve the complete original first document and entire second
functions vector, including all nested values, order and repeated records.
Use the same selected tool, exact two captures, unchanged8MiB streams, original
single180s enclosing export budget, complete64MiB ceiling and complete
before/between/after input identities. Do not add a parser feature, dependency,
lossy reconstruction, new output class or fallback.

Fixture admission does not qualify executable bytes. Complete owning and fresh
public-only standalone collections, genuine selected-LLVM comparisons, strict
coverage at every required scope, source/process/profile/raw/module/native
conservation, resource/model/probe checks, retention, both positive owned
cleanups and independent reviews remain separate requirements.
