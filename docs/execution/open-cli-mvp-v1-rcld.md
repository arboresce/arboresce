# Open engine MVP v1 rolling implementation plan

Status: executing approved implementation design; E001 is `verified` and awaiting its committed checkpoint identity; no implementation slice is active. All other slices remain `not_started`. This plan records work to perform and actual checkpoint evidence separately. It does not claim product implementation, measurements, release or publication has occurred.

This is the single governing rolling plan for the engine in this repository. It is independently usable with the public capabilities listed below. Keep one implementation slice active; a coordinated execution also keeps one active implementation slice across its selected repositories. Independent read-only review may run concurrently. Plan text never grants standing authority to commit, push, sign, deploy or spend.

## Baseline and authority

The observed starting revision is `48b9d6cbaf664ccceeaafc105e16d3227c841c07`. The repository contains documentation, instructions and licensing, without a product manifest or product implementation. Re-observe status before starting; preserve unrelated work and the independent Git history. Existing material remains `MIT OR Apache-2.0` with notices retained.

[Repository guidance](../../AGENTS.md), [documentation ownership](../ownership.md), [release policy](../release-policy.md) and [generated artifact guidance](../generated-artifacts.md) remain applicable. The first slice adopts the owning public requirements. Later contract slices freeze precise wire and domain details before dependent code. This execution plan does not silently amend an owning specification. Record intentional contract changes, compatibility impact and replacement relationships before implementing them.

A normal clone must build, test, document, package and run using committed public resources and explicitly documented external dependencies. No sibling checkout, undisclosed endpoint or paid Arboresce entitlement is required. Users may choose paid third-party inference providers through the published gateway contract; provider terms, credentials, cost and qualified limits remain explicit. A credential-free local model is not an implied supported profile.

All implementation destinations in the slice inventory are planned destinations. Some ancestor directories and documentation files already exist; a listed destination is not a claim that its product code or command currently exists. New files receive the nearest directory guidance. Read that guidance before creating them.

## Adopted specification bundle

E001 introduces public specifications under the prospective `docs/requirements/` boundary, indexed from the documentation index: product/scope, architecture, API, data model, financial records, processing, storage/search, security/privacy, resource profiles, testing/coverage, dependency qualification and acceptance/recovery. Contracts under `contracts/public/` and `contracts/internal/` own machine-readable wire details. The testing owner is prospective `docs/requirements/testing-and-coverage.md`, with validated machine contracts at `contracts/quality/coverage.toml`, `contracts/quality/coverage-profiles.toml` and `contracts/quality/source-inventory.json`. Existing release and generated-artifact documents retain their authority and receive deliberate compatible amendments.

Every operation family must freeze request/response variants, fields and units, preconditions, actor/purpose authorization, errors, state transitions, idempotency, concurrency, limits, pagination and positive/negative fixtures before its first dependent implementation. Generated indexes may derive from this plan but cannot become a second editable requirement authority. Every requirement maps to one owner, slice, oracle, verification lane, gate and evidence record; keep specified, implemented, verified and accepted states distinct.

Included scope covers capture and sealed originals; bounded image/audio/text processing; grounded observations; attributable review and asset history; exact expense recordkeeping and per-currency reports; canonical context builds, evaluations, signatures and channels; exact resolve and bounded search; version-linked feedback; revocation/holds/purge/export; deterministic recovery; independent CLI compatibility; supported native/container distributions and production qualification.

Deferred scope includes paid applications, broad integrations, language bindings, graph databases, universal PDF/video, multi-region writes, tracked context, custom model training, tax filing/certification and unsupported platforms. They require an explicit later specification and acceptance decision; none substitutes for completing the included scope.

## Universal green-state and fixture contract

Every source checkpoint must be a useful complete unit, with no advertised stub, `todo!`, `unimplemented!` or test-only red checkpoint. Inspect authority/status, implement the smallest coherent scope, run the applicable lane, repair a failure, independently review behavior and record evidence. Commit only under the active authorization. Source/contract changes and their verification are delivered together. Dependency preparation is explicit; tests never install tools or silently contact providers.

Coverage is strictly greater than 90%, using raw integer counts: `total > 0 && 10 * covered > 9 * total`. Exactly 90% fails. Gate executable lines separately for this repository/language, each Rust crate, each Python package and each executable source file; gate Rust regions for repository/language and each crate/binary, and Python branches for repository/language, each package and each applicable executable file. A proven non-executable or branchless denominator is documented as not applicable, never invented as 100%. No broad exclusions, averaged cross-repository success or percentage rounding waivers are allowed.

Inventory all first-party runtime libraries, binaries, adapters, build scripts, generators, tools, reusable fixture helpers and generated executable code, including declared generated outputs not discoverable solely from Git tracking. Assertion bodies cannot inflate production/support coverage. Missing/unimported modules, source/profile/lock/tool mismatch, empty suites, absent subprocess profiles and unexplained zero denominators fail. Combine observations only for identical sources, tools and qualified feature/native profiles. Report each supported platform/profile independently. Do not publish reports containing host paths, credentials or evidence content.

Use prepared `cargo-llvm-cov` for stable Rust line/region measurement, nextest for suitable test execution plus separate Cargo/doctests, property testing and actual executable-process assertions. Use pytest/coverage for Python line/branch measurement with explicit async/service-process startup and shutdown. Do not introduce a nightly requirement solely for these gates. E012 and E018 use direct collection before runners exist; E013-coverage-runner and E018-process-coverage then implement measured strict orchestration. Cross-language changes run both language lanes, including Python launched by Rust and the independently built CLI where applicable.

Fixtures have versioned scenarios, source rights, independent expected-value review, digests, fixed clocks/entropy and explicit service identity. Use disposable isolated resources, bounded readiness polling and synchronization barriers; arbitrary sleeps do not establish race ordering. Recorded providers fail on missing input and never fall back to live calls. Failed/denied/cancelled/expired/error paths are required regardless of percentage. Property tests complement independently fixed examples; output generated by the implementation cannot approve its own oracle.

The minimum semantic oracle inventory includes: exact 40.00 + 2.00 + 8.00 = 50.00 components; unknown tax remaining unknown with unresolved reconciliation; CAD/USD totals kept separate; forbidden accept replacement; required complete revise replacement; stale control conflict; succeeded-with-failure rejection; unavailable readiness; active channel requiring a build; revoked-and-held denying reads while preventing purge; revoked required snapshot source preventing partial serving; evidence-audience intersection; required failed attachment preventing readiness; interrupted wait causing neither cancellation nor success claim; reversed text range rejection; invalid JSON-pointer escape rejection. Extend these with required missing total/currency/date, refund/void, locale/date boundaries, negative amounts, shared search budgets, replay, concurrency and cross-language identity cases.

## Verification lanes

These are interfaces to implement and document, not commands claimed to exist at the baseline. Before the first source mutation, verify local preparation, tool versions and any applicable repository build-output routing. Early documentation uses bounded link/contract/disclosure review with exact commands recorded. Never invent an existing check.

| Lane | Required verification |
| --- | --- |
| DOC | Owning specification/compatibility review, local link and structural checks, exact inventory/dependency validation and independent public-content/rights review; no product pass claimed. |
| RUST | `cargo fmt --all -- --check`; `cargo check --locked --workspace --all-targets`; `cargo test --locked --workspace`; `cargo clippy --locked --workspace --all-targets -- -D warnings`; applicable raw coverage gate and explicit supported features/targets. |
| PYTHON | In `python/`: `uv run --locked ruff format --check .`; `uv run --locked ruff check .`; `uv run --locked pyright`; `uv run --locked pytest`; raw line/branch coverage and declared subprocess collection. |
| CONTRACT | RUST/PYTHON for changed executable code, schema/semantic/generation drift tests and source inventory. Before E013/E014, use explicitly recorded bounded manual or prepared standalone validation; no absent runner is required. E014 implements `cargo run --locked -p xtask -- contracts`; E015 implements `cargo run --locked -p xtask -- architecture-check`. Run those commands once implemented, including verification of the implementing slice. |
| SERVICE | Applicable RUST/PYTHON plus prospective `cargo run --locked -p xtask -- integration --suite <registered-name> --require-services`; pinned explicitly prepared disposable services, nonzero test count and actual integration coverage. |
| E2E | Applicable language lanes plus prospective `cargo run --locked -p xtask -- e2e --scenario <registered-name> --cli-binary <verified-artifact>`; independently built compatible CLI, actual API/services and process-level measurements. |
| IMAGE | Exact declared Docker Bake target from prospective `docker/docker-bake.hcl`, digest/config/license review, startup/shutdown/resource smoke on supported native profile; applicable language lanes for source changes. |
| RELEASE | Exact source/locks/toolchains/contracts/artifacts, full required gates and experiments, actual authorized artifact signing and independent verification, incident/recovery/support evidence. |

The prospective coverage interface is `cargo run --locked -p xtask -- coverage --profile <registered-profile> --require-complete`; profile declarations select actual prepared collection tools, source inventories, raw metrics and supported platform/features. E013-coverage-runner and E018-process-coverage implement and test that interface before dependent use. Direct-bootstrap collection must document exact installed-tool commands and prove the same raw-count rules. A future `just` entry point delegates to tested Rust xtask logic. Every required suite is registered when introduced; unknown or empty selection, missing services and missing evidence are failures. Use explicit supported feature sets rather than indiscriminate all-features. Default tests use no commercial credentials. Live qualification uses separately authorized resources and recorded limits; unavailable resources mean blocked, never passed. E019 delivery of local act orchestration is mandatory. Using that convenience wrapper is optional because the underlying direct commands remain supported; hosted GitHub Actions are outside this plan. Qualification needs an independently operated verifier using the same public checks, separated from privileged signing.

## Resource and quality acceptance

These are qualification targets, not measured performance or service promises. Initial bounds are: 1 MiB JSON commands with nesting/collection caps; 10 MiB text/Markdown/JSON originals; JPEG/PNG at 25 MiB encoded and 40 million decoded pixels; WAV or qualified non-DRM M4A at 20 MiB and five decoded minutes; eight attachments and 100 MiB per capture; 100 streamed manifest entries with two CLI transfers by default. Remote acquisition uses HTTPS, zero redirects, five-second connect and 120-second total deadlines with streamed byte limits. API work has a ten-second deadline and two-second ordinary transaction budget. Internal gRPC controls default to 1 MiB; bulk bytes use authorized blob references. Context builds cap 100 asset versions and 256 KiB compiled text.

Search uses k=1..20, two total retrieval rounds and 200 cumulative candidates before deduplication across hybrid sources. Canonical batched authorization precedes reranking; no denied counts, unbounded refill or N+1 authorization. Essential shared admission arrives with the first adapter: two model jobs and 100 pending jobs per organization initially, explicit global worker slots and cost reservations, bounded retries and all parser scratch/CPU/memory/time budgets. Consolidation later hardens the same controls. The initial live-profile reservation is $1 per capture as a cost ceiling to qualify, not a price promise or permission to spend.

