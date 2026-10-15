# Dependency and platform qualification

**Owner:** Engine dependency selection, version pins and supported targets.
**Status:** Normative target. A selected technology is not a qualified deployment.
**Verification:** Recorded build, compatibility, service and platform evidence under [acceptance](acceptance.md) and [testing and coverage](testing-and-coverage.md).

Each supported profile must identify an exact, reproducible dependency graph and
the behavior verified against it. A library's advertised feature, a successful
download or a container starting is insufficient evidence that the engine can
depend on that behavior. Qualification records belong with the versioned profile
and the code that uses it.

This document fixes the baseline below. Other dependency versions are selected
and locked when first needed by a working implementation. No product manifest,
lockfile or executable command is introduced by this requirements document.

## Fixed compiler and SDK baseline

| Component | Required selection |
| --- | --- |
| Rust toolchain | Exactly `1.99.0`. Use an exact pin, not a floating channel or an implicit minor-line upgrade. |
| Rust language edition | `2024`. |
| Cargo dependency resolver | `3`. The lockfile format number is a separate setting. |
| Temporal Rust SDK | `temporalio-sdk` exactly `1.0.0`. |
| Temporal Core dependency | Preserve the SDK's compatible `0.9.0` graph; do not force Core to use the SDK's version number. |

Changes to these selections require a reviewed requirement change, compatible
locks and renewed qualification. A nightly compiler or a patched dependency must
not silently replace the selected baseline to make a failing build pass.

Temporal server, persistence schema, administration tools, CLI and UI have their
own pins and compatibility checks. They must not inherit a version number merely
because the Rust SDK uses it. Isolate SDK-owned non-exhaustive types and handle
unknown cases safely. Use supported public runtime, cancellation and testing APIs;
do not make unsupported Core internals part of the application contract.

## Targets and preparation

Service images target Linux AMD64 (`linux/amd64`). Native macOS and Linux CLI
artifacts are required qualification targets of the independent client. Its
declared architecture matrix is separate from the server-image target. Windows
and additional targets require a later supported-profile decision.

Record the actual host architecture and whether execution was native or emulated.
An emulated build or smoke test cannot establish native capacity, latency or
operating-system integration. Every claimed target and feature profile needs its
own applicable result; one platform cannot supply missing evidence for another.

Preparation is explicit and separate from verification. Select exact tool and
service versions, prepare an isolated environment and record licences and origins
before running the checks that need them. Tests must not install tools, download
a Temporal test executable, resolve an unpinned image or silently contact a model
provider. Prepared offline verification must fail clearly when an input is absent.

Rust manifests and locks, Python project metadata and `uv.lock`, image digests,
generation tools and processing-profile resources form part of the qualified
identity. Update affected locks intentionally with dependency changes, then use
locked verification. Do not update unrelated dependencies as incidental cleanup.
Source, lock, configuration, tool or profile changes invalidate affected evidence.

## Current verification-tool parser selection

The local architecture/dispatch tool selects `serde_json` exactly `1.0.151`,
`syn` exactly `2.0.119` with full/parsing/visit and no default features, and
`proc-macro2` exactly `1.0.106` with no default features. Their locked public
graphs and MIT OR Apache-2.0 rights are prepared before offline verification.
That selection parses Cargo metadata and Rust source; it did not select domain
dependencies, canonical JSON, SDK/runtime qualification or an inference provider.
Actual tool/source/process qualification remains required before dependent use.
It retained standard-library-only domain/proof production graphs. The later
scalar-number selection below deliberately qualifies pure domain edges.

The supplied-report gate adds direct `serde` exactly `1.0.229`, with no default
features and only std. This is the already prepared locked public version;
its MIT OR Apache-2.0 rights and graph are retained. It supplies the explicit
duplicate-key-rejecting JSON visitor. This does not select serde for domain
values, canonical JSON or product transports. No other locked version changes.

The inventory selects `sha2` exactly `0.10.9`, with no default features, for
SHA-256 file identity. Its exact locked public graph is prepared from cached,
checksum-verified crate archives before offline qualification. `generic-array`
retains MIT rights; the other added packages retain their declared MIT/Apache-2.0
dual rights. That tool-only selection did not select product signing,
canonical encoding or domain cryptography. Existing selected package versions and
the then-standard-library-only domain/proof production graphs were unchanged. Availability of
an extracted source directory alone is not package-archive preparation evidence.

