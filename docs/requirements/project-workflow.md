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

The separately versioned project-capture binding preserves its required source
workspace context. Admission compares the selected project/workspace and an exact
independently resolved capture reference with the request. The returned binding
retains the admitted project ID, capture ID/revision, binding revision and source
context. A foreign source workspace, substituted revision or discarded context is
a semantic failure even when each value has a valid parsed shape. These joins do
not replace current authorization, sealed-byte validation or the final transaction
fence owned by the consuming service.

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

### Initial expense association admission

The initial association profile creates a previously absent relationship for a
current proposed expense. Its `expected_control_revision` belongs to the stable
association aggregate. Expense and project control revisions remain independent;
no expense content version, project revision or retained association field can
supply a current association control observation.

For fresh initial creation, the selected workspace has no aggregate with the
supplied association ID and no current association for the selected project and
expense ID. The complete supplied association has `association_revision` content
1 and `expected_control_revision` control 1. These are explicit initial values,
not a fabricated pre-existing control 1 or a zero control. The initial-control
precondition binds the absent aggregate and the required initial result together.
An existing ID or current relationship conflicts rather than silently replacing
or duplicating it. Later association revision and rebinding transitions retain
separate qualification requirements.

At the final admission fence, the selected project reference names its complete
current same-workspace project content revision. The supplied expense reference
names the complete current same-workspace expense head, including exact content
version and digest, and that head is proposed. Current requester, executor,
membership, permitted purpose/action, scoped grant or authorized-human authority,
source disclosure and lifecycle permissions must all still permit this creation.
A financial review label, Party, project acceptance or retained confirmation
cannot provide this authority. The separately registered create request retains
its complete association body and common metadata; protected input does not
replace these semantic preconditions.

One fenced transaction creates the immutable association revision 1, its current
head and independently recorded current association control 1. It binds the
complete owning association preimage/digest, actual executor and attributable
provenance, mutation audit/outbox and terminal idempotency receipt to that exact
request and result. Failure cannot expose a partial association or commit part
of those effects. Initial creation does not revise expense money/content,
advance expense content/control, change its proposed state, issue or consume a
financial review event, or create financial confirmation. It does not rewrite
project content, historical associations, confirmations or report selections.

Complete recorded postconditions distinguish the absent association/control
before admission from the exact new head and current control 1 afterwards.
Current control is an observation bound to the stable association ID; the
immutable resource's retained expected control is a historical precondition.
The existing `observed_association_control_revision` in financial selections
carries the separately observed association control, not an expense version or
an inferred approval. A service must qualify actual current-control observation
and mutation effects before enabling the corresponding runtime capability;
synthetic conformance records establish neither storage nor actual authority.

Eligible exact idempotent replay rechecks current authorization and disclosure
before returning the original complete association and receipt. It preserves the
original request fingerprint and receipt expiry/tombstone deadlines, marks the
receipt replayed, and creates no new association, head, control increment,
mutation audit/outbox or review effect. A changed request under the same key
conflicts. A fresh distinct intent cannot convert an existing ID or relationship
into initial creation. Current denial does not disclose a cached result or
reapply an old financial effect.

This profile defines initial creation and its exact replay only. Association
revision, rebinding, supersession and correction after confirmation remain
required but must receive their own complete owning transitions before affected
consumers. A new proposed composition cannot be implemented by fabricating a
normalized no-op financial revision or rewriting an older confirmation/report.
Initial creation is not acceptance of those later transitions, human assurance
or an installed service/client command.

### Association revision before financial confirmation

The registered `expense_association.create` operation can create a new immutable
revision of an existing association before that relationship is financially
confirmed. This profile is distinct from absent-aggregate initial creation. The
stable association ID, workspace and stable expense ID remain unchanged; rebinding
changes the selected project or the exact revision of that same expense. A new
expense identity requires its own initial relationship, not reuse of another
association identity. The current expense head is proposed. At the final
admission fence, the complete required confirmation history of the selected stable
association aggregate is observed across all its retained revisions, within its
exact workspace. No revision of that aggregate has a retained successful
financial confirmation binding. A confirmed historical revision prevents this
pre-confirmation admission even if a later current association or expense head
is unconfirmed/proposed; the separately versioned post-confirmation composition
transition is required. Missing or unavailable required history cannot establish
eligibility. A confirmation for a different stable association does not supply
this aggregate's history or by itself disqualify this transition.

