# Engine architecture

**Owner:** Engine component and dependency boundaries.
**Status:** Normative target. Components and commands are prospective until implemented and verified.
**Verification:** Architecture and contract checks under [testing and coverage](testing-and-coverage.md), followed by the applicable [acceptance gates](acceptance.md).

The engine receives commands through a public application API, applies domain
rules under current authorization and records accepted state in PostgreSQL.
Durable work runs through Temporal and Activities. A bounded Python service
performs intelligence processing through an authorized model gateway. Sealed
object storage holds exact bytes; search projections provide candidates for
subsequent canonical checks.

The independent CLI and other applications use the same public API. They do not
administer the engine's databases, start workflows directly or embed a second
implementation of its business rules. Local and remotely operated engine
profiles share the same command and authorization semantics.

## Repository ownership

Retain the existing seven component roots. A directory identifies ownership, not
proof that a package, service or executable already exists.

| Root | Owns |
| --- | --- |
| `crates/` | Rust domain and application code, adapters, workflow boundaries and executable composition. |
| `contracts/` | Public and internal machine-readable contracts and their quality declarations. |
| `docker/` | Image definitions and supported local runtime composition. |
| `docs/` | Requirements, architecture, public interfaces, contributor guidance and operating procedures. |
| `python/` | The intelligence service, processing profiles and its packaged resources. |
| `tests/` | Shared fixtures and verification across components. |
| `tools/` | Durable repository verification and generation tooling. |

Create only packages required by an implemented capability. Keep Rust crate
directories flat and use consistent `arboresce-` package names. A new domain noun
does not by itself justify a crate, process or service. Public transport contracts
and stable domain APIs must not expose implementation-specific persistence or
framework types.

Public wire definitions belong under `contracts/public/`; service-internal wire
definitions belong under `contracts/internal/`. The independent CLI may consume
versioned public transport definitions or packages, but it must build from its
own checkout and documented dependencies. Server-internal crates cannot become
a required CLI implementation dependency.

## Dependency direction

Domain code owns pure invariants, value types and state transitions. It has no
database, HTTP, object-storage, model-provider or Temporal dependency. Application
code owns use cases, authorization decisions, transaction boundaries and ports
for effects. It depends on the domain and typed abstractions, with no concrete
infrastructure or Temporal SDK types in its public interfaces.

Adapters implement those ports and depend inward. API, worker, projector and
administration binaries assemble configuration and concrete implementations.
Composition may select adapters, but it must not become another authority for
business state or money calculations. Test support may compose controlled
implementations without becoming a production dependency.

Temporal-specific code belongs in workflow, Activity and worker integration
boundaries. Workflow code may use supported deterministic Temporal APIs. It must
not perform direct network or database I/O, consult ambient clocks or randomness,
or invoke a model. Effectful work belongs in Activities behind application ports.
The [processing contract](processing.md) owns retries, replay and accepted-result
identity. Architecture checks must exercise both permitted SDK use and forbidden
effectful dependencies; a blanket SDK ban is not the intended boundary.

Separate algorithmic libraries from process startup and configuration. Prefer
explicit types, scoped errors, deliberately exported APIs and bounded asynchronous
work. Reject unsupported states through errors rather than intentional production
panics. Avoid production-path `unwrap` and `expect`; forbid unsafe code by default
and isolate any explicitly reviewed exception. Dependency exceptions need an
identified purpose and their own tests.

Use a Python `src` package layout with strict typing and explicit public module
interfaces. Manage its isolated environment and lock with uv, and use Ruff,
Pyright and pytest through the owning verification lane. Reusable test helpers
are maintained source; placing a parser or harness in a test directory does not
remove its architectural or measurement obligations.

## Current local guard profile

The first [repository tool](../../tools/xtask/README.md) checks the actual resolved
Cargo graph and current production source directories. Its initial role registry
recognizes the existing binding/build proof, standard-library-only domain and
verification tool at version 0.0.0. Unknown packages or dependency kinds fail.
The inward application and supported deterministic SDK 1.0.0 profiles have
independent synthetic policy examples; actual application/workflow packages and
runtime qualification remain future consumers. Amend explicit pure dependency,
role and supported API profiles with their first qualified consumers. A current
unselected dependency is not a permanent prohibition on qualified pure libraries.

