# Context contract checkpoint evidence

Date: 2026-09-08. The starting source revision is
`890b3841e18a78d4243e94e2c00bbfc4c781c5e0`, which records the completed
[financial prerequisite](e006-financial-contract-evidence.md).
This checkpoint defines context contracts and fixed signing fixtures. Product
construction, evaluation, approval, channels and complete delivery remain later
implementation work.

The [context owner](../requirements/context-contract.md) defines exact identities,
complete delivery, evaluation receipts, authenticated approval, signatures/current
trust and channel transitions. The [validator guide](../../contracts/validation/README.md)
owns the standalone commands and pinned environment. Existing schema families
and their fixture meanings remain unchanged.

## Independent fixtures and reconciliation

The fixture author froze 2,958 literal cases across 77 named targets before
inspecting the context schema or running it. An initial invocation selected an
unprepared environment because the build-output router replaced the earlier
environment assignment. That attempt failed with exit 2 before comparing values.
The corrected invocation selected the prepared environment after the router.

The first actual comparison took 154.947 seconds and exited 1 with ten
disagreements. Independent review distinguished implementation defects from
oracle mistakes; the original inputs and raw diagnostics remain retained.

Three negative cases correctly exposed a missing 32-member limit on transport
results. The schema now enforces that bound in its shared result definition and
the separately composed operation-read result. Those three expectations remain
unchanged. Six positive cases incorrectly allowed additions inside immutable
record wrappers, and one positive case had impossible all-zero evaluation counts.
The owning prose now states these boundaries explicitly. Only those seven
existing expected booleans changed; their identities, targets and literal values
are unchanged.

Twenty-three independently reviewed cases were added: one zero-passed/one-failed
count tuple and eleven nested result 32/33-member boundary pairs. The corrected
inventory contains 2,981 cases: 374 accepted and 2,607 rejected values across the
same 77 targets. The expectation description records the original independent
freeze and subsequent reviewed corrections. This is not an implementation-derived
oracle or a claim that the first comparison passed.

## Actual parsed-contract qualification

The guarded run completed from 08:24:31 through 08:44:58 UTC.
All 72 qualification commands produced their expected result: 19 successful
checks and 53 deliberate failures, without unexpected exits or timeouts. The
prepared native macOS ARM64 environment retains CPython 3.14.7, uv 0.12.10,
check-jsonschema 0.38.0, jsonschema 4.26.0 and the existing fourteen locked
upstream packages. Host defaults, project configuration and lock are unchanged.
Commands used the required build-output router, explicit isolated environment,
default format checking and the documented ECMAScript regex behavior.

All ten documented checks passed:

| Check | Actual result |
| --- | --- |
| Offline lock check | Exit 0. |
| Locked offline synchronization check | Exit 0. |
| Metaschema check naming all nineteen schema paths | Exit 0. |
| Common exact family | Exit 0; 248 expectations. |
| Acquisition exact family | Exit 0; 217 expectations. |
| Observation exact family | Exit 0; 168 expectations. |
| Candidate review exact family | Exit 0; 237 expectations. |
| Financial records exact family | Exit 0; 519 expectations. |
| Context exact family | Exit 0; 2,981 expectations. |
| Separate signing-vector structure | Exit 0; fourteen vectors. |

The six families check 4,370 expected outcomes. All six and the separate vector
structure passed from a relocated input directory containing spaces and
non-ASCII characters. Sixteen manifest/metadata/value-oracle mutations failed
with validation exit 1. Seven infrastructure probes failed with the expected
reference-resolution diagnostics: broken consumed positive and negative-`not`
references, and missing context, common, observation, candidate review and
acquisition libraries. Each missing library was actually consumed by its probe.
Infrastructure errors are not counted as domain-value rejections.

All thirty off-diagonal fixture-family swaps failed with validation exit 1.
The 53 independent failure probes used four separate disposable mirrors; every
probe restored and rehashed its mirror. Final restored context and vector checks
passed. All 28 machine inputs and seven guidance inputs remained unchanged;
all 144 raw stdout/stderr streams were retained and rehashed. The reference
graph contains 5,202 local references across nineteen schemas, with no remote
application reference, injected identifier or base URI. Relocation is distinct
from a fresh Git consumer and does not establish a network sandbox.

