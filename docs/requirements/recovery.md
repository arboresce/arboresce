# Recovery and portable exit

**Authority:** normative requirements. **Status:** accepted; recovery tooling,
operational policies and restoration results remain separately qualified work.

This document owns restoration, lifecycle-journal completeness, safe reopening
and usable export. [Security and privacy](security-and-privacy.md) owns admission,
revocation, holds and retention policy; [storage and search](storage-and-search.md)
owns canonical persistence and projection rebuild mechanics. [Acceptance](acceptance.md)
owns D0/D1 qualification gates and recovery time/data-loss thresholds.

## Recoverable state and policy

A database backup alone is not a usable or privacy-correct backup. The recovery
set includes canonical PostgreSQL records and relationships, sealed originals and
required derivatives, encryption-key material, signing/trust history, schema and
processing-profile identity, and the current independently retained lifecycle
journal. Record which exact versions and objects the backup represents and how
they are authenticated. Projection snapshots are optional rebuild accelerators;
they cannot become canonical truth or current authority.

Maintain a versioned policy for backup frequency, retention, access, encryption,
storage location, hold handling, destruction, key recovery and journal continuity.
The operator must select and approve real policy and resources before admitting
restricted evidence or performing real purge. Synthetic policy values allow
generic implementation tests; they do not establish legal retention or a supported
operational commitment. A backup promise requires an actual restoration drill.

Keys and recovery credentials have a distinct access boundary from ordinary
application access. Inventory every decryption dependency and the trust material
needed to interpret historic signatures. Unknown, missing or revoked trust must
not expand authority during restoration. Do not destroy a shared key before
confirming its surviving dependencies and required recovery obligations.

## Current lifecycle journal

Retain revocations, deletions, holds and relevant authorization restrictions beyond
the database backup boundary, with independently verifiable continuity and an
explicit recovery cutoff. The journal must be independently retained so that a
lost or rolled-back canonical database cannot also erase all proof of later
restrictions. Its access and integrity controls must preserve minimized lifecycle
evidence without duplicating full customer content.

The concrete journal contract must define event identity, policy/subject revision,
durable recording, authenticated ordering or completeness proof, checkpoints,
retention and gap detection. Qualify that proof before allowing restored serving.
A journal recovered only from the same old backup is insufficient. The highest
allocated outbox or sequence ID is not proof of committed completeness: concurrent
transactions can commit in another order or leave gaps. Do not infer an intact
tail from a locally observed maximum or from successful replay of available rows.

Prove both the intended recovery cutoff and all applicable restrictions through
that cutoff against an independently retained record. Apply later policy before
any new read, processing, external exposure, export or snapshot resolution is
admitted. If the tail, cutoff, authenticity or completeness is unknown, serving
stays closed. A nominal recovery-point target never permits resurrecting a later
revoked or deleted object. Legitimate policy changes and hold releases must retain
their authentic authority and ordering; replay may not invent a new grant.

## Restoration procedure

Implement a bounded operator tool and runbook before a recovery claim. Their
documented interface must distinguish inspection, preparation, mutation and opening
service; this specification does not claim those commands currently exist.

1. Select the exact backup, schema/tool versions, key/trust set, lifecycle cutoff
   and isolated destination. Verify authorized operator access and resource limits.
   Disable all serving, worker admission and outbound provider delivery.
2. Restore canonical PostgreSQL, sealed required bytes and key/trust material.
   Validate digests, lengths, tenant relationships, references and backup completeness;
   report absent or corrupt objects instead of fabricating replacements.
3. Obtain and validate the independent journal through the recovery cutoff.
   Reapply post-backup revocations, deletions, holds and authorization restrictions
   idempotently under the correct policy versions. Confirm that missing, revoked
   and held evidence has the required denial and retention behavior.
4. Reconcile durable operations, accepted stage results, outbox/delivery receipts,
   leases and unfinished cleanup. Expired claims do not justify duplicate canonical
   transitions or another unbounded external side effect. Keep uncertain provider
   outcomes explicit and follow the [processing contract](processing.md).
5. Rebuild derived stores from eligible canonical state using the generation and
   maintenance-barrier protocol in [storage and search](storage-and-search.md).
   Reuse authorized embeddings only when content and processing profiles match.
   Validate membership, counts, per-record digests and current restrictions before
   switching generations. Corrupt projections cannot restore revoked authority.
6. Run restoration integrity, current-admission and exact-resolution checks with
   serving still closed. Verify actual identity configuration, required dependencies,
   readiness and any approved operating limits in the restored environment.
7. Open only the qualified serving scope after all required proofs pass. Record
   operator identity, input/output identities, cutoff/completeness proof, timings,
   failures, unavailable data and the exact authorization to reopen.