D0 is two synthetic organizations and 100 scenarios. D1 is ten organizations, 100,000 captures, 200,000 artifact versions, 1,000,000 observations, 100,000 asset versions, 500,000 1,024-dimensional vectors and 30 GiB original/derived bytes. D2 is ten times D1 and deferred growth qualification. H0 is native macOS with 16 GiB host memory, Docker capped at four vCPU/eight GiB and 50 GiB free disk; hosted inference memory is separate. H1 uses two API instances at two vCPU/two GiB each, Python four/four, application PostgreSQL four/eight, Qdrant four/eight and Temporal two/two with a separate PostgreSQL two/four. Record actual versions, hardware, region, RTT, cold/warm state and costs. Emulation is not native capacity evidence.

Connection budgets explicitly sum maximum instances times per-role pool size plus administration, migration and reserve. No pool per tenant/request. Measure rebuild overlap, queue age, pool wait, model reservations/actual cost, parser memory, admission/denial, exact resolution, search exhaustion, projection lag, orphan bytes and restoration. Operational labels exclude content and unbounded identifiers.

| Experiment | Mandatory target and evidence |
| --- | --- |
| E01 | Exact native builds and 1,000 compatibility vectors; supported pairs work and unsupported pairs fail explicitly from independent public builds. |
| E02 | 64 clients/10,000 commands; zero tenant/session leakage; H1 pool-wait p95 <= 50 ms. Qualify each claimed production pooling profile separately. |
| E03 | 1,000 jobs/32 active Activities; controlled crashes at 25% of tested boundaries; no lost accepted commands or duplicate transitions; terminal state within ten minutes after recovery. |
| E04 | D1 at 100 requests/s, 64 clients, 30 minutes with 70% reads/20% commands/10% conflicts; p95 <= 300 ms, p99 <= 800 ms; exactly one same-revision winner. |
| E05 | 100 transfers of 20 MiB, 16 concurrent, 20% interrupted; no unsafe egress/unverified bytes; added API RSS < 512 MiB and parser RSS < 3 GiB. |
| E06 | D1/1,000 ground-truth queries at 20 queries/s and 32 clients; declared ACL selectivities, top 20, shared 200-candidate/two-round budget; zero leakage, recall >= 0.95, p95 <= 750 ms, p99 <= 2 s, Qdrant RSS < 7 GiB. |
| E07 | 100,000 reordered/duplicate events; canonical reconciliation and D1 rebuild <= 15 minutes with explicit scoped write pause. |
| E08 | 10,000 revoke/promote/download races and 1,000 credential-profile cases; no post-commit new admissions/quota bypass; denial p99 <= 1 s. |
| E09 | D1 restoration plus 500 later revocations/deletions; RPO <= 15 minutes, RTO <= 60 minutes; no resurrected serving; required residency qualified. |
| E10 | 10,000 canonicalization cases, 1,000 invalid signatures and 250 financial cases; exact cross-language agreement and safe rejection. |
| E11 | 300 held-out receipts and 100 voice notes of 30–300 s; readable total-and-currency accuracy >= 98%, with abstention/coverage separately reported; image p95 <= 60 s and audio p95 <= twice duration + 30 s; approved cost ceiling. |
| E12 | Three independent users, target task completion within 30 minutes after prerequisites; 72-hour soak at 25% of E04 with bursts; 100 agent/human tasks; idle memory growth <= 10%; independently usable exports. |

Before media implementation qualification, approve target languages/codecs, corpus rights, readability labels, normalized scoring, denominators and numerical ASR lexical and consequential semantic thresholds (negation, amount, date, attribution). Do not invent missing values. Readable-case abstention counts as unsuccessful for the accuracy denominator and is also reported separately. Separate development corpora from untouched final holdout; tuning against a holdout invalidates its independence. Recorded responses cannot satisfy live quality. Real provider runs require approved data exposure and spending authority.

## Capability gates and external prerequisites

Public capabilities are versioned evidence requirements, not references to another checkout. Record an immutable CLI artifact/digest and compatibility version wherever CLI behavior is required. External blocking evidence does not prevent unrelated dependency-ready synthetic work. Within ready work, prefer the lowest numeric slice; preserve semantic prerequisites and explicit suffix ordering. No implicit previous-number dependency replaces the listed dependencies.

| Capability | Required evidence |
| --- | --- |
| X-ENVIRONMENT | Approved concrete dependency/profile selections, prepared tooling and disposable environments. Actual credentials, spend and deployment policy are supplied only to operations that need them. |
| X-CLI-IDENTITY | Compatible native CLI supports scoped service identity, pinned-issuer PKCE login, safe profile storage, offline logout, cleanup failure reporting and refresh/logout fencing. |
| X-CLI-CAPTURE | CLI supports bounded local/HTTPS acquisition, secure destination/origin handling, interruption and resume. Human paths are explicit; autonomous agents initially use stdin or trusted granted descriptors unless an actual external confinement boundary is qualified. |
| X-MEDIA-PROFILE | One concrete OCR/ASR profile with approved languages/codecs, gateway route, corpus/oracle, numeric ASR thresholds, regions/terms and spend controls. Paid user-selected providers are permitted; no Arboresce entitlement required. |
| X-CLI-PROCESSING | CLI shows accepted versus completed processing, typed state, warnings, evidence and cancellation semantics. |
| X-CLI-REVIEW | CLI renders attributable review, complete safe editor replacement and revision-safe verdicts without assuming human status from a flag. |
| X-CLI-REPORT | CLI provides financial correction/report/export with atomic safe output and exact currency semantics. |
| X-CONSUMER-RESOLVE | Provider-neutral public consumer resolves the exact eligible build and records version/receipt; initial example does not require feedback. |
| X-CONSUMER-FEEDBACK | Extended public consumer submits feedback after feedback API/CLI exist, retaining exact consumed version and retry intent without claiming human approval or processing success. |
| X-CLI-FEEDBACK | CLI supports exact-subject feedback/outcomes and explicit reviewed supersession. |
| X-CLI-EXIT | CLI supports control-revision revocation, export, safe downloads and clean exit. |
| X-CLI-SEARCH | CLI supports bounded filters/search and honest selected-context receipts. |
| X-CLI-DOCUMENTATION | Supported human/agent/noninteractive journeys, native install/uninstall and credential cleanup are documented and verified independently. |
| X-IDENTITY-ARITHMETIC | Experiment E10 passes against exact code/contracts and independent oracles. |
| X-DEPLOYMENT-POLICY | Actual identity/provider/retention/residency/hold/recovery policies are approved for the selected environment before private evidence or real purge. |
| X-PRODUCTION-EVIDENCE | Independently measured mandatory experiments, supported environment, incident/recovery ownership, support limits and exact native CLI compatibility. |
| X-ARTIFACT-SIGNING | Actual authorized signing binds source, locks, manifest, digest, length, architecture and trusted signer; trust rotation/revocation and independent verification of distributed bytes pass. No unsigned candidate is called released. |

| Gate | Required closure |
| --- | --- |
| G0 | Current independent source state and approved public requirements/rights are recorded without contradictory active authority. |
| G1 | Exact toolchains, independent builds, complete contracts, compatibility, negative fixtures and applicable coverage gates pass. |
| G2 | Packaged actual identity, current authorization, tenant policy, sealed upload, acquisition/media limits, shared quotas and audit pass before private evidence. Synthetic capture/processing and approved non-sensitive provider tests do not satisfy this gate. |
| G3 | Real CLI record/report and evidence/review/build/evaluate/promote/resolve/feedback journeys pass; accepted submission never substitutes for completion. |
| G4 | Deterministic real-service races, failures/replay, revocation, export, D0 restoration and projection rebuild pass. Recovery requires current independently retained post-backup revocation/deletion/hold journal; incomplete proof keeps serving closed. |
| G5 | Independently scored live holdout, independent users and required quality/capacity/soak experiments pass. |
| G6 | All required experiments including full D1 E09, exact production profile, independent verifier, signed native distributions/images and operational obligations pass. Content attestations do not prove artifact signing. |

The packaged local profile plus G2 and D0 restore evidence precede private-user qualification. An early D0 restore must recover canonical PostgreSQL, sealed original/derived bytes, keys/trust and current independently retained lifecycle journal, then rebuild derived stores. A journal recovered only from the old backup or the maximum allocated outbox ID is insufficient. Full E09 remains mandatory later.

## Slice inventory

Each slice records its current status below. The named tests are mandatory additions to its verification lane and universal green-state contract. Planned paths are bounded owner destinations; inspect actual layout before creating files and record any justified path amendment. Record actual command, selected test count, exits, source/lock/profile identity, coverage numerators/denominators, review and remaining evidence after execution.

### E001 — Adopt the public implementation boundary

Status: `verified`. Prerequisites: approved public scope and refreshed repository status. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Publish current-state and product-scope requirements, the architecture and testing policy, supported interfaces and deferred scope. Preserve notices and existing files; describe only available commands as available.

Planned paths: `README.md`, `AGENTS.md`, `docs/README.md`, `docs/requirements/`,
and `docs/execution/e001-requirements-evidence.md` for actual owning checkpoint
verification. The requirements index names the single owner of each topic;
this plan records execution without becoming a competing specification.

Definition of green: Independent rights and disclosure review; local links resolve; every requirement has one owner; prospective implementation is clearly labelled.

### E002 — Freeze command and error semantics

Status: `not_started`. Prerequisites: E001. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Specify tenant and domain identifiers, independent revision types, command identity, typed errors, accepted operations and readiness. Define strict command parsing, extensible responses and cross-field validation.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Malformed commands fail; extra response fields remain tolerable; success cannot carry a failure code; unavailable readiness returns 503.

### E003 — Freeze acquisition and sealed-byte contracts

Status: `not_started`. Prerequisites: E002. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Specify upload lifecycle, byte digests/lengths, capture revisions and ordered attachment roles. Separate immutable originals and representations from readiness and control state.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Foreign attachment rejected; required failure blocks readiness; optional failure is visible; expired upload cannot finalize.

### E004 — Freeze grounded observation contracts

Status: `not_started`. Prerequisites: E003. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Define half-open UTF-8 ranges, RFC6901 pointers, page coordinates and audio units bound to exact representations. Preserve assertion, observation and inference distinctions.

Planned paths: `contracts/public/`, `contracts/internal/`, `docs/requirements/`.

Definition of green: Invalid boundaries/escapes/spans rejected; representation mismatches fail; absent grounding stays explicitly absent.

### E005 — Freeze candidate review variants

Status: `not_started`. Prerequisites: E004. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Specify complete candidate identity and digest; only revise accepts a complete replacement and produces a new pending revision. Acceptance pins the reviewed content and current authority.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Replacement on accept fails; incomplete revise fails; scope-only changes alter identity; stale review conflicts.

### E006 — Freeze expense and report semantics

Status: `not_started`. Prerequisites: E003. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Require reviewed total, currency and date before confirmation. Preserve unknown optional components, reconciliation and date precision. Refunds/credits carry positive magnitudes, explicit kind and original-record links; reports pin exact eligible revisions.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Missing required fields remain proposed; unknown tax differs from zero; no cross-currency total; refunds preserve purchase history; authorized exclusions remain explicit.

### E007 — Freeze context and attestation contracts

Status: `not_started`. Prerequisites: E005. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Specify canonical build preimage, ordering, numeric profile and exact content references; keep receipts outside content identity. Define domain-separated signatures, trusted keys and channel generations.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: No self-hash; unknown/revoked keys denied; partial snapshots forbidden; revoked inputs cannot become eligible through rollback.

