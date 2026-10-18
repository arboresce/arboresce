# Owned transport close boundaries

These original public synthetic literals use MIT OR Apache-2.0. Their owners are
the existing [resource](../../../../docs/requirements/resource-profiles.md),
[storage](../../../../docs/requirements/storage-and-search.md),
[security](../../../../docs/requirements/security-and-privacy.md) and
[testing](../../../../docs/requirements/testing-and-coverage.md) requirements.
They preserve every existing migration, authority, shared-admission and cleanup
fixture. They do not change a transaction result or a resource budget.

Only common_inputs and case.input reach a producer. Identity, expected results,
profile fields, counts, rights and file pins remain outside runtime decisions.
Each complete expected object is compared after the producer returns. A native
producer additionally receives the already verified, exclusively owned service;
no fixture supplies a path, PID, connection string or cleanup target.

The five completion cases exercise the same awaited owned-task completion
boundary used by physical close. A controlled task owns a destruction marker.
Observable start/completion channels establish ordering without sleeps.
A completed task before its original deadline permits success. A panicked or
cancelled task fails despite marker destruction. A still-pending task at the
deadline fails without pretending it has completed; its controlled channel is
then released and its actual cleanup awaited separately. A ready successful
task after an already expired original deadline still fails. Completion evidence
records task status and marker destruction at the decision, separately from the
later cleanup. These are controlled task observations, not native PostgreSQL,
socket, transaction, slot, host-failure or recovery evidence.

The six native cases use the real selected PostgreSQL service and exact backend
identity. Raw value cases execute the compiled SELECT $1::bigint, binding case.input.value,
and observe that exact returned integer. The peer-termination value query
finishes before its own backend is deliberately terminated. The raw operation
error case executes the fixed compiled SELECT 1/0 and retains the actual SQLx
error. Peer termination must be acknowledged before close. The operation-error case retains the original boxed SQLx error and its
allocation, including SQLSTATE 22012. Graceful protocol shutdown or a particular
OS error is not an oracle: actual owned transport destruction must complete
within the unchanged 1,000 ms close budget, and a bounded actual server census
must independently show the exact backend absent. The peer-termination cases
require the real server acknowledgement and observed absence before close.

Authority cases close an ordinary session, an externally terminated owned
session, or a session poisoned by an actual failed backend query after that
termination. Each starts with its own verified backend. Its finite local set
remains charged at one available slot before close and returns to two only
after positive owned close. The poisoned query retains Unavailable; it is not
a definite transaction outcome. This family contains no in-flight transaction,
queued-rollback or cancellation schedule. Existing real uncertainty, rollback,
race and poisoned no-driver-I/O scenarios remain mandatory.

A native case never substitutes an empty result for an operation, query, census,
termination or close failure. The unchanged no-adoption guard remains required.
No case requires OS57, retries a failed action or expands a deadline. The four
existing typed/allocation-preserving cleanup-combiner cases remain unchanged.

Static admission qualifies only these finite meanings and public provenance.
It does not qualify the future consumer, coverage, runtime or product behavior.
All original owning and fresh public-only standalone, full unfiltered, strict
coverage, process/profile/native/resource/conservation/retention, both owned
cleanups, current-image model, separate process-probe, independent source,
expected, distinct delivery and whole-checkpoint requirements still apply.
