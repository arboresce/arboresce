# Project-change value comparison profile v1

**Authority:** pure domain API target derived from the owning project-workflow,
candidate-review and common contracts. **Status:** target profile; implementation
and runtime qualification remain open. This profile adds no endpoint or approval
ledger. Acceptance is recorded in its owning checkpoint evidence.

## Stages and input boundary

Parsed change-content and change-item constructors check closed forms, exact
tagged identities, operation/content/target joins and canonical content identity.
They may represent pending proposals subsequently found ineligible. Semantic
comparison is a separate pure operation over an immutable typed change plus
explicit bounded comparison facts. Both stages require independent fixtures
before implementation. A parsed-form pass cannot close semantic criteria.

Comparison facts are application-supplied observations, not caller authority.
The eventual application must resolve them canonically, enforce current
disclosure and repeat consequential fences at its transaction boundary.
Synthetic test facts establish only the pure algorithm's behavior. No
authorized, human-approved or production-safe boolean is part of this interface.

The fixture runtime projection supplies only selected item/value, selected exact
item revision and selected comparison object. Fixture IDs, answers, other worlds,
manifest and expected diagnostics never enter the operation.

## Finite comparison input

A comparison object carries its explicit profile version, selected workspace,
project reference and complete origin. These must match the selected change's
stable scope and retained origin. A later unrelated project-head observation is
not a whole-project stale fence.

The selected item revision is a tagged content revision. For an item it equals
the candidate reference version; a content preimage receives that revision
explicitly, without inventing a revision inside the canonical preimage.

The input has at most 64 exact proposal nodes, 128 exact project-record facts,
64 Party facts, 192 observation facts, 16 default-rule bindings and 32 inference
creation/profile facts. Each node has at most 32 dependencies. Each fact retains
its exact stable identity and tagged revision where applicable, selected
workspace/project, complete needed content and any separately supplied current
head/state observation. Conflicting observations for the same exact identity
are refused. The whole comparison object is at most 1048576 encoded UTF-8 bytes
and 16 nested containers. General object/array bounds remain128/256.
The encoded-byte ceiling is the length of compact common-JCS encoding of the
complete supplied parsed comparison value, not original wire bytes or a
pretty-printed fixture. Strings keep their exact Unicode content; only required
JSON escaping is emitted. Object order does not change length. Numbers use the
existing common lossless canonical-number profile before admission; nonfinite,
unsafe or nonrepresentable values are refused. This does not qualify original
raw numeric tokens. Counters are checked parsed unsigned integers within their
declared bounds; strings, booleans, negative or fractional values are not coerced.
This is numeric meaning after common-number admission: an integral parsed
number such as0.0 denotes index0; it does not accept a string or round0.5.
An original transport may require a stricter raw integer token at its own gate.

All referenced current/exact facts used by comparison must be present. Missing
facts produce a typed incomplete-comparison result; they do not establish
absence, source truth or permission.
A referenced Observation without its needed projected field value is incomplete
(missing_observed_field); an inherited occurrence without its explicit retained
source-field binding is incomplete (missing_retained_binding). Both leave later
stages not_checked. The exact selected proposal node is required alongside the
supplied selected tuple; its absence reports graph=incomplete,
dependency_readiness=missing and missing_selected_node after valid lineage.
A create's absence predicate still needs the real service's canonical transaction
check.

## Consequential fields and lineage

Field paths have1 through128 ASCII characters under the existing lower-case
dotted grammar. Syntax and resolution are separate checks. content.<path>
selects the complete proposed project content. candidate.statement,
candidate.scope, candidate.qualifications and candidate.evidence select the
corresponding review-content fields of the canonical change preimage, not the
wire item's candidate-reference object. Nested paths resolve actual supplied
members; array indices and arbitrary executable patches are unsupported.

An unqualified path is admitted only if exactly one candidate-content or
project-content interpretation exists. A missing member, extraneous path or
multiple interpretation is refused. Resolution never rewrites original path
strings in the canonical preimage.

Every consequential content field other than the fixed top-level content-kind
discriminator has lineage. A binding to an entire typed structured value covers
its descendants; otherwise all consequential leaves require bindings. Every
declared candidate-content changed field also requires lineage. Duplicate
lineage groups retain their complete order and entries; they are not silently
merged or globally banned.

For create, all proposed content is new for changed-field coverage; this does
not prove actual target absence. For revise/associate, the exact supplied base
target content determines changes. Every declared changed path must denote a
genuine change, and declared paths together cover every changed consequential
leaf. A changed structured parent may cover descendants even when some of those
descendants are unchanged. A path to an unchanged value is not a positive
change control.

