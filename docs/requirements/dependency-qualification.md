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
| Rust toolchain | Exactly `1.98.0`. Use an exact pin, not a floating channel or an implicit minor-line upgrade. |
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
| OIDC and service credentials | Qualify one concrete issuer/public-client contract, browser PKCE behavior, issuer/audience/signature checks, rotation and recovery, alongside local and scoped-service bootstrap. Device authorization is supported only when the chosen issuer path is independently tested. E02 and E08 apply. |
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