### E008 — Freeze lifecycle and operation contracts

Status: `not_started`. Prerequisites: E007, E006. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Define revoke, hold, purge, export availability, cancellation and feedback attribution; distinguish current authorization, immutable history and idempotency expiry.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Hold grants no read; cancellation may lose to completion; purged bytes remain unavailable; expired request identity cannot silently repeat a financial action.

### E009 — Freeze bounded selection and listing

Status: `not_started`. Prerequisites: E005. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Specify filter grammar, authenticated cursors, stable ordering and independent selection receipts. Search permits k=1..20, two total retrieval rounds and 200 cumulative pre-dedup candidates across all sources.

Planned paths: `contracts/public/`, `docs/requirements/`.

Definition of green: Changed filter/purpose invalidates cursor; unknown filters fail; inaccessible counts stay hidden; budget exhaustion is explicit and cannot trigger unbounded refill.

### E010 — Freeze intelligence transport

Status: `not_started`. Prerequisites: E004, E008. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Version gRPC input, stage and processing-profile identity, immutable blob references, typed outputs, locators, uncertainty, exposure and cost receipts. Keep credentials outside messages.

Planned paths: `contracts/internal/proto/`, `docs/requirements/`.

Definition of green: Oversize messages fail; incompatible versions fail explicitly; malformed locators fail; no output can approve canonical knowledge.

### E011 — Approve deterministic semantic oracles

Status: `not_started`. Prerequisites: E006, E007, E010, E009. Verify lane: DOC + CONTRACT (executable validators use applicable language lanes).

Scope: Create D0 scenario inventory and independently review expected financial, review, control, capture, policy and context outcomes. Register fixture rights, provenance, digests, deterministic seeds and mutation controls. Freeze live-media scoring criteria separately.

Planned paths: `tests/fixtures/scenarios/`, `docs/requirements/`.

Definition of green: Validate every fixture reference; contradictory expectations fail; minimum scenario inventory below is complete; affected acceptance stays blocked until independent oracle approval exists.

### E012 — Bootstrap a measured Rust workspace

Status: `not_started`. Prerequisites: E011. Verify lane: RUST + CONTRACT.

Scope: Pin Rust 1.97.1, edition 2024 and resolver 3 with reviewed dependency locks and central lints. Introduce a useful opaque identifier type; use prepared coverage tools directly before a custom runner exists.

Planned paths: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `crates/arboresce-domain/`.

Definition of green: Equality, invalid formats and roundtrips pass; every target compiles; raw line/region gates pass including executable support/build code; no placeholder members.

### E013 — Implement the verification dispatcher

Status: `not_started`. Prerequisites: E012. Verify lane: RUST + CONTRACT.

Scope: Add a real xtask workspace package and thin just recipes. Dispatch existing formatting/check/test/lint commands with exact exit propagation and measured execution counts; unsupported operations fail.

Planned paths: `tools/xtask/`, `justfile`, `.cargo/config.toml`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Child failures propagate; empty required selection fails; setup is explicit; no hidden installation; dispatcher source itself meets coverage gates.

### E013-coverage-runner — Enforce measured source checkpoints

Status: `not_started`. Prerequisites: E013. Verify lane: RUST + CONTRACT.

Scope: Implement tested source/package/file inventory and strict Rust coverage collection/aggregation with direct-bootstrap equivalence; prepare the Python inventory contract without claiming Python service collection. Reject incomplete generated/support inventories, stale identities, missing native/subprocess profiles, empty suites and rounded 90% success. Keep the runner and gate implementation inside their own coverage denominator.

Planned paths: `tools/xtask/`, `contracts/quality/`, `tests/contract/coverage/`, `docs/requirements/testing-and-coverage.md`.

Definition of green: boundary fixtures at 0, exactly 90%, just above 90%, missing source, wrong lock/profile, branchless/non-executable justified N/A and lost subprocess profile all produce the specified outcome; useful bootstrap code and runner satisfy gates with independent fixtures.

### E014 — Verify contracts and generated code

Status: `not_started`. Prerequisites: E013, E013-coverage-runner. Verify lane: RUST + CONTRACT.

Scope: Integrate pinned local schema and Protobuf tools, semantic fixtures and generation locks. Commit required generated output; check drift without fetching schemas from the network.

Planned paths: `tools/xtask/`, `contracts/`, `tests/contract/`.

Definition of green: Unknown references fail; positive/negative vectors run; breaking changes require explicit versions; regeneration is reproducible; generated executable code is included in coverage.

### E015 — Enforce architecture and dependency policy

Status: `not_started`. Prerequisites: E013, E013-coverage-runner. Verify lane: RUST + CONTRACT.

Scope: Check inward domain/application dependencies, the workflow API matrix, license/advisory policy and source inventories. Workflows may use deterministic Temporal APIs; direct I/O, ambient time/randomness and adapter dependencies are forbidden.

Planned paths: `tools/xtask/`, `deny.toml`, `crates/README.md`, `tests/contract/`.

Definition of green: Allow supported workflow APIs and legitimate SDK transitives; reject direct domain SQL and workflow network edges; unsafe paths and unknown checks fail; scanner limits are documented.

### E016 — Implement deterministic shared test support

Status: `not_started`. Prerequisites: E013, E013-coverage-runner. Verify lane: RUST + CONTRACT.

Scope: Add narrow clock, identity, fixture, barrier and disposable-service helpers without production bypasses. Inventory executable support and test it independently.

Planned paths: `crates/arboresce-testkit/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Repeated seeds produce identical values; barriers replace timing guesses; isolation prevents cross-test state; deterministic mode denies unexpected network; helper code exceeds coverage gates.

### E017 — Generate isolated intelligence DTO packages

Status: `not_started`. Prerequisites: E014, E016. Verify lane: RUST + CONTRACT.

Scope: Add a non-published transport package from locked Protobuf and preserve domain independence. Commit generated runtime DTOs and their source/version contract.

Planned paths: `crates/arboresce-intelligence-proto/`, `contracts/internal/`, `tools/xtask/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Roundtrip vectors pass; removed tags remain reserved; breaking changes detected; domain has no transport dependency; generated source is measured.

### E018 — Bootstrap measured Python processing

Status: `not_started`. Prerequisites: E017. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Use an isolated src-layout package, pinned Python/uv dependencies, strict lint/type/test configuration and useful validated configuration/health behavior. Measure directly before service-process collection exists.

Planned paths: `python/pyproject.toml`, `python/uv.lock`, `python/.python-version`, `python/src/arboresce_intelligence/`, `python/tests/`.

Definition of green: Unknown or unbounded configuration fails; imports work outside source directory; all source modules are measured even if never imported; package and file line/branch gates pass.

### E018-process-coverage — Collect cross-process Python execution

Status: `not_started`. Prerequisites: E018, E013-coverage-runner. Verify lane: RUST + PYTHON + CONTRACT.

Scope: Implement Python package/file line and branch inventory with explicit child/async process coverage lifecycle, bounded flush/merge and exact source/profile matching. Prove isolated launched-process collection now and extend to actual gRPC, Activities and E2E processes when introduced. No invented coverage for processes not yet implemented.

Planned paths: `tools/xtask/`, `python/tests/`, `python/pyproject.toml`, `contracts/quality/`.

Definition of green: unimported module, abrupt child exit, missing profile, mismatched source and stale report fail; deterministic spawned/async work is measured; both language lanes and all existing source gates pass.

### E019 — Add local orchestration for established checks

Status: `not_started`. Prerequisites: E018, E015, E018-process-coverage. Verify lane: DOC + CONTRACT.

Scope: Place optional local act definitions outside hosted workflow discovery, invoking the same public commands. Pin image and explicit architecture; preserve direct-command use.

Planned paths: `tools/act/`, `.actrc`, `docs/contributing.md`.

Definition of green: Workflow parsing passes; direct and orchestrated commands match; unsupported runner configuration fails; absent tools report preparation requirements.

### E020 — Implement tenant and principal identities

Status: `not_started`. Prerequisites: E016. Verify lane: RUST + CONTRACT.

Scope: Add validated organization/domain/principal identifiers, human/service distinctions and active membership. Personal organizations use the same rules.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Cross-organization references fail; email is never identity authority; disabled membership cannot authorize.

### E021 — Implement independent revision types

Status: `not_started`. Prerequisites: E020. Verify lane: RUST + CONTRACT.

Scope: Separate content revision, control revision and channel generation with validated comparisons and increments.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Overflow rejected; timestamps confer no concurrency authority; stale control revision conflicts.

### E022 — Implement source date and event-time values

Status: `not_started`. Prerequisites: E021. Verify lane: RUST + CONTRACT.

Scope: Preserve source precision and zone independently from injected server recording time.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Date-only values stay dates; ambiguous instants require a zone; canonical UTC encoding is stable.

### E023 — Implement evidence locator values

Status: `not_started`. Prerequisites: E022. Verify lane: RUST + CONTRACT.

Scope: Validate each locator against an exact representation and its bounds, retaining typed units.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Invalid UTF-8 boundaries, pointer escapes, regions and audio ranges fail; digest substitution fails.

### E024 — Implement capture readiness

Status: `not_started`. Prerequisites: E023. Verify lane: RUST + CONTRACT.

Scope: Represent fixed ordered submitted attachments, required/optional roles and amendment through a new capture revision.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Foreign attachments fail; required failure blocks; optional failure remains visible; no silent attachment removal.

### E025 — Implement sealing state transitions

Status: `not_started`. Prerequisites: E024. Verify lane: RUST + CONTRACT.

Scope: Model upload creation through sealing plus expiry, abortion and rejection. An accepted sealed reference is immutable.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Retries cannot rewind state; expired sessions cannot finalize; sealed byte identity cannot change.

### E026 — Implement complete candidate identity

Status: `not_started`. Prerequisites: E025. Verify lane: RUST + CONTRACT.

Scope: Represent statement, scope, assumptions, supporting/refuting/qualifying evidence and invalidation conditions with explicit revision identity.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Scope-only change changes digest; evidence set ordering follows contract; model provenance cannot constitute approval.

### E027 — Implement review transition invariants

Status: `not_started`. Prerequisites: E026. Verify lane: RUST + CONTRACT.

Scope: Model accept/reject/contest/revise as distinct operations. Revision creates pending content and acceptance pins exact content/revision.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Accept cannot replace content; stale revisions fail; contested history is preserved; unique accepted mapping is represented.

### E028 — Implement asset content and availability

Status: `not_started`. Prerequisites: E027. Verify lane: RUST + CONTRACT.

Scope: Separate immutable reviewed asset versions from active, deprecated, revoked and purged controls.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Control changes preserve content identity; purged bytes are unavailable; no asset exists without its valid review mapping.

### E029 — Implement exact monetary values

Status: `not_started`. Prerequisites: E028. Verify lane: RUST + CONTRACT.

Scope: Parse bounded decimal strings and currency scales without floating point; preserve component uncertainty.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Scale/overflow failures are explicit; unknown is distinct from zero; positive magnitude and explicit kind rules are enforced.

### E030 — Implement expense state and refunds

Status: `not_started`. Prerequisites: E029. Verify lane: RUST + CONTRACT.