Source-observation lineage resolves the exact supplied Observation reference.
Its fact preserves the actual public kind: document_observation, user_assertion
or model_inference. There is no assertion alias. A source fact's capture names
that Observation's supplied immutable association; it need not equal the
proposal's analysis-origin capture. Historical and different-capture sources
remain supported. The reference and capture organization agree; a used source
capture remains in the selected workspace. Actual canonical reads, authenticated
attribution and original grounding establish that immutable association before
application; the pure projection cannot detect a forged single fact by guessing
origin equality. Conflicting supplied associations/kinds for one exact fact are
refused. Matching projected values never relabel an assertion or inference as
independent truth. Retained-context lineage resolves its exact record
and the explicitly supplied source-to-destination field binding. Its consequential
record revision is also an explicit dependency. Typed Party references resolve
their exact supplied workspace-scoped Party facts and current head; they are
not fabricated as project-record dependency kinds. Missing, foreign or stale
Party facts produce their corresponding comparison/readiness reason. No
reference is promoted to independent truth or a different provenance category.

Default lineage must name an explicitly selected supported rule binding for its
resolved field. The first comparison fixture profile supports event_date_v1:
for a date-precision attributed event, an explicitly selected exact due-date
field may use that calendar date. Its binding and applies_to description must
agree, and the resulting value/precision must match. It does not interpret an
instant's timezone, invent a due date without an applicable selected binding or
execute rule prose. Unsupported rules and mismatched field/value/applicability
are distinct refusals. Actual selected-profile qualification remains external.

Inference lineage retains its exact profile slug/digest, basis, alternatives and
unresolved qualifications. Origin processing-profile digest and lineage profile
digest are not universally equal. Where comparison claims a creation/profile
join, a fact binds the exact field/entry, creation analysis/revision and profile
tuple; substitution is checked against that fact. Legitimate distinct or
historical provenance remains supported. Absence of a fact cannot be replaced
by a guessed digest equality.

Unresolved field paths must resolve without invented certainty. Their complete
clarification/unknown/conflict requirements are preserved. They can form a
pending proposal while preventing dependency/effect readiness under the
applicable consuming policy.

## Exact dependencies and graph result

Consequential retained records and typed subject-record references retain exact
kind, stable ID and content revision in the explicit dependency list. Missing or
substituted revisions are refused or reported incomplete before any effect.

Proposal nodes are indexed by exact item ID and content revision. For the
selected node and its reachable dependencies, workspace agrees with the selected
workspace and project is explicitly selected in context_projects. The selected
node and dependency declarations agree with the selected change and exact
referenced nodes. Exact self-edges and directed cycles are refused.
Checks apply to the selected item and its consequential reachable dependency
closure. An unrelated rejected item or unrelated cycle cannot block an
independent item. Whole supplied-context syntax, fact consistency and capacity
checks still apply. A valid pending DAG remains a representable proposal;
dependencies need not already be accepted merely to construct it. Different item revisions never
silently substitute for one another.

The result separately reports structural/lineage validity, graph validity and
dependency readiness against the supplied current facts. Pending, rejected,
stale, missing and foreign dependencies receive distinct fixed typed reasons.
Accepted/current dependency facts can establish dependency readiness only.
That result is not actual authorization, human acceptance, target CAS success,
persistence, a financial approval or permission to execute an effect.

## Canonical identity

Within each evidence role, reject repeated exact identities, then sort by exact
ASCII organization_id and observation_id. Different roles remain distinct.
Every other array remains ordered; strings are never trimmed, case-folded or
Unicode-normalized. Canonical project-change preimages are complete and at most
1048576 bytes. A complete encoded item has the same independent ceiling.
Mandatory complete current/exact GET reserve and transport raw-byte limits
remain consuming gates.

An optional supplied preimage must agree under these content semantics. Exact
candidate context/ID/version remains separate from content digest. Equivalent
evidence permutations preserve normalized content/item identity, while submitted
command distinctions remain with the command/fingerprint owner.

## Exact fixture projection and comparison fields

The fixture projection for compare_content has exactly content, item_revision
and comparison. The projection for compare_item has exactly item and comparison;
its revision comes from the exact candidate tuple. These are pure API fixture
bindings, not additions to the closed public command or item wire schema.

All comparison members are required: version, workspace, project,
observed_project_head, origin, selected_item, context_projects, records, parties,
observations, nodes, retained_bindings, default_rules, inference_facts and
previous_candidate_content. Unknown members are refused. The profile version is
arboresce.project-change-comparison.v1. previous_candidate_content is null in
this initial proposal-comparison profile; manual edit history has its own gate.
observed_project_head is a tagged content revision retained as an observation,
not a fence on every proposal. context_projects contains1 through8 explicit
project references; it includes the selected project and may explicitly include
additional same-workspace context. Mere access to another project supplies no
selection or authority.

