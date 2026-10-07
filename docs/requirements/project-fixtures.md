# Contextual project fixture and oracle contract

**Authority:** normative focused CLI workflow fixtures. **Status:** fixture inputs,
runtime suites and live quality remain to be supplied and qualified.

[Project workflow](project-workflow.md) owns behavior;
[testing and coverage](testing-and-coverage.md) owns independent expectations and
measurement; [acceptance](acceptance.md) owns actual-user/live/production claims.
Recorded processing is a deterministic test mode, never a substitute for actual
route qualification.

## Complete canonical case

The canonical case is one receipt photograph, an approximately 30-second related
voice note and an existing project involving several parties. The note reports an
expense and meeting/project changes without restating all known context. Include
one useful omitted detail recoverable from prior context and one unresolved detail
that must remain unknown or require clarification. The case must yield multiple
reviewable items, at least one manual correction and a follow-up revision.

The versioned manifest must include:

| Group | Required independent inputs |
| --- | --- |
| Originals | Exact source files/bytes, SHA-256, length, actual media/container/codec, source roles, rights/access classification and capture revision; never assumed camera/codec support. |
| Time | Attributed recording/meeting date, timezone and precision; separate upload/receipt time; explicit relative-date rule expectations. |
| Prior context | Workspace/domain/project IDs and base revisions, Party IDs/aliases/roles, participants, existing decisions/work/responsibilities/dates/status/facts, and independently defined authorization policies. |
| Representations | Qualified normalization/profile identities, orientation/pixel and channel/time mappings, exact derivative hashes and source relationship. |
| Recordings | Separate OCR, ASR and semantic-synthesis request/output recordings with exact input/context hashes, library/model/prompt/parser/schema versions and complete warnings/errors. |
| Truth | Human-checked source transcript, receipt values/readability, contextual reference labels, ambiguity/unknown/disagreement and expected semantics; independent of the processor under test. |
| Expected proposals | Deterministic JSON item identities, target/base versions, typed proposed content, evidence/context/default/inference lineage, dependencies, unresolved values and expected review eligibility. |
| Review trace | JSON edit/accept/reject/pending commands, supplied preconditions/idempotency intent, expected errors/results and a follow-up correction/reanalysis preserving earlier manual decisions. |
| Final state | Exact canonical revisions, accepted mappings, pending/rejected items, separate reviewed expense/project association, source-linked readback/export, restart and restored-state expectations. |

A supplied Whisper transcription is an ASR recording with its available actual
metadata. It is not automatically a reference transcript, proof of live processing,
or corpus accuracy. Missing metadata is explicit. Human verification labels
inaudible/ambiguous words rather than guessing them. Receipt truth must preserve
unreadable fields; neither OCR nor synthesis output is its own oracle.

Runtime input contains only evidence and authorized prior context. Keep expected
answers, future corrections, final state and held-out labels outside model context
and production resources. Deterministic IDs/clocks use throwaway test identities;
never include real identity secrets, personal originals or private metadata in
public source. Public standalone fixtures use rights-cleared synthetic evidence.
Permitted private originals remain in a separately controlled fixture corpus with
access/retention policy, prepared after the required security/recovery gates.

## Package validation and authoring

The versioned package has independently defined required groups, typed fields,
case identities and category coverage. An oracle cannot define away its own
mandatory inventory. Empty packages, missing collections, malformed or duplicate
identities, unbound preimages and inconsistent cross-group relationships fail.
Bound file/JSON reads and reject unsafe or outside-package source paths before
consumption. A digest must hash its declared canonical preimage, not an item label.

Runtime projection uses an explicit evidence-and-authorized-context shape. It
excludes truth, expected proposals, verdicts, future clarification, final state
and recovery answers regardless of their field names. Substring sentinels cannot
establish isolation or reject otherwise permitted Party names. Record full prior
content/revisions, actor roles and policies, attributed event/timezone context,
derivative/profile preimages and complete separate processor recordings.