Scope: Model proposed, confirmed, superseded and void revisions plus linked refund/credit records. Enforce required-field and reconciliation policy.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Unresolved total/currency/date cannot confirm; optional unknowns remain unknown; void/refund retain original history and exact arithmetic.

### E031 — Implement deterministic report arithmetic

Status: `not_started`. Prerequisites: E030. Verify lane: RUST + CONTRACT.

Scope: Calculate exact eligible revision selections, per-currency totals, known components and authorized exclusions with immutable identity.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: No mixed-currency sum; proposals stay outside default totals; missing components are not guessed; input ordering does not alter set results.

### E032 — Implement durable operation values

Status: `not_started`. Prerequisites: E031. Verify lane: RUST + CONTRACT.

Scope: Separate command, operation, attempt and accepted-result identities with explicit cancellation races.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Terminal invariants hold; success cannot include failure; detached wait does not cancel work; completed effects remain reported.

### E033 — Implement build and channel value types

Status: `not_started`. Prerequisites: E032. Verify lane: RUST + CONTRACT.

Scope: Represent exact build content, external receipts and explicit availability; channels have monotonic generations and empty/active/suspended states.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Active requires a build; empty may omit one; receipts never enter content digest; counters cannot be recycled.

### E034 — Implement feedback and lifecycle subjects

Status: `not_started`. Prerequisites: E033. Verify lane: RUST + CONTRACT.

Scope: Bind feedback/outcome assertions to exact subject versions and actor kinds; distinguish hold, revoke and purge.

Planned paths: `crates/arboresce-domain/src/`, `crates/arboresce-domain/tests/`.

Definition of green: Feedback cannot approve; outcome assertions are not independent proof; hold cannot broaden access.

### E035 — Qualify canonical content encoding

Status: `not_started`. Prerequisites: E033, E014. Verify lane: RUST + CONTRACT.

Scope: Use vetted JCS with explicit numeric/Unicode restrictions and set ordering. Reject duplicate keys and unsupported values.

Planned paths: `crates/arboresce-domain/`, `crates/arboresce-contracts/`, `tests/contract/canonicalization/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: RFC and cross-language vectors agree; UTF-16 key ordering is correct; duplicates fail; receipts and self-hash are excluded.

### E036 — Implement content attestation signatures

Status: `not_started`. Prerequisites: E035. Verify lane: RUST + CONTRACT.

Scope: Use vetted Ed25519 over the specified domain-separated envelope with injected trust metadata; keep key I/O outside domain code.

Planned paths: `crates/arboresce-domain/src/attestation.rs`, `tests/contract/signing/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Wrong domain/build/issuer, malformed/expired/revoked/unknown keys fail; rotation overlap works; signature never bypasses current policy.

### E037 — Define focused application ports

Status: `not_started`. Prerequisites: E034, E036. Verify lane: RUST + CONTRACT.

Scope: Introduce transaction results and focused persistence, policy, blob, projection, intelligence, signer, clock and identity ports. Keep transport/infrastructure outside application logic.

Planned paths: `crates/arboresce-application/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: In-memory conformance tests cover transaction outcomes and cancellation; dependency direction remains inward; no gratuitous helper interfaces.

### E038 — Implement PostgreSQL migration harness

Status: `not_started`. Prerequisites: E037. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add SQLx adapter, ordered migrations, direct versus pooled configuration and committed offline metadata. Prepare pinned disposable database/pooler/storage fixtures explicitly.

Planned paths: `crates/arboresce-postgres/`, `tools/xtask/`, `Cargo.toml`, `Cargo.lock`, `docker/compose/core.yaml`, `docs/testing.md`.

Definition of green: Empty and upgrade migrations pass; invalid roles fail; offline compilation works; tests cannot select a production database; real integration coverage is collected.

### E039 — Persist tenant and membership identities

Status: `not_started`. Prerequisites: E038. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use composite tenant keys, canonical issuer/subject identity and serialized initial-owner creation.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/identity/`.

Definition of green: Duplicate identity and foreign-tenant references fail; simultaneous bootstrap produces one owner.

### E040 — Enforce application roles and RLS

Status: `not_started`. Prerequisites: E039. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use non-owner application roles, forced RLS and transaction-local tenant context with uniform inaccessible-object behavior.

Planned paths: `crates/arboresce-postgres/roles/`, `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/tests/security/`.

Definition of green: No BYPASSRLS; missing context fails; pooled reuse cannot leak tenant/session state; referential errors do not expose foreign data.

### E041 — Implement canonical admission locking

Status: `not_started`. Prerequisites: E040. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Compose current permission and evidence audience intersections using short transactions, policy epochs and consistent lock order. Keep streaming/model calls outside locks.

Planned paths: `crates/arboresce-application/src/policy/`, `crates/arboresce-postgres/src/policy/`, `crates/arboresce-postgres/tests/concurrency/`.

Definition of green: Barrier-driven revoke/read tests prove admission point; bounded deadlock retry; stale caches cannot grant access; source audiences intersect.

### E042 — Persist idempotent command results

Status: `not_started`. Prerequisites: E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Scope key fingerprint, expiry and tombstone state; atomically claim work and replay results only after current authorization.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/commands/`.

Definition of green: Same key/body returns original result; changed body conflicts; lost-response retry is safe; expiry never silently repeats financial effects.

### E043 — Persist atomic audit and outbox

Status: `not_started`. Prerequisites: E042. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Write command/domain/audit/outbox effects together with bounded metadata and stable event identity.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/outbox/`, `crates/arboresce-postgres/src/audit/`.

Definition of green: Rollback leaves none; replay cannot duplicate effects; event allocation is not treated as commit order; audit avoids unrestricted content.

### E044 — Implement fenced outbox leasing

Status: `not_started`. Prerequisites: E043. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Lease bounded due batches, reclaim expired ownership with fencing and track each consumer independently.

Planned paths: `crates/arboresce-postgres/src/outbox/`, `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/tests/concurrency/`.

Definition of green: Concurrent dispatchers cannot share a live lease; stale owner cannot finish; retries/dead letters are bounded; maximum event ID is not completion proof.

### E045 — Persist captures and sealed references

Status: `not_started`. Prerequisites: E043. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Store uploads, captures, attachment roles, artifact versions and sealed byte identities with tenant-safe foreign keys and separate controls.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/artifacts/`.

Definition of green: One accepted finalization; immutable submitted set; foreign references fail; unreferenced objects never become canonical automatically.

### E046 — Persist processing representations and results

Status: `not_started`. Prerequisites: E045. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Store stage, operation, representation and observation identity by exact input/profile, including typed locators and accepted-result fencing.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/processing/`.

Definition of green: Retries yield one canonical result; stale attempts fail; source revision is mandatory; processing cannot persist approval.

### E047 — Persist candidates reviews and assets

Status: `not_started`. Prerequisites: E046. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Store immutable revisions and reviews, current heads, unique accepted mappings and reverse evidence dependencies.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/knowledge/`.

Definition of green: Concurrent accepts create one mapping; stale acceptance conflicts; reverse dependencies remain tenant-scoped.

### E048 — Persist expenses and report selections

Status: `not_started`. Prerequisites: E045, E031. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use exact numeric storage, immutable financial revisions and reports, linked refunds and workload indexes.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/expenses/`.

Definition of green: Scale/overflow guards hold; report pins exact revisions; void/refund preserve history; inaccessible exclusions/counts never leak.

### E049 — Persist builds attestations and channels

Status: `not_started`. Prerequisites: E047, E033. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Separate immutable bytes/input edges from execution/approval receipts and mutable channel generations.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/releases/`.

Definition of green: Stale promotion conflicts; reactivation is explicit; receipts preserve content hash; generations are never reused.

### E050 — Persist lifecycle obligations and journal

Status: `not_started`. Prerequisites: E049, E048. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Store feedback, retention/holds, revocation/purge obligations and deletion-aware recovery records. Design independent journal retention and completeness tracking for later recovery.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/src/lifecycle/`.

Definition of green: Hold blocks physical purge without restoring reads; minimal tombstones persist; feedback does not rewrite content; journal gaps are detectable.

### E051 — Add evidence-driven database indexes

Status: `not_started`. Prerequisites: E050. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Build composite, partial and reverse indexes from required access patterns and deterministic representative D1 data.

Planned paths: `crates/arboresce-postgres/migrations/`, `crates/arboresce-postgres/tests/query_plans/`, `tests/fixtures/`.

Definition of green: Keyset review queue, reverse revocation and due-work queries use bounded plans; representative selectivities are measured; no blanket JSON index policy.

### E052 — Qualify local pooling and tenant isolation

Status: `not_started`. Prerequisites: E051. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Prepare a pinned PostgreSQL/PgBouncer topology with explicit connection allocation and pooled/direct settings. Exercise the local pooling experiment.

Planned paths: `docker/compose/`, `tools/xtask/`, `tests/integration/postgres/`.

Definition of green: 64 clients and 10,000 commands show no tenant/session leakage; prepared-statement compatibility recorded; H1 pool-wait target measured separately.

### E053 — Compose bounded API health behavior

Status: `not_started`. Prerequisites: E041, E037. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add validated configuration, bounded parsing and uniform Problem mapping with liveness/readiness and implemented-only capabilities. Install initial JSON/request deadlines here.

Planned paths: `crates/arboresce-api/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Readiness distinguishes 200/503; unknown routes return 404; health reveals no secrets; oversize/deep inputs fail before expensive processing.

### E054 — Implement first-owner provisioning

Status: `not_started`. Prerequisites: E039, E042. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add a thin administrative binary and scoped bootstrap/recovery use cases requiring actual operator possession; ship no default password.

Planned paths: `crates/arboresce-admin/`, `crates/arboresce-application/src/identity/`, `crates/arboresce-postgres/src/identity/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: One concurrent owner wins; protected credential delivery; existing identity is not overwritten; replacement revokes superseded credentials.

### E055 — Implement scoped service authentication

Status: `not_started`. Prerequisites: E054, E053. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Store suitable credential verifiers, explicit scopes and expiry/revocation; enforce actor kind at the server without commercial entitlement.

Planned paths: `crates/arboresce-application/src/auth/`, `crates/arboresce-postgres/src/auth/`, `crates/arboresce-api/src/auth/`.

Definition of green: Expired/revoked/foreign tokens fail; service credentials cannot claim human approval; logs never expose credentials/verifiers.

### E056 — Implement pinned-issuer OIDC validation

Status: `not_started`. Prerequisites: E053, E039. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Validate issuer, signature, audience, time and canonical subject using bounded JWKS caching and approved issuer configuration.