Missing required blobs, keys, journal tail or completeness evidence prevents
opening the affected restored serving scope. Do not respond with a partially
reconstructed required snapshot. Repeat/repair must remain idempotent and preserve
the failed attempt's evidence. A recovery mechanism cannot quietly skip a required
check to meet a time target.

## Early D0 and later D1 qualification

Implement and exercise actual D0 restoration before independent users rely on
their own evidence. Back up the [D0 corpus](resource-profiles.md), then perform
later revocation, deletion and hold changes. Restore the older canonical snapshot
with the current independent journal and prove no access is resurrected. Include
missing/corrupt blobs, missing keys, journal gaps and unknown tail/cutoff negatives;
each must keep serving closed. Corrupt or remove a derived store and reconstruct
it from canonical state with restrictions intact.

Repeat that drill through the packaged local profile after packaging exists.
Packaged G2 security and this actual D0 recovery proof precede independent-user
qualification. A successful unit fake or a pre-packaging component drill alone is
insufficient. Full D1 restoration, the later restriction workload, measured RPO/RTO
and actual residency qualification remain required by [E09 and G6](acceptance.md).
Early D0 does not waive them.

## Projection outages, deploys and shutdown

Exact authorized build resolution remains independent of search projection
availability. If required canonical state or evidence cannot be established,
resolution denies; a stale projection cannot fill the gap. Search outage and
single-host storage limits must be reflected in readiness and support claims.
Multiple API processes or durable volumes alone do not establish high availability.

Deployment and recovery tooling must preserve accepted data. Use compatible
expand/backfill/contract migrations, documented rollback limits, drain/readiness
checks and supported sequential workflow-service upgrade rehearsals. Do not rely
on destructive down-migrations or an untested version rollback. Replay histories
and reconcile accepted durable work across each relevant workflow/SDK change.

Shutdown stops readiness and new admission, stops acquiring leases, drains bounded
requests/work, propagates cancellation where supported, releases or fences claims,
and terminates within its declared budget. An external call or slow download must
not keep a process alive indefinitely. Another worker or restart must recover
accepted work without pretending every provider effect is exactly once. The
[deployment](deployment.md) and [processing](processing.md) owners define the
component-specific behavior and clocks.

## Blob reconciliation and deletion restoration

Reconcile staging and sealed objects against canonical references, versioned policy
and current holds. Identify and quarantine suspected orphans before controlled
deletion; age or matching digest alone does not establish orphanhood. Purge
rechecks all references and holds at its consequential boundary, records individual
controlled-copy outcomes, and survives interruption without deleting a shared
object twice or losing responsibility for incomplete cleanup.

Retained minimal tombstones and the independent journal must prevent restoration
from making purged or revoked content available again. A held-but-revoked object
stays retained and denied. Restoring a backup for recovery cannot become an export
route around lifecycle policy. Record copies beyond system control and external
deletion requests accurately rather than promising recall.
Retaining a historical digest or identity alone never promises that the original
content remains reconstructable after its bytes have been purged.

## Usable export and import

An authorized export includes a versioned public manifest, exact selected records
and revisions, permitted original/derived bytes, relationships, evidence locators,
corrections, attribution, context builds and relevant evaluations, with digests and
explicit authorized unavailable/purged markers. Do not expose inaccessible objects
or counts through omissions. Pin an authorized selection, bound temporary storage
and expiry, and reauthorize delivery; mid-export revocation must block affected
delivery. Duplicate requests retain their original intent and result semantics.
Cancellation cleans up controlled temporary work without deleting user exports.

Prove usefulness with an independent reader/import validator or a specified
restoration interface in a clean open deployment. Preserve references, originals
and attribution; define identity remapping when importing into a newly authorized
organization. An archive cannot import old membership, credential authority or
approval trust merely by containing their previous identifiers. Evaluate signatures
under explicitly selected current/historical trust instead of accepting stale
signing status as a new grant.

Validate schema, lengths, digests and completeness before use. Reject archive path
traversal, absolute paths, symlinks, special files, overwrite hazards, extraction
bombs and executable content; import must not execute content. Preserve encryption
and access requirements in transit and temporary storage. Financial exports follow
[financial records](financial-records.md), including a lossless structured form
and safe spreadsheet form. A downloaded file alone is not portability evidence.

## Required operational evidence

Record exact engine/client versions, backup and blob identities, schema/lock/image
digests, policy, key/trust references excluding secrets, hardware, region, clocks,
journal cutoff and proof, recovered counts/digests, unavailable records, rebuilt
generations, timings and review. Define the measurement start/end events for data
loss and time to qualified serving. Local artifacts do not prove independent
backup retention; actual retained resources and restore behavior must be tested.

Incident response must be able to suspend affected admission, preserve minimized
evidence, scope exposure, rotate affected credentials under authority, restore and
add regression scenarios. Unprovided resources, policy, keys or journal proof block
the named recovery gate. Record the missing item and continue independent work;
do not invent successful recovery or weaken a required security result.
