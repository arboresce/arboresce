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

[Complete stage inputs](stage-vectors.json) separately freeze literal request
and response frames, complete readable records and independent comparison inputs
under `stages/`, `stage-admission/` and `stage-sources/`. The four output families
include both workspace candidates and project changes. Examples retain exact
accepted dependencies, project origins and four distinct lineage categories,
global stage/reconciliation attempt ordinals, source and locator bytes, finite
quotes, complete earlier/current exposure records and reduced rational costs.
Selected message/record/material boundaries and contradictions retain their
independently chosen answers. Independent admission remains a consuming gate.

The observation files are separate synthetic fixture inputs. `dispatch_state`
and `dispatch_clock` describe the dispatch cut; `current_state` and
`response_clock` describe the later response-check cut. Canonical record bytes,
server timestamps, accepted dependencies, actual invocation sets and receipt
associations are separately supplied observations. No response or profile label
establishes those facts. The text, JSON, two-pixel PNG and one-second PCM WAV
are literal conformance material, without actual media/clock/authority/resource
qualification. The source manifest records their exact identities and envelopes.

The byte author uses literal protobuf field numbers and shortest primitive
varints/lengths, standard-library SHA256 and explicit UTF-16 JSON key ordering.
It imports no protobuf serializer, framing decoder or prospective stage consumer.
Each case pins its exact request and response byte count and digest. Expected
fields belong only to thin assertions. A refused request has a null response
expectation: no response consumer is executed for that request. Late failure
and uncertainty examples preserve already incurred evidence within the original
reconciliation deadline; they authorize neither new work nor a renewed budget.

The boundary literals retain all sixty-four stage declarations and their exact
source preimages, and a complete 256-attempt execution with fixed stage clocks.
They distinguish semantic ordinals and safe worker fences from protobuf integer
capacity. Selected output, warning, context, dependency, lineage, stage, attempt,
attachment, exposure and transcript counts have explicit inclusive/one-over cases.
Pending proposal DAGs preserve exact available item and committed-record versions
without supplying review acceptance or application authority. Transcript examples
retain complete synthetic text bytes and separately supplied audio alignment.

Eleven additive literal budget comparisons distinguish the fixed whole-operation
allowance across stages: verified settlement, prior-over-allowance dispatch,
uncertain and known-but-unsettled reservation floors, greater retained liability,
and identical/conflicting exposure overlap. Equality is inclusive; exact rational
overflow refuses. These synthetic inputs qualify no actual shared admission,
authenticated issuer, gateway invocation or per-capture spending authority.

The complete-stage inventory also retains sixty-four additive semantic literals.
They cover exact snapshot scalar types, malformed comparison containers, counted
progress and reconciliation counters, complete prior-result attempt/history
joins, preserved later settlement, nested proposal identities, document and
transcript originals, all observation provenance branches, reachable inference
basis bounds and attributable supplied context/representations. Those additions brought the inventory to 299 exchanges; the earlier 235 case
rows remain unchanged.

New semantic rows explicitly declare comparison_materials: complete supplied
material keys loaded by the fixture driver, without expected-answer values.
This permits malformed-observation intake without making the loader fabricate
missing files or infer validity from the maintained comparison implementation.
Every listed file must exist. Original rows retain their earlier material
selection. All new literals still require independent admission before semantic
consumption; their presence claims no real model, human, audit or service result.

Fifty further additive literals cover the remaining complete-consumer review
findings. Typed Boolean defaults distinguish Boolean values from equal-looking
integers/floats; declared billing-unit arrays cannot be supplied as objects.
Current optional failed parts retain exact sealed-binding correspondence.
Historical supplied-representation assertions preserve complete ordered capture
intents/readiness and every retained binding, including valid historical drafts.

Every consumed proposal evidence role, expense field basis and project
source-observation lineage is a provenance root. The literals include complete
versus missing supplier attribution, unavailable or cyclic retained inference
bases, and historical document/model locator checks. Historical representations
remain independently resolved outside the current capture/blob/representation
list, with their own conversion and creation profiles. Same content bytes cannot
replace exact original identities. No additional authority flag or assertion
processing profile is introduced.

The aggregate inventory is 359 exchanges: 250 request admits and 109 refusals;
103 response admits, 147 refusals and 109 unconsumed responses. All earlier 299
complete case rows and answers remain unchanged. These new literals also require
independent admission before consumer changes. Canonical-byte/schema self-checks
are author evidence, not independent acceptance or real human/model/read/runtime
qualification.

Four additional counter literals preserve the acquisition state invariant in
current and historical captures. A content revision advances both counters and
leaves a draft; submission advances control again. Submitted content two/control
three is consistent; submitted two/two and draft two/one are not. The seven
submitted historical correspondence snapshots use control three; the valid draft
two/two remains distinct. All preceding 349 complete case rows, answers and frame
pins remain unchanged. These facts model possible recorded states without
inventing event histories or actual service qualification.

Six further literals isolate capture upload-ID uniqueness and the inclusive
100-MiB declared-intent aggregate in current and historical captures. Distinct
upload IDs for equal byte identities remain valid; roles, generations or whole
object uniqueness cannot conceal a repeated upload ID. Every declared part
counts before eligibility and storage deduplication, including unavailable
optional parts retaining earlier sealed bindings.

The aggregate-boundary controls declare one three-byte ready note and four
unavailable optional PNG sources. Each PNG declaration fits its individual
25-MiB envelope; the combined declaration is exactly 100 MiB or one byte over.
Only the ready three-byte material is supplied to processing, within its
unchanged selected limits. These are synthetic metadata comparisons, without
actual 100-MiB media, storage, decoding or resource qualification. The earlier
353 complete case rows, answers and frame pins remain unchanged.