Planned paths: `crates/arboresce-oidc/`, `crates/arboresce-api/src/auth/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Wrong issuer/audience/claim fails; rotation works; tokens cannot choose key-fetch URLs; issuer limits and cache expiry are tested.

### E057 — Expose identity and membership operations

Status: `not_started`. Prerequisites: E055, E056, E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Implement authorized organization/domain discovery, ownership and membership changes with last-owner recovery controls.

Planned paths: `crates/arboresce-api/src/routes/`, `crates/arboresce-application/src/identity/`.

Definition of green: No service self-promotion; inactive owner denied; resources are hidden uniformly; concurrent changes conflict.

### E058 — Compose command admission and errors

Status: `not_started`. Prerequisites: E057, E042. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Wire fingerprints, current admission, idempotency and typed status handling. Install shared atomic rate/backlog reservations with bounded retry guidance at first use.

Planned paths: `crates/arboresce-api/src/`, `crates/arboresce-application/src/commands/`.

Definition of green: 400/401/404/409/422/429 are distinct; unknown state fails safely; parallel instances cannot bypass reservations; rejected work is not falsely accepted.

### E059 — Verify independent CLI identity behavior

Status: `not_started`. Prerequisites: X-CLI-IDENTITY, E058. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Run an independently built CLI artifact against the real API and controlled issuer/service identities; register a nonempty identity scenario with subprocess coverage.

Planned paths: `tests/e2e/identity/`, `tools/xtask/`, `docs/requirements/`.

Definition of green: Authentication challenges, organization isolation and capabilities agree; response extensions remain compatible; both service and CLI failures are observable.

### E060 — Implement streaming object storage

Status: `not_started`. Prerequisites: E037, E045. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add bounded streaming S3-compatible operations with separate staging/sealed roles, actual SHA-256 and scoped grants. Register real storage tests and their prepared fixture.

Planned paths: `crates/arboresce-blob-s3/`, `crates/arboresce-application/src/blob/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Unknown length stays bounded; digest mismatch fails; tenant substitution fails; ETag is not digest authority; signed URLs remain redacted.

### E061 — Implement bounded upload allocation

Status: `not_started`. Prerequisites: E060, E058. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Authorize staging-only sessions and allowlisted destinations/headers with shared quota, media and size admission.

Planned paths: `crates/arboresce-application/src/uploads/`, `crates/arboresce-api/src/routes/uploads.rs`.

Definition of green: Clients cannot write sealed prefixes; unsupported type/oversize/quota fail clearly; concurrent allocations cannot exceed reservations.

### E062 — Implement captures and attachment submission

Status: `not_started`. Prerequisites: E061. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Create drafts, ordered authorized attachments and immutable submitted sets with explicit readiness.

Planned paths: `crates/arboresce-application/src/captures/`, `crates/arboresce-api/src/routes/captures.rs`.

Definition of green: Required absence blocks; optional failure remains; retries create one capture; edits after submission create a revision.

### E063 — Implement fenced streaming sealing

Status: `not_started`. Prerequisites: E062. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Read staged content into a server-only object once, checking digest/length and resource caps, then bind in a short fenced transaction.

Planned paths: `crates/arboresce-application/src/uploads/`, `crates/arboresce-postgres/src/artifacts/`, `tests/integration/storage/`.

Definition of green: Concurrent overwrite cannot change accepted bytes; crashes on either side of binding recover; stale lease cannot replace reference; bounded cleanup respects ownership.

### E064 — Implement orphan reconciliation

Status: `not_started`. Prerequisites: E063, E050. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Track unreferenced staging/sealed objects with safety delays, active leases, shared-reference/hold checks, dry-run reporting and separately authorized deletion.

Planned paths: `crates/arboresce-application/src/reconciliation/`, `crates/arboresce-postgres/src/lifecycle/`, `tests/integration/storage/`.

Definition of green: Live finalizers and shared references survive; held bytes survive; wrong-tenant paths are untouched; interrupted cleanup is recoverable.

### E065 — Implement authorized bounded downloads

Status: `not_started`. Prerequisites: E063, E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Expose immutable content/current controls and proxy exact bytes after canonical admission with transfer deadlines and cancellation.

Planned paths: `crates/arboresce-api/src/routes/artifacts.rs`, `crates/arboresce-application/src/artifacts/`.

Definition of green: Foreign objects hidden; revoked new admission denied; digest/length verified; slow/cancelled clients release resources; index outage does not affect exact access.

### E066 — Verify capture and sealing through CLI

Status: `not_started`. Prerequisites: E065, X-CLI-CAPTURE. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Run real API/storage with a verified CLI, deterministic local inputs and controlled HTTPS endpoint; keep private evidence out of this stage.

Planned paths: `tests/e2e/capture/`, `tools/xtask/`, `docker/compose/`.

Definition of green: Complete receipt/note capture, missing parts, tenant substitution, overwrite and interrupted-transfer cases assert actual object bytes; nonempty scenario and subprocess coverage recorded.

### E067 — Qualify the Temporal dependency set

Status: `not_started`. Prerequisites: E012, X-ENVIRONMENT. Verify lane: DOC + RUST + CONTRACT.

Scope: Pin Rust SDK 1.0.0 and Core 0.9.0 with selected compatible server/admin/schema/UI/CLI versions. Verify test/replay APIs and disable unqualified downloads or experimental behavior.

Planned paths: `docs/architecture/`, `contracts/internal/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Exact graph and licenses recorded; no latest tags; production uses supported SDK surfaces; selected server family is qualified before supported claims.

### E068 — Package Temporal server and administration

Status: `not_started`. Prerequisites: E067. Verify lane: IMAGE + SERVICE; applicable language lanes for source changes.

Scope: Build owned images from reviewed rights-cleared releases; separate schema administration from server startup and keep credentials out of images.

Planned paths: `docker/images/temporal/`, `docker/docker-bake.hcl`, `docs/runbooks/temporal.md`.

Definition of green: Native target build/config smoke passes; no exposed administration by default; schema-version mismatch fails; dry-run administration is safe.

### E069 — Implement deterministic workflow boundaries

Status: `not_started`. Prerequisites: E067, E017, E037. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Create workflow and Activity packages using the accepted dependency matrix and application ports. A representative real workflow has deterministic fake Activities only in tests.

Planned paths: `crates/arboresce-workflows/`, `crates/arboresce-activities/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Allowed SDK calls pass architecture check; direct I/O/time/randomness fail; IDs derive from operations; pinned replay API compiles and runs.

### E070 — Compose bounded durable workers

Status: `not_started`. Prerequisites: E069. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Wire SDK runtime, validated configuration, poller/slot limits and graceful shutdown without moving policy into the binary.

Planned paths: `crates/arboresce-worker/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Unavailable runtime/server reports clearly; shutdown drains/recoverable work; task queue IDs reveal no user content; memory/slots stay bounded.

### E071 — Dispatch operations to stable workflows

Status: `not_started`. Prerequisites: E070, E044. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use operation-derived workflow identity with duplicate/conflict handling and separate fenced outbox receipts. Enforce admitted backlog/slots here and register real Temporal tests.

Planned paths: `crates/arboresce-activities/`, `crates/arboresce-worker/`, `crates/arboresce-postgres/src/outbox/`.

Definition of green: Crash after start before delivery receipt is safe; concurrent dispatch converges; terminal work is not restarted incorrectly; missing service fails qualification.

### E072 — Implement capture-analysis orchestration

Status: `not_started`. Prerequisites: E071, E066, E046. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Sequence sealed readiness, normalization and extraction through Activities using bounded references and accepted-result fencing.

Planned paths: `crates/arboresce-workflows/src/capture.rs`, `crates/arboresce-activities/src/capture.rs`.

Definition of green: Required failure blocks; optional results remain; retry yields one result; history contains neither blob bytes nor secrets.

### E073 — Expose operation status and cancellation

Status: `not_started`. Prerequisites: E072. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Serve typed state/result/progress, expiry and retry metadata under current permission; model cancellation-versus-effect races explicitly.

Planned paths: `crates/arboresce-api/src/routes/operations.rs`, `crates/arboresce-application/src/operations/`.

Definition of green: Success has no failure code; actual committed effects survive cancellation races; foreign status stays hidden; stale revision conflicts.

### E074 — Verify replay restarts and fencing

Status: `not_started`. Prerequisites: E072, E073, E068. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Commit deterministic synthetic histories for success, retry, cancellation and failure points, then exercise worker restarts and accepted-result fencing.

Planned paths: `tests/temporal-replay/`, `tests/integration/temporal/`, `tools/xtask/`.

Definition of green: Replay across code changes passes; duplicated completion cannot duplicate effects; unknown variants fail safely; no live provider is needed.

### E075 — Implement Python gRPC service validation

Status: `not_started`. Prerequisites: E018, E010, E018-process-coverage. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Generate locked bindings and strict request/profile/response validation with deadline propagation; Python has no canonical database credentials.

Planned paths: `python/src/arboresce_intelligence/server.py`, `python/src/arboresce_intelligence/_generated/`, `python/tests/contract/`.

Definition of green: Malformed/oversize/incompatible messages fail; locators are validated; service-process coverage includes async and generated execution.

### E076 — Implement deterministic recorded processing

Status: `not_started`. Prerequisites: E075, E011. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Resolve synthetic responses by canonical request fingerprint and profile digest with explicit fixture selection and no live fallback.

Planned paths: `python/src/arboresce_intelligence/providers/recorded/`, `python/tests/golden/`, `tests/fixtures/scenarios/`.

Definition of green: Repeated input is identical; missing recording/profile mismatch fails; unexpected network is denied; fixture identity is recorded.

### E077 — Implement text and structured normalization

Status: `not_started`. Prerequisites: E076. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Create new text/Markdown/JSON representations with exact original digests, UTF-8 offsets and JSON pointer semantics.

Planned paths: `python/src/arboresce_intelligence/pipelines/normalization/`, `python/tests/unit/`.

Definition of green: Invalid encoding/duplicate keys fail; newline normalization changes representation identity; no fabricated offsets.

### E078 — Implement bounded image decoding

Status: `not_started`. Prerequisites: E077. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Qualify JPEG/PNG decoder and isolate processing under encoded/pixel/CPU/memory/time/scratch budgets with correct orientation mapping.

Planned paths: `python/src/arboresce_intelligence/pipelines/images/`, `python/tests/unit/`, `python/pyproject.toml`, `python/uv.lock`.

Definition of green: Bombs, truncation and unsupported animation fail; orientation locators remain valid; metadata is minimized; timeouts reclaim resources.

### E079 — Implement bounded audio normalization

Status: `not_started`. Prerequisites: E077. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Qualify WAV and supported non-DRM M4A codecs, channels and duration; isolate subprocesses and create immutable normalized audio/time mappings.

Planned paths: `python/src/arboresce_intelligence/pipelines/audio/`, `python/tests/unit/`, `python/pyproject.toml`, `python/uv.lock`.

Definition of green: Unsupported/truncated media and duration overflow fail; no shell interpolation; channel/time mapping is accurate; subprocess cleanup is measured.

### E080 — Implement provider policy and reservations

Status: `not_started`. Prerequisites: E041, E042, E017. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Define generic provider profiles and gateway admission over purpose, data class, region, terms expiry and scope. Reserve shared slots and spend before invocation.

Planned paths: `crates/arboresce-application/src/model_gateway/`, `crates/arboresce-postgres/src/model_gateway/`, `contracts/internal/`.

Definition of green: Expired/unapproved profile denies; concurrent reservations cannot overspend; unauthorized text is never sent; query embeddings use the same policy.

### E081 — Implement the controlled intelligence gateway

Status: `not_started`. Prerequisites: E080, E075. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Provide bounded authenticated internal invocation with job/stage/purpose identity, cancellation and safe exposure/cost receipts; prevent uncontrolled provider egress.

Planned paths: `crates/arboresce-api/src/internal/`, `crates/arboresce-intelligence-grpc/`, `contracts/internal/`, `tests/contract/`.

Definition of green: Wrong purpose/stage fails; retries are idempotent; secrets stay outside RPC; output/deadline budgets enforced; denied requests make no provider call.

### E082 — Implement the selected receipt adapter

Status: `not_started`. Prerequisites: E081, X-MEDIA-PROFILE. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Implement the approved OCR/extraction profile through the gateway with structured fields, original/representation references and explicit abstention.

Planned paths: `python/src/arboresce_intelligence/providers/`, `python/src/arboresce_intelligence/resources/prompts/`, `python/tests/`.

Definition of green: Recorded low-quality/conflicting totals remain unresolved; invalid or absent grounding is explicit; no automatic expense confirmation; generic prompts ship with the package.

### E083 — Implement the selected transcription adapter

Status: `not_started`. Prerequisites: E081, X-MEDIA-PROFILE. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Implement the approved ASR profile with bounded transcript and exact temporal grounding. Speaker labels never establish identity.

Planned paths: `python/src/arboresce_intelligence/providers/`, `python/src/arboresce_intelligence/pipelines/transcription/`, `python/tests/`.

Definition of green: Noise may abstain; spans stay in bounds; unknown speaker retained; output/time caps and redaction pass; semantic scoring cases are registered.

### E084 — Validate fields and multimodal disagreement

Status: `not_started`. Prerequisites: E082, E083. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Convert provider output to typed proposals with exact decimal parsing and revalidated locators; retain observation/assertion/inference distinctions.

Planned paths: `python/src/arboresce_intelligence/pipelines/fields/`, `python/tests/golden/`.

Definition of green: Photo/voice disagreements persist; unknown tax is not zero; invalid location fails; claimed purpose does not become proven fact.

### E085 — Propose scoped claims and counter-evidence

Status: `not_started`. Prerequisites: E084. Verify lane: PYTHON + CONTRACT; SERVICE when a process/provider boundary is exercised.

Scope: Implement bounded candidate synthesis with support/refutation, qualifications and invalidation evidence; preserve uncertain/confounding interpretations.

Planned paths: `python/src/arboresce_intelligence/pipelines/claims/`, `python/src/arboresce_intelligence/resources/prompts/`, `python/tests/golden/`.

Definition of green: No fabricated source or approval; negative evidence retained; cross-document work avoids global all-pairs comparison; output count cap enforced.

### E086 — Integrate durable intelligence Activities

Status: `not_started`. Prerequisites: E085, E072, E081, E018-process-coverage. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add gRPC adapter with deadlines, heartbeat/cancellation, typed revalidation, receipts and reservation settlement. Instrument Rust-launched Python service lifetimes.

Planned paths: `crates/arboresce-intelligence-grpc/`, `crates/arboresce-activities/src/processing/`, `tests/integration/processing/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Retry reuses accepted result; timeout retains uncertain cost; invalid outputs cannot become canonical; no approval side channel; cross-language process coverage is complete.