## Checked revision fixture decoder selection

The checked revision-value tests select `serde_json` exactly `1.0.151` as a
development-only dependency, using the already prepared locked MIT OR Apache-2.0
graph. No external package version or host tool changes. The test decoder reads
independently frozen literal fixture JSON and preserves typed unsigned integer
values without floating-point coercion. Its maintained support is measured and
its full owning and standalone qualification precedes accepted reliance.

This development selection added no normal or build dependency to the domain
runtime. It does not qualify admission of user JSON,
original numeric tokens, tagged wire objects or JCS. Those consumers retain their
own dependency selection, raw-source, semantic and budget gates.

## Selected scalar canonical-number profile

The separate `canonical-json-number-v1` domain API selects
`serde_json_canonicalizer` exactly `0.4.1` and `sha2` exactly `0.10.9` with no
default features and only `force-soft`. The former has no selected features and
brings `ryu-js` exactly `1.0.3`. Preserve every previously locked registry package
record; only those two new packages and the domain's owning dependency edges are
added. Cargo feature unification adds serde default and serde_json float_roundtrip
to the affected workspace graph. This deliberately changes those effective
profiles and requires complete fresh owning and public-only standalone qualification.

The selected `jcs-number-v1` architecture profile binds the conservative unfiltered
23-package normal/build closure, exact registry sources, root and transitive
aliases/kinds/conditions and effective features. Its fixed compact sorted tool-JSON
SHA-256 is `d6d37e664f5971fa19c2d73ae893a46d74a5d89302083987d0f39c5d9f4dd014`.
This tool identity is not product JCS. The lockfile and independently reviewed
[dependency inputs](../../tests/architecture/fixtures/qualified-number-dependencies-v1/README.md)
retain exact package/archive and source-rights facts. Canonicalizer is MIT;
ryu-js is Apache-2.0 OR BSL-1.0. Other transitive rights remain distinct, including
MIT, Unlicense OR MIT and Unicode terms. A successful resolution or download alone
does not qualify behavior or redistribution. Verify exact published archive
checksums and rights before offline consumers; extracted source alone is insufficient.

Only a checked finite primitive f64 from bounded complete numeric grammar reaches
canonicalizer `to_vec`; final original-decimal equality is checked afterward.
Do not expose Serialize, use convenience `pipe`, unsafe string conversion, raw
fragments or generic maps/Values as source admission. The selected source path
emits at most 25 bytes within the chosen 32-byte output capacity. It requests an
initial 128-byte Vec and one boxed writer reference; ryu uses a 132-byte stack
buffer. Object/key/value stacks remain empty. Include those allocations in the
bounded scalar proof and exercise exact native output/capacity against independently
frozen published numeric vectors. This is not an entire-document allocation proof.

The standalone scalar API accepts at most 4096 bytes of complete numeric grammar.
Normalize the original decimal before finite binary64 conversion and require
equality with the shortest JCS decimal before hashing; preserve signed-zero
canonicalization, padded exponents and long insignificant zeros. Reject precision
changes, nonzero underflow and values outside the common safe range. Hash only
complete canonical bytes. Keep immutable canonical-value identity distinct from
representation identity, unsigned protocol tokens, exact money and signatures.
The software feature precedes CPU dispatch; metadata still conservatively retains
cpufeatures/libc without claiming they execute in this subset. No extra SHA feature,
canonicalizer path, version or platform is implicitly qualified.

This selected scalar profile does not select the general source parser's limits.
Full raw-source Unicode/duplicate/numeric admission, UTF16 ordering, aggregate
allocation/work and selected depth/node/output budgets, actual JSON Pointer
selection, cross-language agreement and E10 signing remain required at their
consumers. Preserve every closed Initial fixture meaning and all actual source,
native, service, release and whole-checkpoint gates.

## Selected source JSON profile

