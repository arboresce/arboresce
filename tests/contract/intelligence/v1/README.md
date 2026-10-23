# Intelligence transport inputs v1

These synthetic inputs belong to the typed stage-transport contract. They are
prepared before its maintained framing and semantic consumers. Independent
literal admission remains required; the presence of these files is not acceptance.

[Wire vectors](wire-vectors.json) freeze 42 byte strings and their independently
selected framing results for the named messages in
[processing.proto](../../../../contracts/internal/proto/arboresce/intelligence/v1/processing.proto).
They use the selected closed-version rules from the
[transport requirement](../../../../docs/requirements/intelligence-transport-v1.md)
and the upstream [wire encoding](https://protobuf.dev/programming-guides/encoding/)
and [field presence](https://protobuf.dev/programming-guides/field_presence/)
definitions. The byte strings were chosen directly from field numbers and
primitive encodings, without consulting the future validator.

The framing scope is deliberately explicit. An empty message or opaque JCS
record can have valid protobuf framing while still failing complete stage
admission. Complete required fields, record codecs, trusted request/response
joins, limits and authority retain their own semantic checks before any effect.
The ordinal-zero/absent pair must preserve explicit presence. The two field-order
vectors show why protobuf serialization cannot replace the named JCS preimages.

For this selected version, unknown fields/enums, unspecified enum sentinels,
duplicate singular/oneof members, overlong integers and unsupported wire types
are refused. This closed admission rule is stricter than a general protobuf
decoder's compatibility behavior. Repeated fields remain distinct occurrences.
Expected validity stays in the assertions; it is not supplied to measured
decoders or semantic validation.

[Frame boundaries](wire-boundaries.json) separately freeze two exact byte
recipes at 1,048,576 and 1,048,577 bytes. Each recipe concatenates its literal
hex prefix and the stated number of ASCII `x` bytes. The count and prefix were
chosen from the hard frame cap and the standard length varint, independently
of the future consumer. Accepting the smaller frame proves framing only; its
string is not a semantic organization identity. Keep these expectations out
of the byte producer and decoder inputs.

[Supplemental vectors](wire-supplement.json) retain eight additional literal
frames chosen directly from the declared scalar types and primitive encodings.
They distinguish scalar wire-type failures, exact unsigned maxima, an excessive
varint, valid multibyte strings and opaque non-UTF-8 bytes. Framing admits the
integer maxima and arbitrary bytes without relaxing ordinal, fence or JCS
semantics. These examples require independent admission before consumption;
their expected answers do not come from the decoder.

[Record inputs](record-codecs.json) separately draft complete JCS values for
all fourteen fixed record fields. They contain real computed profile/core,
source, plan, result and proposed initial-content preimages; they do not reuse
placeholder digests as semantic evidence. Candidate and expense examples retain
their complete owning public preimage envelopes and normalize only the declared
sets. Literal permutations preserve their content identity; duplicate identities
and bare-content hashes are refused. Their selected codec is
`intelligence-jcs-v1`. Independent literal admission remains required before
any maintained record consumer executes.

The record scope is canonical bytes, the fixed complete target, own identity/
digest preimages, intrinsic quote fractions/validity and a profile's own limits.
It is not complete StageRequest/StageResponse admission. Cross-record source,
context, allocation, stage, deadline, dependency, locator and exposure/cost joins,
authenticated authority and actual processing remain separate required gates.
The SuccessfulStage and FailedStage labels select their named complete JCS
fields, not the containing protobuf messages.

All payload paths identify literal bytes. Negative files deliberately include
invalid UTF-8, an unpaired Unicode escape, duplicate members, noncanonical
spellings, partial records and shape-valid wrong digests. Expected fields stay
in thin assertions, outside measured decoders and support inputs. The authoring
recipe uses independently declared UTF-16 ordering and standard JSON for its
integer/string/bool/null examples. Numeric boundary spellings were selected
literally from JCS. No prospective consumer supplied those expectations.

The two large additive records have 65,536 and 262,144 scalar elements plus
their containers, all within the transport's byte/depth/collection limits.
They exceed the unrelated source-JSON profile's node quotas. Depth 64 is legal;
depth 65 is refused. Selected byte/depth boundaries reuse the same complete
record. Configuration inputs require explicit integer limits within the declared
ranges; mutable byte views and text are not exact immutable byte intake.

These are synthetic conformance bytes. They qualify no running RPC, model,
provider, pricing authority, persistence, recovery, canonical writer or user
journey. Complete semantic examples and applicable source/generation/process
qualification remain required.