### E087 — Verify synthetic processing across processes

Status: `not_started`. Prerequisites: X-CLI-PROCESSING, E086. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Exercise actual API/Temporal/Python/CLI with recorded synthetic inputs and protected originals; register nonempty processing E2E and all process profiles.

Planned paths: `tests/e2e/processing/`, `tools/xtask/`.

Definition of green: Receipt/voice remain separate assertions; worker restart and policy revocation are safe; no unauthorized provider call; selected suites and subprocess coverage are verified.

### E088 — List authorized candidates and evidence

Status: `not_started`. Prerequisites: E047, E087, E009. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Implement current heads/history with typed filters, authenticated keyset cursors and bounded batch authorization.

Planned paths: `crates/arboresce-application/src/knowledge/`, `crates/arboresce-api/src/routes/knowledge.rs`.

Definition of green: Filter/purpose mismatch fails; no hidden counts or N+1 policy queries; current-state pagination semantics are explicit.

### E089 — Implement complete candidate replacement

Status: `not_started`. Prerequisites: E088, E027. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Create a new pending revision using expected revision/digest and current evidence permissions while preserving history.

Planned paths: `crates/arboresce-application/src/knowledge/`, `crates/arboresce-api/src/routes/reviews.rs`.

Definition of green: Incomplete/unreadable replacement fails; stale revisions conflict; revise never autoaccepts; changed scope alters identity.

### E090 — Implement attributable review verdicts

Status: `not_started`. Prerequisites: E089, E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Lock candidate and current policy, persist verdict and unique accepted asset mapping atomically with audit/outbox.

Planned paths: `crates/arboresce-application/src/reviews/`, `crates/arboresce-postgres/src/knowledge/`.

Definition of green: Fifty contenders yield one mapping; retries reauthorize; conflicting verdict is 409; service actors cannot impersonate humans.

### E091 — Expose reviewed assets and applicability

Status: `not_started`. Prerequisites: E090. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Serve exact immutable content with current control, availability, review and supersession metadata.

Planned paths: `crates/arboresce-api/src/routes/assets.rs`, `crates/arboresce-application/src/assets/`.

Definition of green: Pending candidates cannot appear as assets; revoked metadata remains access-safe; scope/qualifiers survive; revision orders history.

### E092 — Verify CLI review attribution

Status: `not_started`. Prerequisites: X-CLI-REVIEW, E091. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Exercise human/service pending/contested/revised/accepted flows and cross-organization evidence with the actual CLI.

Planned paths: `tests/e2e/review/`, `tools/xtask/`.

Definition of green: Human approval succeeds; service impersonation fails; concurrent review creates one mapping; scope-only revision works; unapproved content cannot be served.

### E093 — Create grounded expense proposals

Status: `not_started`. Prerequisites: E084, E048, E086. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Map validated extraction to proposed financial revisions with evidence and separately attributed user context.

Planned paths: `crates/arboresce-application/src/expenses/`, `crates/arboresce-activities/src/expenses/`.

Definition of green: No model auto-confirmation; unknown components persist; stage retry creates one proposal; duplicate detection remains a suggestion.

### E094 — Implement financial corrections and confirmation

Status: `not_started`. Prerequisites: E093. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Apply expected revision/current authority to correction, confirmation, void and linked positive-magnitude refund/credit commands.

Planned paths: `crates/arboresce-application/src/expenses/`, `crates/arboresce-api/src/routes/expenses.rs`.

Definition of green: Required unknowns block confirmation; currency changes create revisions; concurrent confirm has one winner; exact refund arithmetic and history pass.

### E095 — Compile immutable period reports

Status: `not_started`. Prerequisites: E094, E048, E031. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Select currently authorized confirmed revisions using explicit date/zone policy, currency grouping and authorized exclusions; freeze calculation version.

Planned paths: `crates/arboresce-application/src/reports/`, `crates/arboresce-postgres/src/expenses/`.

Definition of green: Date-only and time-zone boundaries are tested; void/proposal exclusions visible only if authorized; changed expense creates a different report.

### E096 — Export lossless and spreadsheet-safe reports

Status: `not_started`. Prerequisites: E095. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Provide lossless structured reports and separately specified safe CSV with permission-checked source references and bounded streaming.

Planned paths: `crates/arboresce-application/src/exports/`, `crates/arboresce-api/src/routes/reports.rs`.

Definition of green: Formula injection escaped without altering canonical data; no mixed-currency sum; unresolved eligible exclusions explicit; no tax-certification claim.

### E097 — Verify the financial user journey

Status: `not_started`. Prerequisites: X-CLI-REPORT, E096. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Run actual CLI/API/processing from receipt/note through correction, confirmation, report and export without requiring knowledge publication.

Planned paths: `tests/e2e/expenses/`, `tests/fixtures/scenarios/`, `tools/xtask/`.

Definition of green: Refund, void, multicurrency and malicious CSV cases pass; no paid Arboresce entitlement; real result semantics and process coverage recorded.

### E098 — Select exact eligible build inputs

Status: `not_started`. Prerequisites: E091, E049, E035. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Resolve bounded requested asset versions under current tenant/domain/purpose and evidence permission; preserve qualifications and reject unauthorized required inputs.

Planned paths: `crates/arboresce-application/src/context/`, `tests/integration/context/`.

Definition of green: No partial build or truncation; cross-domain selection follows policy; deprecation explicit; asset/byte caps enforced.

### E099 — Compile canonical build content

Status: `not_started`. Prerequisites: E098. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Generate deterministic exact bytes and manifest with compiler version, canonical set ordering and preserved ordered content; exclude mutable receipts.

Planned paths: `crates/arboresce-application/src/context/`, `crates/arboresce-domain/src/context/`, `tests/contract/context/`.

Definition of green: Set permutations are stable; ordered content stays ordered; signature/self-hash excluded; empty/oversize inputs rejected.

### E100 — Persist sealed build bytes and receipts

Status: `not_started`. Prerequisites: E099, E063. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Seal compiler output then bind immutable build edges and operation result once, with separate execution receipts.

Planned paths: `crates/arboresce-application/src/context/`, `crates/arboresce-postgres/src/releases/`, `tests/integration/context/`.

Definition of green: Identical retry reuses identity; crashes leave recoverable orphans; execution time cannot alter digest; foreign hashes are not existence oracles.

### E101 — Orchestrate durable context builds

Status: `not_started`. Prerequisites: E100, E069. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use stable workflow identity and Activities for selection, sealing, persistence and evaluation; recheck eligibility at effect boundaries.

Planned paths: `crates/arboresce-workflows/src/context.rs`, `crates/arboresce-activities/src/context.rs`, `tests/temporal-replay/`.

Definition of green: Success/retry histories replay; mid-build revocation blocks release; cancellation accurately reports committed effects.

### E102 — Implement versioned evaluation suites

Status: `not_started`. Prerequisites: E099, E011. Verify lane: RUST + CONTRACT.

Scope: Define suite/case identity, expected assertions, validity, exact counts and separate deterministic versus live quality classes.

Planned paths: `crates/arboresce-domain/src/evaluations/`, `contracts/public/`, `tests/fixtures/scenarios/`.

Definition of green: Counts and failures agree; unknown versions fail; evaluation cannot silently rewrite oracle expectations; malformed results fail.

### E103 — Execute evaluations and attach receipts

Status: `not_started`. Prerequisites: E102, E101. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Evaluate an exact build using bounded deterministic suites and persist immutable profile/dataset/result identity outside build content.

Planned paths: `crates/arboresce-application/src/evaluations/`, `crates/arboresce-activities/src/evaluations/`, `crates/arboresce-postgres/src/releases/`.

Definition of green: Correct build pinned; duplicate result idempotent; overlimit work fails; failed evaluation cannot confer eligibility.

### E104 — Issue accountable approval receipts

Status: `not_started`. Prerequisites: E103, E036, E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Check principal/purpose, current evidence and required suites before signing the exact content-attestation envelope through an injected key service.

Planned paths: `crates/arboresce-application/src/approvals/`, `crates/arboresce-postgres/src/releases/`, `tests/integration/releases/`.

Definition of green: Wrong actor/purpose and revoked/expired key fail; service cannot claim human approval; signature binds exact build; confirmation flags grant no authority.

