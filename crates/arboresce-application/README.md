# Arboresce application ports

This package owns typed application boundaries and depends inward on the domain.
It exposes no database, framework, transport, credential or ambient-clock types.

The first port serves only the isolated synthetic migration harness. Its immutable
record carries checked organization/record identifiers, content revision and date.
Its explicit write outcome distinguishes a positive commit, a confirmed absence
of accepted effects, and an unknown result. The port supplies no automatic retry.
Actual migration, persistence and uncertainty require the PostgreSQL consumer's
independent service qualification. These types enable no product/bootstrap writer,
authorization, idempotent command, audit/outbox or recovery capability.

The policy module carries an explicitly selected organization/domain workspace,
a server-internal principal context and typed scoped references. Its scope check
compares the complete workspace and preserves the reference family. Checked
identifiers establish format; the consuming canonical transaction still checks
existence, membership and current policy. No caller-supplied authentication flag,
key verifier or human-assurance claim is introduced.

`policy::principal` adds an exact checked profile/version/public-key lookup value,
a checked stored principal record and an asynchronous `PrincipalDirectory` port.
The mapping consumer invokes the port once, distinguishes absence from failure,
rejects ambiguity including duplicate IDs, and checks the returned identity.
Its bounded result carries no alias, creation, workspace permission or assurance.
The independently frozen principal-directory fixtures cover direct constructors,
actual typed port calls, pending completion and finite diagnostic traits.

This is application mapping semantics. Real SQL directory integration, actual
signed-request authentication, I/O deadlines, current policy/revocation and all
product/bootstrap writers remain at their separate qualified consumers.

`policy::authority` evaluates the explicitly selected
[`canonical-authority-evaluation-v1`](../../tests/security/fixtures/canonical-authority-v1/README.md)
processing profile over a bounded trusted canonical projection. It checks the
selected workspace, current canonical principal/membership/policy, restrictive
source intersections, exact immutable delegation links and supplied target CAS.
Trusted configuration supplies supported policy names; no fixture policy is a
production default. Its immutable observation retains actual selected identities
and revisions. The evaluator borrows decoded records and allocates no heap.

The projection is complete for every selected and participating required record.
Unrelated well-formed rows receive structural/uniqueness checks without globally
resolving foreign policies or references. Adapters must qualify bounded decoding
and current canonical SQL reads separately. Re-evaluating a new supplied snapshot
proves pure policy semantics; it supplies no conflicting database lock, current
effect fence or transferable authority. The existing readonly tenant boundary,
actual races, bootstrap, shared admission and pooling remain required. This
component enables no product/bootstrap writer, authentication or human assurance.

`policy::bootstrap::check_scope` checks the explicitly registered namespace and
policy against a current canonical one-use grant and a normalized request. The
authenticated operator is a separate typed argument. All six names have the
independently frozen nonempty ASCII token bound of 96 bytes before comparisons; the
stored lifetime must be nonempty, with inclusive issuance and exclusive expiry.
Consumed, retired, foreign-tenant and mismatching selections fail explicitly.
The borrowed check allocates no heap and performs no I/O or mutation. A matching
scope returns only `()`, with no capability or grant consumption.

The adapter still owns bounded decoding, real operator authentication, healthy
trusted time and current canonical reads. Checked new-owner format establishes
no existence, membership or key proof. The
[`bootstrap-scope-v1` fixtures](../../tests/security/fixtures/bootstrap-scope-v1/README.md)
preserve all ten original observations and 33 finite boundary cases. The measured
input-only helper observes the actual API and unchanged typed grant. Every
product/bootstrap writer still requires authentic independently retained
lifecycle continuity and its separately qualified command and effect boundary.

Use the workspace's Rust 1.99.0 checks and complete native/source collection,
following the repository guidance and the independently frozen migration fixtures.

# Shared reservation boundaries

The admission policy module provides checked quota, claim, lease, owner/fence,
attempt and total-deadline decisions. It accepts typed trusted configuration and
time values; it performs no storage, dispatch or authentication. Shared budgets
and the current canonical membership must be checked by the owning transaction
before a reservation can support a later worker operation. An uncertain external
outcome retains resource responsibility until positive reconciliation.

The independently admitted shared-admission fixtures cover complete boundary
observations and input failures. These pure results do not qualify the actual
PostgreSQL transaction, a product writer, a provider or recovery.

The [supplemental shared fixtures](../../tests/security/fixtures/shared-admission-additional-v1/README.md)
add 41 malformed preparation observations through the same typed input helpers.
Their complete finite errors stay separate from product admission decisions.

# Continuity scope projections

The lifecycle module provides checked six-family names, positive 18-digit
ordinals and organization/stream namespaces. Its pure `match_scope` checks a
supplied beginning or predecessor, a checkpoint range and one through 256 entry
scopes. Namespace mismatches, gaps, duplicate/reversed ordinals, invalid ranges,
broken predecessor links and ordinal exhaustion return finite failures. It
borrows typed inputs, performs no I/O or allocation, and returns only a projected
range. Repeated families are permitted; complete record validation is separate.

The independently committed [continuity literals](../../tests/integration/recovery/fixtures/continuity-ordinal-v1/README.md)
provide 16 family, 13 ordinal and 36 scope observations, including exact values
above 2^53 and the positive maximum 999999999999999999. Measured test support
receives only a case's input and serializes the actual typed values after the
call. Expected objects and case IDs stay in the thin assertions.

These projections authenticate neither a beginning nor a complete tail. Equal
supplied digests do not qualify actual encoded bytes, signatures/trust, complete
binding/abort/inventory, independently durable acknowledgement, cutoff,
recognition/retirement, recovery or serving. All remain mandatory at their
consumers; this API grants no product/bootstrap writing permission. Source,
helper, strict coverage and owning/public-only native qualification remain
required under the declared source-inventory profile.
