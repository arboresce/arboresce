# Workspace and contextual project workflow

**Authority:** normative focused CLI MVP requirements. **Status:** accepted target;
schemas, runtime, identity assurance and qualification remain implementation work.

This document owns workspace/project meaning and the contextual change workflow.
[Identity](identity-contract.md) owns authentication and review authority;
[candidate review](candidate-review-contract.md) owns attributable verdicts and
accepted mappings; [financial records](financial-records.md) owns exact money and
confirmation. [Acceptance](acceptance.md) distinguishes the focused milestone from
the retained complete knowledge and production milestones.

## Runtime and workspace

The first user-facing runtime is the independently built CLI. It is optimized for
agents through stable JSON contracts and remains usable by humans through readable
inspection and review. It calls the public engine API. The engine owns canonical
authorization, transitions, calculations and persistence. Other applications,
embedded bindings and additional product runtimes are outside this milestone.

A server Workspace is the existing Organization tenant aggregate, exposed with
workspace terminology. Initially it has one configured default Domain. It is not
a second tenant identifier or security model. Public wire contracts continue to
use `organization_id` and exact domain references; workspace commands resolve
these canonically. The CLI's local connection profile selects an origin, identity,
workspace and domain. A profile name or local directory never grants access.

Identity provisioning and the controlled first-workspace bootstrap are explicit
exceptions to selecting an existing workspace. Every ordinary product command,
capture, analysis and mutation selects exactly one workspace. Tenant enforcement
applies to every referenced record. One workspace can contain multiple projects
and workspace-level knowledge. Project selection is explicit or a reviewable
proposal; absent selection is not permission to use every accessible project.
The canonical receipt workflow targets one selected project. Capture processing
cannot write into another workspace even if its actor is a member of both.

## Projects, parties and responsibilities

Projects are stable same-workspace aggregates with immutable content revisions,
separate access/lifecycle controls and accountable change history. Required typed
records are project description/context, participants, decisions, work items,
responsibility assignments, due dates, status and contextual facts. Work items
retain planned, in-progress, completed, cancelled and unknown meanings; an assigned
task or future statement is not evidence of completion.

A Party is a stable workspace-scoped reference to a person or external business.
It retains aliases and attributed details, which may be unresolved or unverified.
A Party is distinct from a Nostr Principal, Membership, tenant Organization and
delegation grant. A party need not have an account. A separately reviewed explicit
principal-party association cannot confer authority. Supplier, speaker, participant,
assignee, reviewer and executor are separate roles. Reported attendance,
responsibility or a statement about agreement does not prove identity, consent,
attendance, another party's assent or authorization.

Party alias matching must retain alternatives and ambiguity. Exact identifiers
take precedence over name guesses. All project/party/task relationships enforce
same-workspace ownership. Responsibility records distinguish reported assignment,
accepted product update and independently established agreement. The first profile
does not claim a participant acceptance or electronic contracting system.

## Capture and exact context

A capture is a fixed revision containing the ordered, explicit attachment roles,
required/optional status and attributed user context defined by
[acquisition](acquisition-contract.md). Preserve original receipt/audio/text bytes,
server-sealed hash and byte length, media identity, and exact derived representation
versions. Preserve image orientation/coordinate mappings and audio channel/time
mappings. A supplied transcript is a derivative or attributed statement, never a
replacement for the original recording or independent proof of transcription.

A normal voice note is approximately 30 seconds and describes what changed. It
need not repeat the project description, roster, known responsibilities or existing
dates. Longer input is accepted only within its qualified profile. The analysis
plan binds exact capture/representation references, authenticated source author,
selected project, attributed event date/timezone and a bounded authorized context
snapshot containing exact prior record revisions and processing-profile digests.
Only context admitted for the requested workspace/domain/purpose may reach a model.