The complete retained write intent names the exact held current association
revision/digest and separately observed aggregate control. At final admission,
that held head must still be the current immutable association, and the requested
`expected_control_revision` must equal the current association control. The new
`association_revision` is exactly the old association content revision plus one;
after commit the current association control is exactly its former value plus
one. Content and control counters remain independent. The immutable predecessor's
retained expected control, an expense/project revision or a financial label
cannot supply the current control or infer either successor counter. Reject
exhaustion or a stale fence before any effect.

The existing closed create body carries the complete new association and common
metadata. Its project reference names a complete current same-workspace project;
its expense reference names the complete current proposed head of the same stable
expense, including exact version and content digest. Both the predecessor and
new relationship remain subject to current requester/executor, membership,
permitted action/purpose, scoped grant or independently authorized human, source
disclosure and lifecycle authority. No project approval, Party, cached permission
or retained review supplies fresh authority. The new current project/expense pair
must not already belong to a different current association in the workspace.

The new owning association preimage and digest must differ from the predecessor's
normalized content. Provenance-only changes or refreshing the same project's
reference without changing the owning preimage cannot manufacture a revision.
Changing the project binding or the exact expense revision is explicit reviewed
association content; it never changes closed expense v1 money/content. A normalized
no-op expense revision cannot substitute for an association or composition change.

One fenced transaction retains the predecessor unchanged, appends exactly the
new immutable association, replaces that aggregate's current head, advances its
independently recorded current control once and binds attributable provenance,
mutation audit/outbox, permanent command recognition and complete original
terminal result/receipt. The old association remains resolvable by its exact
revision/digest under current disclosure; supersession changes the current head,
not the immutable historical association or its provenance. Other association
aggregates, expense content/control/state, project content, old confirmations and
frozen report selections remain unchanged. Failure cannot commit a partial set
of these effects or disclose a partial successful result.

Pending financial challenges and unconsumed review events bound to the old
association become ineligible for the new current relationship. Their exact
historical bodies and decisions remain retained; they cannot be rebound, renewed,
reissued or consumed as authority for the successor. A new exact review must name
the new association revision/digest. The association write issues or consumes no
financial review event and creates no financial confirmation.

Each distinct revision or rebind retains a new complete command intent and key.
Eligible exact replay rechecks current authorization before disclosure and returns
the original association and original receipt with replay marked. Later current
heads or controls do not turn replay into fresh revision admission; the original
fingerprint, terminal time and retention deadlines do not change. Replay creates
no new version, head/control change, audit/outbox, recognition or financial effect.
A changed full request under the same scoped key conflicts. Permanent recognition
with unavailable detailed results cannot be treated as an unused key. A retry
with a new key still must satisfy the actual current head/control and no-op fences.

Complete recorded observations distinguish immutable history, held references,
the current head and independent current control before and after the write.
Lost or invalid success observations leave remote admission unknown and retain
the last trusted state; they cannot invent committed or aborted postconditions.
These requirements must receive independent literal, consumer and actual runtime
qualification at their respective gates. They accept no post-confirmation
composition, financial human assurance, storage or installed service/client.

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

### Separately versioned confirmed financial composition