| Context input | SHA-256 |
| --- | --- |
| Exact-family entry | `e52599118a96be35bbe42c0ef8c6838a952cc531c39a9a4e12c86d4b9201ad34` |
| Context schema | `d56e740553d4e405d79ea48765c3c9c19e9229e5fb54219b33083bd2d7c651c7` |
| Case manifest | `0cfa3e2cf7c56f9536d0652b8f9eb927b5940de10dde1c1f80bda22bd3def10a` |
| Expectation schema | `ce702ff9018862071a096b368089a90040078755e0f36daeeb2b7c1d92253c87` |
| Context owner | `f15c68cea7bbd66273a6e321a78443bb670268a4ecfd698a140c39f30f1fd00a` |
| Validator guide | `ae5a5e715a1fcb2535d65612f6a9f57a63ba40c1e8ba537dbd0746a940b8166c` |

## Actual fixed signing verification

Exact input bytes and fourteen expected outcomes were frozen before either
implementation ran. Both installed independent primitives, OpenSSL 3.6.4 and
libsodium 1.0.22, produced the same Pure Ed25519 signature and SHA-256 digest.
Their cross-verification accepted the valid vector and rejected all thirteen
fixed mutations. The complete message is 486 bytes: the 32-byte ASCII project
prefix, an actual NUL byte and the 453-byte canonical ASCII payload.

The public fixture uses the published RFC 8032 TEST1 seed and public key. These
are explicitly public test material, never operational keys or trusted issuer
registration. Its synthetic build/profile digests are input literals, not claims
of computed build or approval identities. Eight mutations change one payload
field each; the remaining five change the prefix, remove NUL, substitute printable
backslash-zero, change one message byte or change one signature bit. Primitive
rejection is distinct from current time, trust or authority rejection.

The initial 75-command batch ran from 08:05:21 through 08:05:49 UTC. The second
104-command batch ran from 08:13:10 through 08:13:43 UTC. It consumed the actual
serialized public vector bytes through both primitive/digest implementations,
checked their metadata, rejected 42 metadata mutations and two missing-reference
probes, and passed relocated/restored checks. All 358 raw stdout/stderr streams
were retained and rehashed. Independent review additionally verified the consumed
artifacts, frozen inputs, library independence and expected diagnostics.

OpenSSL's CLI could not allocate a buffer for the published empty-message test
on this installation. That exact infrastructure limitation is retained; no
OpenSSL empty-message pass is claimed. libsodium reproduced that published
signature. Both implementations passed the published nonempty RFC control;
separate wrong-message and wrong-public-key probes rejected on both.

After the two transport-member limits changed, a separate two-command addendum
rehashed all nine signing-related schema/data/environment inputs and passed the
current metaschema and exact-vector checks. The only changed input was the
context schema, whose exact delta is outside `signed_payload`; signing inputs and
outputs are unchanged. The original receipts remain immutable. Cryptographic
operations were not repeated for unrelated transport limits.

| Fixed input/output | SHA-256 |
| --- | --- |
| Project signing message | `b32ae0afe598574149b216a29552016b4c1f987d8a425315a798205472182d2f` |
| Signing-vector data | `d8d70941cc8846a10a8c5e21302c411e20d1e8e6c6e9e3f8d9522f88c0b80736` |
| Signing-vector schema | `37a58cd1c679eb08dcb1bfb87bcde2f00f1595057c4aeacd176d95438f25d10d` |

## Review and remaining qualification

Three independent design reviews passed before implementation. Independent source
review subsequently found that optional outer JSON escaping invalidated the
claimed response expansion bound. The owner now requires literal UTF-8 chunk
emission except mandatory quote/backslash escaping, and actual near-limit
serializer tests. Reader compatibility remains unchanged. Independent source,
fixture-reconciliation and signing-evidence reviews passed after the documented
corrections.

Parsed values do not qualify raw token/Unicode admission, JCS implementation,
digest/reference/count equations, complete chunk or token delivery, current
source/identity/policy authority, event consumption, trust expiry/revocation,
transactions or generation races. Fixed valid-message verification does not
qualify the stricter malformed-point/subgroup acceptance profile. The full
canonicalization and malformed-signature experiments, actual qualified profiles,
native runtime suites and production evidence remain required by the owning
[contract](../requirements/context-contract.md) and [acceptance](../requirements/acceptance.md).

There is no first-party Rust or Python source change in this checkpoint and no
numerical source-coverage claim. The strict [testing contract](../requirements/testing-and-coverage.md)
applies to the runtime and maintained fixture/test support when introduced.
