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

Use the workspace's Rust 1.99.0 checks and complete native/source collection,
following the repository guidance and the independently frozen migration fixtures.