Post-confirmation relationship changes use the separate
[`financial_composition_*_v1` family](financial-records.md#versioned-composition-transitions),
including first materialization from a legacy association or an explicitly known
unassociated confirmed expense. Each proposal creates a new exact association
and composition atomically; it never rewrites a historical association or borrows
the pre-confirmation profile. Content revisions and observed controls advance
independently. Complete financial binding history remains distinct from current
heads. An unchanged associated binding cannot create a wrapper-only successor.

The dedicated compound review binds exact project revision, money, association,
composition and their independent current controls. Confirmation requires both
financial and association authority, one qualified human decision and one global
event consumption. Proposed incomplete money cannot become confirmed by association.
Original expense confirmations and reports remain immutable. Current authorization
also gates exact reads and replay disclosure; a retained reference grants no access.
The legacy report/readback/export shapes remain unchanged. Their separately
versioned composition selections must distinguish legacy association selections
from compound selections and retain original base expense confirmation alongside
the compound event. This family alone does not qualify those consumer paths.

## Focused export payload and artifact identities

Project export requires both the exact project content revision and expected
project control revision. Neither precondition may be inferred from an ID or a
cached head. `project-export-payload-v1` freezes the complete scoped
`project_readback_view`, the separate complete pending change collection and
source availability. Accepted content and pending proposals remain distinct.
`project-financial-export-payload-v1` freezes the complete project financial
report snapshot, every selected exact financial revision paired with its frozen
selection, and source availability. Financial records use the unchanged closed
financial v1 export record type and retain selection order. Excluded records
contribute neither content nor money. Every included identity, lineage, review
and source descriptor requires current disclosure; inability to provide the
complete authorized payload fails rather than silently omitting entries.

The closed `arboresce.project-export-artifact.v1` core binds workspace/export
identity, exact project snapshot or project report subject with observed control,
payload digest, selected format, media type, literal UTF-8 encoding, emitted
byte length/digest, selected profile, sealing time and fixed exclusive expiry.
The complete resource adds the unchanged lifecycle export reference and control,
observation time and available/unavailable/expired/purged disposition. Its
artifact-record digest hashes JCS of the complete core. Mutable observations
never enter that digest. Project and project-financial formats require their
corresponding subject kind; JSON uses application/json and CSV uses text/csv.
For JSON, payload and emitted digests agree; CSV has its own emitted-byte digest.
Each JSON payload uses shared JCS, with no BOM, newline or insignificant space,
and the financial export's 64 MiB, depth 32 and 1,048,576-node stream bounds.
Money and revision values remain strings. Actual bytes and complete encoded size
require independent qualification; parsed shapes do not establish them.

### Focused reversible CSV profiles

Both focused CSV formats retain the financial owner's exact UTF-8, quoting,
CRLF, typed-cell, reversible string-escape and capacity rules. Quote every cell,
including headers, and terminate every row, including the last, with CRLF.
The decoded data cells use `text:s:`, `text:n:`, `text:b:`, `text:i:` and
the separately declared inapplicable token `text:x:`. A null value, empty
string, empty object, empty array and inapplicable column remain distinct.
Money, IDs and revisions remain strings; counters and container counts use
canonical unsigned integers no greater than 9,007,199,254,740,991. No nested
JSON object or array is placed in a cell. The shared 32,760 UTF-16-code-unit
and 65,536 UTF-8-byte decoded-cell bounds include the token and escaped payload;
CSV quoting is outside the cell bound and inside the stream bound.

`project-csv-v1` has this exact nine-column header:

```text
row_kind,organization_id,domain_id,project_id,content_revision,snapshot_id,path,value_type,value
```

Every data row repeats the payload's exact readback workspace, project ID,
content revision value and snapshot ID. The first data row has row kind
`project`, inapplicable path, value type `string` and value `project-csv-v1`.
All later rows have kind `field`. They encode every node of the complete
`project-export-payload-v1`, including the root, pending changes and source
availability. The row identities are projections checked against that root;
they never replace its controls, provenance or accepted-versus-pending state.

`project-financial-csv-v1` retains the financial owner's exact 24-column
header, cell and overview projections. Every row repeats the dedicated project
report ID and digest; the first `report` row's value is
`project-financial-csv-v1`. The `record` rows project complete selected records
in frozen order. `summary` and `exclusion` rows project
`report.manifest.financial_manifest`, with unchanged currency/bucket ordering
and minimal authorized exclusion identities. Project association selections,
review events and project association exclusions appear in the mandatory
complete field rows, without extra overview transactions or fetching excluded
content. File order is report, records, summaries, financial exclusions, then
all field rows. Every assigned nullable overview cell uses its null token;
every unassigned column uses the inapplicable token. These overviews are
checked projections of the complete payload, never another money authority.

For either format, one `field` row represents every payload node. Its canonical
RFC 6901 pointer is empty at root; object keys escape `~` as `~0` and `/` as
`~1`, and array indices use canonical unsigned decimals. Value type is exactly
`object`, `array`, `string`, `integer`, `boolean` or `null`. Containers carry
their exact immediate count as an integer cell; scalars carry the matching
typed cell. Rows use deterministic preorder, with parents before children,
UTF-16 JCS member ordering and increasing array indices. Financial field rows'
other columns, apart from row kind and repeated report identity, are inapplicable.

An independent reader reconstructs exactly one complete root and validates its
owning payload shape and all repeated identities, profile and overview
projections. Reject wrong headers, row width, row kinds or order; unquoted
cells, malformed UTF-8, BOM, missing final CRLF, noncanonical cells or pointers;
duplicates, missing parents, wrong scalar/container types or counts and
noncontiguous arrays. Reject all stream, depth, node and cell overflows rather
than truncating. JCS of the reconstructed payload must equal the corresponding
JSON bytes and payload digest. An ordinary CSV parser and this public profile
suffice; reconstruction needs neither an engine database nor a proprietary
importer. A separately requested JSON export may fit when CSV exceeds a bound.

The literal [byte vectors](../../contracts/validation/tests/fixtures/project-export-byte-vectors.json)
freeze complete payload, emitted and artifact-core identities for independent
review before maintained encoders or readers consume them. Encoding-only
Unicode/escape/container vectors exercise the same cell/tree convention but
do not assert a domain payload or actual human assurance. Synthetic retained
assurance and source labels remain fixture values, not qualification evidence.

The lifecycle owner retains fixed sealing/expiry, staging, current authorization,
bounded delivery, source restriction and retained receipt semantics. Existing
financial/lifecycle v1 shapes and operation kinds are unchanged. This additive
payload/core and byte-profile completion does not register a new lifecycle operation, qualify
asynchronous admission, or advertise an implemented delivery route. Complete
maintained project CSV encoding and independent readers, additive operation/admission and
exact artifact/content/delivery transport qualification remain required before
CLI consumption. No ID-only export response constitutes a complete artifact.

### Focused asynchronous export operations

`arboresce.project-export-operations.v1` is a separately registered focused
profile for `project.export` and `project_report.export`. It preserves the
original focused and additional inventories, their legacy response
definitions, and the closed financial/lifecycle v1 families. Those prototype
export responses do not establish asynchronous support. Capability discovery
must name the implemented qualified profile and its exact request, exchange,
operation and plan contracts; it cannot silently select a prototype response.
The focused profile registers both existing export POST routes, the existing
exact operation and operation-plan GET routes and the existing operation cancel
POST. No legacy lifecycle operation
kind is added. Current subject authority and complete source disclosure remain
mandatory before admission, replay and each exact read.

The existing closed export request bodies remain the input contracts. Initial
durable admission returns HTTP 202, a common accepted body, the complete queued
or running focused operation and its top-level pending originating receipt.
Location identifies that exact `/v1/operations/{operation_id}`. Authorized replay
and operation GET return HTTP 200 with common succeeded transport body and
`result={operation,idempotency_receipt}`. Pending work has null receipt expiry
pair; terminal work has the original terminal pair. Receipt replay indication
describes the request disposition, not another admission or effect. Failed and
cancelled work remain failed and cancelled inside a successful resource read.
No command receipt is placed inside the operation or at the top of a read.
Cancellation reuses the unchanged lifecycle request and complete cancellation
record/disposition rules, with its own atomic terminal receipt and the full
focused target operation. Resolve the target's immutable subject before current
action/policy checks; membership, a target reference or an original receipt
does not authorize cancellation. No disconnect or polling timeout cancels work.
Status and required headers belong to a separate normalized exchange, not
invented domain-body fields. The focused exchange observes a `WWW-Authenticate`
challenge for 401, using the `Nostr` scheme and at most 512 printable ASCII
characters without CR/LF. The qualified identity producer owns its complete
applicable challenge syntax and interpretation; a string is no proof of identity.
429 requires `Retry-After`; 202 and other errors may carry it where guidance
applies. This profile observes canonical delta seconds `0..99999999` as an
ASCII decimal string without leading zeroes. The finite representation is not
a default wait or permission to retry uncertain dispatched work. Preserve the
qualified producer's actual guidance. Current authorization precedes retained replay.

The complete focused operation retains the common identity/state fields,
command, context, command origin, historical causal command, execution
attribution, exact closed domain input, operation control, execution, effects
and exposures. Reuse the lifecycle state, cancellation, reconciliation, clock,
stage, attempt and ledger rules. Its separately closed execution additionally
retains complete `stage_results`; every accepted result reference has exactly
one corresponding complete record. No hidden, truncated or fabricated empty
collection qualifies a read. Missing required retained detail fails under the
existing lifecycle disclosure/retention outcomes.

`arboresce.project-export-plan.v1` is a closed immutable plan core containing
plan ID, scoped operation, command, context, complete selected profile reference,
execution limits, canonical domain-input digest and complete stage declarations.
Hash JCS of that entire core; exclude its own digest. The separately closed
`arboresce.project-export-operation-profile.v1` selects complete execution
limits, the exact delivery-profile reference and the first focused variant's
single required `compile_export` stage with `export_artifact` output. It also
selects the scoped lifecycle policy and the unchanged lifecycle
`command_retention_limits` in its mandatory `retention` object. Qualify their
actual policy applicability before admission. Replay duration is at least
604800000 ms, diagnostic duration at least 2592000000 ms and strictly longer
than replay. Both are terminal-relative: receipt expiry is terminal time plus
`replay_retention_ms`; tombstone expiry is terminal time plus
`diagnostic_retention_ms`, with checked UTC arithmetic. No incremental duration
is inferred from the first expiry. Polling and replay never refresh either value. Pending receipts
retain the required null pair. Minimal recognition and retirement barriers
remain governed by lifecycle continuity; receipt expiry permits no repeated
effect. These are explicit selected inputs, not defaults. The stage's input digest hashes JCS of
the complete closed domain body; the request fingerprint independently hashes
the full submitted command using the common method/path/purpose/body preimage.
Never retain the raw idempotency key in the public operation or plan.

The complete closed `arboresce.project-export-stage-result.v1` retains its ID,
scoped operation and stage, exact input/profile digests and exact artifact
reference. Its reference's result digest hashes JCS of the complete record.
Operation success requires that retained output, the matching canonical
`export_artifact_bound` effect and a typed result with command, exact subject
and artifact reference. Resolve complete artifact metadata and bytes through
the focused artifact transport; an ID-only result is insufficient. Compile-stage
completion atomically retains the immutable artifact/control, its stage-result
record, canonical effect and audit. It does not by itself claim overall operation
success. The final operation completion checks the selected policy and shares
the success/cancellation fence. If that final fence fails or cancellation wins
after stage completion, preserve the completed stage, attempt, result and effect;
failed/cancelled work still has no successful public operation result. No new
business dispatch or artifact bind follows the winning cancellation. These
focused boundaries do not alter the closed legacy financial operation contract.

Cancellation comparison requires separate qualified decision inputs using
`project_export_cancellation_decision_inputs`: the exact target, prior control,
context, complete own execution attribution and canonical decision time. These
are producer evidence, never a customer body or authority inferred from the
returned cancellation. The own command and authority audit belong to that
decision, separately from the originating command. Its recorded time cannot
precede any event represented in the retained prior operation, including stage
and attempt completion, effect commits, exposure observations, terminalization
or an earlier cancellation. Replay preserves the original decision inputs and
observed target. Queued work can become cancelled in the same one-increment
decision without inventing a worker attempt.

Attempt execution stays within its declared stage interval; no business stage
or attempt dispatch follows a winning cancellation. Retry wait cannot retain a
running attempt, and successful terminalization stays within the fixed work
boundary. Complete ledger comparison checks distinct effect/exposure identities,
scoped references, stage/attempt identity and audit attribution, observed clocks
and exposure reconciliation associations. Ledger presence itself grants no
permission for a canonical effect or external exposure.

The selected export capability admits no canonical business effect before its
compile stage. Each effect names that stage and its qualified worker execution;
stage-less admission effects require a separately defined independent admission
association before any future support. Terminal business stages and attempts
cannot remain running or retryable. Ordinary finish and reconciliation checks
stay within their fixed deadlines, and success observes settlement before its
terminal time. Diagnostic strings and claimed resolved states supply no late
observation qualification. The current conformance inputs contain no independent
late timeout, passive confirmation or post-terminal effect evidence, so those
comparisons deny when such evidence would be required. An owning producer must
define and qualify that evidence before supporting the retained lifecycle late
observation and confirmation paths; no later work or renewed budget is inferred.

Plans, profiles, fingerprints, receipts, UTC deadlines, controls, complete
ledgers, outputs and source/artifact joins require independent expected-value
and semantic qualification. Static declarations implement no canonical
admission, worker, current authority, capacity reservation or live delivery.
Qualify those actual producers and fences before enabling this profile.

### Focused artifact and delivery transport

The canonical artifact kind selects the focused project profile on the existing
`GET /v1/export-artifacts/{export_id}`, read-only
`POST /v1/export-artifacts/{export_id}/content`, and
`GET /v1/export-deliveries/{delivery_id}` routes. This does not widen the closed
financial artifact core or register another legacy lifecycle operation kind.
The separate three-entry `project_export_transport_inventory` binds operation,
method, route, normalized request, success contract, errors and exact grant
requirements. Its operations are `project_export.artifact.get`,
`project_export.content.read` and `project_export.delivery.get`.

Both GETs have exactly the organization, domain and purpose query selectors,
decoded once with duplicates and extra selectors rejected, and no HTTP body or
mutation idempotency metadata. Their normalized selectors contain read command
and context metadata plus the exact route ID. Artifact GET returns the complete
focused resource in common 200 `result.artifact`; delivery GET returns the
unchanged closed lifecycle delivery receipt in common 200 `result.delivery`.
Neither response has a top-level or nested command receipt or operation alias.
Lookup under current authority must return the complete record or fail. An
availability observation or retained delivery receipt never grants byte access.

The content POST reuses the unchanged closed lifecycle `export_content_request`:
context, complete exact export reference and expected control revision. It has
no command metadata or idempotency claim. Before bytes, compare route, workspace,
reference/core digest and current control, then recheck current source, subject,
profile, policy, expiry and grant authority and reserve capacity and an audited
delivery. The existing lifecycle fixed-grant and before/after-body failure rules
apply. No Range, resume, compression, redirect, presigned URL or multipart mode
is introduced. A retry obtains a fresh delivery admission and ID.

For a scoped agent, project artifacts require `project.read`; project financial
artifacts additionally require `report.read`. Both use `project_read` purpose
and an exact same-workspace workspace or project grant. This is a minimum
artifact access mapping, not authorization to disclose every payload member:
each included expense, association, review, source and contextual record still
requires its owning current authority. Direct human access requires current
domain policy and the same complete disclosure checks. Select the requirements
from the canonical artifact subject, never an unverified caller kind; do not
reveal the subject or delivery identity before authorization. A delivery GET
resolves its exact retained artifact for this selection, never a mutable head.

`project_export_delivery_profile` is a separately named closed profile with kind
`arboresce.project-export-delivery-profile.v1`. It has the same mandatory finite
duration, attempt, buffer, slot, staged-byte and lifecycle-policy fields and leaf
bounds as the existing financial delivery profile. All fields are selected
inputs, not defaults. Qualify idle versus total duration, organization versus
global slots, checked representable UTC arithmetic, staging/retry copies and
actual complete 64 MiB generation, transfer and cleanup before enabling support.
The artifact's profile reference binds JCS of this complete selected profile;
its fixed expiry equals sealing time plus its checked artifact lifetime.

Binary success is HTTP 200 with the complete artifact bytes. The typed
`project_export_content_response` describes the observed status and six required
headers; it is a conformance observation, not a JSON body sent instead of bytes.
Headers are Content-Type, Content-Length, Arboresce-Export-Id,
Arboresce-Delivery-Id, Arboresce-Artifact-Sha256 and
Arboresce-Transfer-Expires-At. Preserve and reject duplicate case-insensitive
instances before constructing the canonical header observation. Other ordinary
HTTP headers confer no identity or authority. Required values exactly match
artifact format, emitted length/hash and the admitted delivery ID/deadline.
Transfer expiry is the minimum of fixed artifact expiry, known current authority
expiry, admitted time plus selected transfer total and the qualified server
deadline. None may be inferred from a receipt, refreshed by idle activity or
silently omitted. The client checks the complete byte length/digest and commits
only a protected complete destination after successful verification.

The closed `project_export_delivery_admission_inputs` supplies independent
conformance inputs from the qualified boundary: context, executor, delegation,
audit event, admission time, authority expiry and server deadline. It is neither
a customer POST body nor an authenticated grant. Unknown required deadlines are
representable as null and must deny admission. Qualify their actual trusted
producer and current fences before runtime use; never derive these inputs from
the returned receipt or accept client assertions as their authority.

## Required qualification

[Project fixtures](project-fixtures.md) owns the deterministic corpus and review
trace. Actual CLI/API/PostgreSQL/object-store/Temporal/gRPC integration is required;
recorded inference qualifies deterministic behavior, not live processing. Qualified
OCR, ASR and contextual synthesis require separate actual routes and independently
checked truth. G2-project and actual D0-project restoration precede personal
evidence and independent users. Full knowledge, search/repair, D1 recovery and
production obligations remain retained under [acceptance](acceptance.md).