This snapshot is an analysis input, not a Context Build or search receipt. Initially
it comes from bounded canonical project reads; no vector index or full knowledge
build is a prerequisite. Access and lifecycle restrictions remain current despite
the immutable snapshot. Retain the exact inputs used so a result can be explained.
An unrelated later project edit does not invalidate every proposal: each effect
checks the exact target and consequential dependency revisions it actually used.

Every consequential proposed field carries one or more explicit lineage entries:

- Source observation/assertion with its exact representation and locator.
- Retained-context reference with exact stable record ID and revision.
- Deterministic default/rule with version and applicability, visibly labelled.
- Model inference with profile/basis, alternatives and unresolved qualifications.

Lineage categories are not interchangeable. A context value must not be described
as spoken in the note. Missing information stays unknown unless an applicable
explicit rule or exact prior context supplies it. Resolve relative dates using the
attributed event date and selected IANA timezone/rule version, not upload time or
the process timezone. Preserve date-only versus instant precision. Ambiguous
references, conflicting values, omitted required information and unsupported
temporal expressions require clarification rather than invented certainty.

Receipt totals/currency/date, participant assent/attendance, task completion and
new decisions cannot be manufactured to fill a model response. Reported facts
remain attributed even after product acceptance. Monetary fields retain the exact
financial owner's evidence and unresolved-field rules.

## Typed and composable proposals

One analysis can produce multiple independent project change items and an expense
proposal. A presentation batch groups items; it is neither an indivisible approval
nor a general distributed transaction. Each item has a stable identity, immutable
pending revisions, exact capture/analysis origin and typed operation:
`create`, `revise` or `associate` over the qualified project record kinds above.
Each includes target/project/workspace references, expected target revision or
explicit absence predicate, complete proposed resulting content, changed fields,
supporting/refuting/qualifying evidence, context/lineage, unknowns and dependencies.
The first profile forbids arbitrary executable patches, shell/tool instructions,
permission changes and financial confirmation as project effects.

Dependencies name exact item revisions or committed records. They are acyclic,
same-workspace and explicit. An item depending on an unaccepted/rejected/stale item
remains ineligible with a typed reason; no silent substitute or automatic acceptance
is allowed. Independent items remain reviewable. Accepting a decision does not
automatically accept its suggested task, date, participants or expense.

The change-group read returns complete `project_change_resource` entries,
including typed proposed content, exact candidate identity, origin, lineage,
dependencies, current review and control observations. An ID/revision/state list
cannot substitute for those entries. The group retains the exact workspace,
project revision and capture revision plus a server-issued snapshot ID. A non-null
continuation contains that same snapshot ID and a bounded opaque cursor; null
means the end of that snapshot. It is not a fresh analysis or an approval command.
The serving implementation must bind the cursor to this snapshot, selected scope
and current disclosure authority. It must compare each item's workspace, project
and capture identity against the group and compare the response capture with the
group capture. Those cross-record joins require semantic qualification in addition
to the parsed shape checks; no identifier or cursor grants authority.
The project identity join does not require each item's retained project base
revision to equal a later project-head observation. Target/dependency fences
remain per item; unrelated project changes do not create a whole-project fence.

Project change content requires a versioned extension of the candidate/review
family: a typed effect plus the existing statement, scope, qualification and exact
evidence semantics. Do not insert project fields into the closed legacy candidate
v1 schema, hide them in prose/metadata, or introduce a second approval ledger.
Before consumers, define complete request/result/record variants, canonical
preimages, revision tags, registered commands/operations, numerical capacities,
positive/negative fixtures and exact current/exact reads. Legacy contract fixtures
continue to qualify only their unchanged versions.

## Review, application and follow-up

Each pending item can be edited, accepted, rejected or left pending. Editing creates
a new pending candidate revision with attributable manual changes; it never alters
original media, observations or prior verdicts. Acceptance uses the current domain
policy and either actual human review or the explicit scoped project grant in
[identity](identity-contract.md). Human-required legacy knowledge review retains
its own stronger policy; project acceptance does not waive it.