The following closed fact projections have these exact members:

- A record fact has workspace, project_id, record, content and
  current_content_revision. record is the exact typed project-record reference;
  content is complete typed content. A historical exact record can coexist with
  a separate later head observation.
- A Party fact has record and current_content_revision. record is the complete
  workspace-scoped Party record, not a Principal or Membership.
- An observation fact has reference, capture, kind and observed_fields.
  kind retains one of the three public Observation kinds. Each observed_fields
  row has exactly field and value. A parent-value binding can resolve a supplied
  descendant. Values are finite bounded JSON projections with their exact types;
  they are compared to resolved selected fields only when used for lineage.
  Unconsumed projections may contain bounded arrays/objects or numbers without
  asserting a selected-field match. No Boolean/numeric or string/number coercion
  is allowed. Original kind and attributable reference remain; matching values
  establish neither original grounding nor independent truth.
- A proposal node has item_id, content_revision, workspace, project_id,
  dependencies, state and current_content_revision. State is pending, accepted,
  rejected, contested or superseded. Exact revisions are graph vertices; a
  reference to an earlier revision of the same stable item is not automatically
  an exact self-edge.
- A retained binding has field, group_index, entry_index, record and source_field.
  It binds an exact lineage occurrence to an exact supplied record field.
- A default binding has rule_version, field and applies_to. Only a binding
  explicitly selected for the applicable field can justify a default. Compatible
  repeated fact observations do not invent additional authority.
- An inference fact has workspace, project_id, field, group_index, entry_index,
  profile, profile_digest, analysis_id, analysis_revision, basis, alternatives,
  unresolved and value.
  Its complete provenance and value match that exact lineage occurrence.
  Creation analysis may be historical; digest inequality with the selected
  origin alone is neither substitution evidence nor a rejection rule.

group_index is an integer0 through63 and entry_index is an integer0 through15.
These are checked pure fixture/API counters; parsed-value tests do not qualify
an original transport's raw numeric-token handling. At most64 retained bindings
are accepted. An observation has at most64 observed field/value bindings.
Default and inference capacities remain16 and32, respectively. Compatible
duplicates preserve complete supplied order. The same qualified exact identity
or selected binding has one complete closed fact view in a comparison. Identical
complete copies are compatible; divergent views for that same identity/binding
are refused, including differences in capture association, kind, typed value or
current observation. This consistency check covers the whole supplied context,
including unused facts. A repeated observed-field spelling may retain identical
typed values; conflicting values for that same binding are refused. Typed
record content agrees with its record reference kind. Closed wrappers reject
unknown or missing members independently of whether their fact is consumed.
Referenced dependency projects must be explicitly selected in context_projects and remain in the selected workspace.

A comparison report has lineage, graph, dependency_readiness and reason.
lineage is valid, invalid, incomplete or not_checked. graph is acyclic, cycle,
self_edge, foreign, incomplete or not_checked. dependency_readiness is ready,
pending, rejected, contested, stale, missing, unresolved or not_checked.
reason is null or a fixed typed reason. Invalid or incomplete lineage leaves
graph and dependency_readiness not_checked. In particular a missing Party fact
has incomplete lineage and no later readiness claim, like other missing lineage
facts. There are two explicit structural-availability diagnostics after valid
lineage: a missing required consequential fence reports graph=not_checked and
dependency_readiness=missing; a missing exact graph fact reports graph=incomplete
and dependency_readiness=missing. Those missing classifications do not mean
current-readiness evaluation completed. Cycle/self-edge/foreign graph failures
leave dependency_readiness not_checked. Current target/Party/dependency states
are classified only after a resolved acyclic graph; no unexamined graph is
claimed acyclic.

Invalid forms and malformed comparison objects are refused before semantic
claims. Complete field resolution/coverage and known-source comparison precede
the required-fence and graph checks. Missing declared consequential fences are
reported before current-readiness classification. Acyclic pending/rejected/
contested/stale graphs remain representable; those states prevent dependency
readiness. A full command still requires real current authorization, final target
and consequential CAS, applicable review and every consuming transaction gate.

Owning requirements: [project workflow](project-workflow.md), [candidate review](candidate-review-contract.md), [common contracts](common-contract.md) and [resource profiles](resource-profiles.md). These existing requirements prevail over an incomplete implementation or fixture. Parsed forms and comparison results qualify only their declared layer.