### E105 — Implement channel transitions

Status: `not_started`. Prerequisites: E104, E049. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Apply create/promote/suspend/rollback with expected monotonically increasing generation and current eligibility in one transaction.

Planned paths: `crates/arboresce-application/src/channels/`, `crates/arboresce-postgres/src/releases/`.

Definition of green: Fifty concurrent promotions have one winner; revoked rollback fails; suspended history persists; generations never reset.

### E106 — Resolve exact authorized snapshots

Status: `not_started`. Prerequisites: E105, E065. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Serve complete exact build content under current canonical admission and bounded transfer, issuing accurate usage receipts independently of search.

Planned paths: `crates/arboresce-application/src/resolve/`, `crates/arboresce-api/src/routes/context.rs`.

Definition of green: Any revoked required source blocks; unavailable bytes remain unavailable; no silent filtering; previously delivered content cannot be recalled by assertion.

### E107 — Expose context operation families

Status: `not_started`. Prerequisites: E106, E101. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Bind implemented build/evaluation/attestation/channel operations to typed API contracts, operation resources and honest capabilities.

Planned paths: `crates/arboresce-api/src/routes/releases.rs`, `contracts/public/`, `tests/contract/http/`.

Definition of green: Accepted differs from completed; stale revisions conflict; approvals are enforced; optional index outage leaves exact readiness; extensions remain tolerable.

### E108 — Verify the resolve-only context journey

Status: `not_started`. Prerequisites: X-CONSUMER-RESOLVE, E107, E092. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Run actual capture/process/review/build/evaluate/promote/resolve and changed-evidence cases with fixed fixtures and an independently built resolve-only consumer. Feedback is added only after its capabilities exist.

Planned paths: `tests/e2e/context/`, `tools/xtask/`.

Definition of green: Qualifications preserved; invalid review cannot release; exact digest and tenant boundaries hold; suite executes real runtime and nonzero assertions.

### E109 — Implement exact-version feedback

Status: `not_started`. Prerequisites: E107, E050. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Persist feedback/outcome assertions with author/source identity and exact subject version without rewriting content or approval.

Planned paths: `crates/arboresce-application/src/feedback/`, `crates/arboresce-api/src/routes/feedback.rs`.

Definition of green: Unknown/foreign versions fail safely; retry returns original result; asserted outcomes remain assertions; pending feedback is not completed processing.

### E110 — Implement reviewed supersession

Status: `not_started`. Prerequisites: E109, E090. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Move current asset heads only through explicit reviewed scope rules, preserving evidence/history and channel impact visibility.

Planned paths: `crates/arboresce-application/src/assets/`, `crates/arboresce-postgres/src/knowledge/`.

Definition of green: Cycles and foreign links fail; deprecation preserves bytes; channel never moves silently; replacement knowledge requires review.

### E111 — Implement canonical revocation

Status: `not_started`. Prerequisites: E109, E106, E041. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Apply control revision and current admission locks to revoke evidence and persist dependent invalidation/cleanup obligations atomically.

Planned paths: `crates/arboresce-application/src/revocation/`, `crates/arboresce-postgres/src/lifecycle/`, `crates/arboresce-api/src/routes/artifacts.rs`.

Definition of green: Download/revoke barriers prove admission boundary; stale control conflicts; required components block new resolution; hold cannot broaden reads.

### E112-retention-contract — Define generic lifecycle policy

Status: `not_started`. Prerequisites: E111, E050. Verify lane: DOC + CONTRACT; add applicable language lanes for executable validators.

Scope: Freeze parameterized retention, hold, expiry, reference accounting, purge obligations, key/trust recovery and independent journal completeness semantics. Provide synthetic test policy values with explicit fixture status. Actual deployment policy remains X-DEPLOYMENT-POLICY and gates private-data/purge qualification rather than unrelated synthetic implementation.

Planned paths: `docs/requirements/security-and-privacy.md`, `docs/requirements/recovery.md`, `contracts/public/`, `contracts/internal/`, `tests/fixtures/scenarios/`.

Definition of green: no unsupported retention promise; unknown actual policy cannot enable real purge; held-but-revoked cases deny read while retaining bytes; policy version and journal gap behavior have independent oracle approval.

### E112 — Implement bounded purge and holds

Status: `not_started`. Prerequisites: E111, E064, E112-retention-contract. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Execute policy-driven deletion of controlled raw/derived/projection/export/cache copies with rechecked holds/reference counts and durable receipts. Synthetic tests use the generic lifecycle policy contract.

Planned paths: `crates/arboresce-workflows/src/purge.rs`, `crates/arboresce-activities/src/purge.rs`, `crates/arboresce-application/src/lifecycle/`.

Definition of green: Held bytes remain while reads are denied; retries cannot delete shared references; partial purge is explicit; tombstones survive recovery; remote-copy recall is not promised.

### E113 — Define complete customer exports

Status: `not_started`. Prerequisites: E110, E111. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Export authorized records, versions, relationships, evidence, corrections, context and evaluations with digests and explicit unavailable/purged markers in public formats.

Planned paths: `crates/arboresce-application/src/export/`, `contracts/public/`, `tests/integration/export/`.

Definition of green: References roundtrip; inaccessible entries/counts stay hidden; purged bytes are not fabricated; original attribution remains intact.

### E114 — Produce durable bounded exports

Status: `not_started`. Prerequisites: E113, E101. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Pin an authorized selection, stream bounded temporary output with expiry, reauthorize download and persist operation results.

Planned paths: `crates/arboresce-workflows/src/export.rs`, `crates/arboresce-activities/src/export.rs`, `crates/arboresce-api/src/routes/export.rs`.

Definition of green: Mid-export revocation blocks affected delivery; cancellation cleans controlled temporary work; duplicate requests share result; large exports remain bounded.

### E115 — Verify feedback correction and exit

Status: `not_started`. Prerequisites: X-CLI-FEEDBACK, X-CLI-EXIT, E112, E108, E109, X-CONSUMER-FEEDBACK. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Run a second learning iteration with actual CLI feedback/lifecycle commands and the extended consumer, then interpret export in a clean independent consumer.

Planned paths: `tests/e2e/lifecycle/`, `tests/e2e/export/`, `tools/xtask/`.

Definition of green: Consumed version and retry intent exact; feedback stays pending until reviewed; new build changes correctly; revoked sources deny; usable export preserves attribution.

### E116 — Define projection identity

Status: `not_started`. Prerequisites: E044, E091. Verify lane: RUST + CONTRACT.

Scope: Version immutable tenant/content/chunk/embedding-profile identities and generations with explicit delivery metadata; derived status never grants authority.

Planned paths: `contracts/internal/`, `crates/arboresce-application/src/projections/`, `docs/architecture/`.

Definition of green: Dimension/profile mismatch cannot collide; delayed events cannot redefine current heads; identity is deterministic.

### E117 — Package a protected Qdrant profile

Status: `not_started`. Prerequisites: E013. Verify lane: IMAGE + SERVICE; applicable language lanes for source changes.

Scope: Pin image digest/config/license with authentication, protected bindings, durable volume and explicit smoke checks.

Planned paths: `docker/images/qdrant/`, `docker/compose/retrieval.yaml`, `docker/docker-bake.hcl`.

Definition of green: Unauthenticated access denied; no startup plugin fetch; reported version accurate; target image starts safely.

### E118 — Implement typed projection adapter

Status: `not_started`. Prerequisites: E116, E117. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Use compatible-profile collections, indexed tenant/filter fields, bounded query controls and strict filter construction. Register real search fixture/service suite.

Planned paths: `crates/arboresce-qdrant/`, `Cargo.toml`, `Cargo.lock`, `tests/integration/qdrant/`.

Definition of green: Tenant constraint mandatory; unindexed or oversized query rejected; dimensions checked; delayed delivery cannot overwrite immutable versions.

### E119 — Generate policy-controlled embeddings

Status: `not_started`. Prerequisites: E081, E118. Verify lane: RUST + PYTHON + SERVICE.

Scope: Key embeddings by exact text/profile and permitted scope, reserve cost and persist reproducible references; query embeddings use identical exposure controls.

Planned paths: `crates/arboresce-activities/src/embedding.rs`, `python/src/arboresce_intelligence/providers/`, `tests/integration/embedding/`.

Definition of green: Unchanged content/profile reuses work; denied text never sent; changed dimensions require new profile; shared spend reservation enforced.

### E120 — Compose bounded projection delivery

Status: `not_started`. Prerequisites: E119, E044. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Consume leased outbox work into Qdrant with fenced receipts and bounded retry/backpressure; do not infer commit completeness from event IDs.