`jcs-source-v1` selects the same 23 locked registry packages, archives and effective
features as the scalar profile, adding direct normal `serde` exactly 1.0.229 and
`serde_json` exactly 1.0.151 roots for the private checked AST adapter and bounded
single-string decoder. It retains the canonicalizer 0.4.1 and software sha2 0.10.9
roots. No derive, arbitrary_precision, preserve_order or additional SHA feature
is selected. Preserve every existing registry lock record. The full conservative
normal/build graph's internal tool-JSON SHA-256 is
`5f819d934d2598938c580eafa2b79186a9b34351e82b6976cf27e7dec00beb8f`.
The [selected source dependency inputs](../../tests/architecture/fixtures/qualified-source-dependencies-v1/README.md)
bind exact roots, aliases, conditions, package origins, versions, features,
archive checksums and distinct source rights. Old Initial and number profiles
retain their meanings and separate literal fixtures.

Own complete UTF-8/grammar/duplicate/raw-number validation precedes selection.
Only one already-delimited quoted token enters `serde_json::from_slice::<String>`;
general `Value` source intake and approximate original numeric conversion are
unselected. Only private checked nodes implement Serialize: null, booleans,
admitted finite scalar numbers, strings, sequences and maps. Preflight exact
escaped size, nodes and depth before canonicalizer `to_writer` with a bounded
in-memory destination. Raw fragments, convenience pipe, unsafe conversion,
arbitrary framework values and public Serialize surfaces remain unselected.