The primary synthetic journey reaches project updates and a separately reviewed
exact expense/project association and financial confirmation. Preserve initial
ambiguity and provide independently authored attributable clarification as a
separate future input. Retain held, rejected and unreadable cases. Synthetic
review-mechanism evidence qualifies fixture protocol only, never actual human
assurance. Expected transitions include exact actors/grants, CAS/control/intent,
creation mappings, dependencies, complete history and distinct reanalysis that
preserves manual decisions. Final-state/export/follow-up/recovery groups contain
concrete values and bytes, not desired-behavior summaries.

Every retained matrix case has bounded concrete inputs, actors, revisions,
commands, exact results/errors/exits and effect/no-effect expectations. Preserve
all previously reviewed cases and categories, adding cases needed for new rules.
Expected values remain independent of the implementation or processor under test.

Generators are inert on import and run only through explicit invocation. Qualify
exact tool, codec, voice/font and normalization identities before reproducibility
claims. Generation uses bounded subprocesses, safe temporary staging and package
replacement with verified contents; a failure cannot report a complete package.
Ordinary checks use frozen committed originals and never regenerate them merely
for review. Maintained builders, validators, helpers and actual subprocess paths
join their owning source inventory and strict qualification in the same source
checkpoint. Partial group checks identify their bounded scope and cannot approve
the complete corpus or its runtime consumers.

## Required scenario matrix

The D0-project inventory includes at least:

- Two workspaces with overlapping names; the same actor can access both. Every
  capture/item/expense/reference remains in exactly the selected one. Also test
  inaccessible and revoked workspace/domain/project relationships.
- Known and ambiguous aliases, unknown external parties, supplier versus attendee
  versus speaker/assignee/reviewer, unverified reported participation and assent.
- Omitted established context, genuinely unknown information, conflicting context,
  date-only versus instant, event timezone and relative dates, and context revocation.
- Negation, planned versus completed work, conditional decisions, changed owners,
  partial/uncertain transcript, unreadable receipt fields and financial disagreement.
- Independent items and dependent holds; partial accept/edit/reject; rejected
  prerequisite; cycle/cross-workspace dependency; current target CAS conflict;
  unrelated project update; shared-target races; no blanket whole-project conflict.
- Manual override then reanalysis; accepted/rejected history preservation;
  follow-up supersession; exact replay and different-body conflict; no duplicate
  project task, acceptance, expense or confirmation after a lost response/restart.
- Agent own-key/grant acceptance, revoked/expired chains and unknown assurance;
  financial challenge tampering/expiry, actual-review/executor separation,
  stale project association and one-time competing financial consumption.
- Missing/optional/failed attachments, spoofed media/codec, orientation, truncation
  warnings, parser bombs, unsupported input, worker failure, cancellation before
  and after accepted result, uncertain provider dispatch and finite shared quotas.
- Old-backup recovery with post-backup restrictions/holds/revokes, missing independent
  journal tail, unknown effect, keys/trust, operation/outbox and original-byte
  consistency. Unknown recovery proof keeps serving closed.

## Verification lanes and claims

Deterministic integration uses the actual independently built CLI, public HTTP API,
PostgreSQL, object storage, Temporal and gRPC transport with prepared recorded
inference. It must complete initialization, context setup, capture, processing,
individual review, separate expense review/confirmation, readback/export, follow-up,
restart and packaged restoration. Verify failures at their actual service/process
boundaries. Independent expected-value review precedes implementation; maintained
fixture builders/oracles/harnesses remain measured source.

Live qualification uses actual separately qualified OCR/ASR/synthesis routes and
independently checked evidence. Score lexical transcription and consequential
amount/date/negation/attribution/grounding/abstention separately under frozen
readability labels, denominators and numeric thresholds. Wording variation is
allowed only where the oracle defines semantic equivalence. Record actual routes,
costs and data exposure. The canonical case proves a concrete workflow; population
quality requires an independent held-out corpus, and independent utility requires
actual authorized users. None is implied by fixture generation or recorded output.