Planned paths: `crates/arboresce-projector/`, `crates/arboresce-postgres/src/outbox/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Reordered/duplicate events converge; stale owners cannot acknowledge; restart resumes safely; maximum allocated ID is never a watermark.

### E121 — Implement authorized hybrid search

Status: `not_started`. Prerequisites: E120, E041, E009. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Allocate k=1..20, two total rounds and 200 cumulative candidates before dedup across retrieval sources; batch canonical authorization before reranking permitted content.

Planned paths: `crates/arboresce-application/src/search/`, `crates/arboresce-api/src/routes/search.rs`.

Definition of green: Restrictive ACLs do not leak; no N+1 checks; hybrid branches share one budget; stable ties/exhaustion are tested; limits cannot be bypassed by refill.

### E122 — Record honest selection receipts

Status: `not_started`. Prerequisites: E121, E106. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Bind actual selected IDs, profile, generation, limits and freshness to output; do not reuse a full-build attestation for different selected bytes.

Planned paths: `crates/arboresce-application/src/search/`, `contracts/public/`, `tests/contract/search/`.

Definition of green: Qualifiers cannot vanish silently; receipt matches actual output; partial selection makes no whole-snapshot approval claim.

### E123 — Rebuild projections with a maintenance barrier

Status: `not_started`. Prerequisites: E120, E122. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Pause scoped writes explicitly, snapshot canonical state, reuse versioned embeddings, reconcile a new generation and switch atomically with rollback receipts.

Planned paths: `crates/arboresce-workflows/src/rebuild.rs`, `crates/arboresce-activities/src/rebuild.rs`, `crates/arboresce-application/src/projections/`.

Definition of green: Concurrent writers follow documented pause/queue policy; counts/digests agree; failure retains old generation; no maximum-ID shortcut.

### E124 — Propagate revocation to derived stores

Status: `not_started`. Prerequisites: E123, E112. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Deliver high-priority derived invalidation/deletion and generation-aware cache changes while canonical admission already denies.

Planned paths: `crates/arboresce-projector/`, `crates/arboresce-activities/src/purge.rs`, `tests/integration/qdrant/`.

Definition of green: Paused projector cannot broaden reads; old add events cannot resurrect authority; shared bytes remain; repeated deletion is safe.

### E125 — Verify retrieval disorder through CLI

Status: `not_started`. Prerequisites: X-CLI-SEARCH, E124. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Exercise actual CLI search with delayed/duplicate projections, restrictive authorization, outage and rebuilt generations.

Planned paths: `tests/e2e/search/`, `tools/xtask/`.

Definition of green: Exact resolution stays safe during search failure; no unauthorized results/counts; total budgets hold; actual selection receipts match results; nonempty suite registered.

### E126 — Consolidate shared resource admission

Status: `not_started`. Prerequisites: E080, E073. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Strengthen and stress the existing shared admission/reservation code from initial adapters across capture, processing and model work. Do not create a parallel quota system.

Planned paths: `crates/arboresce-application/src/budgets/`, `crates/arboresce-postgres/src/budgets/`, `crates/arboresce-api/`.

Definition of green: Parallel instances cannot exceed 100 pending jobs or two organization model slots; leases recover; rejected work is explicit; accepted work is not dropped.

### E127 — Consolidate transfer and parser budgets

Status: `not_started`. Prerequisites: E126, E065, E079. Verify lane: RUST + PYTHON + SERVICE.

Scope: Apply the same per-route wire/decode/CPU/memory/time/scratch policy through API, Activities and Python, independent of extractor defaults.

Planned paths: `crates/arboresce-api/src/limits/`, `crates/arboresce-activities/`, `python/src/arboresce_intelligence/`.

Definition of green: Chunked oversize and slow streams fail; 40-million-pixel/five-minute limits enforced; cancellation reclaims resources; no unbounded read remains.

### E128 — Add measured bounded caches

Status: `not_started`. Prerequisites: E121, E111. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Cache only demonstrated repeated work with tenant/principal/delegation/purpose/policy/content/profile/generation keys; every new admission remains canonical.

Planned paths: `crates/arboresce-application/src/cache/`, `tests/security/cache/`.

Definition of green: Revocation is effective regardless of cache; no cross-tenant hash oracle; changed profile misses; eviction and memory bounded.

### E129 — Instrument content-minimizing telemetry

Status: `not_started`. Prerequisites: E127, E128. Verify lane: RUST + PYTHON + SERVICE.

Scope: Measure queue, pool, bytes, cost, stage/retry, authorization and projection behavior with bounded labels and safe correlations; keep audit/exposure separately governed.

Planned paths: `crates/arboresce-observability/`, `crates/arboresce-api/`, `crates/arboresce-worker/`, `python/src/arboresce_intelligence/observability/`, `Cargo.toml`, `Cargo.lock`.

Definition of green: Redaction fixtures catch secrets/documents/URLs; no unbounded tenant/content labels; disabled telemetry is safe; Python/Rust behavior covered.

### E130 — Implement graceful shutdown and recovery

Status: `not_started`. Prerequisites: E129. Verify lane: RUST + PYTHON + SERVICE.

Scope: Stop new admission, bound HTTP draining and manage Activity/projector leases and heartbeats with recoverable operation state.

Planned paths: `crates/arboresce-api/`, `crates/arboresce-worker/`, `crates/arboresce-projector/`, `python/src/arboresce_intelligence/server.py`.

Definition of green: Signals at synchronized stages lose no accepted work; restart recovers; cancellation propagates; unfinished work cannot report success.

### E131 — Implement optional notifications

Status: `not_started`. Prerequisites: E130, E042. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Add a generic mail port and optional SMTP adapter with versioned templates, stable Message-ID and durable attempts. Deterministic integration uses Mailpit.

Planned paths: `crates/arboresce-smtp/`, `crates/arboresce-activities/src/notifications.rs`, `Cargo.toml`, `Cargo.lock`, `tests/integration/smtp/`.

Definition of green: TLS/temporary failure visible; accept-then-crash is reported as ambiguous; duplicate policy tested; no production mail/admin credentials needed.

### E132 — Add bounded fuzz verification

Status: `not_started`. Prerequisites: E014, E127. Verify lane: RUST + CONTRACT.

Scope: Fuzz command JSON, locators, cursors, canonicalization and media metadata using deterministic seeds and explicit time/memory/network restrictions.

Planned paths: `tests/fuzz/`, `tools/xtask/`, `docs/security-testing.md`.

Definition of green: Crash cases reproduce; invalid states reject safely; seed inventory is nonempty; no provider/network calls; harness source is measured where executable.

### E133 — Verify synchronized authorization races

Status: `not_started`. Prerequisites: E111, E126, E105. Verify lane: RUST + SERVICE; CONTRACT for wire/schema changes.

Scope: Exercise membership/revoke/promote/download and shared quota races at explicit barriers, distinguishing already admitted transfers.

Planned paths: `tests/security/`, `tests/integration/concurrency/`, `tools/xtask/`.

Definition of green: 10,000 controlled races show no new post-commit admission or quota bypass; one CAS winner; confirmation flags confer no privilege.

### E134 — Verify clean customer exit

Status: `not_started`. Prerequisites: E114, X-CLI-EXIT. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Implement a public export reader/import validator or explicitly specified restoration entry point preserving identities, references and rights without executing imported content.

Planned paths: `tests/e2e/exit/`, `crates/arboresce-admin/`, `docs/runbooks/export.md`, `tools/xtask/`.

Definition of green: Clean installation reads exports; wrong hashes fail; unavailable content explicit; import cannot grant permissions; actual exit scenario is registered.

### E134-restore — Implement and verify early D0 restoration

Status: `not_started`. Prerequisites: E134, E112-retention-contract, E124, E074. Verify lane: RUST + SERVICE + E2E.

Scope: Add real public restoration tooling and runbook for database, sealed bytes, keys/trust and current independently retained post-backup revocation/deletion/hold journal. Restore in an isolated environment with serving disabled; prove journal completeness, reapply later restrictions, rebuild projections and only then permit qualified serving. Parameterize policy and require separately approved operational resources.

Planned paths: `crates/arboresce-admin/`, `tests/integration/recovery/`, `tests/e2e/recovery/`, `docs/runbooks/recovery.md`, `tools/xtask/`.

Definition of green: D0 backup followed by revocation/deletion/hold changes restores without resurrected access; missing blobs/keys/journal tail or unknown completeness keeps serving closed; derived corruption rebuilds from canonical state. Packaged repetition after E137 is required for G4/private-user qualification; full D1 E09 remains required for G6.

### E135 — Build minimal Rust service images

Status: `not_started`. Prerequisites: E130, E131. Verify lane: IMAGE + SERVICE; applicable language lanes for source changes.

Scope: Package API, worker, projector and admin from pinned multistage inputs with nonroot runtime, configuration validation and OCI identity.

Planned paths: `docker/images/arboresce/`, `docker/docker-bake.hcl`, `.dockerignore`.

Definition of green: Required target builds; no credentials or build caches in final image; each binary starts and stops correctly; read-only filesystem supported where qualified.

### E136 — Build bounded Python runtime image

Status: `not_started`. Prerequisites: E135, E085. Verify lane: IMAGE + SERVICE; applicable language lanes for source changes.

Scope: Package pinned Python/uv/codec dependencies, prompts and generated resources with the selected processing profile; avoid startup installation or unqualified model weights.

Planned paths: `docker/images/intelligence/`, `docker/docker-bake.hcl`, `python/`.

Definition of green: Required target health passes; resources present; no credentials/runtime downloads; parser budgets hold in the actual image.

### E137 — Provide the supported local stack

Status: `not_started`. Prerequisites: E136, E068, E117, E052, E126, E127, E133. Verify lane: IMAGE + SERVICE; applicable language lanes for source changes.

Scope: Compose owned API/worker/Python/database/pooler/storage/Temporal images with optional retrieval/mail. Use protected bindings and explicit setup; require no paid Arboresce entitlement.

Planned paths: `docker/compose/`, `docs/getting-started.md`, `tools/xtask/`.

Definition of green: Clean documented startup works; administration not exposed; readiness accurate; H0 resources measured; selected third-party inference requirements clearly disclosed.

### E138 — Document verified installation and limits

Status: `not_started`. Prerequisites: E137, E108, E097. Verify lane: DOC + E2E.

Scope: Publish implemented operations, dependencies, supported model/media limits, review semantics, exports and independently built CLI compatibility. Distinguish tested support from target behavior.

Planned paths: `README.md`, `docs/`, `CONTRIBUTING.md`, `SECURITY.md`.

Definition of green: Every command exists; public clone is self-contained; no hidden entitlement; no legal/tax-certification claims; committed generated assets work without regeneration.

### E139 — Verify API and CLI parity

Status: `not_started`. Prerequisites: X-CLI-DOCUMENTATION, E138, E125. Verify lane: E2E + RUST; PYTHON when exercised.

Scope: Maintain executable public capability-to-command conformance for authorization, record/report, review/context, lifecycle and export; mark operator-only commands explicitly.

Planned paths: `contracts/public/`, `tests/conformance/`, `tools/xtask/`, `docs/compatibility.md`.

Definition of green: Same input/permissions yield same semantics; every shipped capability has coverage or documented operator distinction; no undocumented essential endpoint.

### E140 — Verify credential and signing-key rotation

Status: `not_started`. Prerequisites: X-IDENTITY-ARITHMETIC, E055, E104. Verify lane: RUST + CONTRACT.

Scope: Test supported credential replacement, trust overlap/revocation and restoration-safe historical trust interpretation without inventing cryptography.

Planned paths: `crates/arboresce-application/src/auth/`, `crates/arboresce-application/src/approvals/`, `tests/security/`.

Definition of green: Revoked authentication fails; historical signatures remain interpretable; unknown keys deny; backup restore cannot expand trust unexpectedly.

### E141 — Prepare verifiable engine distributions

Status: `not_started`. Prerequisites: E135, E136, E140. Verify lane: RUST + IMAGE + CONTRACT; actual release acceptance remains gated below.

Scope: Produce reproducible SBOM, checksums and signing inputs tied to exact source, locks, images and supported platforms. Provide artifact verification tooling distinct from content-attestation verification.

Planned paths: `tools/xtask/`, `docker/`, `docs/release-policy.md`.

Definition of green: Dependency inventory matches image; controlled fixture signatures exercise digest/length/platform/build mismatch and revoked/unknown signer rejection; unpinned bases fail; unsigned candidates are labelled accurately. This tooling checkpoint can complete before actual release signing; X-ARTIFACT-SIGNING and X-PRODUCTION-EVIDENCE remain mandatory for G6 and full-plan completion.

## Execution evidence and reconciliation

All 141 numbered slices and four suffix slices remain unfinished; none is active. Coverage, generic lifecycle policy and early restore additions are part of this plan. Preserve exact IDs when splitting further; a split records replacement relationships, prerequisites, source boundaries and its own green-state criteria. Reconcile against actual code after each checkpoint rather than marking a whole phase complete from documentation alone.

Each checkpoint records source revision/dirty state, contracts, schemas, dependency locks, tool versions, feature/native profile, dataset/scorer identity, exact command, selected tests, raw coverage, duration, results/skips/failures, independent review and gate effect. Redact secrets and evidence content. Record the completed commit hash in subsequent or external evidence rather than recursively embedding its own hash. A local build, a mock, an unsigned candidate and an accepted request have distinct meanings and cannot stand in for live qualification, signed release or completed processing.

Actual identity/media selections, independent oracle approval including numeric ASR thresholds, environment/budget authority, deployment retention/recovery policy, independent-user evidence and signing trust must close before their dependent gates. Do not invent missing facts. Stop only dependent work; continue other eligible work under the active authorization. No failing security/integrity check or required gate is waived. At interruption, name the last green checkpoint, current changes, exact blocked evidence and all remaining slices.