The first pure numeric consumer selects `jcs-number-v1` through the domain's
`package.metadata.arboresce.architecture-profile`. Missing selection uses the
unchanged closed `initial-v1` policy. Unknown or wrong-type selection fails;
the new exception belongs only to `arboresce-domain` version 0.0.0. Actual Cargo
metadata dispatch must check exact unrenamed, untargeted normal canonicalizer
and software-SHA roots, all reachable normal/build conditional edges, registry
sources, exact versions and effective features, including workspace unification.
The [dependency owner](dependency-qualification.md#selected-scalar-canonical-number-profile)
fixes that graph and the bounded finite-f64 subset. A profile marker alone is
not qualification. Preserve all Initial guard meanings and I/O/source boundaries.

Production import/path checks reject configured I/O, clock, random, concrete
infrastructure and unsupported SDK surfaces. They reject root aliases and unsafe
or panic/unwrap paths, including raw spellings, UFCS calls and imported panic
aliases/functions, while preserving safe defaults, pure value imports and supported
workflow namespaces. Platform I/O remains outside pure layers; selected ffi/raw
value imports remain allowed. Cargo production targets must name actual files
under their owning src root; only the exact registered test-peer bin has its own
measured support path. Unknown target kinds fail. Explicit module path attributes
and source include! are unselected until a consumer qualifies complete source
inventory; ordinary modules and text/byte resource inclusion remain allowed. Controlled development dependencies permit isolated test composition
and confer no production import authority. Syntax checks complement actual
compiler checks and independent review; they do not certify arbitrary Rust data
flow, macro expansion, authorization or Temporal compatibility. Source inventory,
maintained helpers and native collection retain the complete testing contract.

## Sources of truth

The focused [project profile](project-workflow.md) adds canonical Party/project
records, exact context snapshots and typed change mappings to application PostgreSQL.
The CLI remains a public HTTP client without database, workflow or gRPC authority.
Python still returns bounded observations/proposals without canonical DB credentials.
Initial context comes from authorized bounded canonical reads; Qdrant and full
Context Builds are unavailable until later qualified consumers. Package only the
actual required local roles, including independent lifecycle continuity and separate
Temporal persistence. No graph database, bindings or additional runtime is an
implicit prerequisite of G3-project.

| Component | Authority and limit |
| --- | --- |
| Application PostgreSQL | Canonical organizations, memberships, policies, evidence relationships, reviews, expenses, assets, builds, channels, operations, audit and outbox records. |
| Blob storage | Exact server-sealed originals and derived representations referenced by canonical records. Storage access alone does not grant domain authority. |
| Temporal | Durable orchestration history and execution state. It is not a second approval or asset database. |
| Python intelligence | Typed processing results and proposals. It has no credentials for writing canonical product tables and cannot approve knowledge. |
| Qdrant | A rebuildable retrieval projection. Index contents, point IDs and stale permissions cannot authorize access or redefine asset identity. |
| Notification transport | Delivery attempts and their observed outcomes. Transport acceptance cannot commit domain transitions or guarantee recipient delivery. |

Keep application data and Temporal persistence in separate databases, with
independent roles, connection budgets and lifecycle qualification. No component
may use a derived store to bypass unavailable canonical authorization. Exact
build serving must remain independent of optional approximate search when its
own required dependencies are healthy.

Content identities, state transitions and policy combination belong to
[the data model](data-model.md). Physical keys, indexes, projections and storage
relationships belong to [storage and search](storage-and-search.md).

## Command and work boundaries

A command authenticates its actor, validates bounded input and establishes current
policy. A short canonical transaction claims the command identity, checks the
expected revision, applies the transition and appends the corresponding audit and
outbox records. Permission changes must participate in the same documented locking
discipline as command admission. A stale cached permission or an object identifier
is insufficient authorization.

Network work, provider calls, parsing and byte transfers occur outside database
transactions. Upload finalization binds verified sealed bytes after streaming
validation; processing persists a validated accepted result with a stable stage
identity. A retry must not create a new canonical result merely because its
transport attempt changed. The detailed rules are owned by [API](api.md),
[processing](processing.md) and [storage](storage-and-search.md).

Use the outbox to connect committed domain changes to durable work and projection
delivery. Consumers record their own completion and reconcile from canonical
state. An allocated event number is not evidence of transaction commit order.
Derived failure may delay a capability, but it cannot retroactively change an
accepted command into an unrecorded operation.

## Intelligence and resources

Start with one coarse-grained Python intelligence service and distinct modules
for the supported processing operations. Split deployment only when measured
compute, isolation or scaling needs justify it. Use typed, versioned internal
gRPC requests and responses, with explicit deadlines, cancellation, resource
limits and idempotency identity. Bulk evidence travels through narrowly authorized
immutable blob references rather than large workflow histories or control
messages.

Rust authorizes work, validates returned types and grounding, and writes canonical
results. Model invocation must pass the approved gateway policy; moving invocation
into another process must preserve equivalent verifiable authorization and egress
enforcement. Processing profiles and ordinary prompt resources are versioned
package inputs. Keep notebooks, model weights and mutable hidden runtime state
out of the product source tree.

Admission is a shared service responsibility from the first expensive adapter,
not a later per-process rate limit. Global worker capacity, tenant reservations,
parser limits and bounded queues must work across the allowed instance count.
The [resource profile](resource-profiles.md) owns shared resource budgets and
capacity equations. [API](api.md) and [storage and search](storage-and-search.md)
own their operation-specific semantic limits. [Security](security-and-privacy.md)
owns exposure and access controls.

## Build and distribution boundaries

[Dependency qualification](dependency-qualification.md) owns exact toolchain pins
and supported targets. Add manifests, locks and executable tooling with their
first working implementation, rather than placeholders for the entire planned
workspace. Rust and Python changes must satisfy their owning verification lanes.
Public local orchestration belongs under `tools/` and calls the same supported
direct checks. Hosted workflow automation is outside the accepted implementation
plan; an independently operated release verifier remains required.

Follow [generated-artifact ownership](../generated-artifacts.md). Use pinned tools
to regenerate public/internal bindings, commit required outputs and test drift.
Consumers of an ordinary checkout must not require access to an unavailable
generator or another source repository. Generated executable code and maintained
tooling remain in the source inventory and coverage scope.

[Deployment](deployment.md) owns packaging, role startup, protected service
topology and upgrade procedures. [Recovery](recovery.md) owns reconstruction and
closed-serving conditions. Future graph features, stable language bindings and
additional product applications remain subject to [the product scope](product-and-scope.md).