The expanded vetted subset includes JCS UTF-16 key sorting, duplicate checks,
key/value buffers, nested object completion, all string escapes and the internal
bounded encoded-key `Value::String` re-decode. It has separate allocation/work/
stack obligations in [the processing profile](resource-profiles.md#bounded-source-json-processing).
Verify exact archive and source-rights hashes before offline consumption. Fresh
owning and public-only standalone source/native/strict qualification is required;
locked resolution is not behavioral qualification. Independently qualified Python
agreement still precedes shared selected identity and whole domain completion.
Signing, money, transport and real service qualification remain separate consumers.

## Technology-specific evidence

The experiment identifiers below refer to the complete criteria in
[acceptance](acceptance.md). They do not narrow the behavior required by the
other owning contracts.

| Dependency boundary | Evidence needed before claiming support |
| --- | --- |
| Rust HTTP and async runtime | Qualify the selected Axum/Tokio graph with the fixed compiler, supported features, cancellation, deadlines, bounded streaming and graceful shutdown. Exercise native compatibility and concurrent authorization paths in E01 and E08. |
| PostgreSQL and SQLx | Verify actual server and driver versions, non-owner roles, tenant enforcement, migrations, required constraints and isolation. Test each claimed direct and transaction-pooled connection profile independently, including prepared-statement/session behavior, leakage, pool limits and regional latency. E02, E04 and E09 apply. |
| Temporal and its PostgreSQL persistence | Verify the exact server/tool/schema/SDK combination, dedicated direct database connections, replay, worker loss, retries, cancellation and upgrade behavior. E03 and E09 apply. Unsupported essential behavior blocks that profile rather than permitting an untested replacement of durable orchestration. |
| S3-compatible object storage | Qualify the selected implementation's credential scope, staged overwrite behavior, one-read sealing, checksums, orphan reconciliation, failure recovery and residency. Do not infer atomicity with PostgreSQL, immutable acceptance or recall of issued grants from an S3-compatible interface. E05 and E09 apply. |
| Qdrant | Pin the service and client, embedding dimensions and model/profile identity. Measure ACL selectivity, stale and reordered writes, generation rebuilds, cold/warm queries, memory peaks and canonical filtering before reranking. E06 and E07 apply. |
| Python media and intelligence service | Pin the interpreter, environment, codecs, OCR/ASR libraries, model route and packaged resources. Test decoder restrictions, bounded parsing, locator validity, cross-language messages and actual held-out processing. E05 and E11 apply. |
| Nostr identity and scoped credentials | Pin exact NIP-01/NIP-98 revisions, vetted secp256k1/BIP-340 libraries and independent vectors. Qualify exact signed request binding, clock/replay/idempotency, key storage/recovery/revocation, protected bootstrap and own-key grants under [identity](identity-contract.md). Actual-human financial review has separate qualified assurance. E02 and E08 remain production obligations. |
| Optional later OIDC | No first-profile issuer prerequisite. Before support, qualify the concrete issuer/public-client browser PKCE/redirect/signature/rotation/recovery contract and any explicitly supported device flow independently. |
| SMTP submission | Qualify the actual certificate-verified transport and submission policy, stable logical notification identity and accept-then-disconnect ambiguity. Test with an isolated mail service, never a production mailbox. E03 includes ambiguous delivery; delivery receipts must preserve uncertainty. |
| Canonical encoding and signatures | Use vetted canonical-JSON and Ed25519 implementations. Verify exact public vectors and Rust/Python agreement, malformed signatures, trust, expiry, rotation and revocation. E10 applies; do not invent cryptographic primitives. |
| Containers and local orchestration | Pin bases, runtime packages and test images; verify the produced digest, resources, startup and shutdown. Local orchestration must invoke the same implemented direct checks. E01 and E12 apply; local success does not replace independent release verification. |
| Public CLI and transport compatibility | Use an independently built, immutable CLI artifact and declared compatibility/capability version. Exercise actual network, credential, acquisition, reporting and export behavior. E01, E05 and E12 apply. No sibling source checkout is a compatibility proof. |

Application PostgreSQL and Temporal persistence remain separate even when one
operator runs both. If a selected database service cannot satisfy Temporal's
required schema or direct-connection behavior, qualify a suitable dedicated
Temporal PostgreSQL service through an explicit profile change. Do not weaken
application constraints or silently redirect application data to solve that
problem. Likewise, a different object-storage implementation requires renewed
behavioral and residency evidence.

Graph-database features are deferred. A later addition needs a demonstrated
workflow, measured workload and review of the exact licence/edition capabilities;
it must not become an implicit dependency of exact-build serving.

## Processing and external-service selections

The focused profile independently qualifies OCR, ASR and contextual synthesis;
selection of one does not select the other two. Pin and verify expected model and
dictionary bytes, runtime/codec versions, prompt/parser/schema, language, endpoint,
data exposure/retention and cost. OCR readiness must not depend on unrelated face
models. Expose skipped/truncated results; detector scores do not prove financial
correctness. Qualify JPEG/PNG and the actual selected non-DRM WAV/M4A profile;
HEIC needs separate decoder evidence or an explicitly preserved derivative relation.
Actual signing/review assurance, Temporal server/tools and finite worker limits
remain consumer-specific unqualified inputs until selected. Exact Rust 1.99.0
supersedes the earlier 1.98.0 baseline and stale execution-plan 1.97.1 entries without implying installation or
compatibility evidence. Public implementations/resources must be standalone and
rights-cleared; reference code is not automatically a dependency or licensed input.

A supported media profile identifies exact input formats, codecs, languages,
parser versions, model and prompt versions, endpoint policy, licence terms,
region, retained-data rules and cost controls. Select and qualify at least one
real OCR and transcription route. A recorded response is a deterministic test
fixture, not proof of live processing support.

The media gate must resolve corpus rights, readability labels, normalized scoring,
denominators and numeric ASR lexical and consequential-semantic thresholds before
claiming quality. Amount, date, negation and attribution errors require explicit
evaluation. Keep final holdouts independent of tuning. Unknown selections or
thresholds remain unresolved inputs; plausible values and successful individual
examples cannot fill them.

Users may explicitly select paid third-party inference under the
[product contract](product-and-scope.md). Before a real call, obtain the required
credentials, resource and spending authority, and enforce the data-exposure rules
in [processing](processing.md) and [security](security-and-privacy.md). A choice of
library or provider does not itself authorize a request. Default deterministic
checks use no commercial credentials.

## Dependency maintenance and evidence

For every release candidate, record exact package and image identities, selected
features, target architectures, generation tools, licences and notices, advisory
review, supported compatibility and actual test results. Inspect unused
dependencies and unnecessary runtime capabilities. A source licence or a
different edition's feature list is not proof of rights in external weights,
codecs, fixtures or redistributed artifacts.

Qualified limits belong to [resource profiles](resource-profiles.md); supported
operation belongs to [deployment](deployment.md). A failed required dependency
test blocks the affected support claim. A changed limit, feature or provider
requires an explicit reviewed profile amendment and new evidence; do not quietly
discard an included MVP capability or replace a mandatory experiment with a mock.

Coverage tools and service/process instrumentation must themselves be prepared
and qualified under [testing and coverage](testing-and-coverage.md). Direct
bootstrap verification must prove the same requirements before an orchestration
runner exists. Distribution checks follow [release policy](../release-policy.md),
including an independent verifier with no requirement to possess privileged
signing or deployment credentials.
