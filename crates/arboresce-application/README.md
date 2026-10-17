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

Use the workspace's Rust 1.99.0 checks and complete native/source collection,
following the repository guidance and the independently frozen migration fixtures.
