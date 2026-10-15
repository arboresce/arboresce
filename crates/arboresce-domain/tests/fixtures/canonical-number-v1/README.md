# Canonical number literals

These independent synthetic decimal and trait cases are original public data
under MIT OR Apache-2.0. The factual IEEE754 serialization pairs are attributed
to [RFC 8785 Appendix B](https://www.rfc-editor.org/rfc/rfc8785.html#appendix-B).
The two additional nonfinite bit patterns complete sign/NaN checks. No RFC prose
or dependency implementation is copied. Dependency licenses remain separate.

The owning [common contract](../../../../../docs/requirements/common-contract.md)
and [data model](../../../../../docs/requirements/data-model.md) retain signed,
fractional binary64 semantics and exact independent money/revision types.
This separately selected scalar profile accepts one complete number token up to
4096 UTF-8 bytes and emits at most32 ASCII bytes, with no newline or BOM.
Original decimal meaning must equal the shortest canonical decimal after binary64
conversion. Equivalent spellings and signed zeros share canonical bytes and hash;
a parsed float alone cannot establish admission. Rust str intake already has valid
UTF-8; this does not qualify a raw-byte or complete-document UTF-8 decoder.

All expected canonical strings are fixed literals. SHA-256 and byte lengths are
independently computed over those exact bytes before Rust consumption. The
4096/4097 cases preserve otherwise valid mathematical values; long padded
exponents do not create incidental integer-overflow restrictions. Zero requires
complete grammar before its exponent may be ignored. Decimal precision and
underflow negatives remain distinct from unsigned protocol-token rules.

Direct published-vector tests call only the dependency's primitive-f64 to_vec;
they inspect its exact output and128-byte capacity. Out-of-domain finite vectors
test the dependency, not domain intake. Its boxed writer allocation and132-byte
ryu stack buffer remain part of the reviewed subset. Object/string/array/key,
raw-fragment, generic Serialize and convenience pipe paths are not qualified.

Independent expected acceptance precedes the consumer. The original28source
pointer cases remain unchanged and unconsumed here. Whole-source duplicate and
Unicode admission, aggregate/selected budgets, UTF16 ordering, actual selection,
cross-language agreement, signatures and whole-checkpoint qualification remain
mandatory at their consumers. Literal vectors alone are no runtime evidence.