For a project item, exact candidate verdict and accepted-asset mapping, target
revision transition, dependencies, audit/outbox and idempotent result bind in one
short canonical transaction. The typed mapping links the accepted item to its exact
project record revision. There is one review history, not a separate project
approval event. Check current authorization, source controls, expected target and
all consequential dependency versions at the final shared fence. CAS conflicts
have no partial application; no automatic merge or whole-project stale fence.
Independent item commands may commit separately. State which commands, if any,
offer bounded atomic groups; the initial profile need not offer batch acceptance.

Acceptance records an accountable product update, not independent truth or another
party's consent. Rejection and supersession preserve exact content and decisions.
A follow-up capture proposes an explicit revision/relationship to the existing
record and accepted mapping. Reuse qualified candidate/asset supersession and
lifecycle semantics; introduce its complete project variants before enabling it.
Basic project corrections do not require full channel/build/retrieval machinery.

Reanalysis creates a distinct versioned result for changed inputs/profile/context.
It must retain manual overrides, prior accepted/rejected history and origin links,
and identify changed or redundant suggestions. It cannot overwrite user corrections,
resurrect rejected revisions, auto-accept new content or duplicate applied actions.
Retries of the same admitted command/stage return their stable results under current
disclosure authority. Cancellation does not undo committed items or prove absence
of an uncertain provider effect; existing operation ledgers retain those outcomes.

## Expense association and readback

An expense can be proposed from the same capture without publishing project
knowledge. Project association is a typed, same-workspace, versioned financial
relationship to the exact expense revision. It must be inspected in financial
review and bound to its exact content/association identities. Implement a versioned
financial composition without changing closed v1 money fields or smuggling a
project ID into business-purpose prose. Financial confirmation needs the distinct
actual-human exact-content event; project-item acceptance cannot provide it.

Project readback uses canonical accepted project revisions and qualified current
expense associations. It distinguishes pending changes, unknown/conflicting facts,
reported attendance, planned/completed work and confirmed/unconfirmed expenses.
It exposes the exact accepted mappings, revisions and source/context provenance
under current disclosure controls. The CLI has no competing project database.
Lossless JSON and reversible CSV financial exports retain existing rules; focused
project read/export preserves references and review history with an independent
reader. A financial export alone is not a complete customer archive.

The scoped `project_readback_view` contains explicit collections of complete
canonical records, exact accepted effect mappings, attributable review history,
expense/project associations and financial expense snapshots. It includes current
project control and a server-issued snapshot ID. Empty collections are explicit;
missing collections cannot stand for a completed read. Each provenance entry binds
an exact record revision to structured field lineage; a source label alone does
not provide that lineage. The response's separate `pending` collection contains
complete pending change resources, not accepted or rejected results disguised as
pending summaries.

Every returned record, mapping, review and expense association must belong to the
selected workspace/project and snapshot. The reader compares exact referenced
candidate, target, expense and association revisions/digests and control
observations. Accepted mappings and provenance identify their returned canonical
records; review history never confers new authority. A financial snapshot exposes
its own proposed/confirmed/superseded/voided state, independent of project-item
acceptance. Bounded reads must report an explicit capacity failure when a complete
authorized view cannot fit; silent collection truncation is forbidden. Semantic
join, completeness and current disclosure checks remain required before a service
or client can claim qualified readback.

## Required qualification

[Project fixtures](project-fixtures.md) owns the deterministic corpus and review
trace. Actual CLI/API/PostgreSQL/object-store/Temporal/gRPC integration is required;
recorded inference qualifies deterministic behavior, not live processing. Qualified
OCR, ASR and contextual synthesis require separate actual routes and independently
checked truth. G2-project and actual D0-project restoration precede personal
evidence and independent users. Full knowledge, search/repair, D1 recovery and
production obligations remain retained under [acceptance](acceptance.md).
